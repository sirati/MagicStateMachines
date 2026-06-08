use crate::{
    StateMachineImpl, StateTrait, StateUnionDiscriminant, StateUnionErased,
    StateUnionSharedEffect, StateUnionState, StateUnionTransition,
};

/// Classifies state marker types.
pub trait StateKind {
    type Proof<T, From, Marker, To>
    where
        T: StateMachineImpl,
        From: StateTrait + UnionTransitionProof<T, Marker, To>,
        Marker: StateUnionDiscriminant
            + StateUnionSharedEffect<T, To>
            + StateMarker<Kind = UnionStateKind>,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>;
}

/// Marker kind for concrete state ZSTs.
pub struct ConcreteStateKind;

impl StateKind for ConcreteStateKind {
    type Proof<T, From, Marker, To> = ()
    where
        T: StateMachineImpl,
        From: StateTrait + UnionTransitionProof<T, Marker, To>,
        Marker: StateUnionDiscriminant
            + StateUnionSharedEffect<T, To>
            + StateMarker<Kind = UnionStateKind>,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>;
}

/// Marker kind for generated union state ZSTs.
pub struct UnionStateKind;

impl StateKind for UnionStateKind {
    type Proof<T, From, Marker, To> = crate::StateUnionTransitionProof<T, From, Marker, To>
    where
        T: StateMachineImpl,
        From: StateTrait + UnionTransitionProof<T, Marker, To>,
        Marker: StateUnionDiscriminant
            + StateUnionSharedEffect<T, To>
            + StateMarker<Kind = UnionStateKind>,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>;
}

/// Common trait implemented by concrete states and generated union markers.
pub trait StateMarker {
    type Kind: StateKind;
}

impl<Marker> StateMarker for StateUnionState<Marker>
where
    Marker: StateUnionDiscriminant,
{
    type Kind = UnionStateKind;
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
