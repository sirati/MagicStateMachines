mod guard;
mod storage;
mod weak;

use crate::{
    Initial, RuntimeStateMarker, SOwned, State, StateMachineImpl, StateMarker,
    StateRuntimeMarkerFor, StateTrait, state_trait,
};
use core::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

pub use guard::{
    SharedBorrowState, StateMut, StateMutTransitionCall, StateRef, StorageStateMut,
    StorageStateRef, transition_mut,
};
pub use storage::{
    MutexStorage, RefCellStorage, RwLockStorage, SharedStateError, SharedStorage, SharedValue,
    WrongStateError,
};
pub use weak::{WeakSArc, WeakSArcMutex, WeakSArcRwLock, WeakSRc, WeakSRcRefCell};

/// Shared state using an explicit, replaceable storage backend.
///
/// `SharedState` is the runtime boundary for this library. Owned state tokens
/// carry their current state only in the type system. Shared containers such
/// as `Rc<RefCell<_>>`, `Arc<Mutex<_>>`, and `Arc<RwLock<_>>` need one
/// authoritative runtime marker because aliases can request typed views at
/// different times.
///
/// A borrow checks that runtime marker first. After the check succeeds, the
/// returned value is again a statically typed `State` view, so ordinary
/// read-only state-machine methods regain compile-time guarantees:
///
/// ```ignore
/// use magicstatemachines::{SArcMutex, transition};
/// use test_def::{Online, states::{Connected, Disconnected}};
///
/// let shared = SArcMutex::<Connection>::new::<Disconnected>(
///     Connection::new("localhost:8080"),
/// );
///
/// {
///     let disconnected = shared.borrow_mut::<Disconnected>()?;
///     let connected = transition!(disconnected);
///     drop(connected); // commits `Connected` back to the shared container.
/// }
///
/// let connected = shared.borrow::<Connected>()?;
/// let online = shared.borrow::<Online>()?;
/// ```
///
/// The storage backend is an explicit type parameter. The built-in aliases
/// cover the common cases:
///
/// - [`SRcRefCell<T>`] for single-threaded shared mutable state;
/// - [`SArcMutex<T>`] for shared state protected by `std::sync::Mutex`;
/// - [`SArcRwLock<T>`] for shared state protected by `std::sync::RwLock`;
/// - [`SRc<Storage, T>`] and [`SArc<Storage, T>`] when you provide a custom
///   [`SharedStorage`] implementation.
///
/// Union markers can be borrowed, but cannot be stored as the committed runtime state:
///
/// ```compile_fail
/// use magicstatemachines::{SArcMutex, StateMachineDefinition, StateMachineImpl, States};
///
/// struct Machine;
/// struct Standin;
///
/// States! {
///     A;
///     B;
/// }
///
/// StateMachineDefinition! {
///     for Standin;
///
///     pub Initial: A;
///     transition A => B();
///     union Any: A | B;
/// }
///
/// StateMachineImpl! {
///     Machine: Standin;
///
///     transition A => B();
/// }
///
/// let _state = SArcMutex::<Machine>::new::<Any>(Machine);
/// ```
pub struct SharedState<P, S, T>
where
    S: SharedStorage,
{
    pub(super) storage: P,
    pub(super) backend: PhantomData<fn() -> S>,
    pub(super) value: PhantomData<fn() -> T>,
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
    /// Creates shared state from a runtime value in an allowed initial state.
    ///
    /// `State` must be a concrete initial state declared by the definition
    /// crate. Union markers are intentionally rejected as committed storage
    /// states; they can be borrowed as views after a concrete state is stored.
    ///
    /// ```ignore
    /// let shared = SArcMutex::<Connection>::new::<Disconnected>(
    ///     Connection::new("localhost:8080"),
    /// );
    /// ```
    #[must_use]
    pub fn new<State>(value: T) -> Self
    where
        T::Standin: Initial<State>,
        State: crate::ConcreteStateTrait,
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

    /// Moves an owned state token into shared storage without changing state.
    ///
    /// This is the shared-storage counterpart to putting an already-created
    /// [`State`] into `Rc`, `Arc`, or another container. The committed runtime
    /// marker is taken from the concrete state token.
    ///
    /// ```ignore
    /// let disconnected: State<SOwned, Connection, Disconnected> =
    ///     State::new(Connection::new("localhost:8080"));
    /// let shared = SArcMutex::<Connection>::from_state(disconnected);
    /// ```
    #[must_use]
    pub fn from_state<StateMarker>(state: State<SOwned, T, StateMarker>) -> Self
    where
        StateMarker: crate::ConcreteStateTrait,
    {
        Self {
            storage: P::from(Backend::new(SharedValue {
                state: state_trait::erased_state::<StateMarker>(),
                value: state.inner.value,
            })),
            backend: PhantomData,
            value: PhantomData,
        }
    }

    /// Borrows the runtime value if the committed state matches `RequestedState`.
    ///
    /// `RequestedState` may be a concrete state or a generated union marker.
    /// Concrete borrows require the exact committed state. Union borrows
    /// succeed when the committed concrete state is a member of that union.
    /// Errors distinguish "the container could not be borrowed/locked" from
    /// "the borrow succeeded but the state was wrong":
    ///
    /// ```ignore
    /// let connected = shared.borrow::<Connected>()?;
    /// let online = shared.borrow::<Online>()?;
    ///
    /// match shared.borrow::<Authenticated>() {
    ///     Err(magicstatemachines::SharedStateError::WrongState(error)) => {
    ///         eprintln!("{error}");
    ///     }
    ///     other => { /* storage errors and success are handled separately */ }
    /// }
    /// ```
    pub fn borrow<RequestedState>(
        &self,
    ) -> Result<
        SRefView<'_, Backend, T, RuntimeStateMarker<RequestedState>>,
        SharedStateError<Backend::ReadError<'_, T>>,
    >
    where
        RequestedState:
            StateTrait + StateMarker + StateRuntimeMarkerFor<<RequestedState as StateMarker>::Kind>,
        RuntimeStateMarker<RequestedState>: SharedBorrowState,
    {
        let guard = Backend::read(self.storage.as_ref()).map_err(SharedStateError::Storage)?;
        StateRef::from_guard(guard).map(State::from_inner)
    }

    /// Mutably borrows the runtime value and tracks the guard's final state.
    ///
    /// When the returned guard is dropped, the shared container is updated to
    /// the guard's pending state. This allows methods on `State<SMutView<...>>`
    /// to retain compile-time transition checks while the committed state is
    /// stored at runtime.
    ///
    /// ```ignore
    /// {
    ///     let connected = shared.borrow_mut::<Connected>()?;
    ///     let authenticated = transition!(connected, "alice".to_owned());
    ///     drop(authenticated); // commits `Authenticated`.
    /// }
    ///
    /// let authenticated = shared.borrow::<Authenticated>()?;
    /// ```
    pub fn borrow_mut<RequestedState>(
        &self,
    ) -> Result<
        SMutView<'_, Backend, T, RuntimeStateMarker<RequestedState>>,
        SharedStateError<Backend::WriteError<'_, T>>,
    >
    where
        RequestedState:
            StateTrait + StateMarker + StateRuntimeMarkerFor<<RequestedState as StateMarker>::Kind>,
        RuntimeStateMarker<RequestedState>: SharedBorrowState,
    {
        let guard = Backend::write(self.storage.as_ref()).map_err(SharedStateError::Storage)?;
        StateMut::from_guard(guard).map(State::from_inner)
    }
}

