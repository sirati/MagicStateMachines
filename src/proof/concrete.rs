use crate::{State, StateMachineImpl, StateStorage, StateTrait, StateUnionDiscriminant};
use core::marker::PhantomData;

/// Proof that a concrete state can transition directly to a concrete target.
#[doc(hidden)]
pub struct StateConcreteTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    marker: PhantomData<fn() -> (T, From, Marker, To)>,
}

impl<T, From, Marker, To> StateConcreteTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    #[doc(hidden)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub fn bind<Storage>(&self, _state: &State<Storage, T, From>)
    where
        Storage: StateStorage,
    {
    }
}

impl<T, From, Marker, To> Default for StateConcreteTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    fn default() -> Self {
        Self::new()
    }
}
