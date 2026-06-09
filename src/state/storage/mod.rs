mod owned;

use crate::{
    ConcreteStateKind, DiscriminatedState, Initial, StateConcreteProvenState,
    StateConcreteTransitionProof, StateKind, StateMachineImpl, StateUnionDiscriminant,
    StateUnionDiscriminatedTransition, StateUnionErased, StateUnionProofTarget,
    StateUnionProvenState, StateUnionSharedEffect, StateUnionSharedTransitionEffect,
    StateUnionTransitionProof, StateWithProof, Transition, UnionStateKind, state_trait,
};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
#[cfg(feature = "tracing")]
use core::panic::Location;
use core::pin::Pin;
#[cfg(feature = "unique-rc-arc")]
use std::rc::UniqueRc;
#[cfg(feature = "unique-rc-arc")]
use std::sync::UniqueArc;

pub use owned::{SOwned, StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox};
#[cfg(feature = "unique-rc-arc")]
pub use owned::{StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc};

/// Owned state whose runtime value is stored in `Box<T>`.
///
/// This is only a type alias for `State<StorageStateOwnedBox, T, S>`. Create it
/// from an already-owned state with [`SBox::new`](State<StorageStateOwnedBox, T, S>::new)
/// so the current state is preserved while changing containers:
///
/// ```ignore
/// let owned: State<SOwned, Connection, Disconnected> = State::new(connection);
/// let boxed: SBox<Connection, Disconnected> = SBox::new(owned);
/// ```
pub type SBox<T, S> = State<StorageStateOwnedBox, T, S>;
/// Owned state whose runtime value is stored in `Pin<Box<T>>`.
///
/// This is the pinned counterpart to [`SBox`]. Pinning consumes a boxed state
/// rather than raw `T`, again preserving the already-proven current state.
///
/// Pinned storage is deliberately not just "boxed plus `SMut`". For
/// `T: !Unpin`, the backend can expose [`Pin<&T>`](core::pin::Pin) and
/// `Pin<&mut T>` through [`SPinRef`] and [`SPinMut`], but it does not expose
/// `&mut T`. That means ordinary transitions requiring `S: SMut` stay
/// unavailable, while pinned transition methods can still be written:
///
/// ```ignore
/// impl Connection {
///     fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
///     where
///         S: magicstatemachines::SPinMut,
///     {
///         magicstatemachines::transition!(pin self)
///     }
/// }
/// ```
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

/// Selects the pinned implementation-side effect for a declared transition.
#[doc(hidden)]
pub trait PinnedTransitionEffectSelector<From, To>: StateMachineImpl {
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

/// Applies implementation-side transition effects through pinned access.
///
/// This is the pinned counterpart to [`TransitionEffect`]. It is used by
/// `pinned transition` entries in [`StateMachineImpl!`](macro@crate::StateMachineImpl)
/// and by [`transition!(pin state, ...)`](macro@crate::transition). The effect
/// receives `Pin<&mut T>`, so a `!Unpin` runtime can update interior fields or
/// call pinned methods without ever exposing `&mut T`.
#[doc(hidden)]
pub trait PinnedTransitionEffect<T, From, To, Args>
where
    T: StateMachineImpl,
{
    fn apply(value: Pin<&mut T>, args: Args);
}

/// Selects where a storage backend's authoritative state marker is inferred.
#[doc(hidden)]
pub trait InferenceKind {
    type Inference: StateInference;
}

/// State marker inference carried by discriminated storage.
#[doc(hidden)]
pub trait StateInference {
    fn new<Storage, T, S>(inner: &Storage::Inner<T, S>) -> Self
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        S: crate::ConcreteStateTrait;

    fn from_erased(state: state_trait::ErasedState) -> Self;

