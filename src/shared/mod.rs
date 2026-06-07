mod guard;
mod storage;

use crate::{Initial, State, StateMachineImpl, StateTrait, state_trait};
use core::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

pub use guard::{StateMut, StateMutTransitionCall, StateRef, StorageStateMut, transition_mut};
pub use storage::{MutexStorage, RefCellStorage, SharedStateError, SharedStorage, SharedValue};

/// Shared state using an explicit, replaceable storage backend.
pub struct SharedState<P, S, T>
where
    S: SharedStorage,
{
    storage: P,
    backend: PhantomData<fn() -> S>,
    value: PhantomData<fn() -> T>,
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
        State: StateTrait,
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

    pub fn borrow<State>(
        &self,
    ) -> Result<StateRef<Backend::ReadGuard<'_, T>, T, State>, SharedStateError>
    where
        State: StateTrait,
    {
        StateRef::from_guard(Backend::read(self.storage.as_ref())?)
    }

    pub fn borrow_mut<StateMarker>(
        &self,
    ) -> Result<StateMutView<'_, Backend, T, StateMarker>, SharedStateError>
    where
        StateMarker: StateTrait,
    {
        StateMut::from_guard(Backend::write(self.storage.as_ref())?).map(State::from_inner)
    }
}

pub type RcState<S, T> = SharedState<Rc<<S as SharedStorage>::Storage<T>>, S, T>;
pub type ArcState<S, T> = SharedState<Arc<<S as SharedStorage>::Storage<T>>, S, T>;
pub type RefCellState<T> = RcState<RefCellStorage, T>;
pub type MutexState<T> = ArcState<MutexStorage, T>;
pub type StateMutView<'a, Backend, T, S> = State<StorageStateMut<'a, Backend>, T, S>;
