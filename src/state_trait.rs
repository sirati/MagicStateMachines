use core::any::TypeId;
use core::ptr::NonNull;
use dynzst::{DynZSTBox, IsZeroSized};

/// A type-erased state marker.
///
/// This trait is implemented for every `'static` zero-sized type.
pub trait StateTrait: IsZeroSized + 'static {
    /// Fully qualified concrete state type name.
    fn type_name(&self) -> &'static str;

    #[doc(hidden)]
    fn type_id(&self) -> TypeId;
}

impl<T> StateTrait for T
where
    T: IsZeroSized + 'static,
{
    fn type_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

pub(crate) type ErasedState = DynZSTBox<dyn StateTrait>;

pub(crate) fn erased_state<T>() -> ErasedState
where
    T: StateTrait,
{
    // SAFETY: `StateTrait: IsZeroSized`, so dereferencing this aligned dangling
    // pointer accesses zero bytes. DynZSTBox then retains only trait metadata.
    let state = unsafe { NonNull::<T>::dangling().as_ref() };
    DynZSTBox::with_dyn(state)
}

pub(crate) fn clone_erased(state: &ErasedState) -> ErasedState {
    DynZSTBox::with_dyn(&**state)
}

pub(crate) fn is_state<T>(state: &ErasedState) -> bool
where
    T: StateTrait,
{
    state.type_id() == TypeId::of::<T>()
}
