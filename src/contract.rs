use core::pin::Pin;
use std::rc::UniqueRc;
use std::sync::UniqueArc;

/// Declares that a definition crate permits `TState` as an initial state.
pub trait Initial<TState> {}

/// Declares that a definition crate permits `TFrom -> TTo`.
///
/// The definition crate owns the stand-in and state types. Rust's orphan rules
/// therefore prevent an implementation crate from adding transitions.
pub trait Transition<TFrom, TTo> {
    /// Function signature required to perform this transition.
    type F = fn();
}

/// Connects an implementation type to a state-machine definition.
///
/// [`crate::StateMachineImpl!`] generates this implementation and keeps the
/// transition capability's construction private:
///
/// ```compile_fail
/// use statemachines::{Initial, State, StateMachineImpl, StorageStateOwned, Transition};
///
/// mod implementation {
///     use super::*;
///
///     pub struct Machine;
///     pub struct Ready;
///     pub struct Running;
///     pub struct Runtime;
///     impl Initial<Ready> for Machine {}
///     impl Transition<Ready, Running> for Machine {}
///
///     statemachines::StateMachineImpl!(Runtime: Machine);
///
///     pub fn ready() -> State<StorageStateOwned, Runtime, Ready> {
///         State::new(Runtime)
///     }
/// }
///
/// let ready = implementation::ready();
/// let _ = statemachines::transition_state::<_, _, _, implementation::Running>(
///     ready,
///     implementation::__StateMachineTransitionToken(())
/// )();
/// ```
///
/// The generated ergonomic method is private to the invocation module:
///
/// ```compile_fail
/// use statemachines::{Initial, State, StorageStateOwned, Transition};
///
/// mod implementation {
///     use super::*;
///
///     pub struct Machine;
///     pub struct Ready;
///     pub struct Running;
///     pub struct Runtime;
///
///     impl Initial<Ready> for Machine {}
///     impl Transition<Ready, Running> for Machine {}
///     statemachines::StateMachineImpl!(Runtime: Machine);
///
///     pub fn ready() -> State<StorageStateOwned, Runtime, Ready> {
///         State::new(Runtime)
///     }
/// }
///
/// let ready = implementation::ready();
/// let _ = ready.transition::<implementation::Running>()();
/// ```
pub trait StateMachineImpl {
    /// Definition-crate ZST used to select the state-machine contract.
    type Standin;

    /// Runtime implementation controlled by the state machine.
    type Impl: StateMachineImpl<Standin = Self::Standin, Impl = Self::Impl>;

    /// Capability required to perform transitions.
    ///
    /// Use [`crate::StateMachineImpl!`] to generate this capability and its
    /// private ergonomic transition helpers.
    type TransitionToken;
}

impl<T> StateMachineImpl for Box<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
    type TransitionToken = T::TransitionToken;
}

impl<T> StateMachineImpl for UniqueRc<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
    type TransitionToken = T::TransitionToken;
}

impl<T> StateMachineImpl for UniqueArc<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
    type TransitionToken = T::TransitionToken;
}

impl<T> StateMachineImpl for Pin<Box<T>>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
    type TransitionToken = T::TransitionToken;
}
