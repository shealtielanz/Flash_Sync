contract;

use std::{
    asset::transfer,
    call_frames::msg_asset_id,
    context::msg_amount,
    hash::{
        Hash,
        sha256,
    },
    storage::storage_string::*,
    string::String,
};
use standards::{src20::SRC20, src6::{Deposit, SRC6, Withdraw}};
use sway_libs::{
    ownership::{
        initialize_ownership,
        only_owner,
    },
    pausable::{
        _is_paused,
        _pause,
        _unpause,
        Pausable,
        require_not_paused,
        require_paused,
    },
};

configurable {
    /// The only sub vault that can be deposited and withdrawn from this vault.
    VAULT_SUB_ID: SubId = SubId::zero(),
    /// The name of the shares asset minted by this contract.
    NAME: str[17] = __to_str_array("Flash Vault Token"),
    /// The symbol of the shares asset minted by this contract.
    SYMBOL: str[3] = __to_str_array("FVT"),
    /// The decimals of the shares asset minted by this contract.
    DECIMALS: u8 = 9,
}

storage {
    // The total amount of assets managed by this vault.
    managed_assets: u64 = 0,
    /// The total amount of shares minted by this vault.
    total_supply: u64 = 0,
    // Whether the vault shares have been minted.
    minted: bool = false,
    // flash predicate for the flash
    flash_loan_predicate: Option<Identity> = Option::None,
}

abi ManagePredicates {
    #[storage(read, write)]
    fn set_flash_loan_predicate(_predicate: Identity);
    #[storage(read, write)]
    fn transfer_to_predicate(amount: u64);
    #[storage(read, write)]
    fn accrue_yield();
    #[storage(read)]
    fn get_flash_predicate() -> Option<Identity>;
}

abi MyConstructor {
    #[storage(read, write)]
    fn my_constructor(new_owner: Identity);
}

impl MyConstructor for Contract {
    #[storage(read, write)]
    fn my_constructor(new_owner: Identity) {
        initialize_ownership(new_owner);
    }
}

impl ManagePredicates for Contract {
    #[storage(read, write)]
    fn set_flash_loan_predicate(_predicate: Identity) {
        only_owner();
        storage.flash_loan_predicate.write(Some(_predicate));
    }

    #[storage(read, write)]
    fn transfer_to_predicate(amount: u64) {
        use std::context::this_balance;

        only_owner();
        let base_asset_id = AssetId::base();

        require(
            this_balance(base_asset_id) >= amount,
            "TransferError::NotEnoughCoins",
        );

        transfer(
            storage
                .flash_loan_predicate
                .read()
                .unwrap(),
            base_asset_id,
            amount,
        );
    }

    #[storage(read, write)]
    fn accrue_yield() {
        use std::context::this_balance;

        let base_asset_id = AssetId::base();

        let gross_yield = this_balance(base_asset_id);

        let asset_managed_bal = storage.managed_assets.read();

        if gross_yield > asset_managed_bal {
            let net_yield = gross_yield - asset_managed_bal;
            storage.managed_assets.write(asset_managed_bal + net_yield);
        }
    }

    #[storage(read)]
    fn get_flash_predicate() -> Option<Identity> {
        storage.flash_loan_predicate.read()
    }
}

impl SRC6 for Contract {
    #[payable]
    #[storage(read, write)]
    fn deposit(receiver: Identity, vault_sub_id: SubId) -> u64 {
        //ensure users cannot deposit during yeidling period.
        require_not_paused();
        require(vault_sub_id == VAULT_SUB_ID, "INVALID_vault_sub_id");

        let underlying_asset = msg_asset_id();
        require(underlying_asset == AssetId::base(), "INVALID_ASSET_ID");

        let asset_amount = msg_amount();
        require(asset_amount != 0, "ZERO_ASSETS");
        let shares = preview_deposit(asset_amount);

        _mint(receiver, shares);

        storage
            .managed_assets
            .write(storage.managed_assets.read() + asset_amount);

        log(Deposit {
            caller: msg_sender().unwrap(),
            receiver,
            underlying_asset,
            vault_sub_id,
            deposited_amount: asset_amount,
            minted_shares: shares,
        });

        shares
    }

