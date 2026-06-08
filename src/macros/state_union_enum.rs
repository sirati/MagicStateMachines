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
        @conversion_trait $marker:ident $enum_name:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[allow(dead_code)]
            pub trait [<$marker IntoEnum>] {
                #[must_use]
                fn into_enum<Storage, T>(
                    state: $crate::State<Storage, T, Self>,
                ) -> $crate::DiscriminatedState<Storage, T, $marker>
                where
                    Self: Sized,
                    Storage: $crate::StateStorage,
                    T: $crate::StateMachineImpl;
            }

            impl [<$marker IntoEnum>] for $first {
                fn into_enum<Storage, T>(
                    state: $crate::State<Storage, T, Self>,
                ) -> $crate::DiscriminatedState<Storage, T, $marker>
                where
                    Storage: $crate::StateStorage,
                    T: $crate::StateMachineImpl,
                {
                    $crate::__private::paste! {
                        $crate::discriminate_state::<Storage, T, Self, $marker>(
                            state,
                            [<$marker Discriminator>]::$first,
                        )
                    }
                }
            }

            $(
                impl [<$marker IntoEnum>] for $state {
                    fn into_enum<Storage, T>(
                        state: $crate::State<Storage, T, Self>,
                    ) -> $crate::DiscriminatedState<Storage, T, $marker>
                    where
                        Storage: $crate::StateStorage,
                        T: $crate::StateMachineImpl,
                    {
                        $crate::__private::paste! {
                            $crate::discriminate_state::<Storage, T, Self, $marker>(
                                state,
                                [<$marker Discriminator>]::$state,
                            )
                        }
                    }
                }
            )*

            impl [<$marker IntoEnum>] for $crate::StateUnionState<$marker> {
                fn into_enum<Storage, T>(
                    state: $crate::State<Storage, T, Self>,
                ) -> $crate::DiscriminatedState<Storage, T, $marker>
                where
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

                        match discriminator {
                            [<$marker Discriminator>]::$first => {
                                $crate::rediscriminate_union_state::<Storage, T, $marker, $marker>(
                                    state,
                                    discriminator,
                                )
                            }
                            $(
                                [<$marker Discriminator>]::$state => {
                                    $crate::rediscriminate_union_state::<Storage, T, $marker, $marker>(
                                        state,
                                        discriminator,
                                    )
                                }
                            )*
                        }
                    }
                }
            }

            impl<Storage, T> [<$marker IntoEnum>] for $enum_name<Storage, T>
            where
                Storage: $crate::StateStorage,
                T: $crate::StateMachineImpl,
            {
                fn into_enum<OtherStorage, OtherT>(
                    _state: $crate::State<OtherStorage, OtherT, Self>,
                ) -> $crate::DiscriminatedState<OtherStorage, OtherT, $marker>
                where
                    OtherStorage: $crate::StateStorage,
                    OtherT: $crate::StateMachineImpl,
                {
                    unreachable!("discriminated states are already enum values")
                }
            }
        }
    };
    (
        @enum $enum_name:ident $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            pub enum [<$marker Discriminator>] {
                $first,
                $(
                    $state,
                )*
            }
        }

        #[allow(dead_code)]
        pub enum $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $first(
                $crate::__private::paste! {
                    $crate::StateUnionVariant<
                        Storage,
                        T,
                        $first,
                        $marker,
                        [<$marker Discriminator>],
                    >
                }
            ),
            $(
                $state(
                    $crate::__private::paste! {
                        $crate::StateUnionVariant<
                            Storage,
                            T,
                            $state,
                            $marker,
                            [<$marker Discriminator>],
                        >
                    }
                ),
            )*
        }

        impl $crate::StateUnionDiscriminant for $marker {
            type Discriminator = $crate::__private::paste! { [<$marker Discriminator>] };

            type Enum<Storage, T>
                = $enum_name<Storage, T>
            where
                Storage: $crate::StateStorage,
                T: $crate::StateMachineImpl;

            fn discriminate<Storage, T>(
                state: $crate::__private::paste! {
                    $crate::State<
                        $crate::SDiscriminated<Storage, [<$marker Discriminator>]>,
                        T,
                        $crate::StateUnionState<$marker>,
                    >
                },
            ) -> Self::Enum<Storage, T>
            where
                Storage: $crate::StateStorage,
                T: $crate::StateMachineImpl,
            {
                let discriminator = $crate::discriminated_state_discriminator(&state);
                match discriminator {
                    $crate::__private::paste! { [<$marker Discriminator>]::$first } => {
                        $enum_name::$first($crate::StateUnionVariant::from_erased(state))
                    }
                    $(
                        $crate::__private::paste! { [<$marker Discriminator>]::$state } => {
                            $enum_name::$state($crate::StateUnionVariant::from_erased(state))
                        }
                    )*
                }
            }
        }

        impl<Storage, T> ::core::ops::Deref for $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            type Target = $crate::__private::paste! {
                $crate::State<
                    $crate::SDiscriminated<Storage, [<$marker Discriminator>]>,
                    T,
                    $crate::StateUnionState<$marker>,
                >
            };

            fn deref(&self) -> &Self::Target {
                match self {
                    Self::$first(state) => state,
                    $(
                        Self::$state(state) => state,
                    )*
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
                    $crate::SDiscriminated<Storage, [<$marker Discriminator>]>,
                    T,
                    $crate::StateUnionState<$marker>,
                >
            } {
                match self {
                    Self::$first(state) => state.into_erased(),
                    $(
                        Self::$state(state) => state.into_erased(),
                    )*
                }
            }
        }

    };
}
