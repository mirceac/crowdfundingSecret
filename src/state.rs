use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{CanonicalAddr, Storage, ReadonlyStorage, StdResult, StdError};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, PrefixedStorage};
use std::{any::type_name};
use serde::de::DeserializeOwned;
use secret_toolkit::serialization::{Bincode2, Serde};
use std::convert::TryFrom;

pub static CONFIG_KEY: &[u8] = b"config";
pub const PREFIX_BALANCES: &[u8] = b"balances";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub campaign_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Campaign {
    pub owner: String,
    pub description: String,
    pub amount: u8,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}

pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    Bincode2::deserialize(
        &storage
            .get(key)
            .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
    )
}

pub fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}

pub struct Balances<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}

impl<'a, S: Storage> Balances<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(PREFIX_BALANCES, storage),
        }
    }

    fn as_readonly(&self) -> ReadonlyBalancesImpl<PrefixedStorage<S>> {
        ReadonlyBalancesImpl(&self.storage)
    }

    pub fn balance(&self, account: &CanonicalAddr) -> u128 {
        self.as_readonly().account_amount(account)
    }

    pub fn set_account_balance(&mut self, account: &CanonicalAddr, amount: u128) {
        self.storage.set(account.as_slice(), &amount.to_be_bytes())
    }
}

/// This struct refactors out the readonly methods that we need for `Balances` and `ReadonlyBalances`
/// in a way that is generic over their mutability.
///
/// This was the only way to prevent code duplication of these methods because of the way
/// that `ReadonlyPrefixedStorage` and `PrefixedStorage` are implemented in `cosmwasm-std`
struct ReadonlyBalancesImpl<'a, S: ReadonlyStorage>(&'a S);

impl<'a, S: ReadonlyStorage> ReadonlyBalancesImpl<'a, S> {
    pub fn account_amount(&self, account: &CanonicalAddr) -> u128 {
        let account_bytes = account.as_slice();
        let result = self.0.get(account_bytes);
        match result {
            // This unwrap is ok because we know we stored things correctly
            Some(balance_bytes) => slice_to_u128(&balance_bytes).unwrap(),
            None => 0,
        }
    }
}

/// Converts 16 bytes value into u128
/// Errors if data found that is not 16 bytes
fn slice_to_u128(data: &[u8]) -> StdResult<u128> {
    match <[u8; 16]>::try_from(data) {
        Ok(bytes) => Ok(u128::from_be_bytes(bytes)),
        Err(_) => Err(StdError::generic_err(
            "Corrupted data found. 16 byte expected.",
        )),
    }
}

pub fn perform_transfer<T: Storage>(
    store: &mut T,
    from: &CanonicalAddr,
    to: &CanonicalAddr,
    amount: u128,
) -> StdResult<()> {
    let mut balances = Balances::from_storage(store);

    let mut from_balance = balances.balance(from);
    if let Some(new_from_balance) = from_balance.checked_sub(amount) {
        from_balance = new_from_balance;
    } else {
        return Err(StdError::generic_err(
            format!("Insufficient funds (from balance {}, required={}). From {} To {}", from_balance, amount, from.to_string(), to.to_string())))
    }
    balances.set_account_balance(from, from_balance);

    let mut to_balance = balances.balance(to);

    if let Some(new_to_balance) = to_balance.checked_add(amount) {
        to_balance = new_to_balance;
    } else {
        return Err(StdError::generic_err(
            format!("Actual funds (to balance {}, too much to add={}). From {} To {}", to_balance, amount, from.to_string(), to.to_string())))
    }

    balances.set_account_balance(to, to_balance);

    Ok(())
}
