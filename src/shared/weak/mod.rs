use super::{MutexStorage, RefCellStorage, RwLockStorage, SArc, SRc, SharedState, SharedStorage};
use core::marker::PhantomData;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Weak as ArcWeak};

/// Weak counterpart to [`SRc`].
///
/// A weak handle must be upgraded before borrowing. This keeps "the shared
/// value was dropped" separate from the existing state/storage borrow errors.
///
/// ```ignore
/// let shared = SRcRefCell::<Connection>::new::<Disconnected>(connection);
/// let weak = shared.downgrade();
///
/// if let Some(shared) = weak.upgrade() {
///     let disconnected = shared.borrow::<Disconnected>()?;
/// }
/// ```
pub struct WeakSRc<Storage, T>
where
    Storage: SharedStorage,
{
    storage: RcWeak<Storage::Storage<T>>,
    backend: PhantomData<fn() -> Storage>,
    value: PhantomData<fn() -> T>,
}

/// Weak counterpart to [`SArc`].
///
/// A weak handle must be upgraded before borrowing. This keeps "the shared
/// value was dropped" separate from the existing state/storage borrow errors.
///
/// ```ignore
/// let shared = SArcMutex::<Connection>::new::<Disconnected>(connection);
/// let weak = shared.downgrade();
///
/// match weak.upgrade() {
///     Some(shared) => {
///         let disconnected = shared.borrow::<Disconnected>()?;
///     }
///     None => {
///         // All strong handles were dropped.
///     }
/// }
/// ```
pub struct WeakSArc<Storage, T>
where
    Storage: SharedStorage,
{
    storage: ArcWeak<Storage::Storage<T>>,
    backend: PhantomData<fn() -> Storage>,
    value: PhantomData<fn() -> T>,
}

impl<Storage, T> Clone for WeakSRc<Storage, T>
where
    Storage: SharedStorage,
{
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            backend: PhantomData,
            value: PhantomData,
        }
    }
}

impl<Storage, T> Clone for WeakSArc<Storage, T>
where
    Storage: SharedStorage,
{
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            backend: PhantomData,
            value: PhantomData,
        }
    }
}

impl<Storage, T> SharedState<Rc<Storage::Storage<T>>, Storage, T>
where
    Storage: SharedStorage,
{
    /// Creates a weak handle to this `Rc`-backed shared state.
    ///
    /// The weak handle remembers the same [`SharedStorage`] backend as the strong
    /// handle. After upgrade, callers use the normal `borrow` and `borrow_mut`
    /// APIs, so state mismatches and backend borrow errors are still reported by
    /// those APIs:
    ///
    /// ```ignore
    /// let shared = SRcRefCell::<Connection>::new::<Disconnected>(connection);
    /// let weak: WeakSRcRefCell<Connection> = shared.downgrade();
    ///
    /// let shared = weak.upgrade().expect("at least one strong handle remains");
    /// let disconnected = shared.borrow::<Disconnected>()?;
    /// ```
    #[must_use]
    pub fn downgrade(&self) -> WeakSRc<Storage, T> {
        WeakSRc {
            storage: Rc::downgrade(&self.storage),
            backend: PhantomData,
            value: PhantomData,
        }
    }
}

impl<Storage, T> SharedState<Arc<Storage::Storage<T>>, Storage, T>
where
    Storage: SharedStorage,
{
    /// Creates a weak handle to this `Arc`-backed shared state.
    ///
    /// Upgrade failure means all strong `Arc` handles were dropped. It is not a
    /// state-machine error and it is independent from wrong-state or lock errors:
    ///
    /// ```ignore
    /// let shared = SArcMutex::<Connection>::new::<Disconnected>(connection);
    /// let weak: WeakSArcMutex<Connection> = shared.downgrade();
    ///
    /// if let Some(shared) = weak.upgrade() {
    ///     let disconnected = shared.borrow::<Disconnected>()?;
    /// }
    /// ```
    #[must_use]
    pub fn downgrade(&self) -> WeakSArc<Storage, T> {
        WeakSArc {
            storage: Arc::downgrade(&self.storage),
            backend: PhantomData,
            value: PhantomData,
        }
    }
}

impl<Storage, T> WeakSRc<Storage, T>
where
    Storage: SharedStorage,
{
    /// Attempts to recover a strong `Rc`-backed shared-state handle.
    ///
    /// `None` means no strong [`SRc`] handle exists anymore. `Some` does not imply
    /// the requested state is currently available; use `borrow::<State>()` on the
    /// upgraded value to perform the state check.
    #[must_use]
    pub fn upgrade(&self) -> Option<SRc<Storage, T>> {
        self.storage.upgrade().map(|storage| SharedState {
            storage,
            backend: PhantomData,
            value: PhantomData,
        })
    }
}

impl<Storage, T> WeakSArc<Storage, T>
where
    Storage: SharedStorage,
{
    /// Attempts to recover a strong `Arc`-backed shared-state handle.
    ///
    /// This mirrors [`std::sync::Weak::upgrade`]. A successful upgrade only
    /// restores ownership of the shared container; typestate validation still
    /// happens at the later `borrow`/`borrow_mut` call.
    #[must_use]
    pub fn upgrade(&self) -> Option<SArc<Storage, T>> {
        self.storage.upgrade().map(|storage| SharedState {
            storage,
            backend: PhantomData,
            value: PhantomData,
        })
    }
}

/// Weak handle for [`SRcRefCell`](crate::SRcRefCell).
///
/// Use this alias when the strong handle has type `SRcRefCell<T>`.
pub type WeakSRcRefCell<T> = WeakSRc<RefCellStorage, T>;
/// Weak handle for [`SArcMutex`](crate::SArcMutex).
///
/// Use this alias when the strong handle has type `SArcMutex<T>`.
pub type WeakSArcMutex<T> = WeakSArc<MutexStorage, T>;
/// Weak handle for [`SArcRwLock`](crate::SArcRwLock).
///
/// Use this alias when the strong handle has type `SArcRwLock<T>`.
pub type WeakSArcRwLock<T> = WeakSArc<RwLockStorage, T>;