    fn state<Storage, T, S>(&self, inner: &Storage::Inner<T, S>) -> state_trait::ErasedState
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        S: crate::StateTrait;
}

/// Inference stored outside the wrapped backend, used by type-only storage.
#[doc(hidden)]
pub struct OuterInference;

/// Inference delegated to the wrapped backend, used by runtime-state storage.
#[doc(hidden)]
pub struct InnerInference;

/// ZST inference value for [`InnerInference`].
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct InnerStateInference;

impl InferenceKind for OuterInference {
    type Inference = state_trait::ErasedState;
}

impl InferenceKind for InnerInference {
    type Inference = InnerStateInference;
}

impl StateInference for state_trait::ErasedState {
    fn new<Storage, T, S>(_inner: &Storage::Inner<T, S>) -> Self
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        S: crate::ConcreteStateTrait,
    {
        state_trait::erased_state::<S>()
    }

    fn from_erased(state: state_trait::ErasedState) -> Self {
        state
    }

    fn state<Storage, T, S>(&self, _inner: &Storage::Inner<T, S>) -> state_trait::ErasedState
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        S: crate::StateTrait,
    {
        state_trait::clone_erased(self)
    }
}

impl StateInference for InnerStateInference {
    fn new<Storage, T, S>(_inner: &Storage::Inner<T, S>) -> Self
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        S: crate::ConcreteStateTrait,
    {
        Self
    }

    fn from_erased(_state: state_trait::ErasedState) -> Self {
        Self
    }

    fn state<Storage, T, S>(&self, inner: &Storage::Inner<T, S>) -> state_trait::ErasedState
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        S: crate::StateTrait,
    {
        Storage::inferred_state(inner)
    }
}

/// Storage backend used by [`State`].
///
/// `State<Storage, T, S>` is only a typed view; the actual representation is
/// selected by this trait. The same implementation methods can therefore work
/// for directly owned values, boxed values, pinned values, shared guard views,
/// and discriminated union views as long as they ask for the right capability
/// bound.
///
/// Backend authors provide an `Inner<T, S>` generic associated type. The
/// library retags that `Inner` after a successful transition, so the backend
/// controls where data is stored while the state-machine contract still
/// controls which retags are legal.
///
/// Most users do not implement this trait. They choose one of the built-in
/// storage aliases and write bounds on methods:
///
/// ```ignore
/// use magicstatemachines::{SRef, SMut, State, transition};
/// use test_def::{InOnline, Online};
/// use test_def::states::{Authenticated, Connected};
///
/// impl Connection {
///     fn endpoint<S>(self: &State<S, Self, impl InOnline>) -> &str
///     where
///         S: SRef,
///     {
///         &self.endpoint
///     }
///
///     fn authenticate<S>(
///         self: State<S, Self, Connected>,
///         user: String,
///     ) -> State<S, Self, Authenticated>
///     where
///         S: SMut,
///     {
///         transition!(self, user)
///     }
/// }
/// ```
pub trait StateStorage: Sized {
    /// Selects how [`SDiscriminated`](crate::SDiscriminated) recovers the current state marker.
    type Inference: InferenceKind = OuterInference;

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

    /// Retags a state after checking that `Args` matches the declared transition signature.
    ///
    /// Storage backends normally delegate this to their inner representation.
    /// Implementation crates should call transitions through [`transition!`](macro@crate::transition)
    /// rather than invoking this method directly.
    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>;

