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

/// A runtime implementation `T` whose compile-time state is `S`.
///
/// Without the `tracing` feature, the state marker has no runtime storage and
/// `State<T, S>` has the same size and alignment as `T`.
///
/// State tokens are linear and shared ownership is not valid state storage:
///
/// ```compile_fail
/// use std::rc::Rc;
/// use statemachines::{Initial, State, StateMachineImpl};
///
/// struct Machine;
/// struct Ready;
/// struct Runtime;
///
/// impl Initial<Ready> for Machine {}
/// impl StateMachineImpl for Runtime {
///     type Standin = Machine;
///     type Impl = Self;
/// }
///
/// let _: State<Rc<Runtime>, Ready> = State::new(Rc::new(Runtime));
/// ```
#[cfg_attr(not(feature = "tracing"), repr(transparent))]
pub struct State<T, S> {
    pub(crate) value: T,
    pub(crate) state: PhantomData<fn() -> S>,
    #[cfg(feature = "tracing")]
    pub(crate) trace: Vec<crate::TraceEntry>,
}

/// A one-shot callable that completes a state transition.
pub struct TransitionCall<T, From, To> {
    state: State<T, From>,
    #[cfg(feature = "tracing")]
    callsite: &'static Location<'static>,
    to: PhantomData<fn() -> To>,
}

impl<T, S> State<T, S> {
    /// Creates a callable transition requiring the definition's arguments.
    ///
    /// Invoke the returned value immediately:
    ///
    /// ```ignore
    /// let connected = disconnected.transition::<Connected>()();
    /// let authenticated = connected.transition::<Authenticated>()("alice");
    /// ```
    #[must_use]
    #[track_caller]
    pub fn transition<Next>(self) -> TransitionCall<T, S, Next>
    where
        T: StateMachineImpl,
        T::Standin: Transition<S, Next>,
    {
        TransitionCall {
            state: self,
            #[cfg(feature = "tracing")]
            callsite: Location::caller(),
            to: PhantomData,
        }
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
    type Output = State<T, To>;

    extern "rust-call" fn call_once(self, _args: Args) -> Self::Output {
        State {
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
    From: crate::tracing::State,
    To: crate::tracing::State,
{
    type Output = State<T, To>;

    extern "rust-call" fn call_once(self, _args: Args) -> Self::Output {
        let mut trace = self.state.trace;
        trace.push(crate::TraceEntry::new::<From, To>(self.callsite));

        State {
            value: self.state.value,
            state: PhantomData,
            trace,
        }
    }
}

impl<T, S> State<T, S> {
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

    /// Recombines state and data produced by the same [`State::decompose`] call.
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

impl<T, S> State<T, S>
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

impl<T, S> Deref for State<T, S> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, S> DerefMut for State<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, S> Clone for State<T, S>
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
impl<T, S> Copy for State<T, S>
where
    T: Copy,
    S: StateClone + StateCopy,
{
}

impl<T: core::fmt::Debug, S> core::fmt::Debug for State<T, S> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(formatter)
    }
}
