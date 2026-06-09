/// Defines the state-machine contract for a stand-in type.
///
/// Use this macro in the crate that owns the state marker types and the
/// stand-in ZST. The generated code contains only contract information:
/// [`Initial`] implementations, [`Transition`] implementations, and optional
/// state-union declarations. It deliberately contains no runtime type and no
/// transition bodies. Think of this as the public state-machine interface: it
/// says which states exist, which states may be constructed initially, and
/// which edges are legal.
///
/// That separation is the main enforcement mechanism. A downstream crate can
/// implement behavior for its runtime type, but it cannot add new legal
/// transitions unless it owns the stand-in or the state markers. In the normal
/// split-crate layout, the definition crate owns both, so Rust's orphan rules
/// make the transition graph a hard public contract.
///
/// The transition argument list declares the signature required by
/// [`transition!`](macro@crate::transition). Argument names are documentation
/// for the contract; only their types participate in the generated
/// [`Transition::F`](crate::Transition::F) signature. For example,
/// `transition Connected => Authenticated(user: String);` emits an impl whose
/// signature is equivalent to:
///
/// ```ignore
/// impl magicstatemachines::Transition<Connected, Authenticated> for ConnectionStandin {
///     type F = fn(String);
/// }
/// ```
///
/// It does not say what happens to the runtime data. The implementation crate
/// supplies that effect later with [`StateMachineImpl!`](macro@crate::StateMachineImpl).
///
/// ```ignore
/// use magicstatemachines::{StateMachineDefinition, States};
///
/// pub struct ConnectionStandin;
///
/// pub mod states {
///     use magicstatemachines::States;
///
///     States! {
///         Disconnected;
///         Reconnecting;
///         Connected;
///         Authenticated;
///         Failed;
///     }
/// }
///
/// use states::*;
///
/// StateMachineDefinition! {
///     for ConnectionStandin;
///
///     Initial: Disconnected | Reconnecting;
///
///     transition Disconnected => Connected | Failed();
///     transition Connected => Authenticated(user: String);
///     transition Connected | Authenticated => Disconnected();
///     transition Authenticated => Connected();
///
///     union All: Disconnected | Connected | Authenticated;
///     union Online: All, Connected | Authenticated;
/// }
/// ```
///
/// `|` on the left or right of a transition expands to every pair. For
/// example, `transition Authenticated => Connected | Failed();` declares both
/// `Authenticated -> Connected` and `Authenticated -> Failed` with the same
/// empty signature. `transition Connected | Authenticated => Disconnected();`
/// declares two incoming edges into `Disconnected`. This is only a declaration;
/// whether the implementation shares a body is decided later by
/// [`StateMachineImpl!`](macro@crate::StateMachineImpl).
///
/// Union declarations are forwarded to [`StateUnion!`](macro@crate::StateUnion).
/// They can be written here for convenience, but they are still independent of
/// the stand-in and may also be written separately. A union such as
/// `union Online: Connected | Authenticated;` does not add transitions by
/// itself; it only gives APIs a way to name "any online state".
///
/// A transition declaration must end in `;`. Bodies are rejected on purpose:
///
/// ```compile_fail
/// use magicstatemachines::{StateMachineDefinition, States};
///
/// pub struct Standin;
/// States! { A; B; }
///
/// StateMachineDefinition! {
///     for Standin;
///
///     Initial: A;
///
///     transition A => B() {
///         // Effects belong in `StateMachineImpl!`, not in the definition.
///     }
/// }
/// ```
///
/// [`Initial`]: crate::Initial
/// [`Transition`]: crate::Transition
#[macro_export]
macro_rules! StateMachineDefinition {
    (
        for $standin:ty;
        Initial: $first_initial:ident $(| $initial:ident)*;
        $($transitions:tt)*
    ) => {
        $crate::__StateMachineDefinition!(@initial_impls $standin; $first_initial $(| $initial)*);

        $crate::__StateMachineDefinition!(
            @parse $standin;
            $($transitions)*
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __StateMachineDefinition {
    (@initial_impls $standin:ty; $first_initial:ident $(| $initial:ident)*) => {
        impl $crate::Initial<$first_initial> for $standin {}
        $(
            impl $crate::Initial<$initial> for $standin {}
        )*
    };
    (@parse $standin:ty;) => {};
    (
        @parse $standin:ty;
        union $name:ident:
        $first:ident $(| $state:ident)*;
        $($rest:tt)*
    ) => {
        $crate::StateUnion!($name: $first $(| $state)*);
        $crate::__StateMachineDefinition!(@parse $standin; $($rest)*);
    };
    (
        @parse $standin:ty;
        union $name:ident:
        $first_super:ident $(+ $supertrait:ident)*,
        $first:ident $(| $state:ident)*;
        $($rest:tt)*
    ) => {
        $crate::StateUnion!($name: $first_super $(+ $supertrait)*, $first $(| $state)*);
        $crate::__StateMachineDefinition!(@parse $standin; $($rest)*);
    };
    (
        @parse $standin:ty;
        transition $first_from:ident $(| $from:ident)* => $first_to:ident $(| $to:ident)*
            ($($arg:ident : $arg_ty:ty),* $(,)?);
        $($rest:tt)*
    ) => {
        $crate::__StateMachineDefinition!(
            @transition_impls $standin; [$first_from $(| $from)*] => [$first_to $(| $to)*]
            ($($arg : $arg_ty),*)
        );
        $crate::__StateMachineDefinition!(@parse $standin; $($rest)*);
    };
    (
        @parse $standin:ty;
        transition $first_from:ident $(| $from:ident)* => $first_to:ident $(| $to:ident)*
            ($($arg:ident : $arg_ty:ty),* $(,)?),
        $($rest:tt)*
    ) => {
        ::core::compile_error!(
            "state-machine definition transitions are declarations and must end with `;`"
        );
    };
    (
        @parse $standin:ty;
        transition $first_from:ident $(| $from:ident)* => $first_to:ident $(| $to:ident)*
            ($($arg:ident : $arg_ty:ty),* $(,)?) { $($body:tt)* }
        $($rest:tt)*
    ) => {
        ::core::compile_error!(
            "state-machine definition transitions cannot contain implementation bodies"
        );
    };
    (
        @parse $standin:ty;
        transition $first_from:ident $(| $from:ident)* => $first_to:ident $(| $to:ident)*
            ($($arg:ident : $arg_ty:ty),* $(,)?) { $($body:tt)* },
        $($rest:tt)*
    ) => {
        ::core::compile_error!(
            "state-machine definition transitions cannot contain implementation bodies"
        );
    };
    (
        @transition_impls $standin:ty; [$first_from:ident $(| $from:ident)*] => [$first_to:ident $(| $to:ident)*]
        $args:tt
    ) => {
        $crate::__StateMachineDefinition!(
            @transition_impls_for_from $standin; $first_from => [$first_to $(| $to)*] $args
        );
        $crate::__StateMachineDefinition!(
            @transition_impls_for_froms $standin; [$($from)|*] => [$first_to $(| $to)*] $args
        );
    };
    (@transition_impls_for_froms $standin:ty; [] => $targets:tt $args:tt) => {};
    (
        @transition_impls_for_froms $standin:ty; [$first_from:ident $(| $from:ident)*] => $targets:tt $args:tt
    ) => {
        $crate::__StateMachineDefinition!(
            @transition_impls_for_from $standin; $first_from => $targets $args
        );
        $crate::__StateMachineDefinition!(
            @transition_impls_for_froms $standin; [$($from)|*] => $targets $args
        );
    };
    (
        @transition_impls_for_from $standin:ty; $from:ident => [$first_to:ident $(| $to:ident)*]
        $args:tt
    ) => {
        $crate::__StateMachineDefinition!(@transition_impl $standin; $from => $first_to $args);
        $(
            $crate::__StateMachineDefinition!(@transition_impl $standin; $from => $to $args);
        )*
    };
    (@transition_impl $standin:ty; $from:ident => $to:ident ()) => {
        impl $crate::Transition<$from, $to> for $standin {}
    };
    (
        @transition_impl $standin:ty; $from:ident => $to:ident
        ($first_arg:ident : $first_arg_ty:ty $(, $arg:ident : $arg_ty:ty)*)
    ) => {
        impl $crate::Transition<$from, $to> for $standin {
            type F = fn($first_arg_ty $(, $arg_ty)*);
        }
    };
}
