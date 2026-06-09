use crate::{In, State, StateMachineImpl, StateStorage, StateUnionDiscriminant};

pub trait EnumExt: StateUnionDiscriminant {
    fn into_enum<S: StateStorage, T: StateMachineImpl>(
        self,
        state: State<S, T, Self>,
    ) -> <Self as StateUnionDiscriminant>::Enum<S, T> {
        <_ as In<Self>>::into_enum(state).discriminate()
    }
}

impl<T: StateUnionDiscriminant> EnumExt for T {}
