script;

use std::{
    asset::transfer,
    constants::ZERO_B256
};

configurable {
    VAULT_CONTRACT_ID: ContractId = ContractId::from(ZERO_B256),
    LOAN_ASSET_ID: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
}

fn main(amount: u64) {
    // transfer funds to the vault.
    transfer(Identity::ContractId(VAULT_CONTRACT_ID), LOAN_ASSET_ID, amount);
}