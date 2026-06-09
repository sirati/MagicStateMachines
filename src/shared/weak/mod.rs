use super::{
    MutexStorage, RefCellStorage, RwLockStorage, SArc, SRc, SharedState, SharedStorage,
};
use core::marker::PhantomData;
use std::rc::{Rc, Weak as RcWeak};
use std::sync::{Arc, Weak as ArcWeak};

/// Weak counterpart to [`SRc`].
///
/// A weak handle must be upgraded before borrowing. This keeps "the shared
/// value was dropped" separate from the existing state/storage borrow errors.
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
    #[must_use]
    pub fn upgrade(&self) -> Option<SArc<Storage, T>> {
        self.storage.upgrade().map(|storage| SharedState {
            storage,
            backend: PhantomData,
            value: PhantomData,
        })
    }
}

pub type WeakSRcRefCell<T> = WeakSRc<RefCellStorage, T>;
pub type WeakSArcMutex<T> = WeakSArc<MutexStorage, T>;
pub type WeakSArcRwLock<T> = WeakSArc<RwLockStorage, T>;
