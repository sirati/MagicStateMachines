#[cfg(not(feature = "tracing"))]
use crate::StateCopy;
use crate::{
    DecomposedData, DecomposedState, Initial, RecomposeError, StateClone, StateMachineImpl,
    Transition,
};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
#[cfg(feature = "tracing")]
use core::panic::Location;
use core::pin::Pin;
use std::rc::UniqueRc;
use std::sync::UniqueArc;

type StateMarker<Storage, T, S> = PhantomData<fn() -> (Storage, T, S)>;
type TransitionMarker<Storage, T, From, To> = PhantomData<fn() -> (Storage, T, From, To)>;

/// A concrete owned runtime implementation `T` whose compile-time state is `S`.
///
/// Without the `tracing` feature, the state marker has no runtime storage and
/// `StateOwned<T, S>` has the same size and alignment as `T`.
///
/// State tokens are linear and shared ownership is not valid state storage:
///
/// ```compile_fail
/// use std::rc::Rc;
/// use statemachines::{Initial, StateMachineImpl, StateOwned};
///
/// struct Machine;
/// struct Ready;
/// struct Runtime;
/// struct Token;
///
/// impl Initial<Ready> for Machine {}
/// impl StateMachineImpl for Runtime {
///     type Standin = Machine;
///     type Impl = Self;
///     type TransitionToken = Token;
/// }
///
/// let _: StateOwned<Rc<Runtime>, Ready> = StateOwned::new(Rc::new(Runtime));
/// ```
#[cfg_attr(not(feature = "tracing"), repr(transparent))]
pub struct StateOwned<T, S> {
    pub(crate) value: T,
    pub(crate) state: PhantomData<fn() -> S>,
    #[cfg(feature = "tracing")]
    pub(crate) trace: Vec<crate::TraceEntry>,
}

pub type StateOwnedBox<T, S> = StateOwned<Box<T>, S>;
pub type StateOwnedPin<T, S> = StateOwned<Pin<T>, S>;
pub type StateOwnedPinBox<T, S> = StateOwned<Pin<Box<T>>, S>;

/// A one-shot callable that completes a state transition.
pub struct TransitionCall<T, From, To> {
    state: StateOwned<T, From>,
    #[cfg(feature = "tracing")]
    callsite: &'static Location<'static>,
    to: PhantomData<fn() -> To>,
}

/// Creates a callable transition requiring the definition's arguments.
///
/// This low-level function requires the implementation's private transition
/// capability. Implementations should use [`StateMachineImpl!`] to expose a
/// private `state.transition()` helper instead.
#[must_use]
#[track_caller]
pub fn transition<T, S, Next>(
    state: StateOwned<T, S>,
    _token: T::TransitionToken,
) -> TransitionCall<T, S, Next>
where
    T: StateMachineImpl,
    T::Standin: Transition<S, Next>,
{
    TransitionCall {
        state,
        #[cfg(feature = "tracing")]
        callsite: Location::caller(),
        to: PhantomData,
    }
}

#[cfg(not(feature = "tracing"))]
impl<T, From, To, Args> FnOnce<Args> for TransitionCall<T, From, To>
where
    T: StateMachineImpl,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
{
    type Output = StateOwned<T, To>;

    extern "rust-call" fn call_once(self, _args: Args) -> Self::Output {
        StateOwned {
            value: self.state.value,
            state: PhantomData,
        }
    }
}

#[cfg(feature = "tracing")]
impl<T, From, To, Args> FnOnce<Args> for TransitionCall<T, From, To>
where
    T: StateMachineImpl,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    From: crate::StateTrait,
    To: crate::StateTrait,
{
    type Output = StateOwned<T, To>;

    extern "rust-call" fn call_once(self, _args: Args) -> Self::Output {
        let mut trace = self.state.trace;
        trace.push(crate::TraceEntry::new::<From, To>(self.callsite));

        StateOwned {
            value: self.state.value,
            state: PhantomData,
            trace,
        }
    }
}

impl<T, S> StateOwned<T, S> {
    /// Separates the compile-time state token from the runtime data.
    ///
    /// The generated UID binds the two returned values together.
    #[must_use]
    pub fn decompose(self) -> (DecomposedState<S>, DecomposedData<T>) {
        let uid = std::random::random(..);

        (
            DecomposedState {
                uid,
                state: PhantomData,
                #[cfg(feature = "tracing")]
                trace: self.trace,
            },
            DecomposedData {
                uid,
                value: self.value,
            },
        )
    }

    /// Recombines state and data produced by the same [`StateOwned::decompose`] call.
    pub fn recompose(
        state: DecomposedState<S>,
        data: DecomposedData<T>,
    ) -> Result<Self, RecomposeError> {
        if state.uid != data.uid {
            return Err(RecomposeError);
        }

        Ok(Self {
            value: data.value,
            state: PhantomData,
            #[cfg(feature = "tracing")]
            trace: state.trace,
        })
    }

    /// Recorded transitions in call order.
    #[cfg(feature = "tracing")]
    #[must_use]
    pub fn trace(&self) -> &[crate::TraceEntry] {
        &self.trace
    }
}

impl<T, S> StateOwned<T, S>
where
    T: StateMachineImpl,
    T::Standin: Initial<S>,
{
    /// Wraps an implementation in a state declared initial by its definition.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            state: PhantomData,
            #[cfg(feature = "tracing")]
            trace: Vec::new(),
        }
    }
}

