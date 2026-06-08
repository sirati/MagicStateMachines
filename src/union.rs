use crate::{
    SMove, SMut, SRef, State, StateMachineImpl, StateStorage, StateTrait, Transition,
    TransitionCallsite,
};
use core::any::Any;
use core::marker::PhantomData;
use core::ops::Deref;

/// State marker shared by every member of a generated state union.
#[doc(hidden)]
pub struct StateUnionState<Marker>(PhantomData<fn() -> Marker>);

/// Implemented by concrete states that can still identify their enum variant.
#[doc(hidden)]
pub auto trait StateUnionConcreteState {}

impl<Marker> !StateUnionConcreteState for StateUnionState<Marker> {}

/// Records that `State` belongs to a generated state union.
#[doc(hidden)]
pub trait StateUnionMember<State> {}

/// Selects the value-carrying discriminated state type for a union marker.
pub trait StateUnionDiscriminant: Sized {
    type Discriminator: Copy + 'static;

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

/// Value-carrying discriminated state for a generated union marker.
pub type DiscriminatedState<Storage, T, Marker> = State<
    SDiscriminated<Storage, <Marker as StateUnionDiscriminant>::Discriminator>,
    T,
    StateUnionState<Marker>,
>;

impl<Storage, T, Marker>
    State<
        SDiscriminated<Storage, <Marker as StateUnionDiscriminant>::Discriminator>,
        T,
        StateUnionState<Marker>,
    >
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
    discriminator: <Marker as StateUnionDiscriminant>::Discriminator,
) -> DiscriminatedState<Storage, T, Marker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant,
{
    State::from_inner(DiscriminatedInner {
        inner: Storage::retag(state.inner),
        discriminator,
    })
}

#[doc(hidden)]
#[must_use]
pub fn rediscriminate_union_state<Storage, T, FromMarker, ToMarker>(
    state: State<Storage, T, StateUnionState<FromMarker>>,
    discriminator: <ToMarker as StateUnionDiscriminant>::Discriminator,
) -> DiscriminatedState<Storage, T, ToMarker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    ToMarker: StateUnionDiscriminant,
{
    State::from_inner(DiscriminatedInner {
        inner: Storage::retag(state.inner),
        discriminator,
    })
}

#[doc(hidden)]
#[must_use]
pub fn undiscriminate_state<Storage, T, S, Discriminator>(
    state: State<SDiscriminated<Storage, Discriminator>, T, S>,
) -> State<Storage, T, S>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Discriminator: Copy + 'static,
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
pub fn discriminated_state_discriminator<Storage, T, S, Discriminator>(
    state: &State<SDiscriminated<Storage, Discriminator>, T, S>,
) -> Discriminator
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Discriminator: Copy + 'static,
{
    state.inner.discriminator
}

#[doc(hidden)]
#[must_use]
pub fn state_union_discriminator<Storage, T, S, Discriminator>(
    state: &State<Storage, T, S>,
) -> Option<Discriminator>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Discriminator: Copy + 'static,
{
    Storage::union_discriminator(&state.inner)
}

/// Storage backend that carries a discriminated union variant alongside another backend.
#[doc(hidden)]
pub struct SDiscriminated<Storage, Discriminator>(PhantomData<fn() -> (Storage, Discriminator)>);

#[doc(hidden)]
pub struct DiscriminatedInner<Inner, Discriminator> {
    pub(crate) inner: Inner,
    pub(crate) discriminator: Discriminator,
}

