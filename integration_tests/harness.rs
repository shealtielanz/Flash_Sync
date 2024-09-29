mod utils;
use fuels::{
    prelude::*, 
    tx::{TxPointer, UtxoId},
    types::{
        input::Input, output::Output, transaction_builders::{
            BuildableTransaction, ScriptTransactionBuilder, TransactionBuilder,
        }, Bits256, Bytes32, ContractId, Identity
    },
};
use utils::setup::abigen_bindings::{flash_loan_predicate_mod, flash_settler_mod};



use crate::utils::setup::{setup, get_default_asset_id, FlashVault};


#[tokio::test]
async fn test_admin_can_refund_flashvault_back_funds_in_flashlaon_predicate() {
    let flash_sync = setup().await;

    let wallets = &flash_sync.wallets;
    let admin = wallets[0].clone();
    let binding = admin.clone();
    let provider = binding.provider().unwrap();
    let base_asset_id = provider.base_asset_id();

    let flash_vault_id = &flash_sync.flash_vault_contract_id;
    let flash_admin_script = flash_sync.flash_admin_script.clone();


    // get a wallet to manually fund the predicate

    let funder = wallets[2].clone();
    let amt_to_forward: u64 = 1_500_000;

    let flash_loan_predicate = &flash_sync.flash_loan_predicate;

    funder
         .transfer(
            flash_loan_predicate.clone().address().into(), 
            amt_to_forward, 
            *base_asset_id, 
            TxPolicies::default()
        ).await.unwrap();
    let bal_predicate = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();

    assert_eq!(bal_predicate, amt_to_forward);

    let bal_vault = provider.get_contract_asset_balance(&flash_vault_id.clone().into(), *base_asset_id).await.unwrap();
    assert!(bal_vault == 0);

    // test that admin can forward balance in the predicate to the vault via the flash_admin_script

    let mut tb: ScriptTransactionBuilder = {
        let mut inputs = vec![
            Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: flash_vault_id.clone(),
        }];

        let input_utxo = flash_loan_predicate.get_asset_inputs_for_amount(*base_asset_id, 5000, None).await.unwrap();

        inputs.extend(input_utxo);

        let outputs = vec![
            Output::Variable {
                to: Address::default(),
                amount: 1u64,
                asset_id: AssetId::default(),
            },
            Output::contract(0, Bytes32::zeroed(), Bytes32::zeroed()),
            Output::Change{
                to: flash_loan_predicate.clone().address().into(),
                amount: 0u64,
                asset_id: *base_asset_id,
            }

         ];

        let tx_policy = TxPolicies::default().with_max_fee(2);
        let amount_to_vault: u64 = 1_000_000;

        ScriptTransactionBuilder::prepare_transfer(
            inputs,
            outputs,
            tx_policy
        )
        .with_script(
            flash_admin_script
                 .main(amount_to_vault.into())
                 .call
                 .script_binary,
        )
        .with_script_data(
              flash_admin_script
                  .main(amount_to_vault.into())
                  .call
                  .encoded_args
                  .unwrap(),
        )

    };
    let amount_to_vault: u64 = 1_000_000;

    tb.add_signer(admin.clone());

    admin.adjust_for_fee(&mut tb, amount_to_vault).await.unwrap();

    let tx = tb.build(
        provider.clone()
        ).await.unwrap();

    provider.send_transaction_and_await_commit(tx)
                                       .await.unwrap();

    let new_bal = provider.get_contract_asset_balance(&flash_vault_id.clone().into(), *base_asset_id).await.unwrap();
    assert!(new_bal == amount_to_vault);
    let predicate_balance = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();


    println!("new balance of contract: {:?}", new_bal);
    println!("new balance of predicate: {:?}", predicate_balance);


    }


