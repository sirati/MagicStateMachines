use crate::connection::Connection;
use statemachines::{SArc, SharedStateError, SharedStorage, SharedValue, StateUnionState};
use std::sync::{Mutex, MutexGuard};
use test_def::{
    Online,
    states::{Connected, Disconnected},
};

/// A custom backend marker. Its GAT selects storage for each runtime type.
pub(crate) struct CustomMutexStorage;

impl SharedStorage for CustomMutexStorage {
    type Storage<T> = Mutex<SharedValue<T>>;
    type ReadGuard<'a, T>
        = MutexGuard<'a, SharedValue<T>>
    where
        T: 'a;
    type WriteGuard<'a, T>
        = MutexGuard<'a, SharedValue<T>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        Mutex::new(value)
    }

    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError> {
        storage.lock().map_err(|_| SharedStateError::Poisoned)
    }

    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError> {
        storage.lock().map_err(|_| SharedStateError::Poisoned)
    }
}

pub(crate) fn run() {
    let shared = SArc::<CustomMutexStorage, _>::from_state(Connection::new("localhost:7070"));
    let alias = shared.clone();

    if let Ok(guard) = shared.borrow_mut::<Disconnected>() {
        let connected = guard.connect();
        drop(connected);
    }

    let connected = alias.borrow::<Connected>().expect("committed state");
    println!("{} uses the custom mutex backend", connected.raw_endpoint());

    let online = alias
        .borrow::<StateUnionState<Online>>()
        .expect("committed online state");
    println!("{} can be borrowed through erasure", online.raw_endpoint());
}
