/// Defines concrete state marker ZSTs.
///
/// Use this in the definition crate for every concrete state in the contract.
/// The generated structs are markers, not runtime state. Runtime data stays in
/// your implementation type; the marker appears only in the type of
/// [`State`](crate::State), [`StateOwned`](crate::StateOwned), shared guards,
/// and generated union enums.
///
/// Attributes placed before a state are copied to the generated struct. This
/// is the intended way to document public state markers:
///
/// ```
/// use magicstatemachines::{ConcreteStateKind, StateMarker, States};
///
/// States! {
///     /// No connection has been established yet.
///     #[derive(Debug, Default)]
///     Disconnected;
///
///     /// The transport is open but no user is authenticated.
///     Connected;
/// }
///
/// fn assert_concrete_state<T>()
/// where
///     T: StateMarker<Kind = ConcreteStateKind>,
/// {
/// }
///
/// assert_concrete_state::<Disconnected>();
/// let _ = format!("{:?}", Disconnected::default());
/// ```
///
/// Each generated type is a public zero-sized struct implementing the traits
/// needed by the rest of the library:
///
/// - [`StateMarker`](crate::StateMarker) with
///   [`ConcreteStateKind`](crate::ConcreteStateKind), so generic code can tell
///   concrete states apart from union markers;
/// - [`StateTrait`](crate::StateTrait), so shared storage and tracing can keep
///   an erased runtime marker;
/// - the concrete-state marker trait used internally to reject storing a union
///   as the authoritative shared state.
///
/// Prefer this macro over hand-written state structs. The library assumes that
/// concrete states are ZST markers whose erased representation still names a
/// concrete state, not a union. Shared storage uses that erased marker at the
/// runtime boundary, tracing stores it in trace entries, and union
/// discrimination compares against it later. Writing the structs by hand means
/// wiring all of those invariants yourself; using `States!` keeps the state
/// definition crate small and makes the generated rustdoc show the state docs
/// exactly where downstream users expect them.
#[macro_export]
macro_rules! States {
    ($($(#[$state_attr:meta])* $state:ident;)*) => {
        $(
            $(#[$state_attr])*
            pub struct $state;

            impl $crate::StateMarker for $state {
                type Kind = $crate::ConcreteStateKind;

                fn erased_state() -> &'static dyn $crate::StateTrait {
                    <$state as $crate::ConcreteStateTrait>::erased_state()
                }
            }

            impl $crate::ConcreteStateTrait for $state {
                fn erased_state() -> &'static dyn $crate::StateTrait {
                    static STATE: $state = $state;
                    &STATE
                }
            }
        )*
    };
}
