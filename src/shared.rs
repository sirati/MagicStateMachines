use crate::{
    Initial, State, StateMachineImpl, StateStorage, StateStorageDeref, StateStorageDerefMut,
    StateTrait, Transition, TransitionCallsite,
    state_trait::{self, ErasedState},
};
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

/// The state marker and runtime data held by a shared-storage backend.
///
/// Its fields are private so backends can synchronize storage without changing
/// the authoritative state directly.
pub struct SharedValue<T> {
    state: ErasedState,
    value: T,
}

/// Failure to acquire a typed view of shared state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedStateError {
    WrongState {
        expected: &'static str,
        actual: &'static str,
    },
    Borrowed,
    Poisoned,
}

impl fmt::Display for SharedStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongState { expected, actual } => {
                write!(formatter, "expected state {expected}, found {actual}")
            }
            Self::Borrowed => formatter.write_str("shared state is already borrowed"),
            Self::Poisoned => formatter.write_str("shared state mutex is poisoned"),
        }
    }
}

impl std::error::Error for SharedStateError {}

/// Replaceable storage backend for [`SharedState`].
pub trait SharedStorage {
    type Storage<T>;

    type ReadGuard<'a, T>: Deref<Target = SharedValue<T>>
    where
        Self: 'a,
        T: 'a;

    type WriteGuard<'a, T>: DerefMut<Target = SharedValue<T>>
    where
        Self: 'a,
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T>;
    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError>;
    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError>;
}

pub struct RefCellStorage;

impl SharedStorage for RefCellStorage {
    type Storage<T> = RefCell<SharedValue<T>>;
    type ReadGuard<'a, T>
        = Ref<'a, SharedValue<T>>
    where
        T: 'a;
    type WriteGuard<'a, T>
        = RefMut<'a, SharedValue<T>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        RefCell::new(value)
    }

    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError> {
        storage.try_borrow().map_err(|_| SharedStateError::Borrowed)
    }

    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError> {
        storage
            .try_borrow_mut()
            .map_err(|_| SharedStateError::Borrowed)
    }
}

pub struct MutexStorage;

impl SharedStorage for MutexStorage {
    type Storage<T> = Mutex<SharedValue<T>>;
    type ReadGuard<'a, T>
        = MutexGuard<'a, SharedValue<T>>
    where
        T: 'a;
    type WriteGuard<'a, T>
        = MutexGuard<'a, SharedValue<T>>
    where
        T: 'a;

    fn new<T>(value: SharedValue<T>) -> Self::Storage<T> {
        Mutex::new(value)
    }

    fn read<T>(storage: &Self::Storage<T>) -> Result<Self::ReadGuard<'_, T>, SharedStateError> {
        storage.lock().map_err(|_| SharedStateError::Poisoned)
    }

    fn write<T>(storage: &Self::Storage<T>) -> Result<Self::WriteGuard<'_, T>, SharedStateError> {
        storage.lock().map_err(|_| SharedStateError::Poisoned)
    }
}

/// Shared state using an explicit, replaceable storage backend.
pub struct SharedState<P, S, T>
where
    S: SharedStorage,
{
    storage: P,
    backend: PhantomData<fn() -> S>,
    value: PhantomData<fn() -> T>,
}

impl<P: Clone, S: SharedStorage, T> Clone for SharedState<P, S, T> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            backend: PhantomData,
            value: PhantomData,
        }
    }
}

impl<P, Backend, T> SharedState<P, Backend, T>
where
    Backend: SharedStorage,
    P: From<Backend::Storage<T>> + AsRef<Backend::Storage<T>>,
    T: StateMachineImpl,
{
    #[must_use]
    pub fn new<State>(value: T) -> Self
    where
        T::Standin: Initial<State>,
        State: StateTrait,
    {
        Self {
            storage: P::from(Backend::new(SharedValue {
                state: state_trait::erased_state::<State>(),
                value,
            })),
            backend: PhantomData,
            value: PhantomData,
        }
    }

    pub fn borrow<State>(
        &self,
    ) -> Result<StateRef<Backend::ReadGuard<'_, T>, T, State>, SharedStateError>
    where
        State: StateTrait,
    {
        StateRef::from_guard(Backend::read(self.storage.as_ref())?)
    }

    pub fn borrow_mut<StateMarker>(
        &self,
    ) -> Result<StateMutView<'_, Backend, T, StateMarker>, SharedStateError>
    where
        StateMarker: StateTrait,
    {
        StateMut::from_guard(Backend::write(self.storage.as_ref())?).map(State::from_inner)
    }
}

