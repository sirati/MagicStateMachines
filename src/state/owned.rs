#[cfg(not(feature = "tracing"))]
use crate::StateCopy;
#[cfg(feature = "decompose")]
use crate::{DecomposedData, DecomposedState, RecomposeError};
use crate::{Initial, StateClone, StateMachineImpl, Transition};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
#[cfg(feature = "tracing")]
use core::panic::Location;
use core::pin::Pin;

/// A concrete owned runtime implementation `T` whose compile-time state is `S`.
///
/// Without the `tracing` feature, the state marker has no runtime storage and
/// `StateOwned<T, S>` has the same size and alignment as `T`.
///
/// State tokens are linear and shared ownership is not valid state storage:
///
/// ```compile_fail
/// use std::rc::Rc;
/// use magicstatemachines::{Initial, StateMachineImpl, StateOwned};
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

pub type SPin<T, S> = StateOwned<Pin<T>, S>;

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
impl<T, From, To> TransitionCall<T, From, To>
where
    T: StateMachineImpl,
{
    pub fn call<Args>(self, _args: Args) -> StateOwned<T, To>
    where
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
    {
        StateOwned {
            value: self.state.value,
            state: PhantomData,
        }
    }
}

#[cfg(feature = "tracing")]
impl<T, From, To> TransitionCall<T, From, To>
where
    T: StateMachineImpl,
{
    pub fn call<Args>(self, _args: Args) -> StateOwned<T, To>
    where
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: crate::TransitionSignature<Args>,
        From: crate::StateTrait,
        To: crate::ConcreteStateTrait,
    {
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
    #[cfg(feature = "decompose")]
    #[must_use]
    pub fn decompose(self) -> (DecomposedState<S>, DecomposedData<T>) {
        let uid = decompose_uid();

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
    #[cfg(feature = "decompose")]
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

#[cfg(all(feature = "decompose", feature = "nightly-random"))]
fn decompose_uid() -> u64 {
    std::random::random(..)
}

#[cfg(all(
    feature = "decompose",
    not(feature = "nightly-random"),
    feature = "decompose-rnd"
))]
fn decompose_uid() -> u64 {
    rnd::random::<u64>()
}

#[cfg(all(
    feature = "decompose",
    not(feature = "nightly-random"),
    not(feature = "decompose-rnd")
))]
compile_error!(
    "feature `decompose` requires a random backend: enable `nightly-random` or `decompose-rnd`"
);

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

#[cfg(not(feature = "tracing"))]
pub(super) fn complete_transition<T, From, To>(
    state: StateOwned<T, From>,
    _callsite: super::TransitionCallsite,
) -> StateOwned<T, To> {
    StateOwned {
        value: state.value,
        state: PhantomData,
    }
}

#[cfg(feature = "tracing")]
pub(super) fn complete_transition<T, From, To>(
    state: StateOwned<T, From>,
    callsite: super::TransitionCallsite,
) -> StateOwned<T, To>
where
    From: crate::StateTrait,
    To: crate::ConcreteStateTrait,
{
    let mut trace = state.trace;
    trace.push(crate::TraceEntry::new::<From, To>(callsite));

    StateOwned {
        value: state.value,
        state: PhantomData,
        trace,
    }
}
