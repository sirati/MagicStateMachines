use crate::{
    StateTrait,
    state_trait::{ErasedState, erased_state},
};
use core::panic::Location;

/// One diagnostic record produced by a completed state transition.
///
/// A trace entry stores three facts: the source state marker, the destination
/// state marker, and the callsite where the transition was requested. The
/// markers are type-erased [`StateTrait`] values, so a single trace vector can
/// record many different transition pairs.
///
/// Tracing is diagnostic only. It does not participate in transition
/// validation, and it should not be used as the authority for the current
/// state. The current state is still represented by the type parameter of
/// [`State`](crate::State) or by the committed erased marker in shared storage.
///
/// `TraceEntry` works with both erased-state backends:
///
/// - without `dynZST`, `from` and `to` are backed by `&'static dyn StateTrait`;
/// - with `dynZST`, they are backed by `dynzst::DynZSTBox<dyn StateTrait>`.
///
/// The public API is the same in both cases. This lets libraries expose traces
/// without leaking which erased-marker backend was selected.
///
/// ```ignore
/// let connected = transition!(disconnected);
/// let trace = connected.trace();
///
/// assert_eq!(trace.len(), 1);
/// assert!(trace[0].from().type_name().ends_with("::Disconnected"));
/// assert!(trace[0].to().type_name().ends_with("::Connected"));
/// eprintln!("transition requested at {}", trace[0].callsite());
/// ```
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
    ///
    /// The returned value is useful for diagnostics such as logs, debug output,
    /// and tests that assert a transition path. It is intentionally erased:
    /// code that needs to branch on a union state should use generated union
    /// enums and `discriminate()` instead of comparing trace entries.
    #[must_use]
    pub fn from(&self) -> &dyn StateTrait {
        &*self.from
    }

    /// Type-erased destination-state marker.
    ///
    /// This is the state the wrapper was retagged to after the transition
    /// effect completed. For shared states, the committed runtime marker is
    /// updated by the guard machinery; the trace is only a historical record.
    #[must_use]
    pub fn to(&self) -> &dyn StateTrait {
        &*self.to
    }

    /// Source location at which a state transition was requested.
    ///
    /// The callsite is captured by the transition wrapper with
    /// `#[track_caller]`, so it points at the user-facing `transition!(...)`
    /// call or the low-level transition function call, not at the internals
    /// that finish retagging the state.
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
