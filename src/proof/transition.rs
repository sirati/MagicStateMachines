use crate::{
    EffectTransitionCall, State, StateConcreteTransitionProof, StateKind, StateMachineImpl,
    StateMarker, StateStorage, StateTrait, StateUnionConcreteState, StateUnionDiscriminant,
    StateUnionErased, StateUnionProofTransitionCall, StateUnionSharedEffect,
    StateUnionTransitionProof, StateWithProof, Transition, TransitionEffectSelector,
    TransitionProof, transition_state_with_concrete_transition_proof,
    transition_state_with_erased_transition_proof,
};

#[doc(hidden)]
pub trait StateProofTransition<Storage, T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait,
    Marker: StateUnionDiscriminant,
    To: crate::ConcreteStateTrait,
{
    type Call;

    fn proven_transition_state(
        state: State<Storage, T, From>,
        token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
    ) -> Self::Call;
}

impl<Storage, T, From, Marker, To> StateProofTransition<Storage, T, From, Marker, To>
    for StateConcreteTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl + TransitionEffectSelector<From, To>,
    Storage: StateStorage,
    T::Standin: Transition<From, To>,
    From: crate::ConcreteStateTrait + StateUnionConcreteState,
    Marker: StateUnionDiscriminant,
    To: crate::ConcreteStateTrait,
{
    type Call = EffectTransitionCall<
        Storage,
        T,
        From,
        To,
        <T as TransitionEffectSelector<From, To>>::Effect,
    >;

    fn proven_transition_state(
        state: State<Storage, T, From>,
        token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
    ) -> Self::Call {
        transition_state_with_concrete_transition_proof(
            StateWithProof {
                state,
                proof: Self::new(),
            },
            token,
        )
    }
}

impl<Storage, T, From, Marker, To> StateProofTransition<Storage, T, From, Marker, To>
    for StateUnionTransitionProof<T, From, Marker, To>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait + StateUnionErased<Marker>,
    Marker: StateUnionDiscriminant + StateUnionSharedEffect<T, To>,
    To: crate::ConcreteStateTrait,
{
    type Call = StateUnionProofTransitionCall<Storage, T, From, Marker, To>;

    fn proven_transition_state(
        state: State<Storage, T, From>,
        token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
    ) -> Self::Call {
        transition_state_with_erased_transition_proof(
            StateWithProof {
                state,
                proof: Self::new(),
            },
            token,
        )
    }
}

impl<Storage, T, From, Marker, To, Inner> StateProofTransition<Storage, T, From, Marker, To>
    for TransitionProof<Storage, T, From, Marker, To, Inner>
where
    T: StateMachineImpl,
    Storage: StateStorage,
    From: StateTrait + StateMarker<Kind = Inner>,
    Marker: StateUnionDiscriminant,
    To: crate::ConcreteStateTrait,
    Inner: StateKind,
    Inner::Proof<T, From, Marker, To>: StateProofTransition<Storage, T, From, Marker, To>,
{
    type Call = <Inner::Proof<T, From, Marker, To> as StateProofTransition<
        Storage,
        T,
        From,
        Marker,
        To,
    >>::Call;

    fn proven_transition_state(
        state: State<Storage, T, From>,
        token: <Storage::Machine<T> as StateMachineImpl>::TransitionToken,
    ) -> Self::Call {
        <Inner::Proof<T, From, Marker, To> as StateProofTransition<
            Storage,
            T,
            From,
            Marker,
            To,
        >>::proven_transition_state(state, token)
    }
}
