use crate::state_trait::ErasedState;
use core::fmt;
use core::ops::{Deref, DerefMut};
use std::cell::{Ref, RefCell, RefMut};
use std::sync::{Mutex, MutexGuard};

/// The state marker and runtime data held by a shared-storage backend.
///
/// Its fields are private so backends can synchronize storage without changing
/// the authoritative state directly.
pub struct SharedValue<T> {
    pub(super) state: ErasedState,
    pub(super) value: T,
}

/// Failure to acquire a typed view of shared state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedStateError {
    WrongState {
        expected: &'static str,
        actual: &'static str,
    },
    Borrowed,
    Poisoned,
}

impl fmt::Display for SharedStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongState { expected, actual } => {
                write!(formatter, "expected state {expected}, found {actual}")
            }
            Self::Borrowed => formatter.write_str("shared state is already borrowed"),
            Self::Poisoned => formatter.write_str("shared state mutex is poisoned"),
        }
    }
}

impl std::error::Error for SharedStateError {}

/// Replaceable storage backend for [`super::SharedState`].
pub trait SharedStorage {
    type Storage<T>;

    type ReadGuard<'a, T>: Deref<Target = SharedValue<T>>
    where
        Self: 'a,
        T: 'a;

    type WriteGuard<'a, T>: DerefMut<Target = SharedValue<T>>
    where
        Self: 'a,
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T>;
    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError>;
    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError>;
}

pub struct RefCellStorage;

impl SharedStorage for RefCellStorage {
    type Storage<T> = RefCell<SharedValue<T>>;
    type ReadGuard<'a, T>
        = Ref<'a, SharedValue<T>>
    where
        T: 'a;
    type WriteGuard<'a, T>
        = RefMut<'a, SharedValue<T>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        RefCell::new(value)
    }

    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError> {
        storage.try_borrow().map_err(|_| SharedStateError::Borrowed)
    }

    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError> {
        storage
            .try_borrow_mut()
            .map_err(|_| SharedStateError::Borrowed)
    }
}

pub struct MutexStorage;

impl SharedStorage for MutexStorage {
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
