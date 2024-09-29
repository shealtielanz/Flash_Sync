### Flash Admin Script

The **Flash Admin Script** is a specialized script that can only be executed by the admin. Its bytecode hash is stored in the predicate, ensuring the script’s authenticity. When executed, the predicate verifies the signatures and ensures that only the vault receives the transferred funds.

### Key Features:
- **Admin-Only Access**: Only the admin, or authorized signers, can use this script.
- **Signature Verification**: The predicate checks that the required number of valid signatures is provided.
- **Funds Transfer**: Ensures that funds are transferred directly to the vault, preventing any unauthorized transfers.

### Core Function: `main()`

The `main()` function in the Flash Admin Script has one key responsibility: transferring funds to the vault.

#### Input:
- **Amount**: The amount of funds to be transferred to the vault.

### Workflow:
1. **Verify Signatures**: The function ensures the transaction is signed by the required number of valid signers.
2. **Transfer Funds**: After signature verification, it transfers the specified amount directly to the vault.

This setup ensures secure fund management and guarantees that only authorized admin actions can impact the vault’s funds.
