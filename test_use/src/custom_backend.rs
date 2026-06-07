use crate::connection::Connection;
use statemachines::{RcState, SharedStateError, SharedStorage, SharedValue};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use test_def::states::{Connected, Disconnected};

/// A non-generic backend marker. Its GAT selects storage for each data type.
pub(crate) struct RwLockStorage;

impl SharedStorage for RwLockStorage {
    type Storage<T> = RwLock<SharedValue<T>>;
    type ReadGuard<'a, T>
        = RwLockReadGuard<'a, SharedValue<T>>
    where
        T: 'a;
    type WriteGuard<'a, T>
        = RwLockWriteGuard<'a, SharedValue<T>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        RwLock::new(value)
    }

    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError> {
        storage.read().map_err(|_| SharedStateError::Poisoned)
    }

    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError> {
        storage.write().map_err(|_| SharedStateError::Poisoned)
    }
}

pub(crate) fn run() {
    let shared = RcState::<RwLockStorage, _>::new(Connection::new("localhost:7070"));
    let alias = shared.clone();

    if let Ok(guard) = shared.borrow_mut::<Disconnected>() {
        let connected = guard.connect();
        drop(connected);
    }

    let connected = alias.borrow::<Connected>().expect("committed state");
    println!(
        "{} uses the custom RwLock backend",
        connected.raw_endpoint()
    );
}
