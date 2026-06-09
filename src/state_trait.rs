use core::any::TypeId;

#[cfg(feature = "dynZST")]
#[doc(hidden)]
pub trait StateTraitZst: dynzst::IsZeroSized {}

#[cfg(feature = "dynZST")]
impl<T> StateTraitZst for T where T: dynzst::IsZeroSized {}

#[cfg(not(feature = "dynZST"))]
#[doc(hidden)]
pub trait StateTraitZst {}

#[cfg(not(feature = "dynZST"))]
impl<T> StateTraitZst for T {}

/// A type-erased state marker.
pub trait StateTrait: StateTraitZst + 'static {
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
    T: crate::StateMarker + StateTraitZst + 'static,
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

#[doc(hidden)]
pub trait ConcreteStateTrait: StateTrait + crate::StateMarker<Kind = crate::ConcreteStateKind> {
    fn erased_state() -> &'static dyn StateTrait
    where
        Self: Sized;
}

#[cfg(feature = "dynZST")]
#[doc(hidden)]
pub type ErasedState = dynzst::DynZSTBox<dyn StateTrait>;

#[cfg(not(feature = "dynZST"))]
#[doc(hidden)]
pub type ErasedState = &'static dyn StateTrait;

#[cfg(feature = "dynZST")]
pub(crate) fn erased_state<T>() -> ErasedState
where
    T: ConcreteStateTrait,
{
    dynzst::DynZSTBox::with_dyn(<T as ConcreteStateTrait>::erased_state())
}

#[cfg(not(feature = "dynZST"))]
pub(crate) fn erased_state<T>() -> ErasedState
where
    T: ConcreteStateTrait,
{
    <T as ConcreteStateTrait>::erased_state()
}

#[cfg(feature = "dynZST")]
#[doc(hidden)]
pub fn clone_erased(state: &ErasedState) -> ErasedState {
    dynzst::DynZSTBox::with_dyn(&**state)
}

#[cfg(not(feature = "dynZST"))]
#[doc(hidden)]
pub fn clone_erased(state: &ErasedState) -> ErasedState {
    *state
}

pub(crate) fn is_state<T>(state: &ErasedState) -> bool
where
    T: StateTrait,
{
    state.type_id() == TypeId::of::<T>()
}

pub(crate) fn static_erased_state<T>() -> ErasedState
where
    T: StateTrait,
{
    #[cfg(feature = "dynZST")]
    {
        dynzst::DynZSTBox::with_dyn(T::erased_state())
    }
    #[cfg(not(feature = "dynZST"))]
    {
        T::erased_state()
    }
}
