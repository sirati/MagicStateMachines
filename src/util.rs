use crate::{In, State, StateMachineImpl, StateStorage, StateUnionDiscriminant};

pub trait EnumExt: StateUnionDiscriminant {
    fn into_enum<S, T, Current>(
        self,
        state: State<S, T, Current>,
    ) -> <Self as StateUnionDiscriminant>::Enum<S, T>
    where
        S: StateStorage,
        T: StateMachineImpl,
        Current: In<Self>,
    {
        <Current as In<Self>>::into_enum(state).discriminate()
    }
}

impl<T: StateUnionDiscriminant> EnumExt for T {}
