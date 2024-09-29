script;
use flash_abi::FlashSettler;

use std::{
    asset::transfer,
    constants::ZERO_B256
};
/*
[Author = @shealtielanz].
[Description = 
This is the flash_script which must be included in the transaction of anyone intending to take a loan from the `flash_loan predicate`,
A simple script that excutes the `flash_settlers.trigger_flashloan()` which sends and calls the the borrower's contract that implemnts the `FlashAbi` 
inorder to make use of whatever amoun the wish to borrow.
]
*/

// the `flash_settler` raw b256 to enable a swift call to the flash_settler's functions when combined with the required abi 
configurable {
    FLASHLOAN_SETTLER:  b256 = 0x3333333333333333333333333333333333333333333333333333333333333333,
    LOAN_ASSET_ID: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
}


/* the `main()` function of the script, which takes the input of the user's contractId and the amount the wish to borrow
The implementation here ensures that the correct amount of fee is charged from the amount the user wishes to borrow
*/
fn main(flash_user: ContractId, amount: u64)  {
    // transfer funds to the flash settler before we trigger the flashloan.
    transfer(Identity::ContractId(ContractId::from(FLASHLOAN_SETTLER)), LOAN_ASSET_ID, amount);
    let flash_loan_settler = abi(FlashSettler, FLASHLOAN_SETTLER);
    flash_loan_settler.trigger_flashloan(amount, flash_user);
}
