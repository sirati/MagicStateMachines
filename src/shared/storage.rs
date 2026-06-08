use crate::state_trait::ErasedState;
use core::fmt;
use core::ops::{Deref, DerefMut};
use std::cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

/// The state marker and runtime data held by a shared-storage backend.
///
/// Its fields are private so backends can synchronize storage without changing
/// the authoritative state directly.
pub struct SharedValue<T> {
    pub(super) state: ErasedState,
    pub(super) value: T,
}

/// Failure caused by asking a shared container for the wrong state marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WrongStateError {
    pub expected: &'static str,
    pub actual: &'static str,
}

impl fmt::Display for WrongStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expected state {}, found {}",
            self.expected, self.actual
        )
    }
}

impl std::error::Error for WrongStateError {}

/// Failure to acquire a typed view of shared state.
#[derive(Debug)]
pub enum SharedStateError<StorageError = core::convert::Infallible> {
    WrongState(WrongStateError),
    Storage(StorageError),
}

impl<StorageError> fmt::Display for SharedStateError<StorageError>
where
    StorageError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongState(error) => error.fmt(formatter),
            Self::Storage(error) => error.fmt(formatter),
        }
    }
}

impl<StorageError> std::error::Error for SharedStateError<StorageError> where
    StorageError: fmt::Debug + fmt::Display
{
}

impl<StorageError> From<WrongStateError> for SharedStateError<StorageError> {
    fn from(error: WrongStateError) -> Self {
        Self::WrongState(error)
    }
}

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

    type ReadError<'a, T>
    where
        Self: 'a,
        T: 'a;

    type WriteError<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T>;
    fn read<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::ReadGuard<'_, T>, Self::ReadError<'_, T>>;
    fn write<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::WriteGuard<'_, T>, Self::WriteError<'_, T>>;
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
    type ReadError<'a, T>
        = BorrowError
    where
        T: 'a;
    type WriteError<'a, T>
        = BorrowMutError
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        RefCell::new(value)
    }

    fn read<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::ReadGuard<'_, T>, Self::ReadError<'_, T>> {
        storage.try_borrow()
    }

    fn write<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::WriteGuard<'_, T>, Self::WriteError<'_, T>> {
        storage.try_borrow_mut()
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
    type ReadError<'a, T>
        = TryLockError<MutexGuard<'a, SharedValue<T>>>
    where
        T: 'a;
    type WriteError<'a, T>
        = TryLockError<MutexGuard<'a, SharedValue<T>>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        Mutex::new(value)
    }

    fn read<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::ReadGuard<'_, T>, Self::ReadError<'_, T>> {
        storage.try_lock()
    }

    fn write<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::WriteGuard<'_, T>, Self::WriteError<'_, T>> {
        storage.try_lock()
    }
}

pub struct RwLockStorage;

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
    type ReadError<'a, T>
        = TryLockError<RwLockReadGuard<'a, SharedValue<T>>>
    where
        T: 'a;
    type WriteError<'a, T>
        = TryLockError<RwLockWriteGuard<'a, SharedValue<T>>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        RwLock::new(value)
    }

    fn read<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::ReadGuard<'_, T>, Self::ReadError<'_, T>> {
        storage.try_read()
    }

    fn write<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::WriteGuard<'_, T>, Self::WriteError<'_, T>> {
        storage.try_write()
    }
}
