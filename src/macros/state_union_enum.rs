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
            pub trait [<$marker IntoEnum>]: [<In $marker>] + $crate::StateUnionConcreteState {
                #[must_use]
                fn into_enum<Storage, T>(
                    state: $crate::State<Storage, T, Self>,
                ) -> $enum_name<Storage, T>
                where
                    Self: Sized,
                    Storage: $crate::StateStorage,
                    T: $crate::StateMachineImpl;
            }

            impl [<$marker IntoEnum>] for $first {
                fn into_enum<Storage, T>(
                    state: $crate::State<Storage, T, Self>,
                ) -> $enum_name<Storage, T>
                where
                    Self: Sized,
                    Storage: $crate::StateStorage,
                    T: $crate::StateMachineImpl,
                {
                    $enum_name::$first($crate::StateUnionVariant::new(state))
                }
            }

            $(
                impl [<$marker IntoEnum>] for $state {
                    fn into_enum<Storage, T>(
                        state: $crate::State<Storage, T, Self>,
                    ) -> $enum_name<Storage, T>
                    where
                        Self: Sized,
                        Storage: $crate::StateStorage,
                        T: $crate::StateMachineImpl,
                    {
                        $enum_name::$state($crate::StateUnionVariant::new(state))
                    }
                }
            )*
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
            $first($crate::StateUnionVariant<Storage, T, $first, $marker>),
            $(
                $state($crate::StateUnionVariant<Storage, T, $state, $marker>),
            )*
        }

        impl<Storage, T> ::core::ops::Deref for $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            type Target = $crate::State<Storage, T, $crate::StateUnionState<$marker>>;

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
            ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>> {
                match self {
                    Self::$first(state) => state.into_erased(),
                    $(
                        Self::$state(state) => state.into_erased(),
                    )*
                }
            }
        }

        impl<Storage, T> ::core::convert::From<$crate::State<Storage, T, $first>>
            for $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            fn from(state: $crate::State<Storage, T, $first>) -> Self {
                Self::$first($crate::StateUnionVariant::new(state))
            }
        }

        $(
            impl<Storage, T> ::core::convert::From<$crate::State<Storage, T, $state>>
                for $enum_name<Storage, T>
            where
                Storage: $crate::StateStorage,
                T: $crate::StateMachineImpl,
            {
                fn from(state: $crate::State<Storage, T, $state>) -> Self {
                    Self::$state($crate::StateUnionVariant::new(state))
                }
            }
        )*
    };
}
