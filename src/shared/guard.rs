use super::{SharedStateError, SharedStorage, SharedValue};
use crate::{
    SMut, SRef, State, StateMachineImpl, StateStorage, StateTrait, StateUnionRuntime,
    StateUnionState, Transition, TransitionCallsite,
    state_trait::{self, ErasedState},
};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub struct StateRef<G, T, S> {
    guard: G,
    marker: PhantomData<fn() -> (T, S)>,
}

impl<G, T, S> StateRef<G, T, S>
where
    G: Deref<Target = SharedValue<T>>,
    S: SharedBorrowState,
{
    pub(super) fn from_guard(guard: G) -> Result<Self, SharedStateError> {
        S::ensure_state(&guard.state)?;
        Ok(Self {
            guard,
            marker: PhantomData,
        })
    }
}

impl<G, T, S> Deref for StateRef<G, T, S>
where
    G: Deref<Target = SharedValue<T>>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.value
    }
}

pub struct StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    guard: Option<G>,
    pending: ErasedState,
    marker: PhantomData<fn() -> (T, S)>,
}

/// Generic [`State`] backend for a mutable shared-state guard.
pub struct StorageStateMut<'a, Backend>(PhantomData<&'a Backend>);

impl<'a, Backend> StateStorage for StorageStateMut<'a, Backend>
where
    Backend: SharedStorage + 'a,
{
    type Inner<T, S>
        = StateMut<Backend::WriteGuard<'a, T>, T, S>
    where
        T: StateMachineImpl;
    type Machine<T>
        = T
    where
        T: StateMachineImpl;

    fn retag<T, From, To>(mut inner: Self::Inner<T, From>) -> Self::Inner<T, To>
    where
        T: StateMachineImpl,
    {
        StateMut {
            guard: inner.guard.take(),
            pending: state_trait::clone_erased(&inner.pending),
            marker: PhantomData,
        }
    }

    fn complete_transition<T, From, To, Args>(
        mut state: State<Self, T, From>,
        _args: Args,
        _callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: StateTrait,
        To: StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        State {
            inner: StateMut {
                guard: state.inner.guard.take(),
                pending: state_trait::erased_state::<To>(),
                marker: PhantomData,
            },
            marker: PhantomData,
        }
    }

    fn complete_transition_after_effect<T, From, To>(
        mut state: State<Self, T, From>,
        _callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: StateTrait,
        To: StateTrait,
    {
        State {
            inner: StateMut {
                guard: state.inner.guard.take(),
                pending: state_trait::erased_state::<To>(),
                marker: PhantomData,
            },
            marker: PhantomData,
        }
    }
}

impl<'a, Backend> SRef for StorageStateMut<'a, Backend>
where
    Backend: SharedStorage + 'a,
{
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        inner
    }
}

impl<'a, Backend> SMut for StorageStateMut<'a, Backend>
where
    Backend: SharedStorage + 'a,
{
    fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
    where
        T: StateMachineImpl,
    {
        inner
    }
}

impl<G, T, S> StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
    S: SharedBorrowState,
{
    pub(super) fn from_guard(guard: G) -> Result<Self, SharedStateError> {
        S::ensure_state(&guard.state)?;
        let pending = S::initial_pending(&guard.state);
        Ok(Self {
            guard: Some(guard),
            pending,
            marker: PhantomData,
        })
    }
}

/// Creates a callable transition for a mutable shared-state guard.
///
/// This is the guarded-state counterpart to [`crate::transition`].
#[must_use]
pub fn transition_mut<G, T, S, Next>(
    state: StateMut<G, T, S>,
    _token: T::TransitionToken,
) -> StateMutTransitionCall<G, T, S, Next>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
    T::Standin: Transition<S, Next>,
{
    StateMutTransitionCall {
        state,
        to: PhantomData,
    }
}

impl<G, T, S> Deref for StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.as_ref().expect("guard is present").value
    }
}

impl<G, T, S> DerefMut for StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.as_mut().expect("guard is present").value
    }
}

impl<G, T, S> Drop for StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    fn drop(&mut self) {
        if let Some(guard) = self.guard.as_mut() {
            guard.state = state_trait::clone_erased(&self.pending);
        }
    }
}

pub struct StateMutTransitionCall<G, T, From, To>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    state: StateMut<G, T, From>,
    to: PhantomData<fn() -> To>,
}

impl<G, T, From, To, Args> FnOnce<Args> for StateMutTransitionCall<G, T, From, To>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    To: StateTrait,
{
    type Output = StateMut<G, T, To>;

    extern "rust-call" fn call_once(mut self, _args: Args) -> Self::Output {
        StateMut {
            guard: self.state.guard.take(),
            pending: state_trait::erased_state::<To>(),
            marker: PhantomData,
        }
    }
}

pub trait SharedBorrowState: StateTrait {
    fn ensure_state(actual: &ErasedState) -> Result<(), SharedStateError>;
    fn initial_pending(actual: &ErasedState) -> ErasedState;
}

pub auto trait ExactSharedBorrowState {}

impl<Marker> !ExactSharedBorrowState for StateUnionState<Marker> {}

impl<S> SharedBorrowState for S
where
    S: StateTrait + ExactSharedBorrowState,
{
    fn ensure_state(actual: &ErasedState) -> Result<(), SharedStateError> {
        if state_trait::is_state::<S>(actual) {
            Ok(())
        } else {
            Err(SharedStateError::WrongState {
                expected: core::any::type_name::<S>(),
                actual: actual.type_name(),
            })
        }
    }

    fn initial_pending(_actual: &ErasedState) -> ErasedState {
        state_trait::erased_state::<S>()
    }
}

impl<Marker> SharedBorrowState for StateUnionState<Marker>
where
    Marker: StateUnionRuntime + 'static,
    StateUnionState<Marker>: StateTrait,
{
    fn ensure_state(actual: &ErasedState) -> Result<(), SharedStateError> {
        if Marker::contains(&**actual) {
            Ok(())
        } else {
            Err(SharedStateError::WrongState {
                expected: Marker::expected_type_name(),
                actual: actual.type_name(),
            })
        }
    }

    fn initial_pending(actual: &ErasedState) -> ErasedState {
        state_trait::clone_erased(actual)
    }
}
