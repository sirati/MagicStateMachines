mod owned;

use crate::{Initial, StateMachineImpl, Transition};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
#[cfg(feature = "tracing")]
use core::panic::Location;

pub use owned::{
    SOwned, StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox,
    StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc,
};

fn retag_owned<T, From, To>(inner: crate::StateOwned<T, From>) -> crate::StateOwned<T, To> {
    crate::StateOwned {
        value: inner.value,
        state: PhantomData,
        #[cfg(feature = "tracing")]
        trace: inner.trace,
    }
}

type StateMarker<Storage, T, S> = PhantomData<fn() -> (Storage, T, S)>;
type TransitionMarker<Storage, T, From, To> = PhantomData<fn() -> (Storage, T, From, To)>;

/// Selects the implementation-side effect for a declared transition.
#[doc(hidden)]
pub trait TransitionEffectSelector<From, To>: StateMachineImpl {
    type Effect;
}

/// Applies implementation-side transition effects before the state is retagged.
#[doc(hidden)]
pub trait TransitionEffect<T, From, To, Args>
where
    T: StateMachineImpl,
{
    fn apply(value: &mut T, args: Args);
}

/// Storage backend used by [`State`].
pub trait StateStorage: Sized {
    /// Concrete state representation used by this storage backend.
    type Inner<T, S>
    where
        T: StateMachineImpl;

    /// Type that carries the state-machine implementation contract.
    type Machine<T>: StateMachineImpl<Standin = T::Standin, Impl = T::Impl, TransitionToken = T::TransitionToken>
    where
        T: StateMachineImpl;

    #[doc(hidden)]
    fn retag<T, From, To>(inner: Self::Inner<T, From>) -> Self::Inner<T, To>
    where
        T: StateMachineImpl;

    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple;

    #[doc(hidden)]
    fn complete_transition_after_effect<T, From, To>(
        state: State<Self, T, From>,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait;
}

pub(crate) fn retag_state<Storage, T, From, To>(
    state: State<Storage, T, From>,
) -> State<Storage, T, To>
where
    Storage: StateStorage,
    T: StateMachineImpl,
{
    State {
        inner: Storage::retag(state.inner),
        marker: PhantomData,
    }
}

#[doc(hidden)]
pub fn complete_transition_after_effect<Storage, T, From, To>(
    state: State<Storage, T, From>,
    callsite: TransitionCallsite,
) -> State<Storage, T, To>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    From: crate::StateTrait,
    To: crate::StateTrait,
{
    Storage::complete_transition_after_effect(state, callsite)
}

/// Storage backend that can create initial owned state.
pub trait StateStorageNew: StateStorage {
    fn new<T, State>(value: T) -> Self::Inner<T, State>
    where
        T: StateMachineImpl,
        <Self::Machine<T> as StateMachineImpl>::Standin: Initial<State>;
}

/// Storage backend that can expose a runtime reference.
pub trait SRef: StateStorage {
    fn s_ref<T, State>(inner: &Self::Inner<T, State>) -> &T
    where
        T: StateMachineImpl;
}

/// Storage backend that can expose a mutable runtime reference.
pub trait SMut: SRef {
    fn s_mut<T, State>(inner: &mut Self::Inner<T, State>) -> &mut T
    where
        T: StateMachineImpl;
}

/// Storage backend whose state token can be consumed by value.
pub trait SMove: StateStorage {}

/// A state token parameterized by its storage backend.
pub struct State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub(crate) inner: Storage::Inner<T, S>,
    pub(crate) marker: StateMarker<Storage, T, S>,
}

/// A result whose success and error values are states of the same machine.
#[allow(type_alias_bounds)]
pub type SResult<Storage, T, OkState, ErrState>
where
    Storage: StateStorage,
    T: StateMachineImpl,
= Result<State<Storage, T, OkState>, State<Storage, T, ErrState>>;

/// A callable transition for generic [`State`] storage.
pub struct StateTransitionCall<Storage, T, From, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    state: State<Storage, T, From>,
    #[cfg(feature = "tracing")]
    callsite: &'static Location<'static>,
    marker: TransitionMarker<Storage, T, From, To>,
}

/// A callable transition that first routes through implementation-owned effects.
#[doc(hidden)]
pub struct EffectTransitionCall<Storage, T, From, To, Effect>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    state: State<Storage, T, From>,
    callsite: TransitionCallsite,
    marker: PhantomData<fn() -> (To, Effect)>,
}

#[cfg(feature = "tracing")]
pub type TransitionCallsite = &'static Location<'static>;

#[cfg(not(feature = "tracing"))]
pub type TransitionCallsite = ();

#[doc(hidden)]
#[track_caller]
pub fn transition_callsite() -> TransitionCallsite {
    #[cfg(feature = "tracing")]
    {
        Location::caller()
    }
    #[cfg(not(feature = "tracing"))]
    {}
}

impl<Storage, T, From, To, Args> FnOnce<Args> for StateTransitionCall<Storage, T, From, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    From: crate::StateTrait,
    To: crate::StateTrait,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        Storage::complete_transition(self.state, args, {
            #[cfg(feature = "tracing")]
            {
                self.callsite
            }
            #[cfg(not(feature = "tracing"))]
            {}
        })
    }
}

impl<Storage, T, From, To, Args, Effect> FnOnce<Args>
    for EffectTransitionCall<Storage, T, From, To, Effect>
where
    T: StateMachineImpl,
    Storage: SMut,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    From: crate::StateTrait,
    To: crate::StateTrait,
    Effect: TransitionEffect<T, From, To, Args>,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
        Effect::apply(Storage::s_mut(&mut self.state.inner), args);
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

/// Creates a callable transition for generic state storage.
#[must_use]
#[track_caller]
pub fn transition_state<Storage, T, S, Next>(
    state: State<Storage, T, S>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> StateTransitionCall<Storage, T, S, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    S: crate::StateTrait,
    Next: crate::StateTrait,
{
    StateTransitionCall {
        state,
        #[cfg(feature = "tracing")]
        callsite: Location::caller(),
        marker: PhantomData,
    }
}

/// Creates a callable transition that runs implementation-side effects first.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_effect<Storage, T, S, Next, Effect>(
    state: State<Storage, T, S>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> EffectTransitionCall<Storage, T, S, Next, Effect>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    S: crate::StateTrait,
    Next: crate::StateTrait,
{
    EffectTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

impl<Storage, T, S> State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: StateStorageNew,
{
    /// Wraps an implementation in a state declared initial by its definition.
    #[must_use]
    pub fn new(value: T) -> Self
    where
        <Storage::Machine<T> as StateMachineImpl>::Standin: Initial<S>,
    {
        Self {
            inner: Storage::new(value),
            marker: PhantomData,
        }
    }
}

impl<Storage, T, S> State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub(crate) fn from_inner(inner: Storage::Inner<T, S>) -> Self {
        Self {
            inner,
            marker: PhantomData,
        }
    }
}

impl<Storage, T, S> Deref for State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: SRef,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Storage::s_ref(&self.inner)
    }
}

impl<Storage, T, S> DerefMut for State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: SMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Storage::s_mut(&mut self.inner)
    }
}