    #[payable]
    #[storage(read, write)]
    fn withdraw(
        receiver: Identity,
        underlying_asset: AssetId,
        vault_sub_id: SubId,
    ) -> u64 {
        //ensure users cannot withdraw during yeidling period.
        require_not_paused();

        // can only  base asset allowd 
        require(underlying_asset == AssetId::base(), "INVALID_ASSET_ID");
        require(vault_sub_id == VAULT_SUB_ID, "INVALID_vault_sub_id");

        let shares = msg_amount();
        require(shares != 0, "ZERO_SHARES");

        let share_asset_id = managed_assetsid();

        require(msg_asset_id() == share_asset_id, "INVALID_ASSET_ID");
        let assets = preview_withdraw(shares);

        storage
            .managed_assets
            .write(storage.managed_assets.read() - shares);

        _burn(share_asset_id, shares);

        transfer(receiver, underlying_asset, assets);

        log(Withdraw {
            caller: msg_sender().unwrap(),
            receiver,
            underlying_asset,
            vault_sub_id,
            withdrawn_amount: assets,
            burned_shares: shares,
        });

        assets
    }

    #[storage(read)]
    fn managed_assets(underlying_asset: AssetId, vault_sub_id: SubId) -> u64 {
        if underlying_asset == AssetId::base() && vault_sub_id == VAULT_SUB_ID {
            // In this implementation managed_assets and max_withdrawable are the same. However in case of lending out of assets, managed_assets should be greater than max_withdrawable.
            storage.managed_assets.read()
        } else {
            0
        }
    }

    #[storage(read)]
    fn max_depositable(
        receiver: Identity,
        underlying_asset: AssetId,
        vault_sub_id: SubId,
    ) -> Option<u64> {
        if underlying_asset == AssetId::base() {
            // This is the max value of u64 minus the current managed_assets. Ensures that the sum will always be lower than u64::MAX.
            Some(u64::max() - storage.managed_assets.read())
        } else {
            None
        }
    }

    #[storage(read)]
    fn max_withdrawable(underlying_asset: AssetId, vault_sub_id: SubId) -> Option<u64> {
        if underlying_asset == AssetId::base() {
            // In this implementation managed_assets and max_withdrawable are the same. However in case of lending out of assets, managed_assets should be greater than max_withdrawable.
            Some(storage.managed_assets.read())
        } else {
            None
        }
    }
}

impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        1
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == AssetId::default() {
            Some(storage.total_supply.read())
        } else {
            None
        }
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(NAME)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(SYMBOL)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == AssetId::default() {
            Some(DECIMALS)
        } else {
            None
        }
    }
}
/// Returns the vault shares assetid for the given assets assetid and the vaults sub id
fn managed_assetsid() -> AssetId {
    let share_asset_id = AssetId::new(ContractId::this(), VAULT_SUB_ID);
    share_asset_id
}

#[storage(read)]
fn preview_deposit(assets: u64) -> u64 {
    let shares_supply = storage.total_supply.try_read().unwrap_or(0);
    if shares_supply == 0 {
        assets
    } else {
        assets * shares_supply / storage.managed_assets.try_read().unwrap_or(0)
    }
}

#[storage(read)]
fn preview_withdraw(shares: u64) -> u64 {
    let supply = storage.total_supply.read();
    if supply == shares {
        storage.managed_assets.read()
    } else {
        // suppose to multiple first to avoid rounding/precision issues 
        // but will divide first to avoid arthimetic oveflow during multiplications
        shares * (storage.managed_assets.read() / supply)
    }
}

#[storage(read, write)]
pub fn _mint(recipient: Identity, amount: u64) {
    use std::asset::mint_to;

    let supply = storage.total_supply.read();
    storage.total_supply.write(supply + amount);
    mint_to(recipient, VAULT_SUB_ID, amount);
}

#[storage(read, write)]
pub fn _burn(asset_id: AssetId, amount: u64) {
    use std::{asset::burn, context::this_balance};

    require(
        this_balance(asset_id) >= amount,
        "BurnError::NotEnoughCoins",
    );
    // If we pass the check above, we can assume it is safe to unwrap.
    let supply = storage.total_supply.read();
    storage.total_supply.write(supply - amount);
    burn(VAULT_SUB_ID, amount);
}

impl Pausable for Contract {
    #[storage(write)]
    fn pause() {
        // Add the `only_owner()` check to ensure only the owner may unpause this contract.
        only_owner();
        _pause();
    }

    #[storage(write)]
    fn unpause() {
        // Add the `only_owner()` check to ensure only the owner may unpause this contract.
        only_owner();
        _unpause();
    }

    #[storage(read)]
    fn is_paused() -> bool {
        _is_paused()
    }
}

abi getter{
    fn get_vault_bal() -> u64;
}

impl getter for Contract{

   fn get_vault_bal() -> u64 {
    use std::context::this_balance;
    
    this_balance(AssetId::base())
   }
}

// allow to recieve eth
#[fallback]
fn fallback() {
    // Ensure ETH is being sent
    let amount = msg_amount();
    require(msg_asset_id() == AssetId::base(), "Not eth, why?");
    require(amount > 0, "ETH must be sent to the contract");
}