#[tokio::test]
#[should_panic]
async fn test_admin_cannot_rugpull_the_flashloan_predicate() {

    let flash_sync = setup().await;

    let wallets = &flash_sync.wallets;
    let admin = wallets[0].clone();
    let binding = admin.clone();
    let provider = binding.provider().unwrap();
    let base_asset_id = provider.base_asset_id();

    let flash_vault_id = &flash_sync.flash_vault_contract_id;
    let flash_admin_script = flash_sync.flash_admin_script.clone();


    // get a wallet to manually fund the predicate

    let funder = wallets[2].clone();
    let amt_to_forward: u64 = 1_500_000;

    let flash_loan_predicate = &flash_sync.flash_loan_predicate;

    funder
         .transfer(
            flash_loan_predicate.clone().address().into(), 
            amt_to_forward, 
            *base_asset_id, 
            TxPolicies::default()
        ).await.unwrap();
    let bal_predicate = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();

    assert_eq!(bal_predicate, amt_to_forward);

    let bal_vault = provider.get_contract_asset_balance(&flash_vault_id.clone().into(), *base_asset_id).await.unwrap();
    assert!(bal_vault == 0);

    // test that admin can forward balance in the predicate to the vault via the flash_admin_script

    let mut tb: ScriptTransactionBuilder = {
        let mut inputs = vec![
            Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: flash_vault_id.clone(),
        }];

        let input_utxo = flash_loan_predicate.get_asset_inputs_for_amount(*base_asset_id, 5000, None).await.unwrap();

        inputs.extend(input_utxo);

        let outputs = vec![
            //if admin tries to siphone funds from the predicate it will fail.
            Output::Coin {
                 to: admin.clone().address().into(), 
                 amount: 1_000_000, 
                 asset_id: *base_asset_id },
            Output::Variable {
                to: Address::default(),
                amount: 1u64,
                asset_id: AssetId::default(),
            },
            Output::contract(0, Bytes32::zeroed(), Bytes32::zeroed()),
            Output::Change{
                to: flash_loan_predicate.clone().address().into(),
                amount: 0u64,
                asset_id: *base_asset_id,
            },

         ];

        let tx_policy = TxPolicies::default().with_max_fee(2);
        let amount_to_vault: u64 = 1_000_000;

        ScriptTransactionBuilder::prepare_transfer(
            inputs,
            outputs,
            tx_policy
        )
        .with_script(
            flash_admin_script
                 .main(amount_to_vault.into())
                 .call
                 .script_binary,
        )
        .with_script_data(
              flash_admin_script
                  .main(amount_to_vault.into())
                  .call
                  .encoded_args
                  .unwrap(),
        )

    };
    let amount_to_vault: u64 = 1_000_000;

    tb.add_signer(admin.clone());

    admin.adjust_for_fee(&mut tb, amount_to_vault).await.unwrap();

    let tx = tb.build(
        provider.clone()
        ).await.unwrap();

    provider.send_transaction_and_await_commit(tx)
                                       .await.unwrap();

    // let new_bal = provider.get_contract_asset_balance(&flash_vault_id.clone().into(), *base_asset_id).await.unwrap();
    // assert!(new_bal == amount_to_vault);
    // let predicate_balance = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();


    // println!("new balance of contract: {:?}", new_bal);
    // println!("new balance of predicate: {:?}", predicate_balance);



}

#[tokio::test]
async fn test_send_funds_to_flash_predicate_from_flashvault() {
    let flash_sync = setup().await;
    let flash_loan_predicate = flash_sync.flash_loan_predicate.clone();
    let flash_vault_admin_instance = &flash_sync.flash_vault_instance.clone();
    let wallets = flash_sync.wallets.clone();
    let user = wallets[1].clone();
    let admin = wallets[0].clone();
    let flash_vault_id = flash_sync.flash_vault_contract_id.clone();
    let binding = admin.clone();
    let provider = binding.provider().unwrap();
    let base_asset_id = provider.base_asset_id();
    // create an instance to allow user make call to the vault
    let user_instance = FlashVault::new(flash_vault_id.clone(), user.clone());

    let flash_predicate_id = Identity::Address(flash_loan_predicate.address().into());

    flash_vault_admin_instance
                  .methods()
                  .set_flash_loan_predicate(flash_predicate_id)
                  .call()
                  .await.unwrap();
    let res = flash_vault_admin_instance.methods().get_flash_predicate().call().await.unwrap();

    println!("predicate address: {:?}", res.value.unwrap());


    // user deposits 
    let amount: u64 = 1_000_000;
    let call_param = CallParameters::default()
                                                        .with_amount(amount)
                                                        .with_asset_id(*base_asset_id);
    let receiver = Identity::Address(user.clone().address().into());
     
    user_instance
                .methods()
                .deposit(receiver, Bits256::zeroed())
                .call_params(call_param).unwrap()
                .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                .call()
                .await.unwrap();

    let new_bal = provider.get_contract_asset_balance(&flash_vault_id.into(), *base_asset_id).await.unwrap();
    println!("new balance of vault after user deposits to it: {:?}", new_bal);

    assert!(new_bal == amount);

    // admin now sends funds to the flash_loan

    flash_vault_admin_instance
                            .methods()
                            .transfer_to_predicate(amount)
                            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                            .call()
                            .await.unwrap();
    let predicate_bal = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();

    assert!(predicate_bal == new_bal);

    println!("new balance of predicate after vault transfers to it: {:?}", predicate_bal);


}


