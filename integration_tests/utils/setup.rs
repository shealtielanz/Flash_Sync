use abigen_bindings::{flash_abi_impl_mod, flash_loan_predicate_mod};
use fuels::{
    accounts::provider,
    prelude::*,
    types::*,
};
use rand::Rng;
use sha2::{Digest, Sha256};

// Load abis from json
abigen!(
    Predicate(
        name = "FlashLoanPredicate",
        abi = "./flash_loan_predicate/out/debug/flash_loan_predicate-abi.json"
    ),
    Contract(
        name = "FlashVault", 
        abi = "./flash_vault/out/debug/flash_vault-abi.json"),
    Contract(
        name = "FlashSettler",
        abi = "./flash_settler/out/debug/flash_settler-abi.json"
    ),
    Contract(
        name = "FlashAbiImpl",
        abi = "./flash_abi_impl/out/debug/flash_abi_impl-abi.json"
    ),
    Script(
        name = "FlashScript",
        abi = "./flash_script/out/debug/flash_script-abi.json"
    ),
    Script(
        name = "FlashAdminScript",
        abi = "./flash_admin_script/out/debug/flash_admin_script-abi.json"
    ),
);




pub async fn get_wallets() -> Vec<WalletUnlocked> {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(3),             /* deployer, user, and other */
            Some(1),             /* Single coin (UTXO) */
            Some(10_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    wallets
}




pub async fn get_flash_vault_instance(wallet: WalletUnlocked) -> (FlashVault<WalletUnlocked>, ContractId,) {
     
    let id = Contract::load_from(
        "./flash_vault/out/debug/flash_vault.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();
    
    let owner = Identity::Address(Address::from(wallet.clone().address()));

    let instance = FlashVault::new(id.clone(), wallet);
    instance
            .methods()
            .my_constructor(owner)
            .call()
            .await.unwrap();

    (instance, id.into())
}


pub async fn get_flash_settler_instance(wallet: WalletUnlocked) -> (FlashSettler<WalletUnlocked>, ContractId,) {
     
    let id = Contract::load_from(
        "./flash_settler/out/debug/flash_settler.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = FlashSettler::new(id.clone(), wallet);

    (instance, id.into())
}


pub async fn get_flash_abi_impl_instance(wallet: WalletUnlocked, flash_settler: Identity) -> (FlashAbiImpl<WalletUnlocked>, ContractId,) {
     
    let id = Contract::load_from(
        "./flash_abi_impl/out/debug/flash_abi_impl.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = FlashAbiImpl::new(id.clone(), wallet);

    instance
            .methods()
            .set_flash_settler(flash_settler)
            .call()
            .await.unwrap();

    (instance, id.into())
}



pub async fn get_flash_script<T: Account>(
    base_asset: AssetId,
    account: T,
    amount: u64,
    flash_abi_impl_id: ContractId,
    flash_settler: Bits256
) -> (FlashScript<T>, Bits256) {
    let configurables = FlashScriptConfigurables::default()
        .with_FLASHLOAN_SETTLER(flash_settler).unwrap()
        .with_LOAN_ASSET_ID(base_asset).unwrap();

    let script = FlashScript::new(account.clone(), "./flash_script/out/debug/flash_script.bin")
        .with_configurables(configurables);
    
    let mut hasher = Sha256::new();
    hasher.update(
        script
            .main(flash_abi_impl_id, amount.into())
            .call
            .script_binary,
    );
    let b256 = Bits256(hasher.finalize().into());

    (script, b256)
}


pub async fn get_flash_admin_script<T: Account>(
    account: T,
    amount: u64,
    flash_vault: ContractId
) -> (FlashAdminScript<T>, Bits256) {
    let provider = account.try_provider().unwrap();
    let binding = provider.clone();
    let base_asset_id = binding.base_asset_id();
    let configurables = FlashAdminScriptConfigurables::default()
        .with_VAULT_CONTRACT_ID(flash_vault).unwrap()
        .with_LOAN_ASSET_ID(*base_asset_id).unwrap();

    let script = FlashAdminScript::new(account.clone(), "./flash_admin_script/out/debug/flash_admin_script.bin")
        .with_configurables(configurables);
    
    let mut hasher = Sha256::new();
    hasher.update(
        script
            .main(amount.into())
            .call
            .script_binary,
    );
    let b256 = Bits256(hasher.finalize().into());

    (script, b256)
}

pub async fn get_flash_loan_predicate<T: Account>(
    admin_script_hash: Bits256,
    script_hash: Bits256,
    signers: Vec<T>,
    flash_settler_contract_id: ContractId,
    flash_vault_contract_id: ContractId,
    provider: &Provider,
) -> Predicate {
    let provider_lifetime = provider.clone();
    let base_asset_id = provider_lifetime.base_asset_id();
    let required_signers: u64 = 1;
    let signers: [Address; 3] = [
        signers[0].clone().address().into(),
        signers[1].clone().address().into(),
        signers[2].clone().address().into(),   
    ];
    // println!("b256 address: {:?}", flash_vault_contract_id.clone());
    // get the configurable arguments. 
   // Convert ContractId to Bits256
let flash_settler_address = Bits256(*flash_settler_contract_id);
let vault_address = Bits256(*flash_vault_contract_id);
//  println!("b256 address: {:?}", vault_address.clone());
    // get the configurable arguments. 
let configurables = FlashLoanPredicateConfigurables::default()
                                     .with_LOAN_ASSET(*base_asset_id).unwrap()
                                     .with_SIGNERS(signers).unwrap()
                                     .with_REQUIRED_SIGNATURES(required_signers.into()).unwrap()
                                     .with_EXPECTED_FLASH_LOAN_SCRIPT_BYTECODE_HASH(script_hash).unwrap()
                                     .with_EXPECTED_FLASH_ADMIN_SCRIPT_BYTECODE_HASH(admin_script_hash).unwrap()
                                     .with_FLASH_SETTLER_ADDRESS(flash_settler_address).unwrap()
                                     .with_VAULT_ADDRESS(vault_address).unwrap();

     let bin_path = "./flash_loan_predicate/out/debug/flash_loan_predicate.bin";

    let _predicate = Predicate::load_from(bin_path).unwrap()
                    .with_provider(provider.clone())
                    .with_configurables(configurables);

    _predicate

}


pub async fn  setup() -> FlashSync<WalletUnlocked> {
    let wallets = get_wallets().await;
    let admin = wallets[0].clone();
    let user = wallets[1].clone();
    let borrower = wallets[2].clone();

    let (
        flash_vault_instance, 
        fvt_contract_id
    )  = get_flash_vault_instance(admin.clone()).await;
    let (
        flash_settler_instance, 
        flash_settler_contract_id
    ) = get_flash_settler_instance(admin.clone()).await;

   let flash_settler_id = Identity::ContractId(flash_settler_contract_id.clone());
    let (
        flash_abi_impl_instance,
        flash_impl_contract_id
    ) = get_flash_abi_impl_instance(user.clone(), flash_settler_id).await;

    let amount_to_borrow: u64 = 900_000;
    let binding = user.provider().unwrap();
    let base_asset_id = binding.base_asset_id();
    let flash_settler_b256 = Bits256(*flash_settler_contract_id);
    let (flash_script, script_hash) = 
                get_flash_script(
                    *base_asset_id,
                    borrower, 
                    amount_to_borrow, 
                    flash_impl_contract_id, 
                    flash_settler_b256
                ).await;
    let amt_to_refund_vault = 1_000_000;

    let (flash_admin_script, admin_script_hash) = 
                        get_flash_admin_script(
                            admin.clone(), 
                            amt_to_refund_vault, 
                            fvt_contract_id.clone()
                        ).await;



let user_dup = user.clone();
let provider = user_dup.provider().unwrap();
let flash_loan_predicate = 
get_flash_loan_predicate(
    admin_script_hash,
    script_hash, wallets.clone(), 
    flash_settler_contract_id.clone(), 
    fvt_contract_id.clone(), 
    provider
).await;

FlashSync {
    wallets: wallets.clone(),
    vault_admin: admin.clone(),
    user: user.clone(),
    flashloan_user: user,
    flash_vault_instance: flash_vault_instance,
    flash_settler_instance: flash_settler_instance,
    flash_vault_contract_id: fvt_contract_id,
    flash_abi_impl: flash_abi_impl_instance,
    flash_script: flash_script,
    flash_admin_script: flash_admin_script,
    flash_loan_predicate: flash_loan_predicate,

}

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
#[derive(Debug)]
pub struct FlashSync<T: Account> {
    pub wallets: Vec<WalletUnlocked>,
    pub vault_admin: WalletUnlocked,
    pub user: WalletUnlocked,
    pub flashloan_user: WalletUnlocked,
    pub flash_vault_instance: FlashVault<WalletUnlocked>,
    pub flash_settler_instance: FlashSettler<WalletUnlocked>,
    pub flash_vault_contract_id: ContractId,
    pub flash_abi_impl: FlashAbiImpl<WalletUnlocked>,
    pub flash_script: FlashScript<T>,
    pub flash_admin_script: FlashAdminScript<T>,
    pub flash_loan_predicate: Predicate,
}