impl<T, S> Deref for StateOwned<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, S> DerefMut for StateOwned<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, S> Clone for StateOwned<T, S>
where
    T: Clone,
    S: StateClone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            state: PhantomData,
            #[cfg(feature = "tracing")]
            trace: self.trace.clone(),
        }
    }
}

#[cfg(not(feature = "tracing"))]
impl<T, S> Copy for StateOwned<T, S>
where
    T: Copy,
    S: StateClone + StateCopy,
{
}

impl<T: core::fmt::Debug, S> core::fmt::Debug for StateOwned<T, S> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(formatter)
    }
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

/// A state token parameterized by its storage backend.
pub struct State<Storage, T, S>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub(crate) inner: Storage::Inner<T, S>,
    pub(crate) marker: StateMarker<Storage, T, S>,
}

/// Backend for directly owned values.
pub struct StorageStateOwned;

/// Backend for `Box<T>` owned values.
pub struct StorageStateOwnedBox;

/// Backend for `Pin<Box<T>>` owned values.
pub struct StorageStateOwnedPinBox;

/// Backend for `UniqueRc<T>` owned values.
pub struct StorageStateOwnedUniqueRc;

/// Backend for `UniqueArc<T>` owned values.
pub struct StorageStateOwnedUniqueArc;

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

#[cfg(feature = "tracing")]
pub type TransitionCallsite = &'static Location<'static>;

#[cfg(not(feature = "tracing"))]
pub type TransitionCallsite = ();

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

impl StateStorage for StorageStateOwned {
    type Inner<T, S>
        = StateOwned<T, S>
    where
        T: StateMachineImpl;
    type Machine<T>
        = T
    where
        T: StateMachineImpl;

    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        _args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        State {
            inner: complete_owned_transition(state.inner, callsite),
            marker: PhantomData,
        }
    }
}

impl StateStorageNew for StorageStateOwned {
    fn new<T, S>(value: T) -> Self::Inner<T, S>
    where
        T: StateMachineImpl,
        T::Standin: Initial<S>,
    {
        StateOwned::new(value)
    }
}

impl SRef for StorageStateOwned {
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        &inner.value
    }
}

impl SMut for StorageStateOwned {
    fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
    where
        T: StateMachineImpl,
    {
        &mut inner.value
    }
}

macro_rules! indirect_owned_storage {
    ($storage:ty, $wrapper:ident) => {
        impl StateStorage for $storage {
            type Inner<T, S>
                = StateOwned<$wrapper<T>, S>
            where
                T: StateMachineImpl;
            type Machine<T>
                = $wrapper<T>
            where
                T: StateMachineImpl;

            fn complete_transition<T, From, To, Args>(
                state: State<Self, T, From>,
                _args: Args,
                callsite: TransitionCallsite,
            ) -> State<Self, T, To>
            where
                T: StateMachineImpl,
                From: crate::StateTrait,
                To: crate::StateTrait,
                T::Standin: Transition<From, To>,
                <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
                Args: core::marker::Tuple,
            {
                State {
                    inner: complete_owned_transition(state.inner, callsite),
                    marker: PhantomData,
                }
            }
        }

        impl StateStorageNew for $storage {
            fn new<T, S>(value: T) -> Self::Inner<T, S>
            where
                T: StateMachineImpl,
                <Self::Machine<T> as StateMachineImpl>::Standin: Initial<S>,
            {
                StateOwned::new($wrapper::new(value))
            }
        }

        impl SRef for $storage {
            fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
            where
                T: StateMachineImpl,
            {
                &inner.value
            }
        }

        impl SMut for $storage {
            fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
            where
                T: StateMachineImpl,
            {
                &mut inner.value
            }
        }
    };
}

indirect_owned_storage!(StorageStateOwnedBox, Box);
indirect_owned_storage!(StorageStateOwnedUniqueRc, UniqueRc);
indirect_owned_storage!(StorageStateOwnedUniqueArc, UniqueArc);

impl StateStorage for StorageStateOwnedPinBox {
    type Inner<T, S>
        = StateOwned<Pin<Box<T>>, S>
    where
        T: StateMachineImpl;
    type Machine<T>
        = Pin<Box<T>>
    where
        T: StateMachineImpl;

    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        _args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        State {
            inner: complete_owned_transition(state.inner, callsite),
            marker: PhantomData,
        }
    }
}

impl StateStorageNew for StorageStateOwnedPinBox {
    fn new<T, S>(value: T) -> Self::Inner<T, S>
    where
        T: StateMachineImpl,
        <Self::Machine<T> as StateMachineImpl>::Standin: Initial<S>,
    {
        StateOwned::new(Box::pin(value))
    }
}

impl SRef for StorageStateOwnedPinBox {
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        &inner.value
    }
}

#[cfg(not(feature = "tracing"))]
fn complete_owned_transition<T, From, To>(
    state: StateOwned<T, From>,
    _callsite: TransitionCallsite,
) -> StateOwned<T, To> {
    StateOwned {
        value: state.value,
        state: PhantomData,
    }
}

#[cfg(feature = "tracing")]
fn complete_owned_transition<T, From, To>(
    state: StateOwned<T, From>,
    callsite: TransitionCallsite,
) -> StateOwned<T, To>
where
    From: crate::StateTrait,
    To: crate::StateTrait,
{
    let mut trace = state.trace;
    trace.push(crate::TraceEntry::new::<From, To>(callsite));

    StateOwned {
        value: state.value,
        state: PhantomData,
        trace,
    }
}
