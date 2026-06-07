#[doc(hidden)]
#[macro_export]
macro_rules! __StateUnion {
    (
        @trait $name:ident [$first_super:ident $(, $supertrait:ident)*]:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct [<__state_union_marker_ $name:snake>];

            #[doc(hidden)]
            mod [<__state_union_seal_ $name:snake>] {
                pub trait Sealed {}
            }

            impl [<__state_union_seal_ $first_super:snake>]::Sealed
                for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            {}
            impl $first_super
                for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            {}
            $(
                impl [<__state_union_seal_ $supertrait:snake>]::Sealed
                    for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
                {}
                impl $supertrait
                    for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
                {}
            )*

            pub trait $name:
                [<__state_union_seal_ $name:snake>]::Sealed
                + $first_super
                $(+ $supertrait)*
            {}

            impl [<__state_union_seal_ $name:snake>]::Sealed
                for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            {}
            impl $name for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]> {}

            impl [<__state_union_seal_ $name:snake>]::Sealed for $first {}
            impl $name for $first {}
            impl $crate::StateUnionMember<$first>
                for [<__state_union_marker_ $name:snake>]
            {}

            $(
                impl [<__state_union_seal_ $name:snake>]::Sealed for $state {}
                impl $name for $state {}
                impl $crate::StateUnionMember<$state>
                    for [<__state_union_marker_ $name:snake>]
                {}
            )*

            impl<Standin, To> $crate::StateUnionTransition<Standin, To>
                for [<__state_union_marker_ $name:snake>]
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
        }
    };
    (
        @trait $name:ident []:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__private::paste! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct [<__state_union_marker_ $name:snake>];

            #[doc(hidden)]
            mod [<__state_union_seal_ $name:snake>] {
                pub trait Sealed {}
            }

            pub trait $name: [<__state_union_seal_ $name:snake>]::Sealed {}

            impl [<__state_union_seal_ $name:snake>]::Sealed
                for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            {}
            impl $name for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]> {}

            impl [<__state_union_seal_ $name:snake>]::Sealed for $first {}
            impl $name for $first {}
            impl $crate::StateUnionMember<$first>
                for [<__state_union_marker_ $name:snake>]
            {}

            $(
                impl [<__state_union_seal_ $name:snake>]::Sealed for $state {}
                impl $name for $state {}
                impl $crate::StateUnionMember<$state>
                    for [<__state_union_marker_ $name:snake>]
                {}
            )*

            impl<Standin, To> $crate::StateUnionTransition<Standin, To>
                for [<__state_union_marker_ $name:snake>]
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
        }
    };
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

            $crate::__StateUnion!(
                @enum $enum_name [<__state_union_marker_ $enum_name:snake>]:
                $first $(| $state)*
            );
        }
    };
    (
        @enum $enum_name:ident $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
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

        impl<Storage, T> $enum_name<Storage, T>
        where
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            pub fn into_joint(
                self,
            ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>> {
                match self {
                    Self::$first(state) => state.into_joint(),
                    $(
                        Self::$state(state) => state.into_joint(),
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
