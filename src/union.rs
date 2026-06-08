use crate::{
    State, StateMachineImpl, StateStorage, StateTrait, Transition, TransitionEffect,
    TransitionEffectSelector,
};
use core::marker::PhantomData;
use core::ops::Deref;

/// State marker shared by every member of a generated state union.
#[doc(hidden)]
pub struct StateUnionState<Marker>(PhantomData<fn() -> Marker>);

/// Implemented by concrete states that can still identify their enum variant.
#[doc(hidden)]
pub auto trait StateUnionConcreteState {}

impl<Marker> !StateUnionConcreteState for StateUnionState<Marker> {}

/// Records that `State` belongs to a generated state union.
#[doc(hidden)]
pub trait StateUnionMember<State> {}

/// Selects the value-carrying discriminated state type for a union marker.
pub trait StateUnionDiscriminant {
    type Discriminated<Storage, T>
    where
        Storage: StateStorage,
        T: StateMachineImpl;
}

/// Value-carrying discriminated state for a generated union marker.
pub type DiscriminatedState<Storage, T, Marker> =
    <Marker as StateUnionDiscriminant>::Discriminated<Storage, T>;

/// Converts a concrete or already-erased member state into a union state.
#[doc(hidden)]
pub trait StateUnionErased<Marker>: StateTrait {
    fn into_union_erased<Storage, T>(
        state: State<Storage, T, Self>,
    ) -> State<Storage, T, StateUnionState<Marker>>
    where
        Self: Sized,
        Storage: StateStorage,
        T: StateMachineImpl;
}

/// Runtime membership check for shared erased-state borrows.
#[doc(hidden)]
pub trait StateUnionRuntime {
    fn contains(state: &dyn StateTrait) -> bool;
    fn expected_type_name() -> &'static str;
}

/// Resolves transitions supported by every member of a generated state union.
#[doc(hidden)]
pub trait StateUnionTransition<Standin, To> {
    type F;
}

/// Resolves implementation-side effects supported by every union member.
#[doc(hidden)]
pub trait StateUnionTransitionEffect<T, To>
where
    T: StateMachineImpl,
{
    type Effect;
}

/// Applies the shared implementation-side effect for a union transition.
#[doc(hidden)]
pub trait StateUnionTransitionEffectApply<T, To, Args>: StateUnionTransitionEffect<T, To>
where
    T: StateMachineImpl,
{
    fn apply(value: &mut T, args: Args);
}

impl<Standin, Marker, To> Transition<StateUnionState<Marker>, To> for Standin
where
    Marker: StateUnionTransition<Standin, To>,
{
    type F = Marker::F;
}

impl<T, Marker, To> TransitionEffectSelector<StateUnionState<Marker>, To> for T
where
    T: StateMachineImpl,
    Marker: StateUnionTransitionEffect<T, To>,
{
    type Effect = Marker::Effect;
}

impl<T, Marker, To, Args, Effect> TransitionEffect<T, StateUnionState<Marker>, To, Args> for Effect
where
    T: StateMachineImpl,
    Marker: StateUnionTransitionEffect<T, To, Effect = Effect>
        + StateUnionTransitionEffectApply<T, To, Args>,
{
    fn apply(value: &mut T, args: Args) {
        Marker::apply(value, args);
    }
}

/// A union variant that preserves its concrete state while exposing the joint state.
#[doc(hidden)]
pub struct StateUnionVariant<Storage, T, Concrete, Marker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
{
    state: State<Storage, T, StateUnionState<Marker>>,
    concrete: PhantomData<fn() -> Concrete>,
}

impl<Storage, T, Concrete, Marker> StateUnionVariant<Storage, T, Concrete, Marker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
    Marker: StateUnionMember<Concrete>,
{
    #[must_use]
    pub fn new(state: State<Storage, T, Concrete>) -> Self {
        Self {
            state: crate::state::retag_state(state),
            concrete: PhantomData,
        }
    }

    #[must_use]
    pub fn into_state(self) -> State<Storage, T, Concrete> {
        crate::state::retag_state(self.state)
    }

    #[must_use]
    pub fn into_erased(self) -> State<Storage, T, StateUnionState<Marker>> {
        self.state
    }
}

impl<Storage, T, Concrete, Marker> Deref for StateUnionVariant<Storage, T, Concrete, Marker>
where
    Storage: StateStorage,
    T: StateMachineImpl,
{
    type Target = State<Storage, T, StateUnionState<Marker>>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
