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
///         Storage: SRef,
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
            Storage: $crate::StateStorage,
            Storage::Machine<T>: $crate::StateMachineImpl,
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
            Storage: $crate::StateStorage,
            Storage::Machine<T>: $crate::StateMachineImpl<
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
