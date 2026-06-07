use crate::{
    Initial, MutexState, RefCellState, SharedStateError, State, StateCopy, StateMachineImpl,
    StateOwned, StorageStateOwned, StorageStateOwnedBox, StorageStateOwnedPinBox,
    StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc, Transition, transition,
    transition_state,
};
use core::marker::PhantomData;
#[cfg(not(feature = "tracing"))]
use core::mem::{align_of, size_of};
use core::pin::Pin;
use std::rc::UniqueRc;
use std::sync::UniqueArc;

struct Machine;
struct Ready;
struct Running;

#[derive(Clone, Copy)]
struct Runtime;
struct TransitionToken;

struct SharedRuntime {
    value: u32,
}

impl !StateCopy for Running {}

impl Initial<Ready> for Machine {}
impl Transition<Ready, Running> for Machine {}

impl StateMachineImpl for Runtime {
    type Standin = Machine;
    type Impl = Self;
    type TransitionToken = TransitionToken;
}

impl StateMachineImpl for SharedRuntime {
    type Standin = Machine;
    type Impl = Self;
    type TransitionToken = TransitionToken;
}

#[test]
#[cfg(not(feature = "tracing"))]
fn state_marker_has_no_layout_cost() {
    assert_eq!(
        size_of::<StateOwned<[u8; 8], Ready>>(),
        size_of::<[u8; 8]>()
    );
    assert_eq!(align_of::<StateOwned<u64, Ready>>(), align_of::<u64>());
}

#[test]
fn declared_transition_changes_only_the_type() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken)();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), 0);
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn generic_state_preserves_storage_across_transitions() {
    let ready = State::<StorageStateOwned, _, Ready>::new(Runtime);
    let _: State<StorageStateOwned, Runtime, Running> = transition_state(ready, TransitionToken)();

    let ready = State::<StorageStateOwnedBox, _, Ready>::new(Runtime);
    let _: State<StorageStateOwnedBox, Runtime, Running> =
        transition_state(ready, TransitionToken)();

    let ready = State::<StorageStateOwnedPinBox, _, Ready>::new(Runtime);
    let _: State<StorageStateOwnedPinBox, Runtime, Running> =
        transition_state(ready, TransitionToken)();

    let ready = State::<StorageStateOwnedUniqueRc, _, Ready>::new(Runtime);
    let _: State<StorageStateOwnedUniqueRc, Runtime, Running> =
        transition_state(ready, TransitionToken)();

    let ready = State::<StorageStateOwnedUniqueArc, _, Ready>::new(Runtime);
    let _: State<StorageStateOwnedUniqueArc, Runtime, Running> =
        transition_state(ready, TransitionToken)();
}

#[test]
fn matching_decomposed_parts_recompose() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let (state, data) = ready.decompose();
    let recomposed = StateOwned::recompose(state, data);

    assert!(recomposed.is_ok());
}

#[test]
fn mismatched_decomposed_parts_do_not_recompose() {
    let first: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let (first_state, _) = first.decompose();
    let second_data = loop {
        let second: StateOwned<_, Ready> = StateOwned::new(Runtime);
        let (_, data) = second.decompose();

        if first_state.uid != data.uid {
            break data;
        }
    };

    assert!(StateOwned::recompose(first_state, second_data).is_err());
}

#[test]
fn boxed_implementation_uses_the_same_contract() {
    let ready: StateOwned<Box<Runtime>, Ready> = StateOwned::new(Box::new(Runtime));
    let running: StateOwned<Box<Runtime>, Running> = transition(ready, TransitionToken)();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<Box<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn pinned_box_uses_the_same_contract() {
    let ready: StateOwned<Pin<Box<Runtime>>, Ready> = StateOwned::new(Box::pin(Runtime));
    let running: StateOwned<Pin<Box<Runtime>>, Running> = transition(ready, TransitionToken)();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<Pin<Box<Runtime>>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn unique_rc_uses_the_same_contract() {
    let ready: StateOwned<UniqueRc<Runtime>, Ready> = StateOwned::new(UniqueRc::new(Runtime));
    let running: StateOwned<UniqueRc<Runtime>, Running> = transition(ready, TransitionToken)();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<UniqueRc<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn unique_arc_uses_the_same_contract() {
    let ready: StateOwned<UniqueArc<Runtime>, Ready> = StateOwned::new(UniqueArc::new(Runtime));
    let running: StateOwned<UniqueArc<Runtime>, Running> = transition(ready, TransitionToken)();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<UniqueArc<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(not(feature = "tracing"))]
fn copying_state_copies_the_runtime_value() {
    let first: StateOwned<Runtime, Ready> = StateOwned::new(Runtime);
    let second = first;

    let _: StateOwned<Runtime, Running> = transition(first, TransitionToken)();
    let _: StateOwned<Runtime, Running> = transition(second, TransitionToken)();
}

#[test]
fn clone_policy_can_allow_clone_without_copy() {
    let first = StateOwned::<Runtime, Running> {
        value: Runtime,
        state: PhantomData,
        #[cfg(feature = "tracing")]
        trace: Vec::new(),
    };
    let _second = first.clone();
}

#[test]
fn rc_state_guard_commits_transition_on_drop() {
    let state = RefCellState::new::<Ready>(SharedRuntime { value: 1 });
    let alias = state.clone();

    let mut guard = state.borrow_mut::<Ready>().expect("initial state");
    guard.value = 2;
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken)();
    guard.value = 3;

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::Borrowed)
    ));

    drop(guard);

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::WrongState { .. })
    ));
    assert_eq!(alias.borrow::<Running>().expect("committed state").value, 3);
}

#[test]
fn arc_state_guard_commits_transition_on_drop() {
    let state = MutexState::new::<Ready>(SharedRuntime { value: 4 });
    let alias = state.clone();

    let guard = state.borrow_mut::<Ready>().expect("initial state");
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken)();
    guard.value = 5;
    drop(guard);

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::WrongState { .. })
    ));
    assert_eq!(alias.borrow::<Running>().expect("committed state").value, 5);
}

#[test]
#[cfg(feature = "tracing")]
fn tracing_records_transition_and_callsite() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let expected_line = line!() + 1;
    let running: StateOwned<_, Running> = transition(ready, TransitionToken)();
    let entry = &running.trace()[0];

    assert!(entry.from().type_name().ends_with("::Ready"));
    assert!(entry.to().type_name().ends_with("::Running"));
    assert_eq!(entry.callsite().file(), file!());
    assert_eq!(entry.callsite().line(), expected_line);
}

#[test]
#[cfg(feature = "tracing")]
fn decomposition_preserves_trace() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken)();
    let (state, data) = running.decompose();
    let running = StateOwned::recompose(state, data).expect("matching provenance");

    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(feature = "tracing")]
fn cloning_state_clones_erased_markers() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken)();
    let cloned = running.clone();

    assert!(cloned.trace()[0].from().type_name().ends_with("::Ready"));
    assert!(cloned.trace()[0].to().type_name().ends_with("::Running"));
}
