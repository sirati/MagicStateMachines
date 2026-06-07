/// Defines marker traits and value-carrying enums for unions of states.
///
/// The supported forms are:
///
/// - `StateUnion!(Online: Connected | Authenticated)` for only a marker trait.
/// - `StateUnion!(Online: Parent, Connected | Authenticated)` with a supertrait.
/// - `StateUnion!(Online, enum OnlineEnum: Connected | Authenticated)` for both.
/// - `StateUnion!(enum OnlineEnum: Connected | Authenticated)` for only an enum.
///
/// Generated marker traits are sealed and cannot be implemented downstream:
///
/// ```compile_fail
/// use statemachines::StateUnion;
///
/// struct Connected;
/// struct Other;
///
/// StateUnion!(Online: Connected);
/// impl Online for Other {}
/// ```
///
/// A joint-state transition exists only when every member supports the same
/// target and function signature:
///
/// ```compile_fail
/// use statemachines::{StateUnion, StateUnionState, Transition};
///
/// struct Machine;
/// struct Connected;
/// struct Authenticated;
/// struct Disconnected;
///
/// impl Transition<Connected, Disconnected> for Machine {}
/// StateUnion!(Online: Connected | Authenticated);
///
/// fn requires_disconnect<From>()
/// where
///     Machine: Transition<From, Disconnected>,
/// {}
///
/// requires_disconnect::<StateUnionState<__state_union_marker_online>>();
/// ```
#[macro_export]
macro_rules! StateUnion {
    (
        $name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(@trait $name []: $first $(| $state)*);
    };
    (
        $name:ident, enum $enum_name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(@trait $name []: $first $(| $state)*);
        $crate::__private::paste! {
            $crate::__StateUnion!(
                @enum $enum_name [<__state_union_marker_ $name:snake>]:
                $first $(| $state)*
            );
        }
    };
    (
        $name:ident:
        $first_super:ident $(+ $supertrait:ident)*,
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(
            @trait $name [$first_super $(, $supertrait)*]:
            $first $(| $state)*
        );
    };
    (
        $name:ident:
        $first_super:ident $(+ $supertrait:ident)*,
        enum $enum_name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(
            @trait $name [$first_super $(, $supertrait)*]:
            $first $(| $state)*
        );
        $crate::__private::paste! {
            $crate::__StateUnion!(
                @enum $enum_name [<__state_union_marker_ $name:snake>]:
                $first $(| $state)*
            );
        }
    };
    (
        enum $enum_name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(@standalone_enum $enum_name: $first $(| $state)*);
    };
}
