#![cfg_attr(feature = "nightly-random", feature(random))]
#![cfg_attr(feature = "unique-rc-arc", feature(unique_rc_arc))]
#![feature(
    arbitrary_self_types,
    associated_type_defaults,
    auto_traits,
    negative_impls,
)]
#![deny(unsafe_code)]

//! Zero-overhead wrappers for externally defined typestate contracts.

mod contract;
#[cfg(feature = "decompose")]
mod decomposed;
mod kind;
mod macros;
mod policy;
mod proof;
mod shared;
mod state;
mod state_trait;
#[cfg(feature = "tracing")]
pub mod tracing;
mod union;
mod util;

pub use contract::{Initial, StateMachineImpl, Transition, TransitionSignature};
#[cfg(feature = "decompose")]
pub use decomposed::{DecomposedData, DecomposedState, RecomposeError};
pub use kind::{
    ConcreteStateKind, RuntimeStateMarker, StateKind, StateMarker, StateRuntimeMarkerFor,
    UnionStateKind,
};
pub use policy::{StateClone, StateCopy};
pub use proof::{
    StateConcreteProvenState, StateConcreteTransitionProof, StateProofTransition,
    StateTransitionProofBind, StateUnionProvenState, StateWithProof, TransitionProof,
};
pub use shared::{
    MutexStorage, RefCellStorage, RwLockStorage, SArc, SArcMutex, SArcRwLock, SMutView, SMutex,
    SRc, SRcRefCell, SRefCell, SRwLock, WeakSArc, WeakSArcMutex, WeakSArcRwLock, WeakSRc,
    WeakSRcRefCell, SharedBorrowState, SharedState, SharedStateError, SharedStorage, SharedValue,
    StateMut, StateMutTransitionCall, StateRef, StorageStateMut,
    WrongStateError, transition_mut,
};
pub use state::{
    ConcreteProofTransitionCall, DiscriminatedTransitionCall, EffectTransitionCall,
    InferenceKind, InnerInference, InnerStateInference, KindProofTransitionCall, OuterInference,
    SBox, SMove, SMut, SOwned, SPin, SPinBox, SRef, SResult, State, StateInference, StateOwned,
    StateStorage, StateStorageNew, StateTransitionCall, StateUnionProofTransitionCall,
    StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox,
    TransitionCall, TransitionCallsite, TransitionEffect, TransitionEffectSelector,
    complete_transition_after_effect, proven_state, proven_union_state, transition,
    transition_callsite, transition_concrete_after_effect, transition_discriminated_state,
    transition_state, transition_state_with_concrete_proof,
    transition_state_with_concrete_transition_proof, transition_state_with_effect,
    transition_state_with_concrete_kind_proof, transition_state_with_erased_transition_proof,
    transition_state_with_kind_proof, transition_state_with_static_union_proof,
    transition_state_with_union_proof, transition_state_with_union_transition_proof,
};
#[cfg(feature = "unique-rc-arc")]
pub use state::{StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc};
#[doc(hidden)]
pub use state_trait::ConcreteStateTrait;
pub use state_trait::StateTrait;
#[doc(hidden)]
pub use state_trait::{ErasedState, clone_erased as clone_erased_state};
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
    concretize_discriminated_state, discriminate_state, discriminated_state_marker,
    erased_state_type_id, rediscriminate_union_state, state_union_marker, undiscriminate_state,
};
pub use util::EnumExt;
#[doc(hidden)]
pub use proof::StateUnionTransitionProof;
pub use union::{DiscriminatedState, In, StateUnionDiscriminant};

#[doc(hidden)]
pub mod __private {
    pub use paste::paste;
}

#[cfg(test)]
mod tests;
