   ![Flashsync (2)](https://github.com/user-attachments/assets/317b7c3d-7bde-4e1d-bdde-8c83cf63d831)
# Flash_Sync:
A Lending Platform powered by Yield aggregating Predicates. - Flash Loan Predicates.

**FlashSync** is a project designed to leverage Fuel’s new technology,**Predicates** to accrue yield for the vaults that power it, and also reduce the state burden on the nodes supporting the ecosystem.
Predicates are stateless smart contracts that significantly reduce the storage burden on the nodes supporting the Fuel system, making Fuel the fastest blockchain available. Flash_Sync is powered by the first-ever **Flash Loan Predicates** in the Fuel ecosystem, which holds funds and offers loans for various financial activities to borrowers for example:

- Arbitrage Trades
- Refinancing Loans
- Liquidating Borrowers
- Hacking Smart Contracts

These loans must be repaid within the same transaction along with a small fee. The listed functionalities are some of the most profitable and well-known methods of generating yield in the web3 space.

## How Flash_Sync Works

Flash_Sync allows users to deposit funds into a Flashvault. Once these funds accumulate, they are used to power(fund) The Flash Loan Predicates, maximizing liquidity utilization and generating yield for users through Flash_Sync vaults. The system is designed to ensure user funds are secure, and the vaults and the predicate cannot be rug-pulled due to the built-in protections within the flash loan predicates and the vault. This guarantees the safety of user funds as the top priority.

### Core Components of Flash_Sync:

1. **Flash Loan Predicates**: A new standard developed by me specifically for flash loans using Fuel’s predicates, this allows for stateless flash loans on the transaction level with maximum security.
2. **Flash Vaults**: Vaults that issue shares to users who deposit assets, these assets are aggregated and used to fund the predicates to  enable flash loans and generate yield, as for every flash loan taken from the predicate a fee is taken from the user.
3. **Flash Settler**: On-chain intermediary that ensures all state-based constraints for flash loans are met during transaction execution.
4. **FlashAbi**: An ABI that must be implemented in the contract of anyone seeking to use flash loan funds.
5. **Flash Script**: The script responsible for enabling the loan and securing the flash loan predicates, basically your transaction must make use of the flash script if you must use the flash loans issued out by the predicates.
6. **Flash Admin Script**: This script enables the admin to pull out funds from the predicate back to the vaults.

Each core component includes a detailed README for better understanding, available in their respective folders.

---

## Overview
![Flashsync (3)](https://github.com/user-attachments/assets/cb63c237-df29-4161-9eb2-f8eb637388ba)

Basically, users lend by depositing their assets into the vault, and those funds are taken to the flash loan predicates where borrowers can use those funds but repay with extra fees.

## System Workflow

1. **Investment Period**: Users deposit assets into the vault.
2. **Admin Action**: Admin pauses withdrawals, powers, and activates the flash loan process using the invested funds.
3. **Yielding Period**: Profits generated are transferred back to the vault.
4. **Profit Period**: Users can withdraw their funds along with profits.

This cycle repeats, ensuring consistent yield growth.

### Features for Different Users:

#### For Lenders (Users):
- Funds are safe, and admins cannot rug-pull (due to protective constraints).
- Users can deposit during the investment period and withdraw with profits during the profit period.
- Anyone can trigger yield for the vault to distribute profits.

#### For Borrowers:
- Borrow any amount but must repay with a fee within the same transaction.
- Implement the FlashAbi in the contract to receive loans.
- Utilize the Flash Script to ensure personal security and protect lenders' funds.

#### For Admins:
- Manage predicate settings.
- Transfer designated funds to predicates.
- Return funds to the vault.
- Ensure yield generation (any user can trigger yield).

---

This simple yet robust structure ensures Flash_Sync enables yield generation and powers the ecosystem for all participants. Future upgrades may include gasless transactions for users without ETH who still wish to withdraw funds from the vault.





### Setup

The tests are written in rust SDK for fuels toolchain, anyone looking to understand should have a good knowledge of rust, and sway.
To build the contracts cd into the directory
```bash
forc build
```
To run the tests 
```bash
cargo test
```
to run with the logs from the `println!()` macro.
```bash
cargo test -- --nocapture
```

ensure all the dependencies are in the `Cargo.toml` and `Forc toml` file to avoid errors in your terminal, also have the current latest toolchain(I used nightly if that would help you out in any way).

### Tests

For the vaults check the `flash_vault` folder, and the `README.md` file to understand more of how it works.
For the core Flash_vaults tests, I ensured to test most of the workflows to ensure every user interaction is safe you can see the test outputs from the terminal below:

```bash
running 7 tests
test test_user_cannot_steal_ownership - should panic ... ok
test test_deposit ... ok
test test_withdraw ... ok
test test_cannot_withdraw_when_paused - should panic ... ok
test test_set_and_transfer_to_flash_predicate ... ok
test test_users_profit_when_yield_accrues ... ok
test test_authority_can_pause_and_unpause ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50
```

For tests involving the core Flash_Loan_Predicates tests, the test for the flash loans and to prove admin can pull funds back to the vault using the flash scripts is in the integrations tests:
Results to show the flash loan predicate actually works yaaaayyyy !!!: 
```bash
running 4 tests
test test_admin_cannot_rugpull_the_flashloan_predicate - should panic ... ok
test test_send_funds_to_flash_predicate_from_flashvault ... ok
test test_admin_can_refund_flashvault_back_funds_in_flashlaon_predicate ... ok
test test_flash_loans_can_be_taken_from_the_flashloan_predicate ... ok

```