    #[doc(hidden)]
    fn complete_transition_after_effect<T, From, To>(
        state: State<Self, T, From>,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait;

    #[doc(hidden)]
    fn inferred_state<T, State>(inner: &Self::Inner<T, State>) -> state_trait::ErasedState
    where
        T: StateMachineImpl,
        State: crate::StateTrait,
    {
        let _ = inner;
        state_trait::static_erased_state::<State>()
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
    To: crate::ConcreteStateTrait,
{
    Storage::complete_transition_after_effect(state, callsite)
}

/// Storage backend that can create initial owned state.
///
/// This is the capability behind [`State::new`]. The `Initial` bound prevents
/// raw runtime data from being introduced in a non-initial state.
pub trait StateStorageNew: StateStorage {
    /// Creates a backend-specific inner value in an allowed initial state.
    fn new<T, State>(value: T) -> Self::Inner<T, State>
    where
        T: StateMachineImpl,
        <Self::Machine<T> as StateMachineImpl>::Standin: Initial<State>;
}

/// Storage backend that can expose a runtime reference.
///
/// Use `S: SRef` for methods that only read the runtime implementation. This
/// bound covers owned states, boxed/pinned states, and read or write guards
/// from shared storage.
pub trait SRef: StateStorage {
    /// Borrows the runtime implementation from this storage backend.
    fn s_ref<T, State>(inner: &Self::Inner<T, State>) -> &T
    where
        T: StateMachineImpl;
}

/// Storage backend that can expose a mutable runtime reference.
///
/// Use `S: SMut` for methods that mutate runtime data or call
/// [`transition!`](macro@crate::transition). Transition effect bodies always
/// receive `&mut T`, so a transition method normally needs this bound even if
/// the body is empty.
pub trait SMut: SRef {
    /// Mutably borrows the runtime implementation from this storage backend.
    fn s_mut<T, State>(inner: &mut Self::Inner<T, State>) -> &mut T
    where
        T: StateMachineImpl;
}

/// Storage backend that can expose a pinned runtime reference.
///
/// This capability is separate from [`SRef`] and [`SMut`] because pinning is
/// stronger than ordinary borrowing. A backend should implement this only when
/// it can prove the runtime value will not be moved for the lifetime of the
/// storage object. [`SPinBox`](crate::SPinBox) does; ordinary owned storage
/// intentionally does not.
///
/// Use this bound for read-only methods that must call APIs taking
/// `Pin<&Self>`:
///
/// ```ignore
/// fn stable_address<S>(self: &State<S, Self, Ready>) -> usize
/// where
///     S: magicstatemachines::SPinRef,
/// {
///     magicstatemachines::pin_ref(self).stable_address()
/// }
/// ```
pub trait SPinRef: SRef {
    /// Borrows the runtime implementation as a pinned shared reference.
    fn s_pin_ref<T, State>(inner: &Self::Inner<T, State>) -> Pin<&T>
    where
        T: StateMachineImpl;
}

/// Storage backend that can expose a pinned mutable runtime reference.
///
/// This is the capability required by pinned transition effects. It does not
/// imply [`SMut`]: a backend may safely provide `Pin<&mut T>` for `!Unpin`
/// values while still refusing to provide `&mut T`.
pub trait SPinMut: SPinRef {
    /// Mutably borrows the runtime implementation as a pinned reference.
    fn s_pin_mut<T, State>(inner: &mut Self::Inner<T, State>) -> Pin<&mut T>
    where
        T: StateMachineImpl;
}

/// Storage backend whose state token can be consumed by value.
///
/// Use this when an API must move the backend representation itself rather
/// than just read or mutate the runtime value. Most transition methods only
/// need [`SMut`].
pub trait SMove: StateStorage {}

/// A state token parameterized by storage, runtime implementation, and state.
///
/// The type parameters carry the whole proof:
///
/// - `Storage` selects how the runtime value is held, for example
///   [`SOwned`](crate::SOwned), [`StorageStateOwnedBox`], or a mutable guard
///   from shared storage.
/// - `T` is the runtime implementation type, such as `Connection`.
/// - `S` is the current state marker, such as `Disconnected` or
///   `impl InOnline`.
///
/// `State` is intentionally a deref-like wrapper with very few inherent
/// methods. State-machine behavior should be implemented on `T` with
/// arbitrary self types, not added as generic convenience methods on `State`.
/// That keeps the transition capability private to the implementation module.
///
/// ```ignore
/// impl Connection {
///     fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
///     where
///         S: SMut,
///     {
///         magicstatemachines::transition!(self)
///     }
///
///     fn endpoint<S>(self: &State<S, Self, impl InOnline>) -> &str
///     where
///         S: SRef,
///     {
///         &self.endpoint
///     }
/// }
/// ```
pub struct State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub(crate) inner: Storage::Inner<T, S>,
    pub(crate) marker: StateMarker<Storage, T, S>,
}

/// A result whose success and error values are states of the same machine.
///
/// This is useful for fallible state-machine methods where both branches
/// return ownership of the same runtime value in different states:
///
/// ```ignore
/// fn authenticate_if<S>(
///     self: State<S, Connection, Connected>,
///     user: Option<String>,
/// ) -> SResult<S, Connection, Authenticated, Connected>
/// where
///     S: SMut,
/// {
///     match user {
///         Some(user) => Ok(magicstatemachines::transition!(self, user)),
///         None => Err(self),
///     }
/// }
/// ```
#[allow(type_alias_bounds)]
pub type SResult<Storage, T, OkState, ErrState>
where
    Storage: StateStorage,
    T: StateMachineImpl,
= Result<State<Storage, T, OkState>, State<Storage, T, ErrState>>;

/// A callable transition for generic [`State`] storage.
#[doc(hidden)]
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

/// A callable transition that runs a pinned implementation-side effect first.
#[doc(hidden)]
pub struct PinnedEffectTransitionCall<Storage, T, From, To, Effect>
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
    To: crate::ConcreteStateTrait,
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
    To: crate::ConcreteStateTrait,
{
    state: State<Storage, T, From>,
    callsite: TransitionCallsite,
    marker: PhantomData<fn() -> (Marker, To)>,
}

/// A callable pinned transition proven through a generated state union.
#[doc(hidden)]
pub struct PinnedStateUnionProofTransitionCall<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    Marker: StateUnionDiscriminant,
    To: crate::ConcreteStateTrait,
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

/// A callable pinned discriminated-union transition.
#[doc(hidden)]
pub struct PinnedDiscriminatedTransitionCall<Storage, T, Marker, To>
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
#[doc(hidden)]
pub type TransitionCallsite = &'static Location<'static>;

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
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

impl<Storage, T, From, To> StateTransitionCall<Storage, T, From, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    #[doc(hidden)]
    pub fn call<Args>(self, args: Args) -> State<Storage, T, To>
    where
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait,
    {
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

impl<Storage, T, From, To, Effect> EffectTransitionCall<Storage, T, From, To, Effect>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub fn call<Args>(mut self, args: Args) -> State<Storage, T, To>
    where
        Storage: SMut,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait,
        Effect: TransitionEffect<T, From, To, Args>,
    {
        Effect::apply(Storage::s_mut(&mut self.state.inner), args);
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, To, Effect> PinnedEffectTransitionCall<Storage, T, From, To, Effect>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub fn call<Args>(mut self, args: Args) -> State<Storage, T, To>
    where
        Storage: SPinMut,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait,
        Effect: PinnedTransitionEffect<T, From, To, Args>,
    {
        Effect::apply(Storage::s_pin_mut(&mut self.state.inner), args);
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, To> ConcreteProofTransitionCall<Storage, T, From, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub fn call<Args>(mut self, args: Args) -> State<Storage, T, To>
    where
        T: TransitionEffectSelector<From, To>,
        Storage: SMut,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait,
        <T as TransitionEffectSelector<From, To>>::Effect: TransitionEffect<T, From, To, Args>,
    {
        <T as TransitionEffectSelector<From, To>>::Effect::apply(
            Storage::s_mut(&mut self.state.inner),
            args,
        );
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, Marker, To>
    KindProofTransitionCall<Storage, T, From, Marker, To, ConcreteStateKind>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    To: crate::ConcreteStateTrait,
    Marker: crate::StateMarker,
{
    pub fn call<Args>(mut self, args: Args) -> State<Storage, T, To>
    where
        T: TransitionEffectSelector<From, To>,
        Storage: SMut,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
        From: crate::ConcreteStateTrait,
        To: crate::ConcreteStateTrait,
        <T as TransitionEffectSelector<From, To>>::Effect: TransitionEffect<T, From, To, Args>,
    {
        <T as TransitionEffectSelector<From, To>>::Effect::apply(
            Storage::s_mut(&mut self.state.inner),
            args,
        );
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, Marker, To>
    KindProofTransitionCall<Storage, T, From, Marker, To, UnionStateKind>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    To: crate::ConcreteStateTrait,
    Marker: StateUnionDiscriminant,
{
    pub fn call<Args>(self, args: Args) -> State<Storage, T, To>
    where
        Storage: SMut,
        From: crate::StateTrait + crate::In<Marker>,
        Marker: StateUnionDiscriminatedTransition<T, To, Args>,
        To: crate::ConcreteStateTrait,
    {
        let state = <From as crate::In<Marker>>::into_discriminated(self.state);
        Marker::transition(state, args, self.callsite)
    }
}

impl<Storage, T, From, Marker, To> StateUnionProofTransitionCall<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    To: crate::ConcreteStateTrait,
    Marker: StateUnionDiscriminant,
{
    pub fn call<Args>(mut self, args: Args) -> State<Storage, T, To>
    where
        Storage: SMut,
        From: StateUnionErased<Marker>,
        Marker: StateUnionSharedTransitionEffect<T, To, Args>,
        To: crate::ConcreteStateTrait,
    {
        Marker::apply(Storage::s_mut(&mut self.state.inner), args);
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, From, Marker, To> PinnedStateUnionProofTransitionCall<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: crate::StateTrait,
    To: crate::ConcreteStateTrait,
    Marker: StateUnionDiscriminant,
{
    pub fn call<Args>(mut self, args: Args) -> State<Storage, T, To>
    where
        Storage: SPinMut,
        From: StateUnionErased<Marker>,
        Marker: crate::StateUnionSharedPinnedTransitionEffect<T, To, Args>,
        To: crate::ConcreteStateTrait,
    {
        Marker::apply_pinned(Storage::s_pin_mut(&mut self.state.inner), args);
        Storage::complete_transition_after_effect(self.state, self.callsite)
    }
}

impl<Storage, T, Marker, To> DiscriminatedTransitionCall<Storage, T, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    Marker: StateUnionDiscriminant,
{
    pub fn call<Args>(self, args: Args) -> State<Storage, T, To>
    where
        Storage: SMut,
        Marker: StateUnionDiscriminatedTransition<T, To, Args>,
        To: crate::ConcreteStateTrait,
    {
        Marker::transition(self.state, args, self.callsite)
    }
}

impl<Storage, T, Marker, To> PinnedDiscriminatedTransitionCall<Storage, T, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    Marker: StateUnionDiscriminant,
{
    pub fn call<Args>(self, args: Args) -> State<Storage, T, To>
    where
        Storage: SPinMut,
        Marker: crate::StateUnionDiscriminatedPinnedTransition<T, To, Args>,
        To: crate::ConcreteStateTrait,
    {
        Marker::pinned_transition(self.state, args, self.callsite)
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
    Next: crate::ConcreteStateTrait,
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
    Next: crate::ConcreteStateTrait,
{
    EffectTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Creates a callable transition that runs pinned implementation-side effects first.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_pinned_effect<Storage, T, S, Next, Effect>(
    state: State<Storage, T, S>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> PinnedEffectTransitionCall<Storage, T, S, Next, Effect>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    S: crate::StateTrait,
    Next: crate::ConcreteStateTrait,
{
    PinnedEffectTransitionCall {
        state,
        callsite: transition_callsite(),
        marker: PhantomData,
    }
}

/// Creates a callable pinned transition from a static union proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_static_union_pinned_proof<Storage, T, S, Marker, Next>(
    state: State<Storage, T, S>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> PinnedStateUnionProofTransitionCall<Storage, T, S, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait + StateUnionErased<Marker>,
    Marker: StateUnionDiscriminant + crate::StateUnionSharedPinnedEffect<T, Next>,
    Next: crate::ConcreteStateTrait,
{
    PinnedStateUnionProofTransitionCall {
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
    Next: crate::ConcreteStateTrait,
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
) -> EffectTransitionCall<Storage, T, S, Next, <T as TransitionEffectSelector<S, Next>>::Effect>
where
    T: StateMachineImpl + TransitionEffectSelector<S, Next>,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    Marker: StateUnionDiscriminant,
    S: crate::StateTrait,
    Next: crate::ConcreteStateTrait,
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
) -> EffectTransitionCall<Storage, T, S, Next, <T as TransitionEffectSelector<S, Next>>::Effect>
where
    T: StateMachineImpl + TransitionEffectSelector<S, Next>,
    Storage: StateStorage,
    T::Standin: Transition<S, Next>,
    Marker: StateUnionDiscriminant,
    S: crate::StateTrait,
    Next: crate::ConcreteStateTrait,
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
    Next: crate::ConcreteStateTrait,
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

/// Creates a callable static union transition from a shared-body proof.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_state_with_static_union_proof<Storage, T, S, Marker, Next>(
    state: State<Storage, T, S>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> StateUnionProofTransitionCall<Storage, T, S, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait + StateUnionErased<Marker> + crate::UnionTransitionProof<T, Marker, Next>,
    Marker: StateUnionDiscriminant + StateUnionSharedEffect<T, Next>,
    Next: crate::ConcreteStateTrait,
{
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
    Next: crate::ConcreteStateTrait,
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
    proven: StateWithProof<
        Storage,
        T,
        S,
        crate::TransitionProof<Storage, T, S, Marker, Next, Kind>,
    >,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> ConcreteProofTransitionCall<Storage, T, S, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait,
    Marker: crate::StateMarker,
    Next: crate::ConcreteStateTrait,
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
    proven: StateWithProof<
        Storage,
        T,
        S,
        crate::TransitionProof<Storage, T, S, Marker, Next, Kind>,
    >,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> KindProofTransitionCall<Storage, T, S, Marker, Next, Kind>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    S: crate::StateTrait,
    Marker: crate::StateMarker,
    Next: crate::ConcreteStateTrait,
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
    To: crate::ConcreteStateTrait,
    Effect: TransitionEffect<T, From, To, Args>,
{
    Effect::apply(Storage::s_mut(&mut state.inner), args);
    Storage::complete_transition_after_effect(state, callsite)
}

#[doc(hidden)]
pub fn transition_concrete_after_pinned_effect<Storage, T, From, To, Args, Effect>(
    mut state: State<Storage, T, From>,
    args: Args,
    callsite: TransitionCallsite,
) -> State<Storage, T, To>
where
    T: StateMachineImpl,
    Storage: SPinMut,
    From: crate::StateTrait,
    To: crate::ConcreteStateTrait,
    Effect: PinnedTransitionEffect<T, From, To, Args>,
{
    Effect::apply(Storage::s_pin_mut(&mut state.inner), args);
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

/// Creates a callable pinned discriminated-union transition that runs the exact concrete effect.
#[doc(hidden)]
#[must_use]
#[track_caller]
pub fn transition_discriminated_state_pinned<Storage, T, Marker, Next>(
    state: DiscriminatedState<Storage, T, Marker>,
    _token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
) -> PinnedDiscriminatedTransitionCall<Storage, T, Marker, Next>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    Marker: StateUnionDiscriminant,
    Next: crate::StateTrait,
{
    PinnedDiscriminatedTransitionCall {
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
    To: crate::ConcreteStateTrait,
{
    StateUnionProvenState {
        state,
        marker: PhantomData,
    }
}

/// Borrows a state's runtime value through its storage's pin guarantee.
///
/// This is a free function rather than an inherent `State` method so generic
/// state-machine APIs stay focused on receiver types and transitions. Use it
/// when a method accepts `&State<S, T, Current>` and needs to call an API on
/// `T` that requires `Pin<&T>`.
#[must_use]
pub fn pin_ref<Storage, T, S>(state: &State<Storage, T, S>) -> Pin<&T>
where
    T: StateMachineImpl,
    Storage: SPinRef,
{
    Storage::s_pin_ref(&state.inner)
}

/// Mutably borrows a state's runtime value through its storage's pin guarantee.
///
/// Most pinned transitions should use `pinned transition` plus
/// [`transition!(pin state, ...)`](macro@crate::transition) instead of calling
/// this directly. It is provided for implementation methods that need to call
/// additional pinned APIs before deciding whether to transition.
pub fn pin_mut<Storage, T, S>(state: &mut State<Storage, T, S>) -> Pin<&mut T>
where
    T: StateMachineImpl,
    Storage: SPinMut,
{
    Storage::s_pin_mut(&mut state.inner)
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
        To: crate::ConcreteStateTrait,
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
    ///
    /// This is the normal entry point for a state machine. It only compiles
    /// when the definition crate declared `S` as an initial state for
    /// `T::Standin`.
    ///
    /// ```ignore
    /// use magicstatemachines::{SOwned, State};
    /// use test_def::states::Disconnected;
    ///
    /// let disconnected: State<SOwned, Connection, Disconnected> =
    ///     State::new(Connection::new("localhost:8080"));
    /// ```
    ///
    /// To move the state into another owned container later, create that
    /// container from the returned state token instead of constructing raw
    /// `T` again.
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
    ///
    /// This mirrors `Box::new`, but it takes `State<SOwned, T, S>` instead of
    /// raw `T` so the current state is preserved:
    ///
    /// ```ignore
    /// let owned: State<SOwned, Connection, Connected> = connection.connect();
    /// let boxed: SBox<Connection, Connected> = SBox::new(owned);
    /// ```
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
    ///
    /// This consumes the box and returns `State<SOwned, T, S>`. The state
    /// marker is unchanged, so methods available before boxing remain
    /// available after unboxing.
    ///
    /// ```ignore
    /// let boxed: SBox<Connection, Connected> = SBox::new(owned);
    /// let owned_again: State<SOwned, Connection, Connected> = SBox::unbox(boxed);
    /// ```
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
    ///
    /// Pinning must happen to the boxed allocation. For that reason this
    /// constructor consumes [`SBox`] rather than `SOwned`:
    ///
    /// ```ignore
    /// let boxed: SBox<Connection, Connected> = SBox::new(owned);
    /// let pinned: SPinBox<Connection, Connected> = SPinBox::new(boxed);
    /// ```
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
    ///
    /// This is only available for `T: Unpin`; otherwise the pinning guarantee
    /// must be preserved.
    ///
    /// ```compile_fail
    /// use core::marker::PhantomPinned;
    /// use magicstatemachines::{
    ///     Initial, SBox, SOwned, SPinBox, State, StateMachineImpl, States,
    /// };
    ///
    /// struct Machine;
    /// struct Runtime {
    ///     _pin: PhantomPinned,
    /// }
    /// struct Token;
    ///
    /// States! {
    ///     Ready;
    /// }
    ///
    /// impl Initial<Ready> for Machine {}
    ///
    /// impl StateMachineImpl for Runtime {
    ///     type Standin = Machine;
    ///     type Impl = Self;
    ///     type TransitionToken = Token;
    /// }
    ///
    /// let ready = State::<SOwned, _, Ready>::new(Runtime {
    ///     _pin: PhantomPinned,
    /// });
    /// let pinned: SPinBox<Runtime, Ready> = SPinBox::new(SBox::new(ready));
    ///
    /// // `Runtime` is `!Unpin`, so the pinned allocation cannot be turned
    /// // back into an ordinary box.
    /// let _boxed = SPinBox::into_boxed(pinned);
    /// ```
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

#[cfg(feature = "unique-rc-arc")]
impl<T, S> State<StorageStateOwnedUniqueRc, T, S>
where
    T: StateMachineImpl,
{
    /// Moves a directly owned state into `UniqueRc` storage without changing its state.
    ///
    /// This API is available only with the `unique-rc-arc` feature because
    /// `UniqueRc` itself is nightly-only.
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

#[cfg(feature = "unique-rc-arc")]
impl<T, S> State<StorageStateOwnedUniqueArc, T, S>
where
    T: StateMachineImpl,
{
    /// Moves a directly owned state into `UniqueArc` storage without changing its state.
    ///
    /// This API is available only with the `unique-rc-arc` feature because
    /// `UniqueArc` itself is nightly-only.
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
