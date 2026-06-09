use crate::{
    StateTrait,
    state_trait::{ErasedState, erased_state},
};
use core::panic::Location;

/// One recorded state transition.
pub struct TraceEntry {
    from: ErasedState,
    to: ErasedState,
    callsite: &'static Location<'static>,
}

impl Clone for TraceEntry {
    fn clone(&self) -> Self {
        Self {
            from: crate::state_trait::clone_erased(&self.from),
            to: crate::state_trait::clone_erased(&self.to),
            callsite: self.callsite,
        }
    }
}

impl TraceEntry {
    pub(crate) fn new<From, To>(callsite: &'static Location<'static>) -> Self
    where
        From: StateTrait,
        To: crate::ConcreteStateTrait,
    {
        Self {
            from: crate::state_trait::static_erased_state::<From>(),
            to: erased_state::<To>(),
            callsite,
        }
    }

    /// Type-erased source-state marker.
    #[must_use]
    pub fn from(&self) -> &dyn StateTrait {
        &*self.from
    }

    /// Type-erased destination-state marker.
    #[must_use]
    pub fn to(&self) -> &dyn StateTrait {
        &*self.to
    }

    /// Source location at which a state transition was requested.
    #[must_use]
    pub const fn callsite(&self) -> &'static Location<'static> {
        self.callsite
    }
}

impl core::fmt::Debug for TraceEntry {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("TraceEntry")
            .field("from", &self.from.type_name())
            .field("to", &self.to.type_name())
            .field("callsite", &self.callsite)
            .finish()
    }
}
