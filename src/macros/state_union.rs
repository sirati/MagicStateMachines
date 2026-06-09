/// Defines a named union of concrete state markers.
///
/// A state union is useful when several states support the same read-only API,
/// or when a method may return one of several concrete states. The macro
/// generates three public concepts:
///
/// - a marker ZST such as `Online`;
/// - a sealed membership trait such as `InOnline`, implemented for the listed
///   concrete states and for the union-erased marker itself;
/// - a value-carrying enum such as `OnlineEnum<Storage, T>`, whose variants
///   hold the concrete `State<Storage, T, Connected>` /
///   `State<Storage, T, Authenticated>` values;
/// - an [`In<Online>`](crate::In) implementation for every member, which is the
///   generic form used by helpers that take the union marker as a type
///   parameter;
/// - a [`StateUnionDiscriminant`](crate::StateUnionDiscriminant)
///   implementation tying `Online` to `OnlineEnum`.
///
/// The generated marker and membership trait are useful at the type level:
///
/// ```
/// use magicstatemachines::{
///     ConcreteStateKind, In, StateMarker, StateUnion, StateUnionDiscriminant,
///     States, UnionStateKind,
/// };
///
/// States! {
///     Connected;
///     Authenticated;
/// }
///
/// StateUnion!(Online: Connected | Authenticated);
///
/// fn accepts_generated_trait<T: InOnline>() {}
/// fn accepts_generic_trait<T: In<Online>>() {}
/// fn assert_union_marker<T: StateMarker<Kind = UnionStateKind>>() {}
/// fn assert_concrete_marker<T: StateMarker<Kind = ConcreteStateKind>>() {}
///
/// accepts_generated_trait::<Connected>();
/// accepts_generic_trait::<Authenticated>();
/// assert_union_marker::<Online>();
/// assert_concrete_marker::<Connected>();
/// ```
///
/// For `StateUnion!(Online: Connected | Authenticated)`, APIs can write
/// `impl InOnline` when they need "any online state":
///
/// ```ignore
/// use magicstatemachines::{SRef, State};
/// use test_def::InOnline;
///
/// impl Connection {
///     fn endpoint<S>(self: &State<S, Self, impl InOnline>) -> &str
///     where
///         S: SRef,
///     {
///         &self.endpoint
///     }
/// }
/// ```
///
/// When runtime branching is needed, convert a concrete member into the
/// generated enum through [`EnumExt`](crate::EnumExt), or first convert to a
/// [`DiscriminatedState`](crate::DiscriminatedState) with
/// [`In::into_discriminated`](crate::In::into_discriminated) and then call
/// [`DiscriminatedState::discriminate`](crate::DiscriminatedState):
///
/// ```ignore
/// use magicstatemachines::{DiscriminatedState, EnumExt, State};
/// use test_def::{Online, OnlineEnum};
///
/// fn handle_online<S>(
///     state: State<S, Connection, impl InOnline>,
/// ) -> DiscriminatedState<S, Connection, Online>
/// where
///     S: magicstatemachines::SRef,
/// {
///     <_>::into_discriminated(state)
/// }
///
/// match handle_online(state).discriminate() {
///     OnlineEnum::Connected(connected) => {
///         // `connected` is State<S, Connection, Connected>.
///     }
///     OnlineEnum::Authenticated(authenticated) => {
///         // `authenticated` is State<S, Connection, Authenticated>.
///     }
/// }
///
/// // Equivalent convenience form when the marker value is in scope:
/// let online_enum = Online.into_enum(state);
/// ```
///
/// In a match, every enum variant contains a concrete state again. That is why
/// a branch can call methods that require `Connected` or `Authenticated`
/// specifically, while the enum as a whole dereferences through the union-erased
/// state.
///
/// A discriminated union state can also be converted back into a union-typed
/// state with the enum's generated `into_erased()` method. That is useful when
/// matching temporarily recovers a concrete variant but the API should return
/// the broader union type again.
///
/// The supported forms are:
///
/// - `StateUnion!(Online: Connected | Authenticated)` creates marker `Online`,
///   trait `InOnline`, and enum `OnlineEnum`.
/// - `StateUnion!(Online: Parent, Connected | Authenticated)` additionally
///   makes `InOnline` extend `InParent`.
/// - `StateUnion!(Online, enum CustomOnline: Connected | Authenticated)`
///   creates marker `Online`, trait `InOnline`, and enum `CustomOnline`.
/// - `StateUnion!(enum OnlineEnum: Connected | Authenticated)` creates only an
///   enum. This is useful when you need a discriminating value but do not want
///   to publish a named union marker.
///
/// Super-unions are written before the comma. In this example every `Online`
/// state is also an `AllMarker` state, so `InOnline` extends `InAllMarker`:
///
/// ```ignore
/// StateUnion!(AllMarker: Disconnected | Connected | Authenticated);
/// StateUnion!(Online: AllMarker, Connected | Authenticated);
/// ```
///
/// Generated membership traits are sealed and cannot be implemented
/// downstream:
///
/// ```compile_fail
/// use magicstatemachines::StateUnion;
///
/// struct Connected;
/// struct Other;
///
/// StateUnion!(Online: Connected);
/// impl InOnline for Other {}
/// ```
///
/// A joint-state transition exists only when every member supports the same
/// target and function signature. [`StateMachineImpl!`](macro@crate::StateMachineImpl)
/// adds one more requirement for the `transition!(const ...)` form: every
/// member must also share the same effect body. If bodies differ, use
/// `transition!(dyn Online state)` instead so the concrete variant is
/// discriminated before the effect is selected.
///
/// This is the type-level reason a static union transition can fail at compile
/// time:
///
/// ```compile_fail
/// use magicstatemachines::{StateUnion, StateUnionState, Transition};
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
/// runtime branching is needed. The marker also names that enum through
/// [`StateUnionDiscriminant`](crate::StateUnionDiscriminant), so
/// `<Online as StateUnionDiscriminant>::Enum<Storage, T>` is
/// `OnlineEnum<Storage, T>`.
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
