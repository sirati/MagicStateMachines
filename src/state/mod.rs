mod owned;
mod storage;

pub use owned::{SPin, StateOwned, TransitionCall, transition};
pub use storage::{
    ConcreteProofTransitionCall, DiscriminatedTransitionCall, EffectTransitionCall,
    InferenceKind, InnerInference, InnerStateInference, KindProofTransitionCall, OuterInference,
    SBox, SMove, SMut, SOwned, SPinBox, SRef, SResult, State, StateInference, StateStorage,
    StateStorageNew, StateTransitionCall,
    StateUnionProofTransitionCall, StorageStateOwned, StorageStateOwnedBox,
    StorageStateOwnedPinBox, TransitionCallsite, TransitionEffect, TransitionEffectSelector,
    complete_transition_after_effect, proven_state, proven_union_state, transition_callsite,
    transition_concrete_after_effect, transition_discriminated_state, transition_state,
    transition_state_with_concrete_proof, transition_state_with_concrete_transition_proof,
    transition_state_with_concrete_kind_proof, transition_state_with_effect,
    transition_state_with_erased_transition_proof, transition_state_with_kind_proof,
    transition_state_with_static_union_proof, transition_state_with_union_proof,
    transition_state_with_union_transition_proof,
};
#[cfg(feature = "unique-rc-arc")]
pub use storage::{StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc};
