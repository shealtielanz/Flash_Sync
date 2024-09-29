use fuels::{
    accounts::provider,
    prelude::*,
    types::{ContractId, *},
};
use rand::Rng;
use sha2::{Digest, Sha256};

// Load abi from json
abigen!(Contract(
    name = "FlashVault",
    abi = "out/debug/flash_vault-abi.json"
));

pub async fn get_instance_wallets_contract_id_sub_id_asset_ids() -> (
    FlashVault<WalletUnlocked>,
    ContractId,
    Vec<WalletUnlocked>,
    Bits256,
    AssetId,
    AssetId,
) {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),              // Single wallet
            Some(1),              // Single coin (UTXO)
            Some(10_000_000_000), // Amount per coin
        ),
        None,
        None,
    )
    .await
    .unwrap();

    let wallet = wallets[0].clone();

    // Load and deploy contract
    let id = Contract::load_from("./out/debug/flash_vault.bin", LoadConfiguration::default())
        .unwrap()
        .deploy(&wallet, TxPolicies::default())
        .await
        .unwrap();

    let instance = FlashVault::new(id.clone(), wallet);

    // Set constructor
    let owner_id: Identity = Identity::Address(Address::from(wallets[0].clone().address()));
    instance
        .methods()
        .my_constructor(owner_id)
        .call()
        .await
        .unwrap();

    // Get asset IDs and sub ID
    let vault_asset_id = get_default_asset_id(id.clone().into());
    let binding = wallets[0].clone();
    let base_asset_id = binding.provider().unwrap().base_asset_id();
    let default_sub_id = Bits256([0; 32]);

    (
        instance,
        id.into(),
        wallets,
        default_sub_id,
        *base_asset_id,
        vault_asset_id,
    )
}

/// Computes the AssetId by hashing the contract ID and sub ID together.
pub fn get_asset_id(sub_id: Bytes32, contract: ContractId) -> AssetId {
    let mut hasher = Sha256::new();
    hasher.update(*contract);
    hasher.update(*sub_id);

    AssetId::new(*Bytes32::from(<[u8; 32]>::from(hasher.finalize())))
}

/// Computes the default AssetId using a sub ID of all zeros.
pub fn get_default_asset_id(contract: ContractId) -> AssetId {
    let default_sub_id = Bytes32::from([0u8; 32]);

    get_asset_id(default_sub_id, contract)
}
