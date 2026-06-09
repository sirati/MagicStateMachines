mod guard;
mod storage;
mod weak;

use crate::{Initial, SOwned, State, StateMachineImpl, state_trait};
use core::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

pub use guard::{
    SharedBorrowState, StateMut, StateMutTransitionCall, StateRef, StorageStateMut, transition_mut,
};
pub use storage::{
    MutexStorage, RefCellStorage, RwLockStorage, SharedStateError, SharedStorage, SharedValue,
    WrongStateError,
};
pub use weak::{
    WeakSArc, WeakSArcMutex, WeakSArcRwLock, WeakSRc, WeakSRcRefCell,
};

/// Shared state using an explicit, replaceable storage backend.
pub struct SharedState<P, S, T>
where
    S: SharedStorage,
{
    pub(super) storage: P,
    pub(super) backend: PhantomData<fn() -> S>,
    pub(super) value: PhantomData<fn() -> T>,
}

impl<P: Clone, S: SharedStorage, T> Clone for SharedState<P, S, T> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            backend: PhantomData,
            value: PhantomData,
        }
    }
}

impl<P, Backend, T> SharedState<P, Backend, T>
where
    Backend: SharedStorage,
    P: From<Backend::Storage<T>> + AsRef<Backend::Storage<T>>,
    T: StateMachineImpl,
{
    #[must_use]
    pub fn new<State>(value: T) -> Self
    where
        T::Standin: Initial<State>,
        State: crate::StateTrait,
    {
        Self {
            storage: P::from(Backend::new(SharedValue {
                state: state_trait::erased_state::<State>(),
                value,
            })),
            backend: PhantomData,
            value: PhantomData,
        }
    }

    #[must_use]
    pub fn from_state<StateMarker>(state: State<SOwned, T, StateMarker>) -> Self
    where
        StateMarker: crate::StateTrait,
    {
        Self {
            storage: P::from(Backend::new(SharedValue {
                state: state_trait::erased_state::<StateMarker>(),
                value: state.inner.value,
            })),
            backend: PhantomData,
            value: PhantomData,
        }
    }

    pub fn borrow<State>(
        &self,
    ) -> Result<
        StateRef<Backend::ReadGuard<'_, T>, T, State>,
        SharedStateError<Backend::ReadError<'_, T>>,
    >
    where
        State: SharedBorrowState,
    {
        let guard = Backend::read(self.storage.as_ref()).map_err(SharedStateError::Storage)?;
        StateRef::from_guard(guard)
    }

    pub fn borrow_mut<StateMarker>(
        &self,
    ) -> Result<SMutView<'_, Backend, T, StateMarker>, SharedStateError<Backend::WriteError<'_, T>>>
    where
        StateMarker: SharedBorrowState,
    {
        let guard = Backend::write(self.storage.as_ref()).map_err(SharedStateError::Storage)?;
        StateMut::from_guard(guard).map(State::from_inner)
    }
}

pub type SRc<Storage, T> = SharedState<Rc<<Storage as SharedStorage>::Storage<T>>, Storage, T>;
pub type SArc<Storage, T> = SharedState<Arc<<Storage as SharedStorage>::Storage<T>>, Storage, T>;
pub type SRcRefCell<T> = SRc<RefCellStorage, T>;
pub type SArcMutex<T> = SArc<MutexStorage, T>;
pub type SArcRwLock<T> = SArc<RwLockStorage, T>;
pub type SRefCell<'a> = StorageStateMut<'a, RefCellStorage>;
pub type SMutex<'a> = StorageStateMut<'a, MutexStorage>;
pub type SRwLock<'a> = StorageStateMut<'a, RwLockStorage>;
pub type SMutView<'a, Backend, T, S> = State<StorageStateMut<'a, Backend>, T, S>;
