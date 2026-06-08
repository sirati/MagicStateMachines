use crate::{
    Initial, SArcMutex, SBox, SMove, SOwned, SPinBox, SRcRefCell, SharedStateError, State,
    StateCopy, StateMachineImpl, StateOwned, StateUnionState, StorageStateOwnedBox,
    StorageStateOwnedPinBox, StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc, Transition,
    transition, transition_state,
};
use core::marker::PhantomData;
#[cfg(not(feature = "tracing"))]
use core::mem::{align_of, size_of};
use core::pin::Pin;
use std::cell::BorrowError;
use std::rc::UniqueRc;
use std::sync::{TryLockError, UniqueArc};

struct Machine;
struct Ready;
crate::States! {
    Running;
}

crate::StateUnion!(Active: Running);

#[derive(Clone, Copy)]
struct Runtime;
struct TransitionToken;

struct SharedRuntime {
    value: u32,
}

struct MultiInitialMachine;
struct FirstInitial;
struct SecondInitial;
struct MultiTargetMachine;
struct MultiFromA;
struct MultiFromB;
struct MultiToA;
struct MultiToB;

crate::StateMachineDefinition! {
    for MultiInitialMachine;

    Initial: FirstInitial | SecondInitial;
}

crate::StateMachineDefinition! {
    for MultiTargetMachine;

    Initial: MultiFromA;

    transition MultiFromA | MultiFromB => MultiToA | MultiToB(flag: bool);
}

fn assert_initial<State>()
where
    MultiInitialMachine: Initial<State>,
{
}

fn assert_multi_target_transition<From, To>()
where
    MultiTargetMachine: Transition<From, To, F = fn(bool)>,
{
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

    let ready = SBox::new(State::<SOwned, _, Ready>::new(Runtime));
    let _: State<StorageStateOwnedBox, Runtime, Running> =
        transition_state(ready, TransitionToken)();

    let ready = SPinBox::new(SBox::new(State::<SOwned, _, Ready>::new(Runtime)));
    let _: State<StorageStateOwnedPinBox, Runtime, Running> =
        transition_state(ready, TransitionToken)();

    let ready =
        State::<StorageStateOwnedUniqueRc, _, Ready>::new(State::<SOwned, _, Ready>::new(Runtime));
    let _: State<StorageStateOwnedUniqueRc, Runtime, Running> =
        transition_state(ready, TransitionToken)();

    let ready =
        State::<StorageStateOwnedUniqueArc, _, Ready>::new(State::<SOwned, _, Ready>::new(Runtime));
    let _: State<StorageStateOwnedUniqueArc, Runtime, Running> =
        transition_state(ready, TransitionToken)();
}

#[test]
fn owned_state_can_change_box_container() {
    let ready = State::<SOwned, _, Ready>::new(Runtime);
    let boxed: SBox<Runtime, Ready> = SBox::new(ready);
    let running: State<StorageStateOwnedBox, Runtime, Running> =
        transition_state(boxed, TransitionToken)();
    let _running: State<SOwned, Runtime, Running> = SBox::unbox(running);
}

#[test]
fn boxed_state_can_be_pinned_in_place() {
    let ready = SBox::new(State::<SOwned, _, Ready>::new(Runtime));
    let pinned: SPinBox<Runtime, Ready> = SPinBox::new(ready);
    let running: State<StorageStateOwnedPinBox, Runtime, Running> =
        transition_state(pinned, TransitionToken)();
    let boxed: SBox<Runtime, Running> = SPinBox::into_boxed(running);
    let _running: State<SOwned, Runtime, Running> = SBox::unbox(boxed);
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
fn state_machine_definition_supports_multiple_initial_states() {
    assert_initial::<FirstInitial>();
    assert_initial::<SecondInitial>();
}

#[test]
fn state_machine_definition_supports_multiple_target_states() {
    assert_multi_target_transition::<MultiFromA, MultiToA>();
    assert_multi_target_transition::<MultiFromA, MultiToB>();
    assert_multi_target_transition::<MultiFromB, MultiToA>();
    assert_multi_target_transition::<MultiFromB, MultiToB>();
}

#[test]
fn rc_state_guard_commits_transition_on_drop() {
    let state = SRcRefCell::new::<Ready>(SharedRuntime { value: 1 });
    let alias = state.clone();

    let mut guard = state.borrow_mut::<Ready>().expect("initial state");
    guard.value = 2;
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken)();
    guard.value = 3;

    match alias.borrow::<Ready>() {
        Err(SharedStateError::Storage(error)) => {
            let _: BorrowError = error;
        }
        Err(SharedStateError::WrongState(_)) => panic!("expected native RefCell borrow error"),
        Ok(_) => panic!("expected native RefCell borrow error"),
    }

    drop(guard);

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::WrongState(_))
    ));
    assert_eq!(alias.borrow::<Running>().expect("committed state").value, 3);
}

