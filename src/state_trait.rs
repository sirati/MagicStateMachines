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

/// Runtime identity for a state marker after the concrete marker type has been erased.
///
/// Most users never implement this trait directly. State marker types should be
/// declared with [`States!`](macro@crate::States), which wires this trait,
/// [`StateMarker`](crate::StateMarker), concrete-state classification, and the
/// static marker instance used by shared storage and tracing.
///
/// The erased marker is intentionally only an identity token. It is not the
/// runtime data controlled by the state machine, and it is not used to make a
/// transition valid. Valid transitions are still proven by the generic
/// `State<Storage, T, S>` type and by the [`Transition`](crate::Transition)
/// contract. `StateTrait` is used at runtime only where the compiler cannot
/// keep one concrete state in the type, for example:
///
/// - a shared [`SRcRefCell`](crate::SRcRefCell) or
///   [`SArcMutex`](crate::SArcMutex) stores the currently committed state next
///   to the data so a later borrow can check `borrow::<Connected>()`;
/// - [`TraceEntry`](crate::TraceEntry) records the source and destination state
///   of each transition without making the trace vector generic over every
///   state pair.
///
/// With the default feature set, erased states are stored as
/// `&'static dyn StateTrait`. With the `dynZST` feature, they are stored through
/// `dynzst::DynZSTBox<dyn StateTrait>`, and marker types must satisfy
/// `dynzst::IsZeroSized`. The [`States!`](macro@crate::States) macro generates
/// zero-sized marker structs, so normal users get the correct invariant without
/// writing any unsafe code.
///
/// ```ignore
/// use magicstatemachines::{StateTrait, States};
///
/// States! {
///     Disconnected;
///     Connected;
/// }
///
/// let state: &'static dyn StateTrait = Disconnected::erased_state();
/// assert!(state.type_name().ends_with("::Disconnected"));
/// ```
pub trait StateTrait: StateTraitZst + 'static {
    /// Fully qualified Rust type name of the concrete state marker.
    ///
    /// This is meant for diagnostics and tests. Use typed APIs such as
    /// `borrow::<Connected>()` or generated union discrimination when the
    /// result should affect control flow.
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
pub trait ConcreteStateTrait:
    StateTrait + crate::StateMarker<Kind = crate::ConcreteStateKind>
{
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
