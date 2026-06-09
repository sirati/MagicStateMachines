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
        #[doc = concat!(
            "Value-carrying enum for the `",
            stringify!($marker),
            "` state union.\n\n",
            "Each variant contains the same runtime value and storage backend, ",
            "but with a concrete state marker. Matching this enum recovers the ",
            "specific concrete state, so state-specific methods and concrete ",
            "transitions become available again.\n\n",
            "Most APIs should name `DiscriminatedState<Storage, T, ",
            stringify!($marker),
            ">` in signatures. Call `discriminate()` when you need this enum ",
            "for branching. After matching, call `into_erased()` on the enum when ",
            "the caller should receive the broader union-typed state again.\n\n",
            "The enum itself is useful when the state machine can return more than ",
            "one possible concrete state and the caller must decide what to do with ",
            "each case. A variant such as `",
            stringify!($enum_name),
            "::",
            stringify!($first),
            "` contains `State<Storage, T, ",
            stringify!($first),
            ">`, not an erased state."
        )]
        #[allow(dead_code)]
        pub enum $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            #[doc = concat!(
                "The union value is currently in state `",
                stringify!($first),
                "`."
            )]
            $first(
                $crate::State<Storage, T, $first>
            ),
            $(
                #[doc = concat!(
                    "The union value is currently in state `",
                    stringify!($state),
                    "`."
                )]
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
            #[doc = concat!(
                "Converts this concrete enum variant back into a discriminated `",
                stringify!($marker),
                "` state.\n\n",
                "Use this after matching when the return type should be the broader ",
                "union state again instead of one concrete variant. The resulting ",
                "`DiscriminatedState<Storage, T, ",
                stringify!($marker),
                ">` still remembers the concrete variant for later dynamic ",
                "transition or `discriminate()` calls.\n\n",
                "This does not run a transition effect and does not change the runtime ",
                "value. It only changes the type-level view from a concrete enum variant ",
                "back to the named union."
            )]
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
