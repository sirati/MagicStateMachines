use crate::{
    Initial, MutexState, RefCellState, SMove, SOwned, SharedStateError, State, StateCopy,
    StateMachineImpl, StateOwned, StateUnionState, StorageStateOwnedBox, StorageStateOwnedPinBox,
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
pub struct Running;

crate::StateUnion!(Active: Running);

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
    fn assert_move_storage<S: SMove>() {}
    assert_move_storage::<SOwned>();
    assert_move_storage::<StorageStateOwnedBox>();
    assert_move_storage::<StorageStateOwnedPinBox>();
    assert_move_storage::<StorageStateOwnedUniqueRc>();
    assert_move_storage::<StorageStateOwnedUniqueArc>();

    let ready = State::<SOwned, _, Ready>::new(Runtime);
    let _: State<SOwned, Runtime, Running> = transition_state(ready, TransitionToken)();

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
fn rc_state_borrows_committed_state_through_erased_union() {
    let state = RefCellState::new::<Ready>(SharedRuntime { value: 10 });
    let alias = state.clone();

    let guard = state.borrow_mut::<Ready>().expect("initial state");
    let guard = transition_state::<_, _, _, Running>(guard, TransitionToken)();
    drop(guard);

    {
        let erased = alias
            .borrow::<StateUnionState<Active>>()
            .expect("running is active");
        assert_eq!(erased.value, 10);
    }

    {
        let erased = alias
            .borrow_mut::<StateUnionState<Active>>()
            .expect("running is active");
        drop(erased);
    }

    assert_eq!(alias.borrow::<Running>().expect("still concrete").value, 10);
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

mod transition_effect_syntax {
    use crate::{Initial, SOwned, State, Transition};

    struct Machine;
    pub struct Ready;
    pub struct Connected;
    pub struct Authenticated;

    struct Runtime {
        value: u32,
    }

    impl Initial<Ready> for Machine {}
    impl Transition<Ready, Connected> for Machine {}
    impl Transition<Connected, Authenticated> for Machine {}
    impl Transition<Connected, Ready> for Machine {}
    impl Transition<Authenticated, Ready> for Machine {}
    impl Transition<Authenticated, Connected> for Machine {}

    crate::StateUnion!(Online: Connected | Authenticated);

    crate::StateMachineImpl! {
        Runtime: Machine;

        transition Ready => Connected();

        transition Connected => Authenticated() {
            self.value += 1;
        }

        transition Connected | Authenticated => Ready(),
        transition Authenticated => Connected() {
            self.value += 10;
        }
    }

    #[test]
    fn semicolon_transition_has_empty_body() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let authenticated: State<SOwned, _, Authenticated> = connected.transition()();

        assert_eq!(authenticated.value, 1);
    }

    #[test]
    fn comma_terminated_transition_shares_next_body() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let ready: State<SOwned, _, Ready> = connected.transition()();

        assert_eq!(ready.value, 10);

        let connected = ready.transition()();
        let authenticated: State<SOwned, _, Authenticated> = connected.transition()();
        let ready: State<SOwned, _, Ready> = authenticated.transition()();

        assert_eq!(ready.value, 21);
    }

    #[test]
    fn erased_union_transition_runs_shared_body() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let authenticated: State<SOwned, _, Authenticated> = connected.transition()();
        let online = <Authenticated as InOnline>::into_erased(authenticated);
        let ready: State<SOwned, _, Ready> = online.transition()();

        assert_eq!(ready.value, 11);
    }
}
