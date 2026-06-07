#![feature(
    arbitrary_self_types,
    associated_type_defaults,
    auto_traits,
    fn_traits,
    generic_const_exprs,
    random,
    tuple_trait,
    unboxed_closures,
    unique_rc_arc
)]
#![allow(incomplete_features)]
#![cfg_attr(test, feature(negative_impls))]
#![deny(unsafe_code)]

//! Zero-overhead wrappers for externally defined typestate contracts.

mod contract;
mod decomposed;
mod policy;
mod shared;
mod state;
#[allow(unsafe_code)]
mod state_trait;
#[cfg(feature = "tracing")]
pub mod tracing;

pub use contract::{Initial, StateMachineImpl, Transition};
pub use decomposed::{DecomposedData, DecomposedState, RecomposeError};
pub use policy::{StateClone, StateCopy};
pub use shared::{
    ArcState, MutexState, MutexStorage, RcState, RefCellState, RefCellStorage, SharedState,
    SharedStateError, SharedStorage, SharedValue, StateMut, StateMutTransitionCall, StateMutView,
    StateRef, StorageStateMut, transition_mut,
};
pub use state::{
    State, StateOwned, StateOwnedBox, StateOwnedPin, StateOwnedPinBox, StateStorage,
    StateStorageDeref, StateStorageDerefMut, StateStorageNew, StateTransitionCall,
    StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox, StorageStateOwnedUniqueArc,
    StorageStateOwnedUniqueRc, TransitionCall, TransitionCallsite, transition, transition_state,
};
pub use state_trait::StateTrait;
#[cfg(feature = "tracing")]
pub use tracing::TraceEntry;

/// Defines a public marker trait implemented by each listed state.
///
/// This allows implementation methods to accept a union of states without
/// weakening the transition contract.
#[macro_export]
macro_rules! StateUnion {
    ($name:ident: $first:ident $(+ $state:ident)* $(,)?) => {
        pub trait $name {}

        impl $name for $first {}

        $(
            impl $name for $state {}
        )*
    };
}

/// Connects a runtime type to a definition and adds private transition helpers.
///
/// Invoke this once in the module that implements the runtime's methods:
///
/// ```ignore
/// StateMachineImpl!(Connection: ConnectionStandin);
///
/// impl Connection {
///     fn connect<Storage>(
///         self: State<Storage, Self, Disconnected>,
///     ) -> State<Storage, Self, Connected>
///     where
///         Storage: StateStorageDeref<Self>,
///     {
///         self.transition()()
///     }
/// }
/// ```
#[macro_export]
macro_rules! StateMachineImpl {
    ($implementation:ty : $standin:ty $(,)?) => {
        #[doc(hidden)]
        pub struct __StateMachineTransitionToken(());

        impl $crate::StateMachineImpl for $implementation {
            type Standin = $standin;
            type Impl = $implementation;
            type TransitionToken = __StateMachineTransitionToken;
        }

        trait __StateTransitionExt<T, From>
        where
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            #[track_caller]
            fn transition<To>(self) -> $crate::TransitionCall<T, From, To>
            where
                T::Standin: $crate::Transition<From, To>;
        }

        impl<T, From> __StateTransitionExt<T, From> for $crate::StateOwned<T, From>
        where
            T: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[track_caller]
            fn transition<To>(self) -> $crate::TransitionCall<T, From, To>
            where
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition(self, __StateMachineTransitionToken(()))
            }
        }

        trait __GenericStateTransitionExt<Storage, T, From>
        where
            T: $crate::StateMachineImpl,
            Storage: $crate::StateStorage<T>,
            Storage::Machine: $crate::StateMachineImpl,
        {
            #[must_use]
            #[track_caller]
            fn transition<To>(self) -> $crate::StateTransitionCall<Storage, T, From, To>
            where
                From: $crate::StateTrait,
                To: $crate::StateTrait,
                T::Standin: $crate::Transition<From, To>;
        }

        impl<Storage, T, From> __GenericStateTransitionExt<Storage, T, From>
            for $crate::State<Storage, T, From>
        where
            T: $crate::StateMachineImpl,
            Storage: $crate::StateStorage<T>,
            Storage::Machine: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            #[track_caller]
            fn transition<To>(self) -> $crate::StateTransitionCall<Storage, T, From, To>
            where
                From: $crate::StateTrait,
                To: $crate::StateTrait,
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition_state(self, __StateMachineTransitionToken(()))
            }
        }

        trait __StateMutTransitionExt<G, T, From>
        where
            G: ::core::ops::DerefMut<Target = $crate::SharedValue<T>>,
            T: $crate::StateMachineImpl,
        {
            #[must_use]
            fn transition<To>(self) -> $crate::StateMutTransitionCall<G, T, From, To>
            where
                T::Standin: $crate::Transition<From, To>;
        }

        impl<G, T, From> __StateMutTransitionExt<G, T, From> for $crate::StateMut<G, T, From>
        where
            G: ::core::ops::DerefMut<Target = $crate::SharedValue<T>>,
            T: $crate::StateMachineImpl<
                    Standin = $standin,
                    Impl = $implementation,
                    TransitionToken = __StateMachineTransitionToken,
                >,
        {
            fn transition<To>(self) -> $crate::StateMutTransitionCall<G, T, From, To>
            where
                T::Standin: $crate::Transition<From, To>,
            {
                $crate::transition_mut(self, __StateMachineTransitionToken(()))
            }
        }
    };
}

#[cfg(test)]
mod tests;
