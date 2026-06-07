use core::mem::size_of;
use core::panic::Location;
use core::ptr::NonNull;

mod sealed {
    use core::mem::size_of;

    pub trait State {}

    pub trait ZeroLengthArray {}
    impl ZeroLengthArray for [u8; 0] {}

    impl<T> State for T
    where
        T: Sized + 'static,
        [u8; size_of::<T>()]: ZeroLengthArray,
    {
    }
}

/// A type-erased state marker.
///
/// This trait is sealed and implemented for every `'static` zero-sized type.
pub trait State: sealed::State + 'static {
    /// Fully qualified concrete state type name.
    fn type_name(&self) -> &'static str;
}

impl<T> State for T
where
    T: sealed::State + 'static,
{
    fn type_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }
}

/// One recorded state transition.
#[derive(Clone, Copy)]
pub struct TraceEntry {
    from: &'static dyn State,
    to: &'static dyn State,
    callsite: &'static Location<'static>,
}

impl TraceEntry {
    pub(crate) fn new<From, To>(callsite: &'static Location<'static>) -> Self
    where
        From: State,
        To: State,
    {
        Self {
            from: zst_ref::<From>(),
            to: zst_ref::<To>(),
            callsite,
        }
    }

    /// Type-erased source-state marker.
    #[must_use]
    pub fn from(&self) -> &dyn State {
        self.from
    }

    /// Type-erased destination-state marker.
    #[must_use]
    pub fn to(&self) -> &dyn State {
        self.to
    }

    /// Source location at which [`crate::State::transition`] was called.
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

fn zst_ref<T>() -> &'static dyn State
where
    T: State + 'static,
{
    assert_eq!(size_of::<T>(), 0, "traced states must be zero-sized");

    // SAFETY: `State` is sealed and implemented only for zero-sized types.
    // An aligned, non-null dangling pointer is dereferenceable for the zero
    // bytes occupied by a ZST. The resulting reference carries only type
    // metadata and can therefore be treated as `'static`.
    unsafe { NonNull::<T>::dangling().as_ref() }
}
