use crate::{
    ConcreteStateKind, State, StateMachineImpl, StateMarker, StateStorage, StateTrait,
    StateUnionDiscriminant, StateUnionErased, StateUnionSharedEffect, StateUnionTransition,
    UnionStateKind,
};
use core::marker::PhantomData;

/// Proof that a state can transition through a generated state union.
#[doc(hidden)]
pub struct StateUnionTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: StateTrait,
{
    marker: PhantomData<fn() -> (T, From, Marker, To)>,
}

impl<T, From, Marker, To> StateUnionTransitionProof<T, From, Marker, To>
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

impl<T, From, Marker, To> Default for StateUnionTransitionProof<T, From, Marker, To>
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

/// Proves that a state can transition through a union marker to a concrete target.
#[doc(hidden)]
pub trait UnionTransitionProof<T, TUnion, TTo>: StateMarker
where
    T: StateMachineImpl,
    TUnion: StateMarker<Kind = UnionStateKind> + StateUnionDiscriminant,
    TTo: StateMarker<Kind = ConcreteStateKind>,
{
}

impl<T, From, TUnion, TTo> UnionTransitionProof<T, TUnion, TTo> for From
where
    T: StateMachineImpl,
    From: StateMarker + StateTrait + StateUnionErased<TUnion>,
    TUnion: StateMarker<Kind = UnionStateKind>
        + StateUnionDiscriminant
        + StateUnionTransition<T::Standin, TTo>
        + StateUnionSharedEffect<T, TTo>,
    TTo: StateMarker<Kind = ConcreteStateKind> + StateTrait,
{
}