#[tokio::test]
async fn test_flash_loans_can_be_taken_from_the_flashloan_predicate() {
    /*
    1) users need to deposit in the vault
    2) users funds need to be aggregated to the flashloan predicate
    3) funds in the flashloan predicate can be used for flashloans
    4) funds can be sent back to the vault 
    5) users will profit from the fee accrued by the flashloan predicate
    6) the flashloan AbiImpl is a dummy strategy contract that will be funded, it represents a yield strategy 
        where the borrower implements the flash_abi, and uses to do something profitable, then send funds back
        PS_ it will just have funds in it to pay the extra fee for flashloan, it just shows how to implement the 
        flash Abi and forward fees back to the lender(flashloan predicate through the flash settler)
     */

    let flash_sync = setup().await;
    let flash_loan_predicate = flash_sync.flash_loan_predicate.clone();
    let flash_vault_admin_instance = &flash_sync.flash_vault_instance.clone();
    let wallets = flash_sync.wallets.clone();
    let user = wallets[1].clone();
    let admin = wallets[0].clone();
    let flash_vault_id = flash_sync.flash_vault_contract_id.clone();
    let binding = admin.clone();
    let provider = binding.provider().unwrap();
    let base_asset_id = provider.base_asset_id();


    // get the flash abi impl 
    let flash_impl = flash_sync.flash_abi_impl;
    let flash_settler = flash_sync.flash_settler_instance;
    let flash_settler_id = Identity::ContractId(flash_settler.clone().id().into());
   // set the flash_settler and predicates in the contracts that need them for interaction.
    flash_impl
            .methods()
            .set_flash_settler(flash_settler_id)
            .call()
            .await.unwrap();
   let flash_predicate_id = Identity::Address(flash_loan_predicate.address().into());
   flash_settler
            .methods()
            .set_flash_loan_predicate(flash_predicate_id.clone())
            .call()
            .await.unwrap();

    flash_vault_admin_instance
                  .methods()
                  .set_flash_loan_predicate(flash_predicate_id.clone())
                  .call()
                  .await.unwrap();


    // create an instance to allow user make call to the vault
    let user_instance = FlashVault::new(flash_vault_id.clone(), user.clone());
    
    // let user deposit to get shares from the vault.
       // user deposits 
    let amount: u64 = 1_000_000;
    let call_param = CallParameters::default()
                                                        .with_amount(amount)
                                                        .with_asset_id(*base_asset_id);
    let receiver = Identity::Address(user.clone().address().into());
     
    user_instance
                .methods()
                .deposit(receiver, Bits256::zeroed())
                .call_params(call_param).unwrap()
                .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                .call()
                .await.unwrap();
    let new_bal = provider.get_contract_asset_balance(&flash_vault_id.into(), *base_asset_id).await.unwrap();
    assert!(new_bal == amount);
    let user_shares = user.get_asset_balance(base_asset_id).await.unwrap();
    println!("user share is: {:?}", user_shares);
    println!("amount of base asset deposited: {:?}", amount);
    // admin now sends funds to the flash_loan

    flash_vault_admin_instance
                            .methods()
                            .transfer_to_predicate(amount)
                            .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
                            .call()
                            .await.unwrap();
    let predicate_bal = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();
    println!("balance of predicate before the flash loan: {:?}", predicate_bal);
    assert!(predicate_bal == amount);
    let new_bal = provider.get_contract_asset_balance(&flash_vault_id.into(), *base_asset_id).await.unwrap();
    assert!(new_bal == 0);


    // now funds are in the flash_loan_predicate flash loan can be used 
    // fund the flash impl to simulate the profit strategy
    // simulation funds = 10% of 900_000
    let amount_to_borrow: u64 = 900_000; // have to be same with amount for script in setup.rs
    let simulation_funds: u64 = 90_000;
    let flash_impl_id: ContractId = flash_impl.clone().id().into();
    admin
        .force_transfer_to_contract(
            &flash_impl_id.into(), 
            simulation_funds, 
            *base_asset_id, 
            TxPolicies::default()
        ).await.unwrap();


    let borrower = wallets[2].clone();
    // borrower will user the flash abi impl for the profit strategy they intended to implement using th flash loan.
    let flash_script = flash_sync.flash_script;

    // create transaction 
    let mut tb: ScriptTransactionBuilder = {
        let mut inputs = vec![
            Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: flash_settler.clone().id().into(),
        }];

        let input1 = vec![
            Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: flash_impl_id.clone()
        }];

        let input_utxo = flash_loan_predicate.get_asset_inputs_for_amount(
                                                            *base_asset_id, 
                                                            amount_to_borrow.into(), 
                                                            None
                                                        ).await.unwrap();

        inputs.extend(input1);
        inputs.extend(input_utxo);

        let outputs = vec![
            Output::Variable {
                to: Address::default(),
                amount: 3u64, // set to 3 for max vaariable outputs that might happen even tho might increase gas fee
                asset_id: AssetId::default(),
            },
            Output::contract(0, Bytes32::zeroed(), Bytes32::zeroed()),
            Output::contract(1, Bytes32::zeroed(), Bytes32::zeroed()),
            Output::Change{
                to: flash_loan_predicate.clone().address().into(),
                amount: 0u64,
                asset_id: *base_asset_id,
            },

         ];

        let tx_policy = TxPolicies::default().with_max_fee(4);

        ScriptTransactionBuilder::prepare_transfer(
            inputs,
            outputs,
            tx_policy
        )
        .with_script(
            flash_script
                 .main(flash_impl_id.clone(), amount_to_borrow.into())
                 .call
                 .script_binary,
        )
        .with_script_data(
              flash_script
                  .main(flash_impl_id.clone(), amount_to_borrow.into())
                  .call
                  .encoded_args
                  .unwrap(),
        )

    };
    tb.add_signer(borrower.clone());

    let tx = tb.build(
        provider.clone()
        ).await.unwrap();

    provider.send_transaction_and_await_commit(tx)
                                       .await.unwrap();


    let predicate_new_bal = flash_loan_predicate.get_asset_balance(base_asset_id).await.unwrap();
    //assert that the yield was generated.
    assert!(predicate_new_bal > predicate_bal);
    println!("predicate balance after the loan was taken and reppaid: {:?}", predicate_new_bal);
    
}





    













































































































    // // let admin refunds the vault back the funds plus the profit
    // let amt_to_withdraw = predicate_new_bal - 1;
    // let withdraw_call_params = CallParameters::default()
    //                     .with_asset_id(vault_asset_id)
    //                     .with_amount(amt_to_withdraw);

    // flash_vault_admin_instance
    //                 .methods()
    //                 .transfer_to_predicate(amount)



    //                 let amt_to_withdraw = predicate_new_bal - 1;
    //                 let withdraw_call_params = CallParameters::default()
    //                                     .with_asset_id(vault_asset_id)
    //                                     .with_amount(amt_to_withdraw);
    //                    // first call `accrue_yield()`
    //                    user_vault_instance
    //                    .methods()
    //                    .accrue_yield()
    //                    .call()
    //                    .await
    //                    .unwrap();
    //                // call withdraw
    //                user_vault_instance
    //                .methods()
    //                .withdraw(recipient.clone(), base_asset_id, default_sub_id)
    //                .call_params(withdraw_call_params)
    //                .unwrap()
    //                .with_variable_output_policy(VariableOutputPolicy::Exactly(1))
    //                .call()
    //                .await
    //                .unwrap();
























































































































































































