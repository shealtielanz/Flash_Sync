# **FlashLoanPredicate**
A Novel Flash Loan Standard for Predicates

The **FlashLoanPredicate** is a groundbreaking new standard that utilizes Fuel’s predicates for flash loans. It holds the loanable funds and allows users to borrow them under the strict condition that the borrowed amount is repaid within the same transaction, including a fee.

![Flash Loan Predicate](https://github.com/user-attachments/assets/e82c9a1a-2e6b-48f3-a030-d31e7082fe7a)




### Core Constraints:
1. **FlashScripts for Security**: The predicate must be used with the designated *FlashScripts* to ensure the security of the funds held.
2. **Repayment During the Same Transaction**: Borrowers must repay the borrowed amount during the transaction, facilitated by an intermediary known as the *Flash Settler*.
3. **Restricted Script Types**: The predicate only accepts two types of scripts:
   - *FlashLoan Script*
   - *Flash Admin Script*

### Core Logic:
1. **Admin Script Validation**: 
   - The predicate verifies if the script is an *Admin Script*. If true, it checks the admin’s signature and ensures funds are sent only to the designated vaults. Any attempt to redirect funds to other addresses is blocked.
   
2. **FlashLoan Script Validation**:
   - If the script is a *FlashLoan Script*, the predicate verifies that funds are handled exclusively through the *Flash Settler*, preventing unauthorized transfers.

### Core Functions:
- `verify_transaction_is_safe() -> bool`: Verifies if the transaction is safe and valid, to avoid siphoning the funds inside the flash loan predicate.
- `verifySigners() -> u64`: Checks and returns the authorized user's signature.
- `main() -> bool`: Main function to execute the core logic and ensure all conditions are met before proceeding with the loan.

This structure ensures both security and efficiency in handling flash loans, protecting funds while facilitating rapid borrowing and repayment within a single transaction.

### Flash Loan Flow
- Flash loans cannot be used by EOAs so anyone intending to use this fund should adapt a smart contract.
- The adapted smart contract used to take funds, must implement the FlashAbi Interface, mainly the `use_flash_loan()` function.
- The transaction to use the flash loan must include the flash_script, this script is enforced in the flash predicate to ensure that funds are forwarded correctly.
- The flash script forwards the required amount of funds(specified as input to the script) to the flash settler, after which the flash settler calls `use_flash_loan()` on the user's contract to trigger the usage of the loan.
- The contract can then use the flash loan for anything within its logic, after which it must return the funds to the flash settler, the user doesn't need to forward directly to the predicate as this is handled by the settler.
- After the funds with the fee are sent back to the settler, the settler validates the amount with the fee and ensures all is well, after which it sends back the funds back to the predicate.
- After that the transaction is over and the predicate and the borrower have made their gain.

### Validation enforced by the Flash Predicate
- The predicate ensures that no UTXO is created after the transaction by validation against the transaction outputs.
- The predicate ensures that the specified `output::change()` receiver is the predicate it self.
- The predicate validates the script to ensure against unsafe transactions that way the funds can never be stolen.
- If the script is a flash loan script, the transaction must be a safe one with no `output::coins` and the change must always be the predicates.
- If the script is of the admin, it ensures that the transaction was signed by the signers and also validates that the transaction is safe to ensure against a rug-pull, or centralization where the admin might want to steal the funds in the predicate,.


