use crate::{
    SMove, SMut, SPinMut, SPinRef, SRef, State, StateInference, StateMachineImpl, StateMarker,
    StateStorage, StateTrait, Transition, TransitionCallsite, TransitionProof, UnionStateKind,
    state_trait,
};
use core::{any::TypeId, marker::PhantomData, pin::Pin};

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

/// Selects the value-carrying enum type for a union marker.
///
/// `StateUnion!` implements this for the generated marker. For
/// `StateUnion!(Online: Connected | Authenticated)`, the associated `Enum` is
/// `OnlineEnum<Storage, T>` unless a custom enum name was supplied.
///
/// Users normally do not implement this trait manually. It is useful in
/// generic APIs that need to name the enum associated with a marker:
///
/// ```ignore
/// type OnlineValue<S, T> =
///     <Online as magicstatemachines::StateUnionDiscriminant>::Enum<S, T>;
/// ```
pub trait StateUnionDiscriminant: Sized + StateMarker<Kind = UnionStateKind> {
    /// Generated enum that can hold any concrete member state of this union.
    ///
    /// For `StateUnion!(Online: Connected | Authenticated)`, this is
    /// `OnlineEnum<Storage, T>` unless a custom enum name was supplied.
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
///
/// Every concrete state implements `In<Self>`. Generated union membership
/// traits such as `InOnline` extend `In<Online>` and add sealing/proof bounds.
///
/// Prefer the generated trait (`InOnline`) in normal function signatures. The
/// generated trait carries the sealed union contract and any generated
/// super-union relationships, so a value accepted as `impl InOnline` can also
/// be used with APIs that accept wider generated traits such as `impl InAll`.
/// A bare `impl In<Online>` proves only membership in that one marker and does
/// not give Rust the same widened trait bounds.
///
/// ```ignore
/// fn endpoint<S>(self: &State<S, Connection, impl InOnline>) -> &str
/// where
///     S: SRef,
/// {
///     &self.endpoint
/// }
/// ```
///
/// Use the generic form (`In<Online>`) when the marker is itself a type
/// parameter, or when calling the associated conversion function explicitly.
/// It is a lower-level building block, not the best ergonomic bound for public
/// methods:
///
/// ```ignore
/// fn to_online<S, Current>(
///     state: State<S, Connection, Current>,
/// ) -> DiscriminatedState<S, Connection, Online>
/// where
///     S: StateStorage,
///     Current: In<Online>,
/// {
///     <Current as In<Online>>::into_discriminated(state)
/// }
/// ```
pub trait In<Marker>: StateTrait + StateMarker
where
    Marker: StateMarker,
{
    /// Converts a concrete member state into the union's discriminated state.
    ///
    /// The returned [`DiscriminatedState`] keeps enough runtime information to
    /// recover the concrete enum variant later with `discriminate()`.
    #[must_use]
    fn into_discriminated<Storage, T>(
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
    fn into_discriminated<Storage, T>(
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

/// Union-typed state that still remembers its concrete variant.
///
/// This is a type alias for `State<SDiscriminated<Storage>, T,
/// StateUnionState<Marker>>`. The outer state marker is the union, so methods
/// can accept it as "any online state". The storage wrapper remembers the
/// exact concrete member, so `discriminate()` can later recover the generated
/// enum:
///
/// ```ignore
/// let online: DiscriminatedState<SOwned, Connection, Online> =
///     <Connected as In<Online>>::into_discriminated(connected);
///
/// match online.discriminate() {
///     OnlineEnum::Connected(connected) => {
///         // `connected` is State<SOwned, Connection, Connected>.
///     }
///     OnlineEnum::Authenticated(authenticated) => {
///         // `authenticated` is State<SOwned, Connection, Authenticated>.
///     }
/// }
/// ```
///
/// Dynamic union transitions use the same discriminator internally. That is
/// why `transition!(dyn Online self)` can run different bodies for different
/// union members.
pub type DiscriminatedState<Storage, T, Marker> =
    State<SDiscriminated<Storage>, T, StateUnionState<Marker>>;

impl<Storage, T, Marker> State<SDiscriminated<Storage>, T, StateUnionState<Marker>>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionDiscriminant,
{
    /// Recovers the generated enum for this discriminated union state.
    ///
    /// This is the runtime branch point: after matching on the enum, each
    /// variant can be converted back into its concrete state.
    ///
    /// ```ignore
    /// match online.discriminate() {
    ///     OnlineEnum::Connected(connected) => {
    ///         let authenticated = connected.authenticate("alice");
    ///     }
    ///     OnlineEnum::Authenticated(authenticated) => {
    ///         let connected = authenticated.logout();
    ///     }
    /// }
    /// ```
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
    state
        .inner
        .inference
        .state::<Storage, T, S>(&state.inner.inner)
}

#[doc(hidden)]
#[must_use]
pub fn state_union_marker<Storage, T, S>(state: &State<Storage, T, S>) -> state_trait::ErasedState
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

impl<Storage> SPinRef for SDiscriminated<Storage>
where
    Storage: SPinRef,
{
    fn s_pin_ref<T, S>(inner: &Self::Inner<T, S>) -> core::pin::Pin<&T>
    where
        T: StateMachineImpl,
    {
        Storage::s_pin_ref(&inner.inner)
    }
}

impl<Storage> SPinMut for SDiscriminated<Storage>
where
    Storage: SPinMut,
{
    fn s_pin_mut<T, S>(inner: &mut Self::Inner<T, S>) -> core::pin::Pin<&mut T>
    where
        T: StateMachineImpl,
    {
        Storage::s_pin_mut(&mut inner.inner)
    }
}

impl<Storage> SMove for SDiscriminated<Storage> where Storage: SMove {}

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

/// Selects the pinned implementation effect shared by every member of a generated state union.
#[doc(hidden)]
pub trait StateUnionSharedPinnedEffect<T, To>: StateUnionDiscriminant
where
    T: StateMachineImpl,
    To: crate::ConcreteStateTrait,
{
    type Effect;
}

/// Applies the shared pinned implementation effect for an erased union state.
#[doc(hidden)]
pub trait StateUnionSharedPinnedTransitionEffect<T, To, Args>:
    StateUnionSharedPinnedEffect<T, To>
where
    T: StateMachineImpl,
    To: crate::ConcreteStateTrait,
{
    fn apply_pinned(value: Pin<&mut T>, args: Args);
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

/// Dispatches a pinned discriminated union transition to the concrete state's effect.
#[doc(hidden)]
pub trait StateUnionDiscriminatedPinnedTransition<T, To, Args>: StateUnionDiscriminant
where
    T: StateMachineImpl,
{
    fn pinned_transition<Storage>(
        state: DiscriminatedState<Storage, T, Self>,
        args: Args,
        callsite: TransitionCallsite,
    ) -> State<Storage, T, To>
    where
        Storage: SPinMut,
        To: crate::ConcreteStateTrait;
}

impl<Standin, Marker, To> Transition<StateUnionState<Marker>, To> for Standin
where
    Marker: StateUnionTransition<Standin, To>,
{
    type F = Marker::F;
}
