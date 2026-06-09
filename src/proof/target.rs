use crate::{StateKind, StateMachineImpl, StateMarker, StateStorage, StateTrait};
use core::marker::PhantomData;

#[doc(hidden)]
pub struct TransitionProof<Storage, T, From, Marker, To, Kind>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    From: StateTrait,
    Marker: StateMarker,
    To: StateTrait + StateMarker,
    Kind: StateKind,
{
    marker: PhantomData<fn() -> (Storage, T, From, Marker, To, Kind)>,
}

impl<Storage, T, From, Marker, To, Kind> TransitionProof<Storage, T, From, Marker, To, Kind>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    From: StateTrait,
    Marker: StateMarker,
    To: StateTrait + StateMarker,
    Kind: StateKind,
{
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}
