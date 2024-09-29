contract;
use flash_abi::{
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
    flash_settler: Option<Identity> = Option::None,
}

impl FlashAbi for Contract {

    #[payable]
    #[storage(read, write)]
    fn use_flashloan(amount: u64) {
        let base_asset_id = AssetId::base();
        require(msg_asset_id() == base_asset_id, "Not the asset requested for");

        require(
            msg_amount() >= amount,
            "TransferError::NotEnoughCoins",
        );

        perform_yield_strategy_and_gain_profit(amount);

        /*
        use the flashloan to implement any desired strategy
        and forward back to the Flashloan settlement contract 
        the borrowed funds plus fee.
        this contract will funded for the test just to have funds to return amount borrowed with the fee.
        */ 

        //fee of 5% per loaned taken
        let fee = calculate_five_percent(amount);

        let amount_with_fee = amount + fee;

        match storage.flash_settler.read() {
           Some(flash_settler)  =>  transfer(
            flash_settler,
            base_asset_id,
            amount_with_fee,
            ),
            None =>  {
                 ()
            }
        };
    }


    #[storage(read, write)]
    fn set_flash_settler(flash_settler: Identity) -> Option<Identity> {
        storage.flash_settler.write(Some(flash_settler));
        storage.flash_settler.read()
    }
}

#[storage(read, write)]
fn perform_yield_strategy_and_gain_profit(amount: u64) {

    /*
    This is the internal function that will use the flashloan to make profit for the user
    logic can be to preform arbitrage, flash swaps, liquidity an unhealthy position or to payback a loan
    where the collateral has risen far more than the token borrowed by the user.
    */
    
}
