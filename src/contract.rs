use core::pin::Pin;
#[cfg(feature = "unique-rc-arc")]
use std::rc::UniqueRc;
#[cfg(feature = "unique-rc-arc")]
use std::sync::UniqueArc;

/// Declares that a definition crate permits `TState` as an initial state.
///
/// This trait is normally emitted by
/// [`StateMachineDefinition!`](macro@crate::StateMachineDefinition). It is the
/// proof used by [`State::new`](crate::State::new),
/// [`StateOwned::new`](crate::StateOwned::new), and shared-state constructors:
/// creating a fresh state token only compiles for states declared in the
/// definition crate.
///
/// ```ignore
/// pub struct ConnectionStandin;
/// pub struct Disconnected;
///
/// impl magicstatemachines::Initial<Disconnected> for ConnectionStandin {}
/// ```
///
/// Most users should not write that impl manually; prefer:
///
/// ```ignore
/// magicstatemachines::StateMachineDefinition! {
///     for ConnectionStandin;
///
///     pub Initial: Disconnected;
/// }
/// ```
pub trait Initial<TState> {}

/// Declares that a definition crate permits `TFrom -> TTo`.
///
/// The definition crate owns the stand-in and state types. Rust's orphan rules
/// therefore prevent an implementation crate from adding transitions. This
/// trait is the graph edge only; it does not define what happens to the
/// runtime value during the edge. Runtime effects are supplied later by
/// [`StateMachineImpl!`](macro@crate::StateMachineImpl).
///
/// `F` is the required positional call signature for the transition. A
/// zero-argument transition can use the default `fn()`. A transition that must
/// be called with a `String` declares `type F = fn(String)`.
///
/// ```ignore
/// pub struct ConnectionStandin;
/// pub struct Connected;
/// pub struct Authenticated;
///
/// impl magicstatemachines::Transition<Connected, Authenticated> for ConnectionStandin {
///     type F = fn(String);
/// }
/// ```
///
/// With the definition macro, the same declaration is usually written as:
///
/// ```ignore
/// magicstatemachines::StateMachineDefinition! {
///     for ConnectionStandin;
///
///     pub Initial: Connected;
///     transition Connected => Authenticated(user: String);
/// }
/// ```
///
/// The name `user` is documentation for the contract and for the matching
/// implementation body. The actual transition call remains positional:
/// `transition!(self, user.into())`.
pub trait Transition<TFrom, TTo> {
    /// Function signature required to perform this transition.
    type F = fn();
}

/// Stable proof that a transition signature accepts a tuple of arguments.
#[doc(hidden)]
pub trait TransitionSignature<Args> {}

impl TransitionSignature<()> for fn() {}

macro_rules! transition_signature_impls {
    ($(($($arg:ident),+)),* $(,)?) => {
        $(
            impl<$($arg),+> TransitionSignature<($($arg,)+)> for fn($($arg),+) {}
        )*
    };
}

transition_signature_impls! {
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
}

/// Connects an implementation type to a state-machine definition.
///
/// Implementations are `'static` so storage backends can provide borrowed
/// guard families without repeating the implementation type in the backend.
/// The associated `Standin` selects the definition-crate contract, while
/// `TransitionToken` is the private capability required to perform retagging.
///
/// [`crate::StateMachineImpl!`] generates this implementation and keeps the
/// transition capability's construction private. Code outside the invocation
/// module cannot manufacture the token:
///
/// ```compile_fail
/// use magicstatemachines::{Initial, State, StateMachineImpl, States, StorageStateOwned, Transition};
///
/// mod implementation {
///     use super::*;
///
///     pub struct Standin;
///     pub struct Runtime;
///     States! {
///         Ready;
///         Running;
///     }
///     impl Initial<Ready> for Standin {}
///     impl Transition<Ready, Running> for Standin {}
///
///     magicstatemachines::StateMachineImpl!(Runtime: Standin; transition Ready => Running(););
///
///     pub fn ready() -> State<StorageStateOwned, Runtime, Ready> {
///         State::new(Runtime)
///     }
/// }
///
/// let ready = implementation::ready();
/// let _ = magicstatemachines::transition_state::<_, _, _, implementation::Running>(
///     ready,
///     implementation::__StateMachineTransitionToken(())
/// ).call(());
/// ```
///
/// The generated ergonomic helpers are also private to the invocation module.
/// This means implementation methods can call [`transition!`](macro@crate::transition),
/// but external callers can only call the methods you expose:
///
/// ```compile_fail
/// use magicstatemachines::{Initial, State, States, StorageStateOwned, Transition};
///
/// mod implementation {
///     use super::*;
///
///     pub struct Standin;
///     pub struct Runtime;
///     States! {
///         Ready;
///         Running;
///     }
///
///     impl Initial<Ready> for Standin {}
///     impl Transition<Ready, Running> for Standin {}
///     magicstatemachines::StateMachineImpl!(Runtime: Standin; transition Ready => Running(););
///
///     pub fn ready() -> State<StorageStateOwned, Runtime, Ready> {
///         State::new(Runtime)
///     }
/// }
///
/// let ready = implementation::ready();
/// let _ = magicstatemachines::transition!(ready);
/// ```
pub trait StateMachineImpl: 'static {
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

#[cfg(feature = "unique-rc-arc")]
impl<T> StateMachineImpl for UniqueRc<T>
where
    T: StateMachineImpl + ?Sized,
{
    type Standin = T::Standin;
    type Impl = T::Impl;
    type TransitionToken = T::TransitionToken;
}

#[cfg(feature = "unique-rc-arc")]
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