// #[tokio::test]
// async fn test_admin_can_send_funds_to_vault() {
//    use fuels::types::output::Output;

//    // get wallets 
//    let flash_sync = setup().await;

//    let donor = &flash_sync.wallets[2];

//    let predicate = flash_sync.flash_loan_predicate;

//    let admin = &flash_sync.vault_admin;

//    let amount_to_fund_predicate: u64 = 5_000_000;
//    let donor_dup = admin.clone();
//    let provider = donor_dup.provider().unwrap();
//    let base_asset_id = provider.base_asset_id();

//    donor
//         .transfer(predicate.address().into(), 
//         amount_to_fund_predicate, 
//         *base_asset_id, 
//         TxPolicies::default()
//     ).await;

//     let bal =
//      provider.get_asset_balance(
//         predicate.address().into(), 
//         *base_asset_id
//     ).await.unwrap();

//     println!("balance of predicate: {:?}", bal);
//     let vault_id = flash_sync.flash_vault_contract_id;
//     let vault_raw_addr = convert_id(vault_id.clone()).await;
//     let bech32Address = Bech32Address::from(vault_raw_addr);
    

//     // now build transaction without script specifying only vault can reciev



//     let mut tb: ScriptTransactionBuilder = {

//         // let inputIndex: u16 = 0;
//         // let balanceRoot = Bytes32::zeroed();
//         // let stateRoot = Bytes32::zeroed();
    
