#[doc(hidden)]
#[macro_export]
macro_rules! __StateUnionEnum {
    (
        @standalone_enum $enum_name:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct [<__state_union_marker_ $enum_name:snake>];

            impl $crate::StateUnionMember<$first>
                for [<__state_union_marker_ $enum_name:snake>]
            {}
            $(
                impl $crate::StateUnionMember<$state>
                    for [<__state_union_marker_ $enum_name:snake>]
                {}
            )*

            impl<Standin, To> $crate::StateUnionTransition<Standin, To>
                for [<__state_union_marker_ $enum_name:snake>]
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

            $crate::__StateUnionEnum!(
                @enum $enum_name [<__state_union_marker_ $enum_name:snake>]:
                $first $(| $state)*
            );
        }
    };
    (
        @enum $enum_name:ident $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        #[allow(dead_code)]
        pub enum $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $first(
                $crate::State<Storage, T, $first>
            ),
            $(
                $state(
                    $crate::State<Storage, T, $state>
                ),
            )*
        }

        impl $crate::StateUnionDiscriminant for $marker {
            type Enum<Storage, T>
                = $enum_name<Storage, T>
            where
                Storage: $crate::StateStorage,
                T: $crate::StateMachineImpl;

            fn discriminate<Storage, T>(
                state: $crate::__private::paste! {
                    $crate::State<
                        $crate::SDiscriminated<Storage>,
                        T,
                        $crate::StateUnionState<$marker>,
                    >
                },
            ) -> Self::Enum<Storage, T>
            where
                Storage: $crate::StateStorage,
                T: $crate::StateMachineImpl,
            {
                let inferred_state_type = $crate::erased_state_type_id(
                    &$crate::discriminated_state_marker(&state),
                );
                match inferred_state_type {
                    state_type if state_type == ::core::any::TypeId::of::<$first>() => {
                        $enum_name::$first(
                            $crate::concretize_discriminated_state::<Storage, T, $marker, $first>(
                                state,
                            )
                        )
                    }
                    $(
                        state_type if state_type == ::core::any::TypeId::of::<$state>() => {
                            $enum_name::$state(
                                $crate::concretize_discriminated_state::<Storage, T, $marker, $state>(
                                    state,
                                )
                            )
                        }
                    )*
                    _ => unreachable!("state union inferred a state outside of its variants"),
                }
            }
        }

        #[allow(dead_code)]
        impl<Storage, T> $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            pub fn into_erased(
                self,
            ) -> $crate::__private::paste! {
                $crate::State<
                    $crate::SDiscriminated<Storage>,
                    T,
                    $crate::StateUnionState<$marker>,
                >
            } {
                match self {
                    Self::$first(state) => {
                        $crate::discriminate_state::<Storage, T, $first, $marker>(state)
                    }
                    $(
                        Self::$state(state) => {
                            $crate::discriminate_state::<Storage, T, $state, $marker>(state)
                        }
                    )*
                }
            }
        }

    };
}
