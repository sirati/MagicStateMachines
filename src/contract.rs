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
pub trait StateMachineImpl {
    /// Definition-crate ZST used to select the state-machine contract.
    type Standin;

    /// Runtime implementation controlled by the state machine.
    type Impl: StateMachineImpl<Standin = Self::Standin, Impl = Self::Impl>;
}

impl<T> StateMachineImpl for Box<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
}

impl<T> StateMachineImpl for &mut T
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
}

impl<T> StateMachineImpl for UniqueRc<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
}

impl<T> StateMachineImpl for UniqueArc<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
}

impl<T> StateMachineImpl for Pin<Box<T>>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
}
