mod owned;
mod storage;

pub use owned::{
    StateOwned, StateOwnedBox, StateOwnedPin, StateOwnedPinBox, TransitionCall, transition,
};
pub(crate) use storage::retag_state;
pub use storage::{
    SMut, SRef, SResult, State, StateStorage, StateStorageNew, StateTransitionCall,
    StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox, StorageStateOwnedUniqueArc,
    StorageStateOwnedUniqueRc, TransitionCallsite, transition_state,
};
