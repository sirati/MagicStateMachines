/// Connects a runtime type to a state-machine definition and generates its effects.
///
/// Invoke this macro in the implementation crate, in the same module as the
/// runtime type's state-specific methods. It implements
/// [`StateMachineImpl`](trait@crate::StateMachineImpl), creates a private
/// transition token, and emits module-local extension traits consumed by
/// [`transition!`](macro@crate::transition). Keeping the token private is what
/// prevents callers from retagging states directly.
///
/// The macro also emits raw-state construction helpers for the implementation:
///
/// - `Runtime::with_state(value)` is public and safe, but only works for
///   states declared `Initial` by the definition crate. Treat `Initial` as a
///   public constructor contract: anyone with raw `Runtime` can attach one of
///   those states.
/// - `Runtime::with_state_priv(value)` is private to the invocation module and
///   works for states listed with `priv Initial: StateName;` inside this
///   macro. Use this for target-owned conversions from another state machine.
/// - `Runtime::with_state_unsafe(value)` is the explicit unsafe escape hatch
///   for arbitrary concrete states.
///
/// A private raw construction state is declared before or between transitions:
///
/// ```ignore
/// StateMachineImpl! {
///     Job: JobStandin;
///
///     priv Initial: Authenticated;
///
///     transition Authenticated => Authorised();
/// }
/// ```
///
/// Transition headers must match transitions declared by
/// [`StateMachineDefinition!`](macro@crate::StateMachineDefinition). The
/// definition macro proves that an edge is legal; this macro decides what
/// effect runs when that edge is taken for a particular runtime type. In this
/// macro, headers may also carry implementation bodies:
///
/// - `;` means the transition has no effect body. The runtime value is only
///   retagged from `From` to `To`.
/// - `{ ... }` runs the body against a mutable runtime value before retagging.
///   Inside the body, `self` refers to `&mut Runtime`, not to the state token.
/// - `,` after a header queues that header and shares the next body with it.
///   This is not punctuation sugar. It records that those transition edges use
///   the same effect type. Static union transitions depend on that fact:
///   `transition!(const Online self)` only compiles when every concrete member
///   uses the same body and signature for the chosen target.
/// - `pinned transition From => To(...) { ... }` declares an effect that runs
///   with `self` bound to `Pin<&mut Runtime>` instead of `&mut Runtime`. Call
///   it with [`transition!(pin state, ...)`](macro@crate::transition) from a
///   method whose storage is bounded by `S: SPinMut`. This is the form to use
///   when the runtime is `!Unpin` and the effect must call pinned methods or
///   update fields through interior mutability without ever exposing `&mut T`.
///   Pinned and ordinary effects are separate contracts, so the same
///   `From => To` edge may have both a normal body and a pinned body. The call
///   form decides which one is used.
///
/// The macro intentionally generates awkward hidden helper names such as
/// `_magicsm_transition`, `_magicsm_transitionConst`, `_magicsm_transitionDyn`,
/// `_magicsm_transitionPin`, `_magicsm_transitionPinConst`, and
/// `_magicsm_transitionPinDyn`. User code should call them through
/// [`transition!`](macro@crate::transition), which keeps transition calls
/// uniform and prevents method-name collisions with normal implementation
/// methods.
///
/// A typical implementation keeps the public methods ordinary Rust methods and
/// uses `State<Storage, Self, StateMarker>` as the receiver. The storage bound
/// documents what the method needs: `SRef` for reading, `SMut` for mutation and
/// transitions, and `SMove` when a backend must be movable by value.
///
/// ```ignore
/// use magicstatemachines::{transition, SMut, State, StateMachineImpl};
/// use test_def::{ConnectionStandin, InOnline, Online};
/// use test_def::states::{Authenticated, Connected, Disconnected};
///
/// pub struct Connection {
///     user: Option<String>,
/// }
///
/// StateMachineImpl! {
///     Connection: ConnectionStandin;
///
///     transition Disconnected => Connected();
///
///     transition Connected => Authenticated(user: String) {
///         self.user = Some(user);
///     }
///
///     transition Connected | Authenticated => Disconnected(),
///     transition Authenticated => Connected() {
///         self.user = None;
///     }
/// }
///
/// impl Connection {
///     pub fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
///     where
///         S: SMut,
///     {
///         transition!(self)
///     }
///
///     pub fn authenticate<S>(
///         self: State<S, Self, Connected>,
///         user: impl Into<String>,
///     ) -> State<S, Self, Authenticated>
///     where
///         S: SMut,
///     {
///         transition!(self, user.into())
///     }
///
///     pub fn disconnect<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
///     where
///         S: SMut,
///     {
///         transition!(dyn Online self)
///     }
/// }
/// ```
///
/// In the example, the two `=> Disconnected` headers share the same body, so
/// either `transition!(const Online self)` or `transition!(dyn Online self)`
/// can be used. If `Connected => Disconnected` and
/// `Authenticated => Disconnected` had different bodies, the static form would
/// stop compiling and the dynamic form would remain valid by discriminating the
/// concrete runtime state first.
///
/// Use the static form when the union proof is part of the API contract:
///
/// ```ignore
/// fn disconnect<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
/// where
///     S: SMut,
/// {
///     transition!(const Online self)
/// }
/// ```
///
/// Use the dynamic form when the body may differ per member, or when the value
/// is already stored as a discriminated union state:
///
/// ```ignore
/// fn stop<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
/// where
///     S: SMut,
/// {
///     transition!(dyn Online self,)
/// }
/// ```
///
/// Pinned transitions are intentionally separate from ordinary transitions.
/// `SPinBox<T, S>` does not implement `SMut` for `T: !Unpin`, so a normal
/// body cannot accidentally receive `&mut T`. The pinned body receives
/// `Pin<&mut T>` and the method advertises that by requiring `S: SPinMut`:
///
/// ```ignore
/// use core::{cell::Cell, marker::PhantomPinned, pin::Pin};
/// use magicstatemachines::{transition, SPinMut, State};
///
/// struct Connection {
///     ready: Cell<bool>,
///     _pin: PhantomPinned,
/// }
///
/// impl Connection {
///     fn mark_ready(self: Pin<&mut Self>) {
///         self.as_ref().get_ref().ready.set(true);
///     }
/// }
///
/// StateMachineImpl! {
///     Connection: ConnectionStandin;
///
///     pinned transition Disconnected => Connected() {
///         self.as_mut().mark_ready();
///     }
/// }
///
/// impl Connection {
///     fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
///     where
///         S: SPinMut,
///     {
///         transition!(pin self)
///     }
/// }
/// ```
///
/// If an edge can be used from both movable and pinned storage, define both
/// bodies. The state-machine definition still has only one edge; the
/// implementation macro records two effects for that edge:
///
/// ```ignore
/// StateMachineImpl! {
///     Connection: ConnectionStandin;
///
///     transition Disconnected => Connected() {
///         self.connected = true;
///     }
///
///     pinned transition Disconnected => Connected() {
///         self.as_ref().connected.set(true);
///     }
/// }
///
/// impl Connection {
///     fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
///     where
///         S: SMut,
///     {
///         transition!(self)
///     }
///
///     fn connect_pinned<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
///     where
///         S: SPinMut,
///     {
///         transition!(pin self)
///     }
/// }
/// ```
///
/// Pinned transitions also have static and dynamic union forms. Use
/// `transition!(pin const Online self)` when every member of the union shares
/// the same pinned body and signature for the selected target. Use
/// `transition!(pin dyn Online self)` when the current concrete member should
/// be discriminated first and each member's own pinned body should run.
#[macro_export]
#[cfg_attr(not(feature = "gen_no_unsafe"), allow(unsafe_code))]
#[cfg_attr(not(feature = "gen_no_unsafe"), allow_internal_unsafe)]
macro_rules! StateMachineImpl {
    ($($input:tt)*) => {
        $crate::__StateMachineImplPublic!($($input)*);
    };
}