/// Shared state backed by `Rc<Storage::Storage<T>>`.
///
/// Use this alias when you want single-threaded aliasing but want to choose
/// the synchronization cell yourself. The first type parameter is the
/// [`SharedStorage`] backend, not the actual `Rc` payload:
///
/// ```ignore
/// use magicstatemachines::{RefCellStorage, SRc};
///
/// let shared: SRc<RefCellStorage, Connection> =
///     SRc::new::<Disconnected>(Connection::new("localhost:8080"));
/// ```
///
/// Most code should use [`SRcRefCell`] unless it is intentionally exercising a
/// custom backend.
pub type SRc<Storage, T> = SharedState<Rc<<Storage as SharedStorage>::Storage<T>>, Storage, T>;
/// Shared state backed by `Arc<Storage::Storage<T>>`.
///
/// This is the thread-safe counterpart to [`SRc`]. It is useful when the
/// backend is selected by a public type alias or a generic parameter:
///
/// ```ignore
/// use magicstatemachines::{MutexStorage, SArc};
///
/// type SharedConnection = SArc<MutexStorage, Connection>;
///
/// let shared = SharedConnection::new::<Disconnected>(
///     Connection::new("localhost:8080"),
/// );
/// ```
///
/// Use [`SArcMutex`] or [`SArcRwLock`] for the built-in backends when no
/// custom storage choice is needed.
pub type SArc<Storage, T> = SharedState<Arc<<Storage as SharedStorage>::Storage<T>>, Storage, T>;
/// Shared state backed by `Rc<RefCell<...>>`.
///
/// This is the default single-threaded shared-state container. It preserves
/// the native `RefCell` error behavior: borrowing mutably while an immutable
/// borrow is alive returns [`SharedStateError::Storage`] containing
/// `std::cell::BorrowMutError`; asking for a state that is not committed
/// returns [`SharedStateError::WrongState`].
pub type SRcRefCell<T> = SRc<RefCellStorage, T>;
/// Shared state backed by `Arc<Mutex<...>>`.
///
/// `borrow` and `borrow_mut` both acquire the mutex with `try_lock`, so a
/// concurrent borrow reports the standard `TryLockError` through
/// [`SharedStateError::Storage`] instead of blocking the caller.
pub type SArcMutex<T> = SArc<MutexStorage, T>;
/// Shared state backed by `Arc<RwLock<...>>`.
///
/// Immutable borrows use `try_read` and can coexist with other immutable
/// borrows. Mutable borrows use `try_write` and fail with the backend's
/// `TryLockError` while readers or another writer are alive.
pub type SArcRwLock<T> = SArc<RwLockStorage, T>;
/// Mutable-guard storage backend for [`RefCellStorage`].
///
/// This is the `Storage` parameter of a state returned by
/// [`SRcRefCell::borrow_mut`]. It is useful in signatures when a method wants
/// to specifically accept a `RefCell` guard rather than any [`crate::SMut`]
/// storage:
///
/// ```ignore
/// fn only_ref_cell_guard(
///     state: magicstatemachines::State<
///         magicstatemachines::SRefCell<'_>,
///         Connection,
///         Connected,
///     >,
/// ) {
///     drop(state);
/// }
/// ```
///
/// State-machine implementation methods usually prefer `S: SMut` so they also
/// work with owned, boxed, mutex, and custom storage.
pub type SRefCell<'a> = StorageStateMut<'a, RefCellStorage>;
/// Mutable-guard storage backend for [`MutexStorage`].
///
/// This is the concrete guard storage used by [`SArcMutex::borrow_mut`].
/// Prefer a generic `S: SMut` bound unless you intentionally need to restrict
/// a function to mutex-backed shared state.
pub type SMutex<'a> = StorageStateMut<'a, MutexStorage>;
/// Mutable-guard storage backend for [`RwLockStorage`].
///
/// This is the concrete guard storage used by [`SArcRwLock::borrow_mut`].
/// It represents an active write guard whose final typestate will be committed
/// back to the `RwLock` when the returned [`State`] is dropped.
pub type SRwLock<'a> = StorageStateMut<'a, RwLockStorage>;
/// State view held by a mutable guard from a shared storage backend.
///
/// The alias is mainly documentation for return types. For example,
/// `SArcMutex<T>::borrow_mut::<Connected>()` returns an
/// `SMutView<'_, MutexStorage, T, Connected>`. In user-facing methods, prefer
/// the shorter arbitrary-self receiver form:
///
/// ```ignore
/// fn authenticate<S>(
///     self: magicstatemachines::State<S, Self, Connected>,
///     user: String,
/// ) -> magicstatemachines::State<S, Self, Authenticated>
/// where
///     S: magicstatemachines::SMut,
/// {
///     magicstatemachines::transition!(self, user)
/// }
/// ```
pub type SMutView<'a, Backend, T, S> = State<StorageStateMut<'a, Backend>, T, S>;
/// State view held by an immutable guard from a shared storage backend.
///
/// This is the return type of [`SharedState::borrow`]. It implements [`SRef`](crate::SRef)
/// but not [`SMut`](crate::SMut), so it supports read-only arbitrary-self
/// receivers such as `self: &State<impl SRef, Self, impl InOnline>` while
/// preventing generated transitions from completing.
pub type SRefView<'a, Backend, T, S> = State<StorageStateRef<'a, Backend>, T, S>;