pub type RcState<S, T> = SharedState<Rc<<S as SharedStorage>::Storage<T>>, S, T>;
pub type ArcState<S, T> = SharedState<Arc<<S as SharedStorage>::Storage<T>>, S, T>;
pub type RefCellState<T> = RcState<RefCellStorage, T>;
pub type MutexState<T> = ArcState<MutexStorage, T>;
pub type StateMutView<'a, Backend, T, S> =
    State<StorageStateMut<<Backend as SharedStorage>::WriteGuard<'a, T>>, T, S>;

pub struct StateRef<G, T, S> {
    guard: G,
    marker: PhantomData<fn() -> (T, S)>,
}

impl<G, T, S> StateRef<G, T, S>
where
    G: Deref<Target = SharedValue<T>>,
    S: StateTrait,
{
    fn from_guard(guard: G) -> Result<Self, SharedStateError> {
        ensure_state::<S>(&guard.state)?;
        Ok(Self {
            guard,
            marker: PhantomData,
        })
    }
}

impl<G, T, S> Deref for StateRef<G, T, S>
where
    G: Deref<Target = SharedValue<T>>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.value
    }
}

pub struct StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    guard: Option<G>,
    pending: ErasedState,
    marker: PhantomData<fn() -> (T, S)>,
}

/// Generic [`State`] backend for a mutable shared-state guard.
pub struct StorageStateMut<G>(PhantomData<fn() -> G>);

impl<G, T> StateStorage<T> for StorageStateMut<G>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
{
    type Inner<S> = StateMut<G, T, S>;
    type Machine = T;
    fn complete_transition<From, To, Args>(
        mut state: State<Self, T, From>,
        _args: Args,
        _callsite: TransitionCallsite,
    ) -> State<Self, T, To>
    where
        From: StateTrait,
        To: StateTrait,
        T::Standin: Transition<From, To>,
        <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
        Args: core::marker::Tuple,
    {
        State {
            inner: StateMut {
                guard: state.inner.guard.take(),
                pending: state_trait::erased_state::<To>(),
                marker: PhantomData,
            },
            marker: PhantomData,
        }
    }
}

impl<G, T> StateStorageDeref<T> for StorageStateMut<G>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
{
    fn deref<State>(inner: &Self::Inner<State>) -> &T {
        inner
    }
}

impl<G, T> StateStorageDerefMut<T> for StorageStateMut<G>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
{
    fn deref_mut<State>(inner: &mut Self::Inner<State>) -> &mut T {
        inner
    }
}

impl<G, T, S> StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
    S: StateTrait,
{
    fn from_guard(guard: G) -> Result<Self, SharedStateError> {
        ensure_state::<S>(&guard.state)?;
        Ok(Self {
            guard: Some(guard),
            pending: state_trait::erased_state::<S>(),
            marker: PhantomData,
        })
    }
}

/// Creates a callable transition for a mutable shared-state guard.
///
/// This is the guarded-state counterpart to [`crate::transition`].
#[must_use]
pub fn transition_mut<G, T, S, Next>(
    state: StateMut<G, T, S>,
    _token: T::TransitionToken,
) -> StateMutTransitionCall<G, T, S, Next>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
    T::Standin: Transition<S, Next>,
{
    StateMutTransitionCall {
        state,
        to: PhantomData,
    }
}

impl<G, T, S> Deref for StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard.as_ref().expect("guard is present").value
    }
}

impl<G, T, S> DerefMut for StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard.as_mut().expect("guard is present").value
    }
}

impl<G, T, S> Drop for StateMut<G, T, S>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    fn drop(&mut self) {
        if let Some(guard) = self.guard.as_mut() {
            guard.state = state_trait::clone_erased(&self.pending);
        }
    }
}

pub struct StateMutTransitionCall<G, T, From, To>
where
    G: DerefMut<Target = SharedValue<T>>,
{
    state: StateMut<G, T, From>,
    to: PhantomData<fn() -> To>,
}

impl<G, T, From, To, Args> FnOnce<Args> for StateMutTransitionCall<G, T, From, To>
where
    G: DerefMut<Target = SharedValue<T>>,
    T: StateMachineImpl,
    T::Standin: Transition<From, To>,
    <T::Standin as Transition<From, To>>::F: FnOnce<Args, Output = ()>,
    Args: core::marker::Tuple,
    To: StateTrait,
{
    type Output = StateMut<G, T, To>;

    extern "rust-call" fn call_once(mut self, _args: Args) -> Self::Output {
        StateMut {
            guard: self.state.guard.take(),
            pending: state_trait::erased_state::<To>(),
            marker: PhantomData,
        }
    }
}

fn ensure_state<S>(actual: &ErasedState) -> Result<(), SharedStateError>
where
    S: StateTrait,
{
    if state_trait::is_state::<S>(actual) {
        Ok(())
    } else {
        Err(SharedStateError::WrongState {
            expected: core::any::type_name::<S>(),
            actual: actual.type_name(),
        })
    }
}
