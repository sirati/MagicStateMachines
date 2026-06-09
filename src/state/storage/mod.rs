mod owned;

use crate::{
    ConcreteStateKind, DiscriminatedState, Initial, StateConcreteProvenState, StateMachineImpl,
    StateConcreteTransitionProof, StateKind, StateUnionDiscriminant,
    StateUnionDiscriminatedTransition, StateUnionErased, StateUnionProofTarget,
    StateUnionProvenState, StateUnionSharedEffect, StateUnionSharedTransitionEffect,
    StateUnionTransitionProof, StateWithProof, Transition, UnionStateKind,
};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
#[cfg(feature = "tracing")]
use core::panic::Location;
use core::pin::Pin;
use std::rc::UniqueRc;
use std::sync::UniqueArc;

pub use owned::{
    SOwned, StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox,
    StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc,
};

pub type SBox<T, S> = State<StorageStateOwnedBox, T, S>;
pub type SPinBox<T, S> = State<StorageStateOwnedPinBox, T, S>;

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

    #[doc(hidden)]
    fn union_discriminator<T, State, Discriminator>(
        inner: &Self::Inner<T, State>,
    ) -> Option<Discriminator>
    where
        T: StateMachineImpl,
        Discriminator: Copy + 'static,
    {
        let _ = inner;
        None
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

/// A callable concrete transition selected by [`StateKind`].
#[doc(hidden)]
pub struct ConcreteProofTransitionCall<Storage, T, From, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    state: State<Storage, T, From>,
    callsite: TransitionCallsite,
    marker: PhantomData<fn() -> To>,
}

/// A callable transition selected by the receiver state's marker kind.
#[doc(hidden)]
pub struct KindProofTransitionCall<Storage, T, From, Marker, To, Kind>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    Marker: crate::StateMarker,
    To: crate::StateTrait + crate::StateMarker<Kind = ConcreteStateKind>,
    Kind: StateKind,
{
    state: State<Storage, T, From>,
    callsite: TransitionCallsite,
    marker: PhantomData<fn() -> (Marker, To, Kind)>,
}

/// A callable transition proven through a generated state union.
#[doc(hidden)]
pub struct StateUnionProofTransitionCall<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    Marker: StateUnionDiscriminant,
    To: crate::StateTrait,
{
    state: State<Storage, T, From>,
    callsite: TransitionCallsite,
    marker: PhantomData<fn() -> (Marker, To)>,
}

/// A callable discriminated-union transition that returns to the original storage after transition.
#[doc(hidden)]
pub struct DiscriminatedTransitionCall<Storage, T, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    Marker: StateUnionDiscriminant,
{
    state: DiscriminatedState<Storage, T, Marker>,
    callsite: TransitionCallsite,
    marker: PhantomData<fn() -> To>,
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

impl<Storage, T, From, To, Args> FnOnce<Args>
    for ConcreteProofTransitionCall<Storage, T, From, To>
where
    T: StateMachineImpl + TransitionEffectSelector<From, To>,
    Storage: SMut,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    From: crate::StateTrait,
    To: crate::StateTrait,
    <T as TransitionEffectSelector<From, To>>::Effect: TransitionEffect<T, From, To, Args>,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
        <T as TransitionEffectSelector<From, To>>::Effect::apply(
            Storage::s_mut(&mut self.state.inner),
            args,
        );
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, Marker, To, Args> FnOnce<Args>
    for KindProofTransitionCall<Storage, T, From, Marker, To, ConcreteStateKind>
where
    T: StateMachineImpl + TransitionEffectSelector<From, To>,
    Storage: SMut,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    From: crate::StateTrait + crate::StateMarker<Kind = ConcreteStateKind>,
    Marker: crate::StateMarker,
    To: crate::StateTrait + crate::StateMarker<Kind = ConcreteStateKind>,
    <T as TransitionEffectSelector<From, To>>::Effect: TransitionEffect<T, From, To, Args>,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
        <T as TransitionEffectSelector<From, To>>::Effect::apply(
            Storage::s_mut(&mut self.state.inner),
            args,
        );
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, Marker, To, Args> FnOnce<Args>
    for KindProofTransitionCall<Storage, T, From, Marker, To, UnionStateKind>
where
    T: StateMachineImpl,
    Storage: SMut,
    From: crate::StateTrait + crate::StateMarker + crate::In<Marker>,
    Marker: StateUnionDiscriminant + StateUnionDiscriminatedTransition<T, To, Args>,
    Args: core::marker::Tuple,
    To: crate::StateTrait + crate::StateMarker<Kind = ConcreteStateKind>,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        let state = <From as crate::In<Marker>>::into_enum(self.state);
        Marker::transition(state, args, self.callsite)
    }
}

impl<Storage, T, From, Marker, To, Args> FnOnce<Args>
    for StateUnionProofTransitionCall<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: SMut,
    From: StateUnionErased<Marker>,
    Marker: StateUnionSharedTransitionEffect<T, To, Args>,
    Args: core::marker::Tuple,
    To: crate::StateTrait,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
        Marker::apply(Storage::s_mut(&mut self.state.inner), args);
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, Marker, To, Args> FnOnce<Args>
    for DiscriminatedTransitionCall<Storage, T, Marker, To>
