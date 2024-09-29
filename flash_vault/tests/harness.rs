mod utils;

use crate::setup::{get_instance_wallets_contract_id_sub_id_asset_ids, FlashVault};
use fuels::{
    accounts::{predicate, provider},
    prelude::*,
    types::{ContractId, *},
};
use setup::abigen_bindings::flash_vault_mod::sway_libs::ownership;
use utils::*;

#[tokio::test]
async fn test_deposit() {
    let (_vault, _id, wallets, default_sub_id, base_asset_id, vault_asset_id) =
        get_instance_wallets_contract_id_sub_id_asset_ids().await;

    // Check user balance before deposit
    let user = wallets[1].clone();
    let user_bal = user.get_asset_balance(&vault_asset_id).await.unwrap();
    assert_eq!(user_bal, 0);
    println!(
        "User balance of FVT tokens(vault shares) before deposit: {:?}",
        user_bal
    );

    // Create the call params for the deposit call
    let amount_to_deposit = 5_000_000_000;
    let call_params = CallParameters::default()
        .with_asset_id((*base_asset_id).into())
        .with_amount(amount_to_deposit);
    let recipient = Identity::Address(Address::from(wallets[1].clone().address()));

    // Call deposit
    _vault
        .clone()
        .with_account(user.clone())
        .methods()
        .deposit(recipient, default_sub_id)
        .call_params(call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    // Check total supply after deposit
    let res = _vault
        .methods()
        .total_supply(vault_asset_id)
        .call()
        .await
        .unwrap();
    println!("Total supply after deposit: {:?}", res.value.unwrap());

    let user_bal = user.get_asset_balance(&vault_asset_id).await.unwrap();
    assert_eq!(user_bal, amount_to_deposit);
    println!(
        "User balance of FVT tokens(vault shares) after deposit: {:?}",
        user_bal
    );
}

#[tokio::test]
async fn test_withdraw() {
    let (_vault, _id, wallets, default_sub_id, base_asset_id, vault_asset_id) =
        get_instance_wallets_contract_id_sub_id_asset_ids().await;

    // Check user balance before deposit
    let user = wallets[1].clone();
    let user_bal = user.get_asset_balance(&vault_asset_id).await.unwrap();
    assert_eq!(user_bal, 0);
    println!(
        "User balance of FVT tokens(vault shares) before deposit: {:?}",
        user_bal
    );

    // Create the call params for the deposit call
    let amount_to_deposit = 5_000_000_000;
    let call_params = CallParameters::default()
        .with_asset_id((*base_asset_id).into())
        .with_amount(amount_to_deposit);
    let recipient = Identity::Address(Address::from(wallets[1].clone().address()));

    // Call deposit
    _vault
        .clone()
        .with_account(user.clone())
        .methods()
        .deposit(recipient.clone(), default_sub_id)
        .call_params(call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    // log user balance of FVT tokens
    let user_bal = user.get_asset_balance(&vault_asset_id).await.unwrap();
    assert_eq!(user_bal, amount_to_deposit);
    println!(
        "User balance of FVT tokens(vault shares) before withdraw: {:?}",
        user_bal
    );

    // log user balance of base_asset_tokens tokens
    let user_bal = user.get_asset_balance(&base_asset_id).await.unwrap();
    assert_eq!(user_bal, amount_to_deposit - 1);
    println!("User balance of Base asset before withdraw: {:?}", user_bal);
    // test that the user can withdraw their assets back
    let amt_to_withdraw = 2_500_000_000;
    let withdraw_call_params = CallParameters::default()
        .with_asset_id(vault_asset_id)
        .with_amount(amt_to_withdraw);

    _vault
        .with_account(user.clone())
        .methods()
        .withdraw(recipient.clone(), base_asset_id, default_sub_id)
        .call_params(withdraw_call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    // log user balance of FVT tokens
    let user_bal = user.get_asset_balance(&vault_asset_id).await.unwrap();

    println!(
        "User balance of FVT tokens(vault shares) after withdraw: {:?}",
        user_bal
    );

    // log user balance of base_asset_tokens tokens
    let user_bal = user.get_asset_balance(&base_asset_id).await.unwrap();

    println!("User balance of Base asset after withdraw: {:?}", user_bal);
}

#[tokio::test]
async fn test_authority_can_pause_and_unpause() {
    let (_vault, _, wallets, _, _, _) = get_instance_wallets_contract_id_sub_id_asset_ids().await;

    // Call pause method and check if the contract is paused
    _vault.methods().pause().call().await.unwrap();

    // Check if the contract is paused
    let res = _vault.methods().is_paused().call().await.unwrap();
    let is_paused = res.value;

    println!("the contract is paused: {:?}", is_paused);

    // Call unpause method
    _vault.methods().unpause().call().await.unwrap();

    // Check if the contract is unpaused
    let res2 = _vault.methods().is_paused().call().await.unwrap();
    let is_paused = res2.value;

    println!("the contract is pause should be false: {:?}", is_paused);
}

// commented out cause it makes the logs to noisy
#[tokio::test]
#[should_panic(expected = "CannotReinitialized")]
async fn test_user_cannot_steal_ownership() {
    let (_vault, _, wallets, _, _, _) = get_instance_wallets_contract_id_sub_id_asset_ids().await;

    // try to reintialize the constructor
    let owner_id: Identity = Identity::Address(Address::from(wallets[1].clone().address()));
    _vault
        .with_account(wallets[1].clone())
        .methods()
        .my_constructor(owner_id)
        .call()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_set_and_transfer_to_flash_predicate() {
    let (_vault, _, wallets, default_sub_id, base_asset_id, _) =
        get_instance_wallets_contract_id_sub_id_asset_ids().await;

    let flash_predicate = wallets[1].clone();
    // set the predicate
    let flash_predicate_id = Identity::Address(Address::from(flash_predicate.address()));
    _vault
        .methods()
        .set_flash_loan_predicate(flash_predicate_id)
        .call()
        .await
        .unwrap();

    let res = _vault.methods().get_flash_predicate().call().await.unwrap();
    println!("the flash loan predicate address is: {:?}", res.value);

    // Create the call params for the deposit call
    let amount_to_deposit = 5_000_000_000;
    let call_params = CallParameters::default()
        .with_asset_id((*base_asset_id).into())
        .with_amount(amount_to_deposit);
    let recipient = Identity::Address(Address::from(wallets[0].clone().address()));

    // Call deposit
    _vault
        .methods()
        .deposit(recipient.clone(), default_sub_id)
        .call_params(call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();
    let predicate_bal = flash_predicate
        .get_asset_balance(&base_asset_id)
        .await
        .unwrap();
    // assert that it is same as when it was set up
    assert!(predicate_bal == 10_000_000_000);

    let amt_to_send = 3_000_000_000;

    // transfer funds to predicate
    _vault
        .methods()
        .transfer_to_predicate(amt_to_send)
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();
    let predicate_bal = flash_predicate
        .get_asset_balance(&base_asset_id)
        .await
        .unwrap();
    // assert that it is same as when it was set up
    assert!(predicate_bal > 10_000_000_000);
    println!(
        "validate the predicate has 13_000_000_00: {:?}",
        predicate_bal
    );
}

#[tokio::test]
async fn test_users_profit_when_yield_accrues() {
    let (_vault, _id, wallets, default_sub_id, base_asset_id, vault_asset_id) =
        get_instance_wallets_contract_id_sub_id_asset_ids().await;
    let admin = wallets[0].clone();
    let user = wallets[1].clone();

    let user_vault_instance = FlashVault::new(_id.clone(), user.clone());
    let bal_before_investing = user.get_asset_balance(&base_asset_id).await.unwrap();
    println!(
        "User's balance before investing in the vault: {:?}",
        bal_before_investing
    );

    // user deposits to gain shares
    let amount_to_deposit = 900_000;
    let call_params = CallParameters::default()
        .with_asset_id((*base_asset_id).into())
        .with_amount(amount_to_deposit);
    let recipient = Identity::Address(Address::from(user.clone().address()));

    // Call deposit
    user_vault_instance
        .methods()
        .deposit(recipient.clone(), default_sub_id)
        .call_params(call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    // transfer yield to the contract
    let bech32_contract_id = Bech32ContractId::from(_id);

    let _ = admin
        .force_transfer_to_contract(
            &bech32_contract_id,
            500_000,
            base_asset_id,
            TxPolicies::default(),
        )
        .await;

    let bal_while_investing = user
        .clone()
        .get_asset_balance(&base_asset_id)
        .await
        .unwrap();
    println!(
        "User's balance during investing in the vault: {:?}",
        bal_while_investing
    );
    // initiate withdraw
    let amt_to_withdraw = 900_000;
    let withdraw_call_params = CallParameters::default()
        .with_asset_id(vault_asset_id)
        .with_amount(amt_to_withdraw);

    // first call `accrue_yield()`
    user_vault_instance
        .methods()
        .accrue_yield()
        .call()
        .await
        .unwrap();
    // call withdraw
    user_vault_instance
        .methods()
        .withdraw(recipient.clone(), base_asset_id, default_sub_id)
        .call_params(withdraw_call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

    let bal_after_investing = user.get_asset_balance(&base_asset_id).await.unwrap();
    println!(
        "User's balance after investing in the vault: {:?}",
        bal_after_investing
    );
}

#[should_panic]
#[tokio::test]
async fn test_cannot_withdraw_when_paused() {
    let (_vault_instance_admin, _id, wallets, default_sub_id, base_asset_id, vault_asset_id) =
    get_instance_wallets_contract_id_sub_id_asset_ids().await;

    let user = wallets[1].clone();

    let user_vault_instance = FlashVault::new(_id.clone(), user.clone());


    // user deposits to gain shares
    let amount_to_deposit = 900_000;
    let call_params = CallParameters::default()
        .with_asset_id((*base_asset_id).into())
        .with_amount(amount_to_deposit);
    let recipient = Identity::Address(Address::from(user.clone().address()));

    // Call deposit for user to deposit into the vault
    user_vault_instance
        .methods()
        .deposit(recipient.clone(), default_sub_id)
        .call_params(call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

   // simulate admin pauses the vault to disable withdrawals.
   let user_bal = user.clone().get_asset_balance(&vault_asset_id).await.unwrap();
   println!("balance of shares user has in the vault: {:?}", user_bal);

    _vault_instance_admin.methods().pause().call().await.unwrap();


    let amt_to_withdraw = 900_000;
    let withdraw_call_params = CallParameters::default()
        .with_asset_id(vault_asset_id)
        .with_amount(amt_to_withdraw);
    // let user try and withdraw funds from the vault -PS should panic.
    user_vault_instance
        .methods()
        .withdraw(recipient.clone(), base_asset_id, default_sub_id)
        .call_params(withdraw_call_params)
        .unwrap()
        .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
        .call()
        .await
        .unwrap();

}


// TESTS RESULTS
/*

running 7 tests
test test_user_cannot_steal_ownership - should panic ... ok
test test_deposit ... ok
test test_withdraw ... ok
test test_cannot_withdraw_when_paused - should panic ... ok
test test_set_and_transfer_to_flash_predicate ... ok
test test_users_profit_when_yield_accrues ... ok
test test_authority_can_pause_and_unpause ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s
 */