#[cfg(not(feature = "gen_no_unsafe"))]
#[doc(hidden)]
#[macro_export]
#[allow(unsafe_code)]
#[allow_internal_unsafe]
macro_rules! __StateMachineImplUnsafeConstructor {
    ($implementation:ty) => {
        #[doc(hidden)]
        pub unsafe fn with_state_unsafe<S>(
            value: $implementation,
        ) -> $crate::ConcreteStated<$implementation, S>
        where
            S: $crate::ConcreteStateTrait,
        {
            // This unsafe function intentionally uses only safe Rust in its body.
            $crate::__private::concrete_stated_new(value, __StateMachineTransitionToken(()))
        }
    };
}

#[cfg(feature = "gen_no_unsafe")]
#[doc(hidden)]
#[macro_export]
macro_rules! __StateMachineImplUnsafeConstructor {
    ($implementation:ty) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __StateMachineImplPublic {
    (
        $implementation:ty : $standin:ty;
        $($transitions:tt)*
    ) => {
        #[doc(hidden)]
        pub struct __StateMachineTransitionToken(());

        impl $crate::StateMachineImpl for $implementation {
            type Standin = $standin;
            type Impl = $implementation;
            type TransitionToken = __StateMachineTransitionToken;
        }

        trait __StateMachineRawState<S> {}

        #[allow(dead_code)]
        impl $implementation {
            #[doc(hidden)]
            fn with_state_priv<S>(value: $implementation) -> $crate::ConcreteStated<$implementation, S>
            where
                S: $crate::ConcreteStateTrait,
                __StateMachineTransitionToken: __StateMachineRawState<S>,
            {
                $crate::__private::concrete_stated_new(value, __StateMachineTransitionToken(()))
            }

            #[doc(hidden)]
            pub fn with_state<S>(value: $implementation) -> $crate::ConcreteStated<$implementation, S>
            where
                S: $crate::ConcreteStateTrait,
                $standin: $crate::Initial<S>,
            {
                $crate::__private::concrete_stated_new(value, __StateMachineTransitionToken(()))
            }

            $crate::__StateMachineImplUnsafeConstructor!($implementation);
        }

        $crate::__StateMachineImpl!(
            @parse $implementation; $standin; [];
            $($transitions)*
        );

        #[doc(hidden)]
        pub struct __StateMachineUnionTransitionEffect<Marker, To>(
            ::core::marker::PhantomData<fn() -> (Marker, To)>,
        );

        impl<Marker, To> $crate::TransitionEffectSelector<$crate::StateUnionState<Marker>, To>
            for $implementation
        where
            Marker: $crate::StateUnionSharedEffect<$implementation, To>,
            To: $crate::ConcreteStateTrait,
        {
            type Effect = __StateMachineUnionTransitionEffect<Marker, To>;
        }

        impl<Marker, To, Args> $crate::TransitionEffect<
            $implementation,
            $crate::StateUnionState<Marker>,
            To,
            Args,
        > for __StateMachineUnionTransitionEffect<Marker, To>
        where
            Marker: $crate::StateUnionSharedTransitionEffect<$implementation, To, Args>,
            To: $crate::ConcreteStateTrait,
        {
            fn apply(value: &mut $implementation, args: Args) {
                <Marker as $crate::StateUnionSharedTransitionEffect<
                    $implementation,
                    To,
                    Args,
                >>::apply(value, args);
            }
        }

        #[allow(dead_code)]
        trait __GenericStateTransitionExt<Storage, From>
        where
            Storage: $crate::StateStorage,
        {
            #[must_use]
            #[track_caller]
            fn _magicsm_transition<To>(
                self,
            ) -> $crate::EffectTransitionCall<
                Storage,
                $implementation,
                From,
                To,
                <$implementation as $crate::TransitionEffectSelector<From, To>>::Effect,
            >
            where
                From: $crate::StateTrait,
                To: $crate::ConcreteStateTrait,
                From: $crate::StateUnionConcreteState,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::TransitionEffectSelector<From, To>;

        }

        impl<Storage, From> __GenericStateTransitionExt<Storage, From>
            for $crate::State<Storage, $implementation, From>
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[track_caller]
            fn _magicsm_transition<To>(
                self,
            ) -> $crate::EffectTransitionCall<
                Storage,
                $implementation,
                From,
                To,
                <$implementation as $crate::TransitionEffectSelector<From, To>>::Effect,
            >
            where
                From: $crate::StateTrait,
                To: $crate::ConcreteStateTrait,
                From: $crate::StateUnionConcreteState,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::TransitionEffectSelector<From, To>,
            {
                $crate::transition_state_with_effect(self, __StateMachineTransitionToken(()))
            }

        }

        #[allow(dead_code, non_snake_case)]
        trait __GenericStatePinnedTransitionExt<Storage, From>
        where
            Storage: $crate::StateStorage,
        {
            #[allow(non_snake_case)]
            #[must_use]
            #[track_caller]
            fn _magicsm_transitionPin<To>(
                self,
            ) -> $crate::PinnedEffectTransitionCall<
                Storage,
                $implementation,
                From,
                To,
                <$implementation as $crate::PinnedTransitionEffectSelector<From, To>>::Effect,
            >
            where
                From: $crate::StateTrait,
                To: $crate::ConcreteStateTrait,
                From: $crate::StateUnionConcreteState,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::PinnedTransitionEffectSelector<From, To>;

        }

        impl<Storage, From> __GenericStatePinnedTransitionExt<Storage, From>
            for $crate::State<Storage, $implementation, From>
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionPin<To>(
                self,
            ) -> $crate::PinnedEffectTransitionCall<
                Storage,
                $implementation,
                From,
                To,
                <$implementation as $crate::PinnedTransitionEffectSelector<From, To>>::Effect,
            >
            where
                From: $crate::StateTrait,
                To: $crate::ConcreteStateTrait,
                From: $crate::StateUnionConcreteState,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::PinnedTransitionEffectSelector<From, To>,
            {
                $crate::transition_state_with_pinned_effect(self, __StateMachineTransitionToken(()))
            }

        }

        #[allow(dead_code, non_snake_case)]
        trait __GenericStateMarkerPinnedTransitionExt<Storage, From>
        where
            Storage: $crate::StateStorage,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionPinDyn<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::PinnedDiscriminatedTransitionCall<
                Storage,
                $implementation,
                Marker,
                To,
            >
            where
                From: $crate::StateTrait + $crate::In<Marker>,
                Marker: $crate::StateUnionDiscriminant,
                To: $crate::ConcreteStateTrait;
        }

        impl<Storage, From> __GenericStateMarkerPinnedTransitionExt<Storage, From>
            for $crate::State<Storage, $implementation, From>
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionPinDyn<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::PinnedDiscriminatedTransitionCall<
                Storage,
                $implementation,
                Marker,
                To,
            >
            where
                From: $crate::StateTrait + $crate::In<Marker>,
                Marker: $crate::StateUnionDiscriminant,
                To: $crate::ConcreteStateTrait,
            {
                let state = <From as $crate::In<Marker>>::into_discriminated(self);
                $crate::transition_discriminated_state_pinned(state, __StateMachineTransitionToken(()))
            }

        }

        #[allow(dead_code, non_snake_case)]
        trait __GenericStateMarkerTransitionExt<Storage, From>
        where
            Storage: $crate::StateStorage,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionDyn<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::KindProofTransitionCall<
                Storage,
                $implementation,
                From,
                Marker,
                To,
                <Marker as $crate::StateMarker>::Kind,
            >
            where
                From: $crate::StateTrait + $crate::In<Marker>,
                Marker: $crate::StateMarker,
                To: $crate::ConcreteStateTrait;
        }

        impl<Storage, From> __GenericStateMarkerTransitionExt<Storage, From>
            for $crate::State<Storage, $implementation, From>
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionDyn<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::KindProofTransitionCall<
                Storage,
                $implementation,
                From,
                Marker,
                To,
                <Marker as $crate::StateMarker>::Kind,
            >
            where
                From: $crate::StateTrait + $crate::In<Marker>,
                Marker: $crate::StateMarker,
                To: $crate::ConcreteStateTrait,
            {
                $crate::transition_state_with_kind_proof::<
                    Storage,
                    $implementation,
                    From,
                    Marker,
                    To,
                    <Marker as $crate::StateMarker>::Kind,
                >(
                    self.with(<From as $crate::In<Marker>>::prove()),
                    __StateMachineTransitionToken(()),
                )
            }

        }

        #[allow(dead_code, non_snake_case)]
        trait __GenericStateMarkerStaticTransitionExt<Storage, From>
        where
            Storage: $crate::StateStorage,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionConst<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::StateUnionProofTransitionCall<Storage, $implementation, From, Marker, To>
            where
                From: $crate::StateTrait
                    + $crate::In<Marker>
                    + $crate::StateUnionErased<Marker>
                    + $crate::UnionTransitionProof<$implementation, Marker, To>,
                Marker: $crate::StateUnionDiscriminant
                    + $crate::StateUnionTransition<$standin, To>
                    + $crate::StateUnionSharedEffect<$implementation, To>,
                To: $crate::ConcreteStateTrait,
                $standin: $crate::Transition<
                    $crate::StateUnionState<Marker>,
                    To,
                    F = <Marker as $crate::StateUnionTransition<$standin, To>>::F,
                >;
        }

        impl<Storage, From> __GenericStateMarkerStaticTransitionExt<Storage, From>
            for $crate::State<Storage, $implementation, From>
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionConst<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::StateUnionProofTransitionCall<Storage, $implementation, From, Marker, To>
            where
                From: $crate::StateTrait
                    + $crate::In<Marker>
                    + $crate::StateUnionErased<Marker>
                    + $crate::UnionTransitionProof<$implementation, Marker, To>,
                Marker: $crate::StateUnionDiscriminant
                    + $crate::StateUnionTransition<$standin, To>
                    + $crate::StateUnionSharedEffect<$implementation, To>,
                To: $crate::ConcreteStateTrait,
                $standin: $crate::Transition<
                    $crate::StateUnionState<Marker>,
                    To,
                    F = <Marker as $crate::StateUnionTransition<$standin, To>>::F,
                >,
            {
                $crate::transition_state_with_static_union_proof::<
                    Storage,
                    $implementation,
                    From,
                    Marker,
                    To,
                >(
                    self,
                    __StateMachineTransitionToken(()),
                )
            }

        }

        #[allow(dead_code, non_snake_case)]
        trait __GenericStateMarkerPinnedStaticTransitionExt<Storage, From>
        where
            Storage: $crate::StateStorage,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionPinConst<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::PinnedStateUnionProofTransitionCall<Storage, $implementation, From, Marker, To>
            where
                From: $crate::StateTrait
                    + $crate::In<Marker>
                    + $crate::StateUnionErased<Marker>,
                Marker: $crate::StateUnionDiscriminant
                    + $crate::StateUnionTransition<$standin, To>
                    + $crate::StateUnionSharedPinnedEffect<$implementation, To>,
                To: $crate::ConcreteStateTrait,
                $standin: $crate::Transition<
                    $crate::StateUnionState<Marker>,
                    To,
                    F = <Marker as $crate::StateUnionTransition<$standin, To>>::F,
                >;
        }

        impl<Storage, From> __GenericStateMarkerPinnedStaticTransitionExt<Storage, From>
            for $crate::State<Storage, $implementation, From>
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[allow(non_snake_case)]
            #[track_caller]
            fn _magicsm_transitionPinConst<Marker, To>(
                self,
                _marker: Marker,
            ) -> $crate::PinnedStateUnionProofTransitionCall<Storage, $implementation, From, Marker, To>
            where
                From: $crate::StateTrait
                    + $crate::In<Marker>
                    + $crate::StateUnionErased<Marker>,
                Marker: $crate::StateUnionDiscriminant
                    + $crate::StateUnionTransition<$standin, To>
                    + $crate::StateUnionSharedPinnedEffect<$implementation, To>,
                To: $crate::ConcreteStateTrait,
                $standin: $crate::Transition<
                    $crate::StateUnionState<Marker>,
                    To,
                    F = <Marker as $crate::StateUnionTransition<$standin, To>>::F,
                >,
            {
                $crate::transition_state_with_static_union_pinned_proof::<
                    Storage,
                    $implementation,
                    From,
                    Marker,
                    To,
                >(
                    self,
                    __StateMachineTransitionToken(()),
                )
            }

        }

        #[allow(dead_code)]
        trait __GenericStateConcreteProofTransitionExt<Storage, From, Marker, To>
        where
            Storage: $crate::StateStorage,
        {
            #[track_caller]
            fn _magicsm_transition(
                self,
            ) -> $crate::EffectTransitionCall<
                Storage,
                $implementation,
                From,
                To,
                <$implementation as $crate::TransitionEffectSelector<From, To>>::Effect,
            >
            where
                Storage: $crate::SRef,
                Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
                From: $crate::StateTrait + $crate::StateUnionConcreteState,
                Marker: $crate::StateUnionDiscriminant,
                To: $crate::ConcreteStateTrait,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::TransitionEffectSelector<From, To>;
        }

        impl<Storage, From, Marker, To>
            __GenericStateConcreteProofTransitionExt<Storage, From, Marker, To>
            for $crate::StateConcreteProvenState<
                Storage,
                $implementation,
                From,
                Marker,
                To,
            >
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
            From: $crate::StateTrait,
            Marker: $crate::StateUnionDiscriminant,
            To: $crate::ConcreteStateTrait,
        {
            #[track_caller]
            fn _magicsm_transition(
                self,
            ) -> $crate::EffectTransitionCall<
                Storage,
                $implementation,
                From,
                To,
                <$implementation as $crate::TransitionEffectSelector<From, To>>::Effect,
            >
            where
                From: $crate::StateTrait + $crate::StateUnionConcreteState,
                To: $crate::ConcreteStateTrait,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::TransitionEffectSelector<From, To>,
                Marker: $crate::StateUnionDiscriminant,
            {
                $crate::transition_state_with_concrete_proof(
                    self,
                    __StateMachineTransitionToken(()),
                )
            }

        }

        #[allow(dead_code)]
        impl $implementation {
            #[track_caller]
            fn transition<Storage, From, Marker, To>(
                self: $crate::StateUnionProvenState<
                    Storage,
                    $implementation,
                    From,
                    Marker,
                    To,
                >,
            ) -> $crate::StateUnionProofTransitionCall<
                Storage,
                $implementation,
                From,
                Marker,
                To,
            >
            where
                Storage: $crate::SRef,
                Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
                From: $crate::StateUnionErased<Marker>,
                Marker: $crate::StateUnionSharedEffect<$implementation, To>,
                To: $crate::ConcreteStateTrait,
            {
                $crate::transition_state_with_union_proof(
                    self,
                    __StateMachineTransitionToken(()),
                )
            }
        }

        #[allow(dead_code)]
        trait __GenericStateUnionTransitionExt<Storage, Marker>
        where
            Storage: $crate::StateStorage,
            Marker: $crate::StateUnionDiscriminant,
        {
            #[must_use]
            #[track_caller]
            fn _magicsm_transition_discriminated<To>(
                self,
            ) -> $crate::DiscriminatedTransitionCall<
                Storage,
                $implementation,
                Marker,
                To,
            >
            where
                To: $crate::ConcreteStateTrait;
        }

        impl<Storage, Marker> __GenericStateUnionTransitionExt<Storage, Marker>
            for $crate::DiscriminatedState<Storage, $implementation, Marker>
        where
            Storage: $crate::StateStorage,
            Marker: $crate::StateUnionDiscriminant,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[track_caller]
            fn _magicsm_transition_discriminated<To>(
                self,
            ) -> $crate::DiscriminatedTransitionCall<
                Storage,
                $implementation,
                Marker,
                To,
            >
            where
                To: $crate::ConcreteStateTrait,
            {
                $crate::transition_discriminated_state(self, __StateMachineTransitionToken(()))
            }
        }
    };
    ($implementation:ty : $standin:ty $(,)?) => {
        #[doc(hidden)]
        pub struct __StateMachineTransitionToken(());

        impl $crate::StateMachineImpl for $implementation {
            type Standin = $standin;
            type Impl = $implementation;
            type TransitionToken = __StateMachineTransitionToken;
        }

        trait __StateMachineRawState<S> {}

        #[allow(dead_code)]
        impl $implementation {
            #[doc(hidden)]
            fn with_state_priv<S>(value: $implementation) -> $crate::ConcreteStated<$implementation, S>
            where
                S: $crate::ConcreteStateTrait,
                __StateMachineTransitionToken: __StateMachineRawState<S>,
            {
                $crate::__private::concrete_stated_new(value, __StateMachineTransitionToken(()))
            }

            #[doc(hidden)]
            pub fn with_state<S>(value: $implementation) -> $crate::ConcreteStated<$implementation, S>
            where
                S: $crate::ConcreteStateTrait,
                $standin: $crate::Initial<S>,
            {
                $crate::__private::concrete_stated_new(value, __StateMachineTransitionToken(()))
            }

            $crate::__StateMachineImplUnsafeConstructor!($implementation);
        }

        #[allow(non_snake_case)]
        trait __StateTransitionExt<T, From>
        where
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            #[track_caller]
            fn _magicsm_transition<To>(self) -> $crate::TransitionCall<T, From, To>
            where
                T::Standin: $crate::Transition<From, To>;
        }

        impl<T, From> __StateTransitionExt<T, From> for $crate::StateOwned<T, From>
        where
            T: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[track_caller]
            fn _magicsm_transition<To>(self) -> $crate::TransitionCall<T, From, To>
            where
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition(self, __StateMachineTransitionToken(()))
            }
        }

        #[allow(non_snake_case)]
        trait __GenericStateTransitionExt<Storage, T, From>
        where
            T: $crate::StateMachineImpl,
            Storage: $crate::StateStorage,
            Storage::Machine<T>: $crate::StateMachineImpl,
        {
            #[must_use]
            #[track_caller]
            fn _magicsm_transition<To>(self) -> $crate::StateTransitionCall<Storage, T, From, To>
            where
                From: $crate::StateTrait,
                To: $crate::ConcreteStateTrait,
                T::Standin: $crate::Transition<From, To>;
        }

        impl<Storage, T, From> __GenericStateTransitionExt<Storage, T, From>
            for $crate::State<Storage, T, From>
        where
            T: $crate::StateMachineImpl,
            Storage: $crate::StateStorage,
            Storage::Machine<T>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[track_caller]
            fn _magicsm_transition<To>(self) -> $crate::StateTransitionCall<Storage, T, From, To>
            where
                From: $crate::StateTrait,
                To: $crate::ConcreteStateTrait,
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition_state(self, __StateMachineTransitionToken(()))
            }
        }

        #[allow(non_snake_case)]
        trait __StateMutTransitionExt<G, T, From>
        where
            G: ::core::ops::DerefMut<Target = $crate::SharedValue<T>>,
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            fn _magicsm_transition<To>(self) -> $crate::StateMutTransitionCall<G, T, From, To>
            where
                T::Standin: $crate::Transition<From, To>;
        }

        impl<G, T, From> __StateMutTransitionExt<G, T, From> for $crate::StateMut<G, T, From>
        where
            G: ::core::ops::DerefMut<Target = $crate::SharedValue<T>>,
            T: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            fn _magicsm_transition<To>(self) -> $crate::StateMutTransitionCall<G, T, From, To>
            where
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition_mut(self, __StateMachineTransitionToken(()))
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __StateMachineImpl {
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
    ) => {
        $crate::__StateMachineImpl!(@finish_pending [$($pending)*]);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        priv Initial: $first_state:ident $(| $state:ident)*;
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; {}; $($pending)*);
        $crate::__StateMachineImpl!(@raw_state_impls $first_state $(| $state)*);
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        Initial: $first_state:ident $(| $state:ident)*;
        $($rest:tt)*
    ) => {
        ::core::compile_error!(
            "`Initial:` inside `StateMachineImpl!` would declare a public initial state, but public initial states belong in `StateMachineDefinition!` and are already available through `with_state`. Use `priv Initial:` here only for private implementation-owned initial states, and do not repeat public definition initial states."
        );
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        with_state $first_state:ident $(| $state:ident)*;
        $($rest:tt)*
    ) => {
        ::core::compile_error!(
            "`with_state` has been renamed to `priv Initial:` in `StateMachineImpl!`; use `priv Initial: StateName;` for private implementation-owned initial states."
        );
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        pinned transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?);
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; {}; $($pending)*);
        $crate::__StateMachineImpl!(
            @pinned_effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) {}
        );
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        pinned transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?) { $($body:tt)* },
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; { $($body)* }; $($pending)*);
        $crate::__StateMachineImpl!(
            @pinned_effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) { $($body)* }
        );
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        pinned transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?) { $($body:tt)* }
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; { $($body)* }; $($pending)*);
        $crate::__StateMachineImpl!(
            @pinned_effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) { $($body)* }
        );
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?),
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(
            @parse $implementation; $standin;
            [
                $($pending)*
                { $first_from $(| $from)* => $to ($($arg : $arg_ty),*) }
            ];
            $($rest)*
        );
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?);
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; {}; $($pending)*);
        $crate::__StateMachineImpl!(
            @effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) {}
        );
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?) { $($body:tt)* },
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; { $($body)* }; $($pending)*);
        $crate::__StateMachineImpl!(
            @effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) { $($body)* }
        );
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (
        @parse $implementation:ty; $standin:ty; [$($pending:tt)*];
        transition $first_from:ident $(| $from:ident)* => $to:ident
            ($($arg:ident : $arg_ty:ty),* $(,)?) { $($body:tt)* }
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(@emit_pending $implementation; $standin; { $($body)* }; $($pending)*);
        $crate::__StateMachineImpl!(
            @effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) { $($body)* }
        );
        $crate::__StateMachineImpl!(@parse $implementation; $standin; []; $($rest)*);
    };
    (@finish_pending []) => {};
    (@finish_pending [$($pending:tt)+]) => {
        ::core::compile_error!(
            "comma-terminated state-machine transitions must be followed by a transition body or semicolon"
        );
    };
    (
        @emit_pending $implementation:ty; $standin:ty; { $($body:tt)* };
    ) => {};
    (
        @emit_pending $implementation:ty; $standin:ty; { $($body:tt)* };
        { $first_from:ident $(| $from:ident)* => $to:ident ($($arg:ident : $arg_ty:ty),*) }
        $($rest:tt)*
    ) => {
        $crate::__StateMachineImpl!(
            @effect_impls $implementation; $standin; $first_from $(| $from)* => $to
            ($($arg : $arg_ty),*) { $($body)* }
        );
        $crate::__StateMachineImpl!(
            @emit_pending $implementation; $standin; { $($body)* };
            $($rest)*
        );
    };
    (@raw_state_impls $first_state:ident $(| $state:ident)*) => {
        impl __StateMachineRawState<$first_state> for __StateMachineTransitionToken {}
        $(
            impl __StateMachineRawState<$state> for __StateMachineTransitionToken {}
        )*
    };
    (
        @effect_impls $implementation:ty; $standin:ty; $first_from:ident $(| $from:ident)* => $to:ident
        $args:tt $body:tt
    ) => {
        $crate::__private::paste! {
            #[doc(hidden)]
            pub struct [<__StateMachineTransitionEffect $first_from To $to>];
        }

        $crate::__StateMachineImpl!(
            @effect_impl $implementation; $standin; $first_from $first_from => $to
            $args $body
        );
        $(
            $crate::__StateMachineImpl!(
                @effect_impl $implementation; $standin; $from $first_from => $to
                $args $body
            );
        )*

    };
    (
        @pinned_effect_impls $implementation:ty; $standin:ty; $first_from:ident $(| $from:ident)* => $to:ident
        $args:tt $body:tt
    ) => {
        $crate::__private::paste! {
            #[doc(hidden)]
            pub struct [<__StateMachinePinnedTransitionEffect $first_from To $to>];
        }

        $crate::__StateMachineImpl!(
            @pinned_effect_impl $implementation; $standin; $first_from $first_from => $to
            $args $body
        );
        $(
            $crate::__StateMachineImpl!(
                @pinned_effect_impl $implementation; $standin; $from $first_from => $to
                $args $body
            );
        )*

    };
    (
        @effect_impl $implementation:ty; $standin:ty; $from:ident $effect_from:ident => $to:ident
        ($($arg:ident : $arg_ty:ty),*) { $($body:tt)* }
    ) => {
        $crate::__private::paste! {
            impl $crate::TransitionEffectSelector<$from, $to> for $implementation {
                type Effect = [<__StateMachineTransitionEffect $effect_from To $to>];
            }

            impl $crate::TransitionEffect<$implementation, $from, $to, ($($arg_ty,)*)>
                for [<__StateMachineTransitionEffect $effect_from To $to>]
            where
                $standin: $crate::Transition<$from, $to, F = fn($($arg_ty),*)>,
            {
                fn apply(
                    __state_machine_value: &mut $implementation,
                    ($($arg,)*): ($($arg_ty,)*),
                ) {
                    let __state_machine_self = __state_machine_value;
                    $crate::__StateMachineImpl!(@replace_self __state_machine_self; []; $($body)*);
                }
            }
        }
    };
    (
        @pinned_effect_impl $implementation:ty; $standin:ty; $from:ident $effect_from:ident => $to:ident
        ($($arg:ident : $arg_ty:ty),*) { $($body:tt)* }
    ) => {
        $crate::__private::paste! {
            impl $crate::PinnedTransitionEffectSelector<$from, $to> for $implementation {
                type Effect = [<__StateMachinePinnedTransitionEffect $effect_from To $to>];
            }

            impl $crate::PinnedTransitionEffect<$implementation, $from, $to, ($($arg_ty,)*)>
                for [<__StateMachinePinnedTransitionEffect $effect_from To $to>]
            where
                $standin: $crate::Transition<$from, $to, F = fn($($arg_ty),*)>,
            {
                fn apply(
                    __state_machine_value: ::core::pin::Pin<&mut $implementation>,
                    ($($arg,)*): ($($arg_ty,)*),
                ) {
                    let mut __state_machine_self = __state_machine_value;
                    $crate::__StateMachineImpl!(@replace_self __state_machine_self; []; $($body)*);
                }
            }
        }
    };
    (@replace_self $self_ident:ident; [$($out:tt)*];) => {
        $($out)*
    };
    (@replace_self $self_ident:ident; [$($out:tt)*]; self $($rest:tt)*) => {
        $crate::__StateMachineImpl!(
            @replace_self $self_ident; [$($out)* $self_ident]; $($rest)*
        )
    };
    (@replace_self $self_ident:ident; [$($out:tt)*]; $token:tt $($rest:tt)*) => {
        $crate::__StateMachineImpl!(
            @replace_self $self_ident; [$($out)* $token]; $($rest)*
        )
    };
}
