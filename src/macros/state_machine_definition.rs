/// Defines the initial state and legal transitions for a state-machine stand-in.
///
/// This macro is intended for the definition crate. It emits only [`Initial`]
/// and [`Transition`] implementations, so implementation crates cannot extend
/// the contract through orphan rules.
///
/// ```ignore
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
