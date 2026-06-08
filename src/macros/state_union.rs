/// Defines marker traits and value-carrying enums for unions of states.
///
/// The supported forms are:
///
/// - `StateUnion!(Online: Connected | Authenticated)` for marker `Online`, trait `InOnline`, and `OnlineEnum`.
///   APIs can name the value-carrying type as `DiscriminatedState<Storage, T, Online>`.
/// - `StateUnion!(Online: Parent, Connected | Authenticated)` with supertrait `InParent` and `OnlineEnum`.
///   APIs can name the value-carrying type as `DiscriminatedState<Storage, T, Online>`.
/// - `StateUnion!(Online, enum CustomOnline: Connected | Authenticated)` with a custom enum name.
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
/// impl InOnline for Other {}
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
/// requires_disconnect::<StateUnionState<Online>>();
/// ```
///
/// `DiscriminatedState<Storage, T, Online>` carries the concrete variant in
/// its storage. Calling `discriminate()` recovers the generated enum when
/// runtime branching is needed.
#[macro_export]
macro_rules! StateUnion {
    (
        $name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__private::paste! {
            $crate::__StateUnion!(
                @trait $name [] [enum [<$name Enum>]]:
                $first $(| $state)*
            );
            $crate::__StateUnion!(
                @enum [<$name Enum>] $name:
                $first $(| $state)*
            );
        }
    };
    (
        $name:ident, enum $enum_name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(@trait $name [] [enum $enum_name]: $first $(| $state)*);
        $crate::__private::paste! {
            $crate::__StateUnion!(
                @enum $enum_name $name:
                $first $(| $state)*
            );
        }
    };
    (
        $name:ident:
        $first_super:ident $(+ $supertrait:ident)*,
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__private::paste! {
            $crate::__StateUnion!(
                @trait $name [$first_super $(, $supertrait)*] [enum [<$name Enum>]]:
                $first $(| $state)*
            );
            $crate::__StateUnion!(
                @enum [<$name Enum>] $name:
                $first $(| $state)*
            );
        }
    };
    (
        $name:ident:
        $first_super:ident $(+ $supertrait:ident)*,
        enum $enum_name:ident:
        $first:ident $(| $state:ident)* $(,)?
    ) => {
        $crate::__StateUnion!(
            @trait $name [$first_super $(, $supertrait)*] [enum $enum_name]:
            $first $(| $state)*
        );
        $crate::__private::paste! {
            $crate::__StateUnion!(
                @enum $enum_name $name:
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
