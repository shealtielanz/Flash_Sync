contract;
use flash_abi::{
    FlashSettler,
    FlashAbi,
    calculate_five_percent,
     };

use std::{
    context::this_balance,
    asset::transfer,
    call_frames::msg_asset_id,
    context::msg_amount,
    constants::*,
};

storage {
    non_reenter: u8 = 0,
    flash_predicate: Identity = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000)),
    already_set_predicate: bool = false,
}


impl FlashSettler for Contract {

    #[storage(read, write)]
    fn trigger_flashloan(amount: u64, flash_user: ContractId) {
        let non_reenter = storage.non_reenter.read();

        require(non_reenter == 0, "avoid reentrancy");
        storage.non_reenter.write(1);

        let base_asset_id = AssetId::base();
        let bal_this = this_balance(base_asset_id);

        require(amount <= bal_this, "not enough balance");

        let flash_user_contract = abi(FlashAbi, flash_user.into());
        /*
        assume that on the call to usd the flash loan the user will pay back the assets with the specified fee
        this is due to fuel uses native assets, so no transferform()
        */
        flash_user_contract.use_flashloan{
            asset_id: base_asset_id.into(), coins: amount
        }(amount);

        forward_funds_to_predicate(amount);
        storage.non_reenter.write(0);

    }


    #[storage(read, write)]
    fn set_flash_loan_predicate(_predicate: Identity){
      require(!storage.already_set_predicate.read(), "predicate id already set");
      storage.flash_predicate.write(_predicate);
      storage.already_set_predicate.write(true);
    }

    #[storage(read)]
    fn send_dusts_to_predicate(){
        let base_asset_id = AssetId::base();
        let bal_this = this_balance(base_asset_id);
        if (bal_this > 0){
           transfer(
               storage.flash_predicate.read(),
               base_asset_id,
               bal_this
           )
        }

    }


    #[storage(read)]
    fn get_predicate_id() -> Option<Identity>{
        Some(storage.flash_predicate.read())

    }
}


#[payable]
#[storage(read, write)]
fn forward_funds_to_predicate(amount: u64)  {
    // get base asset 
    let base_asset_id = AssetId::base();
    let fee = calculate_five_percent(amount);

    let loan_with_fee = amount + fee;

    let bal_this = this_balance(base_asset_id);

    require(loan_with_fee <= bal_this, "not enough balance in settler");

    // then do the transfer

    transfer(storage.flash_predicate.read(), base_asset_id, loan_with_fee);
}
