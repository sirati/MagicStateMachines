#![feature(
    arbitrary_self_types,
    associated_type_defaults,
    auto_traits,
    fn_traits,
    generic_const_exprs,
    negative_impls,
    random,
    tuple_trait,
    unboxed_closures,
    unique_rc_arc
)]
#![allow(incomplete_features)]
#![deny(unsafe_code)]

//! Zero-overhead wrappers for externally defined typestate contracts.

mod contract;
mod decomposed;
mod kind;
mod macros;
mod policy;
mod proof;
mod shared;
mod state;
#[allow(unsafe_code)]
mod state_trait;
#[cfg(feature = "tracing")]
pub mod tracing;
mod union;

pub use contract::{Initial, StateMachineImpl, Transition};
pub use decomposed::{DecomposedData, DecomposedState, RecomposeError};
pub use kind::{ConcreteStateKind, StateKind, StateMarker, UnionStateKind};
pub use policy::{StateClone, StateCopy};
pub use proof::{
    StateConcreteProvenState, StateConcreteTransitionProof, StateProofTransition,
    StateTransitionProofBind, StateUnionProvenState, StateWithProof, TransitionProof,
};
pub use shared::{
    MutexStorage, RefCellStorage, RwLockStorage, SArc, SArcMutex, SArcRwLock, SMutView, SMutex,
    SRc, SRcRefCell, SRefCell, SRwLock, SharedBorrowState, SharedState, SharedStateError,
    SharedStorage, SharedValue, StateMut, StateMutTransitionCall, StateRef, StorageStateMut,
    WrongStateError, transition_mut,
};
pub use state::{
    ConcreteProofTransitionCall, DiscriminatedTransitionCall, EffectTransitionCall,
    KindProofTransitionCall, SBox, SMove, SMut, SOwned, SPin, SPinBox, SRef, SResult, State,
    StateOwned, StateStorage, StateStorageNew, StateTransitionCall, StateUnionProofTransitionCall,
    StorageStateOwned, StorageStateOwnedBox,
    StorageStateOwnedPinBox, StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc,
    TransitionCall, TransitionCallsite, TransitionEffect, TransitionEffectSelector,
    complete_transition_after_effect, proven_state, proven_union_state, transition,
    transition_callsite, transition_concrete_after_effect, transition_discriminated_state,
    transition_state, transition_state_with_concrete_proof,
    transition_state_with_concrete_transition_proof, transition_state_with_effect,
    transition_state_with_concrete_kind_proof, transition_state_with_erased_transition_proof,
    transition_state_with_kind_proof, transition_state_with_union_proof,
    transition_state_with_union_transition_proof,
};
pub use state_trait::StateTrait;
#[cfg(feature = "tracing")]
pub use tracing::TraceEntry;
#[doc(hidden)]
pub use union::StateUnionConcreteState;
#[doc(hidden)]
pub use union::StateUnionDiscriminatedTransition;
#[doc(hidden)]
pub use proof::UnionTransitionProof;
#[doc(hidden)]
pub use union::{
    DiscriminatedInner, SDiscriminated, StateUnionErased, StateUnionMember,
    StateUnionProofMembership, StateUnionProofTarget, StateUnionRuntime, StateUnionSharedEffect,
    StateUnionSharedTransitionEffect, StateUnionState, StateUnionTransition,
    StateUnionVariant, concretize_discriminated_state, discriminate_state,
    discriminated_state_discriminator, rediscriminate_union_state, state_union_discriminator,
    undiscriminate_state,
};
#[doc(hidden)]
pub use proof::StateUnionTransitionProof;
pub use union::{DiscriminatedState, In, StateUnionDiscriminant};

#[doc(hidden)]
pub mod __private {
    pub use paste::paste;
}

#[cfg(test)]
mod tests;
