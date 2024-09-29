library;

abi FlashAbi {
    /*
    every user who intends to make use of this flashloan must 
    implement this `abi` in their contract to be able to make use of the loans.
    */
    #[payable]
    #[storage(read, write)]
    fn use_flashloan(amount: u64);
    /*
    used for setting the flash settler that the funds will be paid back too
    */
    #[storage(read, write)]
    fn set_flash_settler(flash_settler: Identity) -> Option<Identity>;
}

abi FlashSettler{
    // trigger _flashloan() to be called by the script to trigger the flashloan

    #[storage(read, write)]
    fn trigger_flashloan(amount: u64, flash_user: ContractId);

    #[storage(read, write)]
    fn set_flash_loan_predicate(_predicate: Identity);

    #[storage(read)]
    fn send_dusts_to_predicate();

    #[storage(read)]
    fn get_predicate_id() -> Option<Identity>;

}

pub fn calculate_five_percent(amount: u64) -> u64 {
   // Calculate 5% of the amount
    let result = amount / 20;
    result
}
