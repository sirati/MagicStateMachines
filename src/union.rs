use crate::{State, StateMachineImpl, StateStorage, Transition};
use core::marker::PhantomData;
use core::ops::Deref;

/// State marker shared by every member of a generated state union.
#[doc(hidden)]
pub struct StateUnionState<Marker>(PhantomData<fn() -> Marker>);

/// Records that `State` belongs to a generated state union.
#[doc(hidden)]
pub trait StateUnionMember<State> {}

/// Resolves transitions supported by every member of a generated state union.
#[doc(hidden)]
pub trait StateUnionTransition<Standin, To> {
    type F;
}

impl<Standin, Marker, To> Transition<StateUnionState<Marker>, To> for Standin
where
    Marker: StateUnionTransition<Standin, To>,
{
    type F = Marker::F;
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
    pub fn into_joint(self) -> State<Storage, T, StateUnionState<Marker>> {
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
