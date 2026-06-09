/// Defines concrete state ZSTs and marks them as concrete state markers.
#[macro_export]
macro_rules! States {
    ($($state:ident;)*) => {
        $(
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
