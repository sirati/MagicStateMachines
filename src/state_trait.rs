use core::any::TypeId;

/// A type-erased state marker.
pub trait StateTrait: 'static {
    /// Fully qualified concrete state type name.
    fn type_name(&self) -> &'static str;

    #[doc(hidden)]
    fn type_id(&self) -> TypeId;

    #[doc(hidden)]
    fn erased_state() -> &'static dyn StateTrait
    where
        Self: Sized;
}

impl<T> StateTrait for T
where
    T: crate::StateMarker + 'static,
{
    fn type_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn erased_state() -> &'static dyn StateTrait {
        <T as crate::StateMarker>::erased_state()
    }
}

#[cfg(feature = "dynZST")]
pub(crate) type ErasedState = dynzst::DynZSTBox<dyn StateTrait>;

#[cfg(not(feature = "dynZST"))]
pub(crate) type ErasedState = &'static dyn StateTrait;

#[cfg(feature = "dynZST")]
pub(crate) fn erased_state<T>() -> ErasedState
where
    T: StateTrait,
{
    dynzst::DynZSTBox::with_dyn(T::erased_state())
}

#[cfg(not(feature = "dynZST"))]
pub(crate) fn erased_state<T>() -> ErasedState
where
    T: StateTrait,
{
    T::erased_state()
}

#[cfg(feature = "dynZST")]
pub(crate) fn clone_erased(state: &ErasedState) -> ErasedState {
    dynzst::DynZSTBox::with_dyn(&**state)
}

#[cfg(not(feature = "dynZST"))]
pub(crate) fn clone_erased(state: &ErasedState) -> ErasedState {
    *state
}

pub(crate) fn is_state<T>(state: &ErasedState) -> bool
where
    T: StateTrait,
{
    state.type_id() == TypeId::of::<T>()
}