//         // let outputs = vec![
//         //     Output::Contract{
//         //         inputIndex,
//         //         balanceRoot,
//         //         stateRoot,

//         //     }

//         // ];
//         let mut inputs = vec![Input::Contract {
//             utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
//             balance_root: Bytes32::zeroed(),
//             state_root: Bytes32::zeroed(),
//             tx_pointer: TxPointer::default(),
//             contract_id: flash_sync.flash_vault_contract_id,
//         }];


//           let inputs1 = predicate
//                                        .get_asset_inputs_for_amount(
//                                         *base_asset_id, 
//                                         4_000_000, 
//                                         None
//                                     ).await.unwrap();
//                                     println!("inputs {:#?}", inputs.clone());
//                                     inputs.extend(inputs1);

//           let outputs = vec![
//             Output::Coin {  
//                 to : Address::new(*vault_id),
//                 amount: 4_000_000,
//                 asset_id: *base_asset_id,
//             },
//                 Output::contract(0, Bytes32::zeroed(), Bytes32::zeroed())  ,
//                 Output::Change {
//                     to: predicate.clone().address().into(),
//                     amount: 0,
//                     asset_id: *base_asset_id,
//                 }

//         ];
          
          
          
          
//         //   predicate
//         //   get_asset_outputs_for_amount( &bech32Address, *base_asset_id, 4_000_000);

          

//          ScriptTransactionBuilder::prepare_transfer(
//             inputs,
//             outputs,
//             TxPolicies::default()
//          )
          
//     };

//     tb.add_signer(admin.clone());


//     let tx = tb.build(
//         provider.clone()
//         ).await.unwrap();

//     provider.send_transaction_and_await_commit(tx)
//                                        .await.unwrap();


//     let bal =
//     provider.get_asset_balance(
//        predicate.address().into(), 
//        *base_asset_id
//    ).await.unwrap();
//    admin
//         .transfer(&bech32Address, 1000, *base_asset_id, TxPolicies::default()).await.unwrap();
//    println!("balance of predicate after: {:?}", bal);
//    let vault_bal =
//    provider.get_contract_balances(
//       &vault_id.into()
//   ).await.unwrap();

//   let vault = flash_sync.flash_vault_instance;
//   let res = vault
//        .methods()
//        .get_vault_bal()
//        .call()
//        .await.unwrap();
//     println!("vault can know it has money(Holy spirit stake control in Jesus Name: {:?}", res.value);

   


//   println!("balance of vault after: {:?}", vault_bal);



   

// }

async fn convert_id(contract_id: ContractId) -> Address {
    Address::new(*contract_id)
}






