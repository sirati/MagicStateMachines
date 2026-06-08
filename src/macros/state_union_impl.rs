#[doc(hidden)]
#[macro_export]
macro_rules! __StateUnion {
    (
        @trait $marker:ident [$first_super:ident $(, $supertrait:ident)*] $enum:tt:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[allow(dead_code)]
            pub struct $marker;

            impl $crate::StateMarker for $marker {
                type Kind = $crate::UnionStateKind;
            }

            #[doc(hidden)]
            #[allow(dead_code)]
            mod [<__state_union_seal_ $marker:snake>] {
                #[allow(dead_code)]
                pub trait Sealed {}
            }

            impl [<__state_union_seal_ $first_super:snake>]::Sealed
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::StateUnionProofMembership<$first_super>
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::StateUnionErased<$first_super>
                for $crate::StateUnionState<$marker>
            {
                $crate::__StateUnion!(
                    @erased_union_variant_impl $marker => $first_super:
                    $first $(| $state)*
                );
            }
            impl [<In $first_super>]
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::In<$first_super>
                for $crate::StateUnionState<$marker>
            {
                $crate::__StateUnion!(
                    @into_enum_union_variant_impl $marker => $first_super:
                    $first $(| $state)*
                );
            }
            impl $crate::StateUnionMember<
                $crate::StateUnionState<$marker>
            > for $first_super {}
            $(
                impl [<__state_union_seal_ $supertrait:snake>]::Sealed
                    for $crate::StateUnionState<$marker>
                {}
                impl $crate::StateUnionProofMembership<$supertrait>
                    for $crate::StateUnionState<$marker>
                {}
                impl $crate::StateUnionErased<$supertrait>
                    for $crate::StateUnionState<$marker>
                {
                    $crate::__StateUnion!(@erased_identity_impl $supertrait);
                }
                impl [<In $supertrait>]
                    for $crate::StateUnionState<$marker>
                {}
                impl $crate::In<$supertrait>
                    for $crate::StateUnionState<$marker>
                {
                    $crate::__StateUnion!(@into_enum_identity_impl $supertrait);
                }
                impl $crate::StateUnionMember<
                    $crate::StateUnionState<$marker>
                > for $supertrait {}
            )*

            #[allow(dead_code)]
            pub trait [<In $marker>]:
                $crate::StateTrait
                + $crate::StateMarker
                + [<__state_union_seal_ $marker:snake>]::Sealed
                + $crate::In<$marker>
                + $crate::StateUnionErased<$marker>
                + $crate::StateUnionProofMembership<$marker>
                + [<In $first_super>]
                $(+ [<In $supertrait>])*
            {
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::StateUnionProofMembership<$marker>
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::StateUnionErased<$marker>
                for $crate::StateUnionState<$marker>
            {
                $crate::__StateUnion!(@erased_identity_impl $marker);
            }
            impl [<In $marker>] for $crate::StateUnionState<$marker> {}
            impl $crate::In<$marker> for $crate::StateUnionState<$marker> {
                $crate::__StateUnion!(@into_enum_identity_impl $marker);
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed for $first {}
            impl $crate::StateUnionProofMembership<$marker> for $first {}
            impl $crate::StateUnionErased<$marker> for $first {
                $crate::__StateUnion!(@erased_variant_impl $marker $first);
            }
            impl [<In $marker>] for $first {}
            impl $crate::In<$marker> for $first {
                $crate::__StateUnion!(@into_enum_variant_impl $marker $first);
            }
            impl $crate::StateUnionMember<$first>
                for $marker
            {}

            $(
                impl [<__state_union_seal_ $marker:snake>]::Sealed for $state {}
                impl $crate::StateUnionProofMembership<$marker> for $state {}
                impl $crate::StateUnionErased<$marker> for $state {
                    $crate::__StateUnion!(@erased_variant_impl $marker $state);
                }
                impl [<In $marker>] for $state {}
                impl $crate::In<$marker> for $state {
                    $crate::__StateUnion!(@into_enum_variant_impl $marker $state);
                }
                impl $crate::StateUnionMember<$state>
                    for $marker
                {}
            )*

            impl<Standin, To> $crate::StateUnionTransition<Standin, To>
                for $marker
            where
                Standin: $crate::Transition<$first, To>,
                $(
                    Standin: $crate::Transition<
                        $state,
                        To,
                        F = <Standin as $crate::Transition<$first, To>>::F,
                    >,
                )*
            {
                type F = <Standin as $crate::Transition<$first, To>>::F;
            }

            $crate::__StateUnion!(
                @shared_effect $marker:
                $first $(| $state)*
            );

            $crate::__StateUnion!(
                @discriminated_transition $marker:
                $first $(| $state)*
            );

            impl $crate::StateUnionRuntime for $marker {
                fn contains(state: &dyn $crate::StateTrait) -> bool {
                    state.type_id() == ::core::any::TypeId::of::<$first>()
                        $(
                            || state.type_id() == ::core::any::TypeId::of::<$state>()
                        )*
                        || state.type_id()
                            == ::core::any::TypeId::of::<$crate::StateUnionState<$marker>>()
                }

                fn expected_type_name() -> &'static str {
                    ::core::any::type_name::<$crate::StateUnionState<$marker>>()
                }
            }

            $crate::__StateUnion!(
                @maybe_conversion_trait $enum $marker:
                $first $(| $state)*
            );
        }
    };
    (
        @trait $marker:ident [] $enum:tt:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[allow(dead_code)]
            pub struct $marker;

            impl $crate::StateMarker for $marker {
                type Kind = $crate::UnionStateKind;
            }

            #[doc(hidden)]
            #[allow(dead_code)]
            mod [<__state_union_seal_ $marker:snake>] {
                #[allow(dead_code)]
                pub trait Sealed {}
            }

            #[allow(dead_code)]
            pub trait [<In $marker>]:
                $crate::StateTrait
                + $crate::StateMarker
                + [<__state_union_seal_ $marker:snake>]::Sealed
                + $crate::In<$marker>
                + $crate::StateUnionErased<$marker>
                + $crate::StateUnionProofMembership<$marker>
            {
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::StateUnionProofMembership<$marker>
                for $crate::StateUnionState<$marker>
            {}
            impl $crate::StateUnionErased<$marker>
                for $crate::StateUnionState<$marker>
            {
                $crate::__StateUnion!(@erased_identity_impl $marker);
            }
            impl [<In $marker>] for $crate::StateUnionState<$marker> {}
            impl $crate::In<$marker> for $crate::StateUnionState<$marker> {
                $crate::__StateUnion!(@into_enum_identity_impl $marker);
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed for $first {}
            impl $crate::StateUnionProofMembership<$marker> for $first {}
            impl $crate::StateUnionErased<$marker> for $first {
                $crate::__StateUnion!(@erased_variant_impl $marker $first);
            }
            impl [<In $marker>] for $first {}
            impl $crate::In<$marker> for $first {
                $crate::__StateUnion!(@into_enum_variant_impl $marker $first);
            }
            impl $crate::StateUnionMember<$first>
                for $marker
            {}

            $(
                impl [<__state_union_seal_ $marker:snake>]::Sealed for $state {}
                impl $crate::StateUnionProofMembership<$marker> for $state {}
                impl $crate::StateUnionErased<$marker> for $state {
                    $crate::__StateUnion!(@erased_variant_impl $marker $state);
                }
                impl [<In $marker>] for $state {}
                impl $crate::In<$marker> for $state {
                    $crate::__StateUnion!(@into_enum_variant_impl $marker $state);
                }
                impl $crate::StateUnionMember<$state>
                    for $marker
                {}
            )*

            impl<Standin, To> $crate::StateUnionTransition<Standin, To>
                for $marker
            where
                Standin: $crate::Transition<$first, To>,
                $(
                    Standin: $crate::Transition<
                        $state,
                        To,
                        F = <Standin as $crate::Transition<$first, To>>::F,
                    >,
                )*
            {
                type F = <Standin as $crate::Transition<$first, To>>::F;
            }

            $crate::__StateUnion!(
                @shared_effect $marker:
                $first $(| $state)*
            );

            $crate::__StateUnion!(
                @discriminated_transition $marker:
                $first $(| $state)*
            );

            impl $crate::StateUnionRuntime for $marker {
                fn contains(state: &dyn $crate::StateTrait) -> bool {
                    state.type_id() == ::core::any::TypeId::of::<$first>()
                        $(
                            || state.type_id() == ::core::any::TypeId::of::<$state>()
                        )*
                        || state.type_id()
                            == ::core::any::TypeId::of::<$crate::StateUnionState<$marker>>()
                }

                fn expected_type_name() -> &'static str {
                    ::core::any::type_name::<$crate::StateUnionState<$marker>>()
                }
            }

            $crate::__StateUnion!(
                @maybe_conversion_trait $enum $marker:
                $first $(| $state)*
            );
        }
    };
    (
        @standalone_enum $enum_name:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__StateUnionEnum!(@standalone_enum $enum_name: $first $(| $state)*);
    };
    (
        @discriminated_transition $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        impl<T, To, Args> $crate::StateUnionDiscriminatedTransition<T, To, Args> for $marker
        where
            T: $crate::StateMachineImpl
                + $crate::TransitionEffectSelector<$first, To>,
            To: $crate::StateTrait,
            <T as $crate::TransitionEffectSelector<$first, To>>::Effect:
                $crate::TransitionEffect<T, $first, To, Args>,
            $(
                T: $crate::TransitionEffectSelector<$state, To>,
                <T as $crate::TransitionEffectSelector<$state, To>>::Effect:
                    $crate::TransitionEffect<T, $state, To, Args>,
            )*
        {
            fn transition<Storage>(
                state: $crate::DiscriminatedState<Storage, T, Self>,
                args: Args,
                callsite: $crate::TransitionCallsite,
            ) -> $crate::State<Storage, T, To>
            where
                Storage: $crate::SMut,
                To: $crate::StateTrait,
            {
                $crate::__private::paste! {
                    let discriminator = $crate::discriminated_state_discriminator(&state);
                    match discriminator {
                        [<$marker Discriminator>]::$first => {
                            let state =
                                $crate::concretize_discriminated_state::<Storage, T, $marker, $first>(
                                    state,
                                );
                            $crate::transition_concrete_after_effect::<
                                Storage,
                                T,
                                $first,
                                To,
                                Args,
                                <T as $crate::TransitionEffectSelector<$first, To>>::Effect,
                            >(state, args, callsite)
                        }
                        $(
                            [<$marker Discriminator>]::$state => {
                                let state =
                                    $crate::concretize_discriminated_state::<Storage, T, $marker, $state>(
                                        state,
                                    );
                                $crate::transition_concrete_after_effect::<
                                    Storage,
                                    T,
                                    $state,
                                    To,
                                    Args,
                                    <T as $crate::TransitionEffectSelector<$state, To>>::Effect,
                                >(state, args, callsite)
                            }
                        )*
                    }
                }
            }
        }
    };
    (
        @shared_effect $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        impl<T, To> $crate::StateUnionSharedEffect<T, To> for $marker
        where
            T: $crate::StateMachineImpl
                + $crate::TransitionEffectSelector<$first, To>,
            To: $crate::StateTrait,
            $marker: $crate::StateUnionTransition<T::Standin, To>,
            $(
                T: $crate::TransitionEffectSelector<
                    $state,
                    To,
                    Effect = <T as $crate::TransitionEffectSelector<$first, To>>::Effect,
                >,
            )*
        {
            type Effect = <T as $crate::TransitionEffectSelector<$first, To>>::Effect;
        }

        impl<T, To, Args> $crate::StateUnionSharedTransitionEffect<T, To, Args>
            for $marker
        where
            T: $crate::StateMachineImpl
                + $crate::TransitionEffectSelector<$first, To>,
            To: $crate::StateTrait,
            $marker: $crate::StateUnionSharedEffect<
                T,
                To,
                Effect = <T as $crate::TransitionEffectSelector<$first, To>>::Effect,
            >,
            <T as $crate::TransitionEffectSelector<$first, To>>::Effect:
                $crate::TransitionEffect<T, $first, To, Args>,
        {
            fn apply(value: &mut T, args: Args) {
                <<T as $crate::TransitionEffectSelector<$first, To>>::Effect as $crate::TransitionEffect<
                    T,
                    $first,
                    To,
                    Args,
                >>::apply(value, args);
            }
        }
    };
    (@into_enum_method $marker:ident) => {
        type Marker: $crate::StateUnionDiscriminant
            + $crate::StateMarker<Kind = $crate::UnionStateKind>;

        #[must_use]
        fn into_enum<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $marker>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl;
    };
    (@erased_identity_impl $marker:ident) => {
        fn into_union_erased<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $marker>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
            $marker: $crate::StateUnionDiscriminant,
        {
            $crate::__private::paste! {
                let discriminator =
                    $crate::state_union_discriminator::<
                        Storage,
                        T,
                        Self,
                        [<$marker Discriminator>],
                    >(&state)
                    .expect("state union discriminator is unavailable");
                $crate::rediscriminate_union_state::<Storage, T, $marker, $marker>(
                    state,
                    discriminator,
                )
            }
        }

    };
    (@erased_variant_impl $marker:ident $variant:ident) => {
        fn into_union_erased<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $marker>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
            $marker: $crate::StateUnionDiscriminant,
        {
            $crate::__private::paste! {
                $crate::discriminate_state::<Storage, T, Self, $marker>(
                    state,
                    [<$marker Discriminator>]::$variant,
                )
            }
        }

    };
    (
        @erased_union_variant_impl $source:ident => $target:ident:
        $first:ident $(| $state:ident)*
    ) => {
        fn into_union_erased<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $target>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
            $target: $crate::StateUnionDiscriminant,
        {
            $crate::__private::paste! {
                let discriminator =
                    $crate::state_union_discriminator::<
                        Storage,
                        T,
                        Self,
                        [<$source Discriminator>],
                    >(&state)
                    .expect("state union discriminator is unavailable");
                let discriminator = match discriminator {
                    [<$source Discriminator>]::$first => [<$target Discriminator>]::$first,
                    $(
                        [<$source Discriminator>]::$state => [<$target Discriminator>]::$state,
                    )*
                };
                $crate::rediscriminate_union_state::<Storage, T, $source, $target>(
                    state,
                    discriminator,
                )
            }
        }

    };
    (@into_enum_identity_impl $marker:ident) => {
        fn into_enum<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $marker>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $crate::__private::paste! {
                let discriminator =
                    $crate::state_union_discriminator::<
                        Storage,
                        T,
                        Self,
                        [<$marker Discriminator>],
                    >(&state)
                    .expect("state union discriminator is unavailable");
                $crate::rediscriminate_union_state::<Storage, T, $marker, $marker>(
                    state,
                    discriminator,
                )
            }
        }

    };
    (@into_enum_variant_impl $marker:ident $variant:ident) => {
        fn into_enum<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $marker>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $crate::__private::paste! {
                $crate::discriminate_state::<Storage, T, Self, $marker>(
                    state,
                    [<$marker Discriminator>]::$variant,
                )
            }
        }

    };
    (
        @into_enum_union_variant_impl $source:ident => $target:ident:
        $first:ident $(| $state:ident)*
    ) => {
        fn into_enum<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::DiscriminatedState<Storage, T, $target>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $crate::__private::paste! {
                let discriminator =
                    $crate::state_union_discriminator::<
                        Storage,
                        T,
                        Self,
                        [<$source Discriminator>],
                    >(&state)
                    .expect("state union discriminator is unavailable");
                let discriminator = match discriminator {
                    [<$source Discriminator>]::$first => [<$target Discriminator>]::$first,
                    $(
                        [<$source Discriminator>]::$state => [<$target Discriminator>]::$state,
                    )*
                };
                $crate::rediscriminate_union_state::<Storage, T, $source, $target>(
                    state,
                    discriminator,
                )
            }
        }

    };
    (
        @maybe_conversion_trait [enum $enum_name:ident] $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {};
    (
        @maybe_conversion_trait [] $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {};
    (
        @enum $enum_name:ident $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__StateUnionEnum!(@enum $enum_name $marker: $first $(| $state)*);
    };
}
