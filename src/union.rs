use crate::{
    SMove, SMut, SRef, State, StateInference, StateMachineImpl, StateMarker, StateStorage,
    StateTrait, Transition, TransitionCallsite, TransitionProof, UnionStateKind, state_trait,
};
use core::{any::TypeId, marker::PhantomData};

/// State marker shared by every member of a generated state union.
#[doc(hidden)]
pub struct StateUnionState<Marker>(PhantomData<fn() -> Marker>);

impl<Marker> StateUnionState<Marker> {
    #[doc(hidden)]
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Implemented by concrete states that can still identify their enum variant.
#[doc(hidden)]
pub auto trait StateUnionConcreteState {}

impl<Marker> !StateUnionConcreteState for StateUnionState<Marker> {}

/// Records that `State` belongs to a generated state union.
#[doc(hidden)]
pub trait StateUnionMember<State> {}

/// Selects the value-carrying discriminated state type for a union marker.
pub trait StateUnionDiscriminant: Sized + StateMarker<Kind = UnionStateKind> {
    type Enum<Storage, T>
    where
        Storage: StateStorage,
        T: StateMachineImpl;

    #[doc(hidden)]
    fn discriminate<Storage, T>(
        state: DiscriminatedState<Storage, T, Self>,
    ) -> Self::Enum<Storage, T>
    where
        Storage: StateStorage,
        T: StateMachineImpl;
}

/// Marks a state as being a concrete marker or a member of a generated state union.
pub trait In<Marker>: StateTrait + StateMarker
where
    Marker: StateMarker,
{
    #[must_use]
    fn into_enum<Storage, T>(
        state: State<Storage, T, Self>,
    ) -> DiscriminatedState<Storage, T, Marker>
    where
        Self: Sized,
        Storage: StateStorage,
        T: StateMachineImpl,
        Marker: StateUnionDiscriminant;

    #[doc(hidden)]
    #[must_use]
    fn prove<Storage, T, To>()
    -> TransitionProof<Storage, T, Self, Marker, To, <Marker as StateMarker>::Kind>
    where
        Self: Sized,
        Storage: StateStorage,
        T: StateMachineImpl,
        To: StateTrait + StateMarker,
    {
        TransitionProof::new()
    }
}

impl<StateMarkerType> In<StateMarkerType> for StateMarkerType
where
    StateMarkerType: StateTrait + StateMarker,
{
    fn into_enum<Storage, T>(
        _state: State<Storage, T, Self>,
    ) -> DiscriminatedState<Storage, T, StateMarkerType>
    where
        Self: Sized,
        Storage: StateStorage,
        T: StateMachineImpl,
        StateMarkerType: StateUnionDiscriminant,
    {
        unreachable!("concrete identity states are not generated state unions")
    }
}

/// Value-carrying discriminated state for a generated union marker.
pub type DiscriminatedState<Storage, T, Marker> = State<
    SDiscriminated<Storage>,
    T,
    StateUnionState<Marker>,
>;

impl<Storage, T, Marker>
    State<SDiscriminated<Storage>, T, StateUnionState<Marker>>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant,
{
    #[must_use]
    pub fn discriminate(self) -> <Marker as StateUnionDiscriminant>::Enum<Storage, T> {
        Marker::discriminate(self)
    }
}

#[doc(hidden)]
#[must_use]
pub fn discriminate_state<Storage, T, From, Marker>(
    state: State<Storage, T, From>,
) -> DiscriminatedState<Storage, T, Marker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    From: crate::ConcreteStateTrait,
    Marker: StateUnionDiscriminant,
{
    let inference =
        <<Storage::Inference as crate::InferenceKind>::Inference as crate::StateInference>::new::<
            Storage,
            T,
            From,
        >(&state.inner);
    State::from_inner(DiscriminatedInner {
        inner: Storage::retag(state.inner),
        inference,
    })
}

#[doc(hidden)]
#[must_use]
pub fn rediscriminate_union_state<Storage, T, FromMarker, ToMarker>(
    state: State<Storage, T, StateUnionState<FromMarker>>,
) -> DiscriminatedState<Storage, T, ToMarker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    FromMarker: StateUnionDiscriminant,
    StateUnionState<FromMarker>: StateTrait,
    ToMarker: StateUnionDiscriminant,
{
    let inferred_state = Storage::inferred_state(&state.inner);
    let inference =
        <<Storage::Inference as crate::InferenceKind>::Inference as crate::StateInference>::from_erased(
            inferred_state,
        );
    State::from_inner(DiscriminatedInner {
        inner: Storage::retag(state.inner),
        inference,
    })
}

#[doc(hidden)]
#[must_use]
pub fn undiscriminate_state<Storage, T, S>(
    state: State<SDiscriminated<Storage>, T, S>,
) -> State<Storage, T, S>
where
    Storage: StateStorage,
    T: StateMachineImpl,
{
    State::from_inner(state.inner.inner)
}

#[doc(hidden)]
#[must_use]
pub fn concretize_discriminated_state<Storage, T, Marker, Concrete>(
    state: DiscriminatedState<Storage, T, Marker>,
) -> State<Storage, T, Concrete>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant,
{
    State::from_inner(Storage::retag(state.inner.inner))
}

#[doc(hidden)]
#[must_use]
pub fn discriminated_state_marker<Storage, T, S>(
    state: &State<SDiscriminated<Storage>, T, S>,
) -> state_trait::ErasedState
where
    Storage: StateStorage,
    T: StateMachineImpl,
    S: StateTrait,
{
    state.inner.inference.state::<Storage, T, S>(&state.inner.inner)
}

#[doc(hidden)]
#[must_use]
pub fn state_union_marker<Storage, T, S>(
    state: &State<Storage, T, S>,
) -> state_trait::ErasedState
where
    Storage: StateStorage,
    T: StateMachineImpl,
    S: StateTrait,
{
    Storage::inferred_state(&state.inner)
}

#[doc(hidden)]
#[must_use]
pub fn erased_state_type_id(state: &state_trait::ErasedState) -> TypeId {
    state.type_id()
}

/// Storage backend that carries a discriminated union variant alongside another backend.
#[doc(hidden)]
pub struct SDiscriminated<Storage>(PhantomData<fn() -> Storage>);

#[doc(hidden)]
pub struct DiscriminatedInner<Inner, Inference> {
    pub(crate) inner: Inner,
    pub(crate) inference: Inference,
}

impl<Storage> StateStorage for SDiscriminated<Storage>
where
    Storage: StateStorage,
{
    type Inference = Storage::Inference;

    type Inner<T, S>
        = DiscriminatedInner<
            Storage::Inner<T, S>,
            <Storage::Inference as crate::InferenceKind>::Inference,
        >
    where
        T: StateMachineImpl;
    type Machine<T>
        = Storage::Machine<T>
    where
        T: StateMachineImpl;

    fn retag<T, From, To>(inner: Self::Inner<T, From>) -> Self::Inner<T, To>
    where
        T: StateMachineImpl,
    {
        DiscriminatedInner {
            inner: Storage::retag(inner.inner),
            inference: inner.inference,
        }
    }

    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: StateTrait,
        To: crate::ConcreteStateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
    {
        let state = State::<Storage, T, From>::from_inner(state.inner.inner);
        let state = Storage::complete_transition(state, args, callsite);
        let inference =
            <<Storage::Inference as crate::InferenceKind>::Inference as crate::StateInference>::new::<
                Storage,
                T,
                To,
            >(&state.inner);
        State::from_inner(DiscriminatedInner {
            inner: state.inner,
            inference,
        })
    }

    fn complete_transition_after_effect<T, From, To>(
        state: State<Self, T, From>,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: StateTrait,
        To: crate::ConcreteStateTrait,
    {
        let state = State::<Storage, T, From>::from_inner(state.inner.inner);
        let state = Storage::complete_transition_after_effect(state, callsite);
        let inference =
            <<Storage::Inference as crate::InferenceKind>::Inference as crate::StateInference>::new::<
                Storage,
                T,
                To,
            >(&state.inner);
        State::from_inner(DiscriminatedInner {
            inner: state.inner,
            inference,
        })
    }

    fn inferred_state<T, State>(inner: &Self::Inner<T, State>) -> state_trait::ErasedState
    where
        T: StateMachineImpl,
        State: StateTrait,
    {
        inner.inference.state::<Storage, T, State>(&inner.inner)
    }
}

impl<Storage> SRef for SDiscriminated<Storage>
where
    Storage: SRef,
{
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        Storage::s_ref(&inner.inner)
    }
}

impl<Storage> SMut for SDiscriminated<Storage>
where
    Storage: SMut,
{
    fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
    where
        T: StateMachineImpl,
    {
        Storage::s_mut(&mut inner.inner)
    }
}

impl<Storage> SMove for SDiscriminated<Storage>
where
    Storage: SMove,
{
}

/// Converts a concrete or already-erased member state into a union state.
#[doc(hidden)]
pub trait StateUnionErased<Marker>: StateTrait {
    fn into_union_erased<Storage, T>(
        state: State<Storage, T, Self>,
    ) -> DiscriminatedState<Storage, T, Marker>
    where
        Self: Sized,
        Storage: StateStorage,
        T: StateMachineImpl,
        Marker: StateUnionDiscriminant;
}

/// Runtime membership check for shared erased-state borrows.
#[doc(hidden)]
pub trait StateUnionRuntime {
    fn contains(state: &dyn StateTrait) -> bool;
    fn expected_type_name() -> &'static str;
}

/// Resolves transitions supported by every member of a generated state union.
#[doc(hidden)]
pub trait StateUnionTransition<Standin, To> {
    type F;
}

/// Proof that a state marker is viewed through a specific generated union trait.
#[doc(hidden)]
pub trait StateUnionProofMembership<Marker>: StateUnionErased<Marker>
where
    Marker: StateUnionDiscriminant,
{
}

/// Selects the union marker used to prove a transition to this target state.
#[doc(hidden)]
pub trait StateUnionProofTarget<T, From>: crate::ConcreteStateTrait + Sized
where
    T: StateMachineImpl,
    From: StateTrait,
{
    type Marker: StateUnionDiscriminant + StateUnionSharedEffect<T, Self>;
}

/// Selects the implementation effect shared by every member of a generated state union.
#[doc(hidden)]
pub trait StateUnionSharedEffect<T, To>: StateUnionDiscriminant
where
    T: StateMachineImpl,
    To: crate::ConcreteStateTrait,
{
    type Effect;
}

/// Applies the shared implementation effect for an erased union state.
#[doc(hidden)]
pub trait StateUnionSharedTransitionEffect<T, To, Args>: StateUnionSharedEffect<T, To>
where
    T: StateMachineImpl,
    To: crate::ConcreteStateTrait,
{
    fn apply(value: &mut T, args: Args);
}

/// Dispatches a discriminated union transition to the concrete state's effect.
#[doc(hidden)]
pub trait StateUnionDiscriminatedTransition<T, To, Args>: StateUnionDiscriminant
where
    T: StateMachineImpl,
{
    fn transition<Storage>(
        state: DiscriminatedState<Storage, T, Self>,
        args: Args,
        callsite: TransitionCallsite,
    ) -> State<Storage, T, To>
    where
        Storage: SMut,
        To: crate::ConcreteStateTrait;
}

impl<Standin, Marker, To> Transition<StateUnionState<Marker>, To> for Standin
where
    Marker: StateUnionTransition<Standin, To>,
{
    type F = Marker::F;
}
