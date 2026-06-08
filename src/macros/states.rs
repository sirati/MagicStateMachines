/// Defines concrete state ZSTs and marks them as concrete state markers.
#[macro_export]
macro_rules! States {
    ($($state:ident;)*) => {
        $(
            pub struct $state;

            impl $crate::StateMarker for $state {
                type Kind = $crate::ConcreteStateKind;
            }
        )*
    };
}
