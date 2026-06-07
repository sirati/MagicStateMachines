use super::{SMove, SMut, SRef, State, StateStorage, StateStorageNew, TransitionCallsite};
use crate::state::owned::{StateOwned, complete_transition};
use crate::{Initial, StateMachineImpl, Transition};
use core::marker::PhantomData;
use core::pin::Pin;
use std::rc::UniqueRc;
use std::sync::UniqueArc;

/// Backend for directly owned values.
pub struct StorageStateOwned;

pub type SOwned = StorageStateOwned;

/// Backend for `Box<T>` owned values.
pub struct StorageStateOwnedBox;

/// Backend for `Pin<Box<T>>` owned values.
pub struct StorageStateOwnedPinBox;

/// Backend for `UniqueRc<T>` owned values.
pub struct StorageStateOwnedUniqueRc;

/// Backend for `UniqueArc<T>` owned values.
pub struct StorageStateOwnedUniqueArc;

impl StateStorage for StorageStateOwned {
    type Inner<T, S>
        = StateOwned<T, S>
    where
        T: StateMachineImpl;
    type Machine<T>
        = T
    where
        T: StateMachineImpl;

    fn retag<T, From, To>(inner: Self::Inner<T, From>) -> Self::Inner<T, To>
    where
        T: StateMachineImpl,
    {
        super::retag_owned(inner)
    }

    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        _args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        State {
            inner: complete_transition(state.inner, callsite),
            marker: PhantomData,
        }
    }

    fn complete_transition_after_effect<T, From, To>(
        state: State<Self, T, From>,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
    {
        State {
            inner: complete_transition(state.inner, callsite),
            marker: PhantomData,
        }
    }
}

impl StateStorageNew for StorageStateOwned {
    fn new<T, S>(value: T) -> Self::Inner<T, S>
    where
        T: StateMachineImpl,
        T::Standin: Initial<S>,
    {
        StateOwned::new(value)
    }
}

impl SRef for StorageStateOwned {
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        &inner.value
    }
}

impl SMut for StorageStateOwned {
    fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
    where
        T: StateMachineImpl,
    {
        &mut inner.value
    }
}

impl SMove for StorageStateOwned {}

macro_rules! indirect_owned_storage {
    ($storage:ty, $wrapper:ident) => {
        impl StateStorage for $storage {
            type Inner<T, S>
                = StateOwned<$wrapper<T>, S>
            where
                T: StateMachineImpl;
            type Machine<T>
                = $wrapper<T>
            where
                T: StateMachineImpl;

            fn retag<T, From, To>(inner: Self::Inner<T, From>) -> Self::Inner<T, To>
            where
                T: StateMachineImpl,
            {
                super::retag_owned(inner)
            }

            fn complete_transition<T, From, To, Args>(
                state: State<Self, T, From>,
                _args: Args,
                callsite: TransitionCallsite,
            ) -> State<Self, T, To>
            where
                T: StateMachineImpl,
                From: crate::StateTrait,
                To: crate::StateTrait,
                T::Standin: Transition<From, To>,
                <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
                Args: core::marker::Tuple,
            {
                State {
                    inner: complete_transition(state.inner, callsite),
                    marker: PhantomData,
                }
            }

            fn complete_transition_after_effect<T, From, To>(
                state: State<Self, T, From>,
                callsite: TransitionCallsite,
            ) -> State<Self, T, To>
            where
                T: StateMachineImpl,
                From: crate::StateTrait,
                To: crate::StateTrait,
            {
                State {
                    inner: complete_transition(state.inner, callsite),
                    marker: PhantomData,
                }
            }
        }

        impl StateStorageNew for $storage {
            fn new<T, S>(value: T) -> Self::Inner<T, S>
            where
                T: StateMachineImpl,
                <Self::Machine<T> as StateMachineImpl>::Standin: Initial<S>,
            {
                StateOwned::new($wrapper::new(value))
            }
        }

        impl SRef for $storage {
            fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
            where
                T: StateMachineImpl,
            {
                &inner.value
            }
        }

        impl SMut for $storage {
            fn s_mut<T, S>(inner: &mut Self::Inner<T, S>) -> &mut T
            where
                T: StateMachineImpl,
            {
                &mut inner.value
            }
        }

        impl SMove for $storage {}
    };
}

indirect_owned_storage!(StorageStateOwnedBox, Box);
indirect_owned_storage!(StorageStateOwnedUniqueRc, UniqueRc);
indirect_owned_storage!(StorageStateOwnedUniqueArc, UniqueArc);

impl StateStorage for StorageStateOwnedPinBox {
    type Inner<T, S>
        = StateOwned<Pin<Box<T>>, S>
    where
        T: StateMachineImpl;
    type Machine<T>
        = Pin<Box<T>>
    where
        T: StateMachineImpl;

    fn retag<T, From, To>(inner: Self::Inner<T, From>) -> Self::Inner<T, To>
    where
        T: StateMachineImpl,
    {
        super::retag_owned(inner)
    }

    fn complete_transition<T, From, To, Args>(
        state: State<Self, T, From>,
        _args: Args,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        State {
            inner: complete_transition(state.inner, callsite),
            marker: PhantomData,
        }
    }

    fn complete_transition_after_effect<T, From, To>(
        state: State<Self, T, From>,
        callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        T: StateMachineImpl,
        From: crate::StateTrait,
        To: crate::StateTrait,
    {
        State {
            inner: complete_transition(state.inner, callsite),
            marker: PhantomData,
        }
    }
}

impl StateStorageNew for StorageStateOwnedPinBox {
    fn new<T, S>(value: T) -> Self::Inner<T, S>
    where
        T: StateMachineImpl,
        <Self::Machine<T> as StateMachineImpl>::Standin: Initial<S>,
    {
        StateOwned::new(Box::pin(value))
    }
}

impl SRef for StorageStateOwnedPinBox {
    fn s_ref<T, S>(inner: &Self::Inner<T, S>) -> &T
    where
        T: StateMachineImpl,
    {
        &inner.value
    }
}

impl SMove for StorageStateOwnedPinBox {}
