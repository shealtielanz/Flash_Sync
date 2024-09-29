### FlashLoan Script

![Flash script](https://github.com/user-attachments/assets/4ae652f9-7c4a-4901-a0ba-d7a932f90d84)


The **FlashLoan Script** is a simple yet essential component in the flash loan process. Its primary function is to facilitate the transfer of funds from the predicate to the *Flash Settler*, which then initiates the loan to the user. 

### Key Features:
- **Triggers Loan Disbursement**: The script sends the borrowed funds to the *Flash Settler*, which ensures proper handling of the loan.
- **Mandatory in Every Flash Loan**: Every flash loan must include the FlashLoan Script, as its bytecode hash is stored in the flash loan predicate and is checked during the TX. This serves as a validation step to ensure only approved transactions can interact with the predicate.

The inclusion of the FlashLoan Script guarantees that loans are executed securely and that all actions are traceable and governed by the smart contract's logic.