impl<Storage, Discriminator> StateStorage for SDiscriminated<Storage, Discriminator>
where
    Storage: StateStorage,
    Discriminator: Copy + 'static,
{
    type Inner<T, S>
        = DiscriminatedInner<Storage::Inner<T, S>, Discriminator>
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
            discriminator: inner.discriminator,
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
        To: StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        let discriminator = state.inner.discriminator;
        let state = State::<Storage, T, From>::from_inner(state.inner.inner);
        let state = Storage::complete_transition(state, args, callsite);
        State::from_inner(DiscriminatedInner {
            inner: state.inner,
            discriminator,
        })
    }

    fn complete_transition_after_effect<T, From, To>(
        state: State<Self, T, From>,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: StateTrait,
        To: StateTrait,
    {
        let discriminator = state.inner.discriminator;
        let state = State::<Storage, T, From>::from_inner(state.inner.inner);
        let state = Storage::complete_transition_after_effect(state, callsite);
        State::from_inner(DiscriminatedInner {
            inner: state.inner,
            discriminator,
        })
    }

    fn union_discriminator<T, State, OtherDiscriminator>(
        inner: &Self::Inner<T, State>,
    ) -> Option<OtherDiscriminator>
    where
        T: StateMachineImpl,
        OtherDiscriminator: Copy + 'static,
    {
        (&inner.discriminator as &dyn Any)
            .downcast_ref::<OtherDiscriminator>()
            .copied()
            .or_else(|| Storage::union_discriminator(&inner.inner))
    }
}

impl<Storage, Discriminator> SRef for SDiscriminated<Storage, Discriminator>
where
    Storage: SRef,
    Discriminator: Copy + 'static,
{
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        Storage::s_ref(&inner.inner)
    }
}

impl<Storage, Discriminator> SMut for SDiscriminated<Storage, Discriminator>
where
    Storage: SMut,
    Discriminator: Copy + 'static,
{
    fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
    where
        T: StateMachineImpl,
    {
        Storage::s_mut(&mut inner.inner)
    }
}

impl<Storage, Discriminator> SMove for SDiscriminated<Storage, Discriminator>
where
    Storage: SMove,
    Discriminator: Copy + 'static,
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

/// Selects the implementation effect shared by every member of a generated state union.
#[doc(hidden)]
pub trait StateUnionSharedEffect<T, To>: StateUnionDiscriminant
where
    T: StateMachineImpl,
    To: StateTrait,
{
    type Effect;
}

/// Applies the shared implementation effect for an erased union state.
#[doc(hidden)]
pub trait StateUnionSharedTransitionEffect<T, To, Args>: StateUnionSharedEffect<T, To>
where
    T: StateMachineImpl,
    To: StateTrait,
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
        To: StateTrait;
}

impl<Standin, Marker, To> Transition<StateUnionState<Marker>, To> for Standin
where
    Marker: StateUnionTransition<Standin, To>,
{
    type F = Marker::F;
}

/// A union variant that preserves its concrete state while exposing the joint state.
#[doc(hidden)]
pub struct StateUnionVariant<Storage, T, Concrete, Marker, Discriminator>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant<Discriminator = Discriminator>,
    Discriminator: Copy + 'static,
{
    state: DiscriminatedState<Storage, T, Marker>,
    concrete: PhantomData<fn() -> (Concrete, Discriminator)>,
}

impl<Storage, T, Concrete, Marker, Discriminator>
    StateUnionVariant<Storage, T, Concrete, Marker, Discriminator>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant<Discriminator = Discriminator> + StateUnionMember<Concrete>,
    Discriminator: Copy + 'static,
{
    #[must_use]
    pub fn new(state: State<Storage, T, Concrete>, discriminator: Discriminator) -> Self {
        Self {
            state: discriminate_state::<Storage, T, Concrete, Marker>(state, discriminator),
            concrete: PhantomData,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn from_erased(state: DiscriminatedState<Storage, T, Marker>) -> Self {
        Self {
            state,
            concrete: PhantomData,
        }
    }

    #[must_use]
    pub fn into_state(self) -> State<Storage, T, Concrete> {
        State::from_inner(Storage::retag(self.state.inner.inner))
    }

    #[must_use]
    pub fn into_erased(self) -> DiscriminatedState<Storage, T, Marker> {
        self.state
    }
}

impl<Storage, T, Concrete, Marker, Discriminator> Deref
    for StateUnionVariant<Storage, T, Concrete, Marker, Discriminator>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant<Discriminator = Discriminator>,
    Discriminator: Copy + 'static,
{
    type Target = State<SDiscriminated<Storage, Discriminator>, T, StateUnionState<Marker>>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
