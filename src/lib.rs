#![feature(
    arbitrary_self_types,
    associated_type_defaults,
    auto_traits,
    fn_traits,
    generic_const_exprs,
    random,
    tuple_trait,
    unboxed_closures,
    unique_rc_arc
)]
#![allow(incomplete_features)]
#![cfg_attr(test, feature(negative_impls))]
#![deny(unsafe_code)]

//! Zero-overhead wrappers for externally defined typestate contracts.

mod contract;
mod decomposed;
mod macros;
mod policy;
mod shared;
mod state;
#[allow(unsafe_code)]
mod state_trait;
#[cfg(feature = "tracing")]
pub mod tracing;
mod union;

pub use contract::{Initial, StateMachineImpl, Transition};
pub use decomposed::{DecomposedData, DecomposedState, RecomposeError};
pub use policy::{StateClone, StateCopy};
pub use shared::{
    ArcState, MutexState, MutexStorage, RcState, RefCellState, RefCellStorage, SharedState,
    SharedStateError, SharedStorage, SharedValue, StateMut, StateMutTransitionCall, StateMutView,
    StateRef, StorageStateMut, transition_mut,
};
pub use state::{
    SMut, SRef, SResult, State, StateOwned, StateOwnedBox, StateOwnedPin, StateOwnedPinBox,
    StateStorage, StateStorageNew, StateTransitionCall, StorageStateOwned, StorageStateOwnedBox,
    StorageStateOwnedPinBox, StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc, TransitionCall,
    TransitionCallsite, transition, transition_state,
};
pub use state_trait::StateTrait;
#[cfg(feature = "tracing")]
pub use tracing::TraceEntry;
#[doc(hidden)]
pub use union::{StateUnionMember, StateUnionState, StateUnionTransition, StateUnionVariant};

#[doc(hidden)]
pub mod __private {
    pub use paste::paste;
}

#[cfg(test)]
mod tests;
