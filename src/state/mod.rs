mod owned;
mod storage;

pub use owned::{SPin, StateOwned, TransitionCall, transition};
pub(crate) use storage::retag_state;
pub use storage::{
    EffectTransitionCall, SBox, SMove, SMut, SOwned, SPinBox, SRef, SResult, State, StateStorage,
    StateStorageNew, StateTransitionCall, StorageStateOwned, StorageStateOwnedBox,
    StorageStateOwnedPinBox, StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc,
    TransitionCallsite, TransitionEffect, TransitionEffectSelector,
    complete_transition_after_effect, transition_callsite, transition_state,
    transition_state_with_effect,
};
