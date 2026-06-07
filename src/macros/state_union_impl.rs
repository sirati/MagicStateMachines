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

            #[doc(hidden)]
            #[allow(dead_code)]
            mod [<__state_union_seal_ $marker:snake>] {
                #[allow(dead_code)]
                pub trait Sealed {}
            }

            impl [<__state_union_seal_ $first_super:snake>]::Sealed
                for $crate::StateUnionState<$marker>
            {}
            impl [<In $first_super>]
                for $crate::StateUnionState<$marker>
            {
                $crate::__StateUnion!(@into_erased_variant_impl $first_super);
            }
            impl $crate::StateUnionMember<
                $crate::StateUnionState<$marker>
            > for $first_super {}
            $(
                impl [<__state_union_seal_ $supertrait:snake>]::Sealed
                    for $crate::StateUnionState<$marker>
                {}
                impl [<In $supertrait>]
                    for $crate::StateUnionState<$marker>
                {
                    $crate::__StateUnion!(@into_erased_variant_impl $supertrait);
                }
                impl $crate::StateUnionMember<
                    $crate::StateUnionState<$marker>
                > for $supertrait {}
            )*

            #[allow(dead_code)]
            pub trait [<In $marker>]:
                $crate::StateTrait
                + [<__state_union_seal_ $marker:snake>]::Sealed
                + [<In $first_super>]
                $(+ [<In $supertrait>])*
            {
                $crate::__StateUnion!(@into_erased_method $marker);
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed
                for $crate::StateUnionState<$marker>
            {}
            impl [<In $marker>] for $crate::StateUnionState<$marker> {
                $crate::__StateUnion!(@into_erased_identity_impl $marker);
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed for $first {}
            impl [<In $marker>] for $first {
                $crate::__StateUnion!(@into_erased_variant_impl $marker);
            }
            impl $crate::StateUnionMember<$first>
                for $marker
            {}

            $(
                impl [<__state_union_seal_ $marker:snake>]::Sealed for $state {}
                impl [<In $marker>] for $state {
                    $crate::__StateUnion!(@into_erased_variant_impl $marker);
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
                @transition_effect $marker:
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

            #[doc(hidden)]
            #[allow(dead_code)]
            mod [<__state_union_seal_ $marker:snake>] {
                #[allow(dead_code)]
                pub trait Sealed {}
            }

            #[allow(dead_code)]
            pub trait [<In $marker>]:
                $crate::StateTrait + [<__state_union_seal_ $marker:snake>]::Sealed
            {
                $crate::__StateUnion!(@into_erased_method $marker);
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed
                for $crate::StateUnionState<$marker>
            {}
            impl [<In $marker>] for $crate::StateUnionState<$marker> {
                $crate::__StateUnion!(@into_erased_identity_impl $marker);
            }

            impl [<__state_union_seal_ $marker:snake>]::Sealed for $first {}
            impl [<In $marker>] for $first {
                $crate::__StateUnion!(@into_erased_variant_impl $marker);
            }
            impl $crate::StateUnionMember<$first>
                for $marker
            {}

            $(
                impl [<__state_union_seal_ $marker:snake>]::Sealed for $state {}
                impl [<In $marker>] for $state {
                    $crate::__StateUnion!(@into_erased_variant_impl $marker);
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
                @transition_effect $marker:
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
        @transition_effect $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        impl<T, To> $crate::StateUnionTransitionEffect<T, To> for $marker
        where
            T: $crate::StateMachineImpl
                + $crate::TransitionEffectSelector<$first, To>,
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

        impl<T, To, Args> $crate::StateUnionTransitionEffectApply<T, To, Args>
            for $marker
        where
            T: $crate::StateMachineImpl
                + $crate::TransitionEffectSelector<$first, To>,
            <T as $crate::TransitionEffectSelector<$first, To>>::Effect:
                $crate::TransitionEffect<T, $first, To, Args>,
            $(
                T: $crate::TransitionEffectSelector<
                    $state,
                    To,
                    Effect = <T as $crate::TransitionEffectSelector<$first, To>>::Effect,
                >,
                <T as $crate::TransitionEffectSelector<$first, To>>::Effect:
                    $crate::TransitionEffect<T, $state, To, Args>,
            )*
        {
            fn apply(value: &mut T, args: Args) {
                <<T as $crate::TransitionEffectSelector<$first, To>>::Effect as
                    $crate::TransitionEffect<T, $first, To, Args>>::apply(value, args);
            }
        }
    };
    (@into_erased_method $marker:ident) => {
        #[must_use]
        fn into_erased<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl;
    };
    (@into_erased_identity_impl $marker:ident) => {
        fn into_erased<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            state
        }
    };
    (@into_erased_variant_impl $marker:ident) => {
        fn into_erased<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $crate::StateUnionVariant::<Storage, T, Self, $marker>::new(state).into_erased()
        }
    };
    (
        @maybe_conversion_trait [enum $enum_name:ident] $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__StateUnionEnum!(
            @conversion_trait $marker $enum_name:
            $first $(| $state)*
        );
    };
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
