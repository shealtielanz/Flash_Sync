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
- `verifySigners() -> u64`: Checks and returns the signature of the authorized user.
- `main() -> bool`: Main function to execute the core logic and ensure all conditions are met before proceeding with the loan.

This structure ensures both security and efficiency in handling flash loans, protecting funds while facilitating rapid borrowing and repayment within a single transaction.
