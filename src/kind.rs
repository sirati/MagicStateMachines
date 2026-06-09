use crate::{
    StateMachineImpl, StateStorage, StateTrait, StateUnionDiscriminant, StateUnionState,
    StateUnionTransitionProof, TransitionProof,
};

/// Classifies state marker types.
pub trait StateKind: Sized {
    type RuntimeState<Marker>: StateTrait + StateMarker
    where
        Marker: StateRuntimeMarkerFor<Self>;

    type Proof<T, From, Marker, To>
    where
        T: StateMachineImpl,
        From: StateTrait + StateMarker<Kind = Self>,
        Marker: StateUnionDiscriminant,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>;

    #[doc(hidden)]
    #[must_use]
    fn prove<Storage, T, From, Marker, To>() -> TransitionProof<Storage, T, From, Marker, To, Self>
    where
        Storage: StateStorage,
        T: StateMachineImpl,
        From: StateTrait + StateMarker<Kind = Self>,
        Marker: StateUnionDiscriminant,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>,
    {
        TransitionProof::new()
    }
}

/// Marker kind for concrete state ZSTs.
pub struct ConcreteStateKind;

impl StateKind for ConcreteStateKind {
    type RuntimeState<Marker>
        = <Marker as StateRuntimeMarkerFor<Self>>::RuntimeState
    where
        Marker: StateRuntimeMarkerFor<Self>;

    type Proof<T, From, Marker, To>
        = crate::StateConcreteTransitionProof<T, From, Marker, To>
    where
        T: StateMachineImpl,
        From: StateTrait + StateMarker<Kind = Self>,
        Marker: StateUnionDiscriminant,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>;
}

/// Marker kind for generated union state ZSTs.
pub struct UnionStateKind;

impl StateKind for UnionStateKind {
    type RuntimeState<Marker>
        = <Marker as StateRuntimeMarkerFor<Self>>::RuntimeState
    where
        Marker: StateRuntimeMarkerFor<Self>;

    type Proof<T, From, Marker, To>
        = StateUnionTransitionProof<T, From, Marker, To>
    where
        T: StateMachineImpl,
        From: StateTrait + StateMarker<Kind = Self>,
        Marker: StateUnionDiscriminant,
        To: StateTrait + StateMarker<Kind = ConcreteStateKind>;
}

/// Common trait implemented by concrete states and generated union markers.
pub trait StateMarker: 'static {
    type Kind: StateKind;

    #[doc(hidden)]
    fn erased_state() -> &'static dyn StateTrait
    where
        Self: Sized;
}

impl<Marker> StateMarker for StateUnionState<Marker>
where
    Marker: StateUnionDiscriminant + 'static,
{
    type Kind = UnionStateKind;

    fn erased_state() -> &'static dyn StateTrait {
        panic!("union state markers cannot be stored as ErasedState")
    }
}

#[doc(hidden)]
pub trait StateRuntimeMarkerFor<Kind: StateKind>: StateTrait + StateMarker {
    type RuntimeState: StateTrait + StateMarker;
}

impl<Marker> StateRuntimeMarkerFor<ConcreteStateKind> for Marker
where
    Marker: StateTrait + StateMarker<Kind = ConcreteStateKind>,
{
    type RuntimeState = Marker;
}

impl<Marker> StateRuntimeMarkerFor<UnionStateKind> for Marker
where
    Marker: StateUnionDiscriminant + StateTrait,
    StateUnionState<Marker>: StateTrait,
{
    type RuntimeState = StateUnionState<Marker>;
}

pub type RuntimeStateMarker<Marker> =
    <<Marker as StateMarker>::Kind as StateKind>::RuntimeState<Marker>;
