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
    /// Requested state or union marker type name.
    pub expected: &'static str,
    /// Committed concrete state type name.
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
    /// The container was borrowed successfully, but the committed state did not match.
    WrongState(WrongStateError),
    /// The backing storage could not be borrowed or locked.
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
///
/// Implement this trait when the built-in [`RefCellStorage`],
/// [`MutexStorage`], and [`RwLockStorage`] do not match the container you want
/// to use. The backend owns the actual synchronization primitive and returns
/// guard types that dereference to [`SharedValue<T>`].
///
/// The library does not collapse backend errors into a custom borrowed/poisoned
/// enum. Your `ReadError` and `WriteError` associated types are preserved and
/// returned as [`SharedStateError::Storage`].
///
/// A custom backend has this shape:
///
/// ```ignore
/// use magicstatemachines::{SArc, SharedStorage, SharedValue};
/// use std::sync::{Mutex, MutexGuard, TryLockError};
///
/// pub struct MyMutexStorage;
///
/// impl SharedStorage for MyMutexStorage {
///     type Storage<T> = Mutex<SharedValue<T>>;
///     type ReadGuard<'a, T> = MutexGuard<'a, SharedValue<T>> where T: 'a;
///     type WriteGuard<'a, T> = MutexGuard<'a, SharedValue<T>> where T: 'a;
///     type ReadError<'a, T> = TryLockError<MutexGuard<'a, SharedValue<T>>> where T: 'a;
///     type WriteError<'a, T> = TryLockError<MutexGuard<'a, SharedValue<T>>> where T: 'a;
///
///     fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
///         Mutex::new(value)
///     }
///
///     fn read<T>(
///         storage: &Self::Storage<T>,
///     ) -> Result<Self::ReadGuard<'_, T>, Self::ReadError<'_, T>> {
///         storage.try_lock()
///     }
///
///     fn write<T>(
///         storage: &Self::Storage<T>,
///     ) -> Result<Self::WriteGuard<'_, T>, Self::WriteError<'_, T>> {
///         storage.try_lock()
///     }
/// }
///
/// type SArcMyMutex<T> = SArc<MyMutexStorage, T>;
/// ```
pub trait SharedStorage {
    /// Concrete cell or lock type containing [`SharedValue<T>`].
    type Storage<T>;

    /// Guard returned by read access.
    type ReadGuard<'a, T>: Deref<Target = SharedValue<T>>
    where
        Self: 'a,
        T: 'a;

    /// Guard returned by write access.
    type WriteGuard<'a, T>: DerefMut<Target = SharedValue<T>>
    where
        Self: 'a,
        T: 'a;

    /// Error returned by read access.
    type ReadError<'a, T>
    where
        Self: 'a,
        T: 'a;

    /// Error returned by write access.
    type WriteError<'a, T>
    where
        Self: 'a,
        T: 'a;

    /// Creates backend storage containing the authoritative state and runtime data.
    fn new<T>(value: SharedValue<T>) -> Self::Storage<T>;
    /// Attempts read access to the backend storage.
    fn read<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::ReadGuard<'_, T>, Self::ReadError<'_, T>>;
    /// Attempts write access to the backend storage.
    fn write<T>(
        storage: &Self::Storage<T>,
    ) -> Result<Self::WriteGuard<'_, T>, Self::WriteError<'_, T>>;
}

/// [`SharedStorage`] implementation backed by [`RefCell`].
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

/// [`SharedStorage`] implementation backed by [`Mutex`].
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

/// [`SharedStorage`] implementation backed by [`RwLock`].
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
