#[doc(hidden)]
#[macro_export]
macro_rules! __StateUnion {
    (
        @trait $name:ident [$first_super:ident $(, $supertrait:ident)*] $enum:tt:
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
            {
                $crate::__StateUnion!(@into_joint_variant_impl [<__state_union_marker_ $first_super:snake>]);
            }
            impl $crate::StateUnionMember<
                $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            > for [<__state_union_marker_ $first_super:snake>] {}
            $(
                impl [<__state_union_seal_ $supertrait:snake>]::Sealed
                    for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
                {}
                impl $supertrait
                    for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
                {
                    $crate::__StateUnion!(@into_joint_variant_impl [<__state_union_marker_ $supertrait:snake>]);
                }
                impl $crate::StateUnionMember<
                    $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
                > for [<__state_union_marker_ $supertrait:snake>] {}
            )*

            pub trait $name:
                [<__state_union_seal_ $name:snake>]::Sealed
                + $first_super
                $(+ $supertrait)*
            {
                $crate::__StateUnion!(@into_joint_method [<__state_union_marker_ $name:snake>]);
                $crate::__StateUnion!(@into_enum_method $enum);
            }

            impl [<__state_union_seal_ $name:snake>]::Sealed
                for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            {}
            impl $name for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]> {
                $crate::__StateUnion!(@into_joint_identity_impl [<__state_union_marker_ $name:snake>]);
                $crate::__StateUnion!(@into_enum_joint_impl $enum);
            }

            impl [<__state_union_seal_ $name:snake>]::Sealed for $first {}
            impl $name for $first {
                $crate::__StateUnion!(@into_joint_variant_impl [<__state_union_marker_ $name:snake>]);
                $crate::__StateUnion!(@into_enum_impl $enum $first);
            }
            impl $crate::StateUnionMember<$first>
                for [<__state_union_marker_ $name:snake>]
            {}

            $(
                impl [<__state_union_seal_ $name:snake>]::Sealed for $state {}
                impl $name for $state {
                    $crate::__StateUnion!(@into_joint_variant_impl [<__state_union_marker_ $name:snake>]);
                    $crate::__StateUnion!(@into_enum_impl $enum $state);
                }
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
        @trait $name:ident [] $enum:tt:
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

            pub trait $name: [<__state_union_seal_ $name:snake>]::Sealed {
                $crate::__StateUnion!(@into_joint_method [<__state_union_marker_ $name:snake>]);
                $crate::__StateUnion!(@into_enum_method $enum);
            }

            impl [<__state_union_seal_ $name:snake>]::Sealed
                for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]>
            {}
            impl $name for $crate::StateUnionState<[<__state_union_marker_ $name:snake>]> {
                $crate::__StateUnion!(@into_joint_identity_impl [<__state_union_marker_ $name:snake>]);
                $crate::__StateUnion!(@into_enum_joint_impl $enum);
            }

            impl [<__state_union_seal_ $name:snake>]::Sealed for $first {}
            impl $name for $first {
                $crate::__StateUnion!(@into_joint_variant_impl [<__state_union_marker_ $name:snake>]);
                $crate::__StateUnion!(@into_enum_impl $enum $first);
            }
            impl $crate::StateUnionMember<$first>
                for [<__state_union_marker_ $name:snake>]
            {}

            $(
                impl [<__state_union_seal_ $name:snake>]::Sealed for $state {}
                impl $name for $state {
                    $crate::__StateUnion!(@into_joint_variant_impl [<__state_union_marker_ $name:snake>]);
                    $crate::__StateUnion!(@into_enum_impl $enum $state);
                }
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
        $crate::__StateUnionEnum!(@standalone_enum $enum_name: $first $(| $state)*);
    };
    (@into_joint_method $marker:ident) => {
        #[must_use]
        fn into_joint<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl;
    };
    (@into_joint_identity_impl $marker:ident) => {
        fn into_joint<Storage, T>(
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
    (@into_joint_variant_impl $marker:ident) => {
        fn into_joint<Storage, T>(
            state: $crate::State<Storage, T, Self>,
        ) -> $crate::State<Storage, T, $crate::StateUnionState<$marker>>
        where
            Self: Sized,
            Storage: $crate::StateStorage,
            T: $crate::StateMachineImpl,
        {
            $crate::StateUnionVariant::<Storage, T, Self, $marker>::new(state).into_joint()
        }
    };
    (@into_enum_method $enum:tt) => {
        $crate::__StateUnionEnum!(@into_enum_method $enum);
    };
    (@into_enum_impl $enum:tt $state:ident) => {
        $crate::__StateUnionEnum!(@into_enum_impl $enum $state);
    };
    (@into_enum_joint_impl $enum:tt) => {
        $crate::__StateUnionEnum!(@into_enum_joint_impl $enum);
    };
    (
        @enum $enum_name:ident $marker:ident:
        $first:ident $(| $state:ident)*
    ) => {
        $crate::__StateUnionEnum!(@enum $enum_name $marker: $first $(| $state)*);
    };
}