where
    T: StateMachineImpl,
    Storage: SMut,
    Marker: StateUnionDiscriminatedTransition<T, To, Args>,
    Args: core::marker::Tuple,
    To: crate::StateTrait,
{
    type Output = State<Storage, T, To>;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        Marker::transition(self.state, args, self.callsite)
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

/// Creates a callable transition from a union-membership proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_union_proof<Storage, T, S, Marker, Next>(
    proven: StateUnionProvenState<Storage, T, S, Marker, Next>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> StateUnionProofTransitionCall<Storage, T, S, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: StateUnionErased<Marker>,
    Marker: StateUnionSharedEffect<T, Next>,
    Next: crate::StateTrait,
{
    StateUnionProofTransitionCall {
        state: proven.state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Creates a callable transition from a concrete-state proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_concrete_proof<Storage, T, S, Marker, Next>(
    proven: StateConcreteProvenState<Storage, T, S, Marker, Next>,
    token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> EffectTransitionCall<
    Storage,
    T,
    S,
    Next,
    <T as TransitionEffectSelector<S, Next>>::Effect,
>
where
    T: StateMachineImpl + TransitionEffectSelector<S, Next>,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    Marker: StateUnionDiscriminant,
    S: crate::StateTrait,
    Next: crate::StateTrait,
{
    transition_state_with_effect(proven.state, token)
}

/// Creates a callable transition from an unresolved concrete-state proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_concrete_transition_proof<Storage, T, S, Marker, Next>(
    proven: StateWithProof<Storage, T, S, StateConcreteTransitionProof<T, S, Marker, Next>>,
    token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> EffectTransitionCall<
    Storage,
    T,
    S,
    Next,
    <T as TransitionEffectSelector<S, Next>>::Effect,
>
where
    T: StateMachineImpl + TransitionEffectSelector<S, Next>,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    Marker: StateUnionDiscriminant,
    S: crate::StateTrait,
    Next: crate::StateTrait,
{
    let StateWithProof {
        state,
        proof: _proof,
    } = proven;
    transition_state_with_effect(state, token)
}

/// Creates a callable transition from an unresolved union-membership proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_union_transition_proof<Storage, T, S, Marker, Next>(
    proven: StateWithProof<Storage, T, S, StateUnionTransitionProof<T, S, Marker, Next>>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> StateUnionProofTransitionCall<Storage, T, S, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: StateUnionErased<Marker>,
    Marker: StateUnionSharedEffect<T, Next>,
    Next: crate::StateTrait,
{
    let StateWithProof {
        state,
        proof: _proof,
    } = proven;
    StateUnionProofTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Creates a callable erased transition from a kind-selected proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_erased_transition_proof<Storage, T, S, Marker, Next, Proof>(
    proven: StateWithProof<Storage, T, S, Proof>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> StateUnionProofTransitionCall<Storage, T, S, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait,
    Marker: StateUnionDiscriminant,
    Next: crate::StateTrait,
{
    let StateWithProof {
        state,
        proof: _proof,
    } = proven;
    StateUnionProofTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Creates a concrete callable transition from a kind-selected proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_concrete_kind_proof<Storage, T, S, Marker, Next, Kind>(
    proven: StateWithProof<Storage, T, S, crate::TransitionProof<Storage, T, S, Marker, Next, Kind>>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> ConcreteProofTransitionCall<Storage, T, S, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait,
    Marker: crate::StateMarker,
    Next: crate::StateTrait + crate::StateMarker<Kind = crate::ConcreteStateKind>,
    Kind: StateKind,
{
    let StateWithProof {
        state,
        proof: _proof,
    } = proven;
    ConcreteProofTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Creates a callable transition from a kind-selected proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_kind_proof<Storage, T, S, Marker, Next, Kind>(
    proven: StateWithProof<Storage, T, S, crate::TransitionProof<Storage, T, S, Marker, Next, Kind>>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> KindProofTransitionCall<Storage, T, S, Marker, Next, Kind>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait,
    Marker: crate::StateMarker,
    Next: crate::StateTrait + crate::StateMarker<Kind = crate::ConcreteStateKind>,
    Kind: StateKind,
{
    let StateWithProof {
        state,
        proof: _proof,
    } = proven;
    KindProofTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

#[doc(hidden)]
pub fn transition_concrete_after_effect<Storage, T, From, To, Args, Effect>(
    mut state: State<Storage, T, From>,
    args: Args,
    callsite: TransitionCallsite,
) -> State<Storage, T, To>
where
    T: StateMachineImpl,
    Storage: SMut,
    From: crate::StateTrait,
    To: crate::StateTrait,
    Effect: TransitionEffect<T, From, To, Args>,
{
    Effect::apply(Storage::s_mut(&mut state.inner), args);
    Storage::complete_transition_after_effect(state, callsite)
}

/// Creates a callable discriminated-union transition that runs the exact concrete effect.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_discriminated_state<Storage, T, Marker, Next>(
    state: DiscriminatedState<Storage, T, Marker>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> DiscriminatedTransitionCall<Storage, T, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    Marker: StateUnionDiscriminant,
    Next: crate::StateTrait,
{
    DiscriminatedTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Binds a generated union-transition proof selected by the target state.
#[doc(hidden)]
#[must_use]
pub fn proven_state<To, Storage, T, S>(
    state: State<Storage, T, S>,
) -> StateUnionProvenState<Storage, T, S, <To as StateUnionProofTarget<T, S>>::Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: StateUnionErased<<To as StateUnionProofTarget<T, S>>::Marker>,
    To: StateUnionProofTarget<T, S>,
{
    StateUnionProvenState {
        state,
        marker: PhantomData,
    }
}

/// Binds a generated union-transition proof selected by a union marker.
#[doc(hidden)]
#[must_use]
pub fn proven_union_state<Marker, To, Storage, T, S>(
    state: State<Storage, T, S>,
) -> StateUnionProvenState<Storage, T, S, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: StateUnionErased<Marker>,
    Marker: StateUnionSharedEffect<T, To>,
    To: crate::StateTrait,
{
    StateUnionProvenState {
        state,
        marker: PhantomData,
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

    /// Binds a generated union-transition proof to this state.
    #[doc(hidden)]
    #[must_use]
    pub fn with<Marker, To, Kind>(
        self,
        proof: crate::TransitionProof<Storage, T, S, Marker, To, Kind>,
    ) -> StateWithProof<Storage, T, S, crate::TransitionProof<Storage, T, S, Marker, To, Kind>>
    where
        S: crate::StateTrait,
        Marker: crate::StateMarker,
        To: crate::StateTrait + crate::StateMarker<Kind = crate::ConcreteStateKind>,
        Kind: crate::StateKind,
    {
        StateWithProof { state: self, proof }
    }
}

impl<T, S> State<StorageStateOwned, T, S>
where
    T: StateMachineImpl,
{
    /// Wraps an implementation in an initial directly owned state.
    #[must_use]
    pub fn new(value: T) -> Self
    where
        T::Standin: Initial<S>,
    {
        State {
            inner: <StorageStateOwned as StateStorageNew>::new(value),
            marker: PhantomData,
        }
    }
}

impl<T, S> State<StorageStateOwnedBox, T, S>
where
    T: StateMachineImpl,
{
    /// Moves a directly owned state into `Box` storage without changing its state.
    #[must_use]
    pub fn new(state: State<StorageStateOwned, T, S>) -> Self {
        State {
            inner: crate::StateOwned {
                value: Box::new(state.inner.value),
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: state.inner.trace,
            },
            marker: PhantomData,
        }
    }

    /// Moves this boxed state back into direct owned storage.
    #[must_use]
    pub fn unbox(state: Self) -> State<StorageStateOwned, T, S> {
        State {
            inner: crate::StateOwned {
                value: *state.inner.value,
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: state.inner.trace,
            },
            marker: PhantomData,
        }
    }
}

impl<T, S> State<StorageStateOwnedPinBox, T, S>
where
    T: StateMachineImpl,
{
    /// Pins an already boxed state in place without changing its state.
    #[must_use]
    pub fn new(state: State<StorageStateOwnedBox, T, S>) -> Self {
        State {
            inner: crate::StateOwned {
                value: Box::into_pin(state.inner.value),
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: state.inner.trace,
            },
            marker: PhantomData,
        }
    }

    /// Converts pinned box storage back to box storage when the runtime is `Unpin`.
    #[must_use]
    pub fn into_boxed(state: Self) -> State<StorageStateOwnedBox, T, S>
    where
        T: Unpin,
    {
        State {
            inner: crate::StateOwned {
                value: Pin::into_inner(state.inner.value),
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: state.inner.trace,
            },
            marker: PhantomData,
        }
    }
}

impl<T, S> State<StorageStateOwnedUniqueRc, T, S>
where
    T: StateMachineImpl,
{
    /// Moves a directly owned state into `UniqueRc` storage without changing its state.
    #[must_use]
    pub fn new(state: State<StorageStateOwned, T, S>) -> Self {
        State {
            inner: crate::StateOwned {
                value: UniqueRc::new(state.inner.value),
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: state.inner.trace,
            },
            marker: PhantomData,
        }
    }
}

impl<T, S> State<StorageStateOwnedUniqueArc, T, S>
where
    T: StateMachineImpl,
{
    /// Moves a directly owned state into `UniqueArc` storage without changing its state.
    #[must_use]
    pub fn new(state: State<StorageStateOwned, T, S>) -> Self {
        State {
            inner: crate::StateOwned {
                value: UniqueArc::new(state.inner.value),
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: state.inner.trace,
            },
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
