use crate::{
    ConcreteStateKind, State, StateConcreteProvenState, StateConcreteTransitionProof,
    StateMachineImpl, StateMarker, StateStorage, StateTrait, StateUnionDiscriminant,
    StateUnionErased, StateUnionProvenState, StateUnionSharedEffect, StateUnionTransitionProof,
    Transition, TransitionEffectSelector, UnionTransitionProof,
};
use core::marker::PhantomData;

/// Binds a generated transition proof to a state receiver.
#[doc(hidden)]
pub trait StateTransitionProofBind<Storage, T, From>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait,
{
    type Output;

    fn bind(self, state: State<Storage, T, From>) -> Self::Output;
}

impl<Storage, T, From, Marker, To> StateTransitionProofBind<Storage, T, From>
    for StateUnionTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait + StateUnionErased<Marker> + UnionTransitionProof<T, Marker, To>,
    Marker: StateUnionSharedEffect<T, To>,
    To: StateTrait + StateMarker<Kind = ConcreteStateKind>,
{
    type Output = StateUnionProvenState<Storage, T, From, Marker, To>;

    fn bind(self, state: State<Storage, T, From>) -> Self::Output {
        StateUnionTransitionProof::bind(&self, &state);
        StateUnionProvenState {
            state,
            marker: PhantomData,
        }
    }
}

impl<Storage, T, From, Marker, To> StateTransitionProofBind<Storage, T, From>
    for StateConcreteTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl + TransitionEffectSelector<From, To>,
    Storage: StateStorage,
    T::Standin: Transition<From, To>,
    From: StateTrait + StateMarker<Kind = ConcreteStateKind>,
    Marker: StateUnionDiscriminant,
    To: StateTrait + StateMarker<Kind = ConcreteStateKind>,
{
    type Output = StateConcreteProvenState<Storage, T, From, Marker, To>;

    fn bind(self, state: State<Storage, T, From>) -> Self::Output {
        StateConcreteTransitionProof::bind(&self, &state);
        StateConcreteProvenState {
            state,
            marker: PhantomData,
        }
    }
}
