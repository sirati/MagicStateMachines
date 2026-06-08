/// Connects a runtime type to a definition and adds private transition helpers.
///
/// Invoke this once in the module that implements the runtime's methods:
///
/// ```ignore
/// StateMachineImpl!(Connection: ConnectionStandin);
///
/// impl Connection {
///     fn connect<Storage>(
///         self: State<Storage, Self, Disconnected>,
///     ) -> State<Storage, Self, Connected>
///     where
///         Storage: SRef,
///     {
///         self.transition()()
///     }
/// }
/// ```
#[macro_export]
macro_rules! StateMachineImpl {
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
            To: $crate::StateTrait,
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
            To: $crate::StateTrait,
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
            fn transition<To>(
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
                To: $crate::StateTrait,
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
            fn transition<To>(
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
                To: $crate::StateTrait,
                From: $crate::StateUnionConcreteState,
                $standin: $crate::Transition<From, To>,
                $implementation: $crate::TransitionEffectSelector<From, To>,
            {
                $crate::transition_state_with_effect(self, __StateMachineTransitionToken(()))
            }

        }

        #[allow(dead_code)]
        trait __GenericStateWithProofTransitionExt<Storage, From, Marker, To, Kind>
        where
            Storage: $crate::StateStorage,
            From: $crate::StateTrait,
            Marker: $crate::StateMarker,
            To: $crate::StateTrait + $crate::StateMarker<Kind = $crate::ConcreteStateKind>,
            Kind: $crate::StateKind,
        {
            #[track_caller]
            fn proven_transition(
                self,
            ) -> $crate::KindProofTransitionCall<
                Storage,
                $implementation,
                From,
                Marker,
                To,
                Kind,
            >;
        }

        impl<Storage, From, Marker, To, Kind>
            __GenericStateWithProofTransitionExt<Storage, From, Marker, To, Kind>
            for $crate::StateWithProof<
                Storage,
                $implementation,
                From,
                $crate::TransitionProof<
                    Storage,
                    $implementation,
                    From,
                    Marker,
                    To,
                    Kind,
                >,
            >
        where
            Storage: $crate::StateStorage,
            Storage::Machine<$implementation>: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
            From: $crate::StateTrait,
            Marker: $crate::StateMarker,
            To: $crate::StateTrait + $crate::StateMarker<Kind = $crate::ConcreteStateKind>,
            Kind: $crate::StateKind,
        {
            #[track_caller]
            fn proven_transition(
                self,
            ) -> $crate::KindProofTransitionCall<
                Storage,
                $implementation,
                From,
                Marker,
                To,
                Kind,
            >
            {
                $crate::transition_state_with_kind_proof::<
                    Storage,
                    $implementation,
                    From,
                    Marker,
                    To,
                    Kind,
                >(self, __StateMachineTransitionToken(()))
            }

        }

        #[allow(dead_code)]
        trait __GenericStateConcreteProofTransitionExt<Storage, From, Marker, To>
        where
            Storage: $crate::StateStorage,
        {
            #[track_caller]
            fn transition(
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
                To: $crate::StateTrait,
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
            To: $crate::StateTrait,
        {
            #[track_caller]
            fn transition(
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
                To: $crate::StateTrait,
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
                To: $crate::StateTrait,
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
            fn transition_discriminated<To>(
                self,
            ) -> $crate::DiscriminatedTransitionCall<
                Storage,
                $implementation,
                Marker,
                To,
            >
            where
                To: $crate::StateTrait;
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
            fn transition_discriminated<To>(
                self,
            ) -> $crate::DiscriminatedTransitionCall<
                Storage,
                $implementation,
                Marker,
                To,
            >
            where
                To: $crate::StateTrait,
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

        trait __StateTransitionExt<T, From>
        where
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            #[track_caller]
            fn transition<To>(self) -> $crate::TransitionCall<T, From, To>
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
            fn transition<To>(self) -> $crate::TransitionCall<T, From, To>
            where
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition(self, __StateMachineTransitionToken(()))
            }
        }

        trait __GenericStateTransitionExt<Storage, T, From>
        where
            T: $crate::StateMachineImpl,
            Storage: $crate::StateStorage,
            Storage::Machine<T>: $crate::StateMachineImpl,
        {
            #[must_use]
            #[track_caller]
            fn transition<To>(self) -> $crate::StateTransitionCall<Storage, T, From, To>
            where
                From: $crate::StateTrait,
                To: $crate::StateTrait,
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
            fn transition<To>(self) -> $crate::StateTransitionCall<Storage, T, From, To>
            where
                From: $crate::StateTrait,
                To: $crate::StateTrait,
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition_state(self, __StateMachineTransitionToken(()))
            }
        }

        trait __StateMutTransitionExt<G, T, From>
        where
            G: ::core::ops::DerefMut<Target = $crate::SharedValue<T>>,
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            fn transition<To>(self) -> $crate::StateMutTransitionCall<G, T, From, To>
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
            fn transition<To>(self) -> $crate::StateMutTransitionCall<G, T, From, To>
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
