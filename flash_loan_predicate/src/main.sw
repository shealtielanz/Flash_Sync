predicate;

use std::{
    b512::B512,
    constants::ZERO_B256,
    ecr::ec_recover_address,
    hash::{
        Hash,
        sha256,
    },
    inputs::{
        Input,
        input_amount,
        input_asset_id,
        input_coin_owner,
        input_count,
        input_type,
    },
    outputs::{
        Output,
        output_amount,
        output_asset_id,
        output_asset_to,
        output_count,
        output_type,
    },
    tx::{
        tx_id,
        tx_script_bytecode_hash,
        tx_script_length,
        tx_witness_data,
        tx_witnesses_count,
    },
};


configurable {
    EXPECTED_FLASH_LOAN_SCRIPT_BYTECODE_HASH: b256 = ZERO_B256,
    EXPECTED_FLASH_ADMIN_SCRIPT_BYTECODE_HASH: b256 = ZERO_B256,
    VAULT_ADDRESS: b256 = ZERO_B256,
    FLASH_SETTLER_ADDRESS: b256 = ZERO_B256,
    LOAN_ASSET: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    REQUIRED_SIGNATURES: u64 = 0,
    SIGNERS: [Address; 3] = [
        Address::from(0x0000000000000000000000000000000000000000000000000000000000000000),
        Address::from(0x0000000000000000000000000000000000000000000000000000000000000000),
        Address::from(0x0000000000000000000000000000000000000000000000000000000000000000),
    ],
}


fn main() -> bool {
    if (tx_script_length().unwrap() == 0) {
        return false;
    }
    
    let script_bytecode_hash: b256 = tx_script_bytecode_hash().unwrap();

    if(script_bytecode_hash == EXPECTED_FLASH_ADMIN_SCRIPT_BYTECODE_HASH) {

        let mut validated_sig = 0;
        let mut i = 0;

        while i < REQUIRED_SIGNATURES {
            validated_sig += verify_sig(i);
            i += 1;
        }

        let sending_to_vault: bool = validate_transaction_is_safe();

        if (validated_sig >= REQUIRED_SIGNATURES)
            && (sending_to_vault)
        {
            return true;
        }

        return false;
    } else if (script_bytecode_hash == EXPECTED_FLASH_LOAN_SCRIPT_BYTECODE_HASH) {
        // if the script used is for the flashloan, validate the tx is always safe.
        if (validate_transaction_is_safe()) {
            return true;
        }
    }

    false
    

}


fn validate_transaction_is_safe() -> bool {

  let mut i = 0;
  let output_num = output_count().as_u64();
  let predicate_addr = get_predicate_addr().unwrap();

  while i < output_num {
    let output_type = output_type(i).unwrap();
     match output_type {
        Output::Coin => return false,
        Output::Change => {
      //  @note this check is required to avoid a critical exploit, will fix later.
            if output_asset_to(i).unwrap() != predicate_addr {
                return false;
            } 
            
        },
        _ => (),
    }
    i += 1;
  }
  return true;

}
fn get_predicate_addr() -> Option<Address>{
    let index = predicate_input_index();
    let addr = input_coin_owner(index);
    addr
}
// @note having doubts whether this actually points to the index where the predicate inputs are located.
// @audit-ok after multiple tests, i have confirem that 
// this actually points to the index where the predicates inputs are loacated.
fn predicate_input_index() -> u64 {
    asm(r1) {
        gm r1 i3;
        r1: u64
    }
}


fn verify_sig(i: u64) -> u64 {
    if (i >= tx_witnesses_count()) {
        return 0;
    }

    let tx_hash = tx_id();

    let mut j = 0;

    while j < 1 {
        let current_sig = tx_witness_data::<B512>(j).unwrap();

        let current_addr = ec_recover_address(current_sig, tx_hash).unwrap();

        if current_addr == SIGNERS[j] {
            return 1;
        }

        j += 1;
    }

    return 0;
}