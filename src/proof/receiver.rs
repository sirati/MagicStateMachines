use crate::{
    SMut, SRef, State, StateMachineImpl, StateStorage, StateTrait, StateUnionDiscriminant,
};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// A state receiver carrying an unresolved generated transition proof.
#[doc(hidden)]
pub struct StateWithProof<Storage, T, From, Proof>
where
    T: StateMachineImpl,
    Storage: StateStorage,
{
    pub(crate) state: State<Storage, T, From>,
    pub(crate) proof: Proof,
}

/// A state receiver carrying a generated union-transition proof.
#[doc(hidden)]
pub struct StateUnionProvenState<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    pub(crate) state: State<Storage, T, From>,
    pub(crate) marker: PhantomData<fn() -> (Marker, To)>,
}

/// A state receiver carrying a generated concrete-transition proof.
#[doc(hidden)]
pub struct StateConcreteProvenState<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    pub(crate) state: State<Storage, T, From>,
    pub(crate) marker: PhantomData<fn() -> (Marker, To)>,
}

impl<Storage, T, From, Marker, To> Deref for StateUnionProvenState<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: SRef,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Storage::s_ref(&self.state.inner)
    }
}

impl<Storage, T, From, Marker, To> DerefMut
    for StateUnionProvenState<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: SMut,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Storage::s_mut(&mut self.state.inner)
    }
}

impl<Storage, T, From, Marker, To> Deref
    for StateConcreteProvenState<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: SRef,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Storage::s_ref(&self.state.inner)
    }
}

impl<Storage, T, From, Marker, To> DerefMut
    for StateConcreteProvenState<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: SMut,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Storage::s_mut(&mut self.state.inner)
    }
}