#[test]
fn rc_state_borrows_committed_state_through_erased_union() {
    let state = SRcRefCell::new::<Ready>(SharedRuntime { value: 10 });
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
    let state = SArcMutex::new::<Ready>(SharedRuntime { value: 4 });
    let alias = state.clone();

    let guard = state.borrow_mut::<Ready>().expect("initial state");
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken)();
    guard.value = 5;
    drop(guard);

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::WrongState(_))
    ));
    assert_eq!(alias.borrow::<Running>().expect("committed state").value, 5);
}

#[test]
fn mutex_state_reports_native_error_when_already_borrowed() {
    let state = SArcMutex::new::<Ready>(SharedRuntime { value: 6 });
    let alias = state.clone();
    let guard = state.borrow::<Ready>().expect("initial state");

    match alias.borrow::<Ready>() {
        Err(SharedStateError::Storage(TryLockError::WouldBlock)) => {}
        Err(SharedStateError::Storage(TryLockError::Poisoned(_))) => {
            panic!("mutex should not be poisoned")
        }
        Err(SharedStateError::WrongState(_)) => panic!("state should still be ready"),
        Ok(_) => panic!("second mutex borrow should not succeed"),
    }

    drop(guard);
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
    crate::States! {
        Ready;
        Connected;
        Authenticated;
        Stopped;
    }

    struct Runtime {
        value: u32,
    }

    impl Initial<Ready> for Machine {}
    impl Transition<Ready, Connected> for Machine {}
    impl Transition<Connected, Authenticated> for Machine {}
    impl Transition<Connected, Ready> for Machine {}
    impl Transition<Authenticated, Ready> for Machine {}
    impl Transition<Authenticated, Connected> for Machine {}
    impl Transition<Connected, Stopped> for Machine {}
    impl Transition<Authenticated, Stopped> for Machine {}

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

        transition Connected => Stopped() {
            self.value += 2;
        }

        transition Authenticated => Stopped() {
            self.value += 20;
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
    fn erased_union_transition_runs_concrete_body_with_normal_transition() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let authenticated: State<SOwned, _, Authenticated> = connected.transition()();
        let online = <Authenticated as crate::In<Online>>::into_enum(authenticated);
        let ready: State<SOwned, _, Ready> =
            crate::undiscriminate_state(online.transitionExp2(Online)());

        assert_eq!(ready.value, 11);
    }

    #[test]
    fn union_proof_transition_infers_proof_from_receiver_and_target() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let authenticated: State<SOwned, _, Authenticated> = connected.transition()();
        let ready: State<SOwned, _, Ready> = authenticated.transitionExp2(Online)();

        assert_eq!(ready.value, 11);
    }

    #[test]
    fn discriminated_union_transition_runs_exact_body_when_bodies_differ() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let stopped: State<SOwned, _, Stopped> =
            <Connected as crate::In<Online>>::into_enum(connected).transition_discriminated()();

        assert_eq!(stopped.value, 2);

        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = ready.transition()();
        let authenticated: State<SOwned, _, Authenticated> = connected.transition()();
        let stopped: State<SOwned, _, Stopped> =
            <Authenticated as crate::In<Online>>::into_enum(authenticated)
                .transition_discriminated()();

        assert_eq!(stopped.value, 21);
    }
}
