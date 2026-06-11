use crate::{
    Initial, MutexStorage, RefCellStorage, RwLockStorage, SArc, SArcMutex, SArcRwLock, SBox,
    SDiscriminated, SMove, SMutex, SOwned, SPinBox, SRc, SRcRefCell, SRefCell, SRefView, SRwLock,
    SharedStateError, State, StateCopy, StateMachineImpl, StateOwned, StateUnionState,
    StorageStateOwnedBox, StorageStateOwnedPinBox, StorageStateRef, Transition, WeakSArcMutex,
    WeakSArcRwLock, WeakSRcRefCell, transition, transition_state,
};
#[cfg(feature = "unique-rc-arc")]
use crate::{StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc};
#[cfg(feature = "tracing")]
use core::any::TypeId;
use core::marker::{PhantomData, PhantomPinned};
#[cfg(not(feature = "tracing"))]
use core::mem::align_of;
use core::mem::size_of;
use core::pin::Pin;
use std::cell::{BorrowError, Cell};
#[cfg(feature = "unique-rc-arc")]
use std::rc::UniqueRc;
use std::sync::TryLockError;
#[cfg(feature = "unique-rc-arc")]
use std::sync::UniqueArc;

struct Machine;
crate::States! {
    Ready;
    Running;
}

#[allow(dead_code)]
mod documented_state_markers {
    crate::States! {
        /// State marker documented with a normal doc comment.
        #[derive(Debug, Default, PartialEq, Eq)]
        #[allow(dead_code)]
        DocCommentState;

        #[doc = "State marker documented with an explicit doc attribute."]
        #[derive(Debug, Default, PartialEq, Eq)]
        #[allow(dead_code)]
        DocAttributeState;
    }
}

crate::StateUnion!(Active: Running);

#[derive(Clone, Copy)]
struct Runtime;
struct TransitionToken;

struct SharedRuntime {
    value: u32,
}

struct MultiInitialMachine;
struct MultiTargetMachine;

crate::States! {
    FirstInitial;
    SecondInitial;
    MultiFromA;
    MultiFromB;
    MultiToA;
    MultiToB;
}

crate::StateMachineDefinition! {
    for MultiInitialMachine;

    pub Initial: FirstInitial | SecondInitial;
}

crate::StateMachineDefinition! {
    for MultiTargetMachine;

    pub Initial: MultiFromA;

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
    let running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());

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
    #[cfg(feature = "unique-rc-arc")]
    assert_move_storage::<StorageStateOwnedUniqueRc>();
    #[cfg(feature = "unique-rc-arc")]
    assert_move_storage::<StorageStateOwnedUniqueArc>();

    let ready = State::<SOwned, _, Ready>::new(Runtime);
    let _: State<SOwned, Runtime, Running> = transition_state(ready, TransitionToken).call(());

    let ready = SBox::new(State::<SOwned, _, Ready>::new(Runtime));
    let _: State<StorageStateOwnedBox, Runtime, Running> =
        transition_state(ready, TransitionToken).call(());

    let ready = SPinBox::new(SBox::new(State::<SOwned, _, Ready>::new(Runtime)));
    let _: State<StorageStateOwnedPinBox, Runtime, Running> =
        transition_state(ready, TransitionToken).call(());

    #[cfg(feature = "unique-rc-arc")]
    {
        let ready = State::<StorageStateOwnedUniqueRc, _, Ready>::new(
            State::<SOwned, _, Ready>::new(Runtime),
        );
        let _: State<StorageStateOwnedUniqueRc, Runtime, Running> =
            transition_state(ready, TransitionToken).call(());

        let ready = State::<StorageStateOwnedUniqueArc, _, Ready>::new(
            State::<SOwned, _, Ready>::new(Runtime),
        );
        let _: State<StorageStateOwnedUniqueArc, Runtime, Running> =
            transition_state(ready, TransitionToken).call(());
    }
}

#[test]
fn owned_state_can_round_trip_through_concrete_state_proof() {
    let ready = State::<SOwned, _, Ready>::new(Runtime);
    let concrete = State::into_concrete(ready);
    let raw = concrete.into_raw();
    let concrete = crate::__private::concrete_stated_new::<_, Ready>(raw, TransitionToken);
    let ready = State::<SOwned, Runtime, Ready>::from_concrete(concrete);

    fn assert_ready_state(_: State<SOwned, Runtime, Ready>) {}
    assert_ready_state(ready);
}

#[test]
fn owned_state_can_change_box_container() {
    let ready = State::<SOwned, _, Ready>::new(Runtime);
    let boxed: SBox<Runtime, Ready> = SBox::new(ready);
    let running: State<StorageStateOwnedBox, Runtime, Running> =
        transition_state(boxed, TransitionToken).call(());
    let _running: State<SOwned, Runtime, Running> = SBox::unbox(running);
}

#[test]
fn boxed_state_can_be_pinned_in_place() {
    let ready = SBox::new(State::<SOwned, _, Ready>::new(Runtime));
    let pinned: SPinBox<Runtime, Ready> = SPinBox::new(ready);
    let running: State<StorageStateOwnedPinBox, Runtime, Running> =
        transition_state(pinned, TransitionToken).call(());
    let boxed: SBox<Runtime, Running> = SPinBox::into_boxed(running);
    let _running: State<SOwned, Runtime, Running> = SBox::unbox(boxed);
}

mod pinned_transition_support {
    use super::{Cell, PhantomPinned};
    use crate::{Initial, SBox, SMut, SOwned, SPinBox, SPinMut, SPinRef, State, Transition};
    use core::pin::Pin;

    struct PinnedMachine;
    crate::States! {
        PinReady;
        PinRunning;
        PinDone;
        PinChoiceLeft;
        PinChoiceRight;
        PinChoiceDone;
    }
    crate::StateUnion!(PinChoice: PinChoiceLeft | PinChoiceRight);

    struct PinnedRuntime {
        value: Cell<u32>,
        touched_through_mut: Cell<bool>,
        touched_through_pin: Cell<bool>,
        _pin: PhantomPinned,
    }

    impl Initial<PinReady> for PinnedMachine {}
    impl Initial<PinChoiceLeft> for PinnedMachine {}
    impl Transition<PinReady, PinRunning> for PinnedMachine {
        type F = fn(u32);
    }
    impl Transition<PinRunning, PinDone> for PinnedMachine {}
    impl Transition<PinChoiceLeft, PinChoiceDone> for PinnedMachine {}
    impl Transition<PinChoiceRight, PinChoiceDone> for PinnedMachine {}

    crate::StateMachineImpl! {
        PinnedRuntime: PinnedMachine;

        transition PinReady => PinRunning(value: u32) {
            self.value.set(value + 1);
            self.touched_through_mut.set(true);
        }

        pinned transition PinReady => PinRunning(value: u32) {
            self.as_ref().set_value(value);
        }

        pinned transition PinRunning => PinDone() {
            self.as_mut().mark_done();
        }

        pinned transition PinChoiceLeft | PinChoiceRight => PinChoiceDone() {
            self.as_mut().mark_done();
        }
    }

    impl PinnedRuntime {
        fn new() -> State<SOwned, Self, PinReady> {
            State::<SOwned, Self, PinReady>::new(Self {
                value: Cell::new(0),
                touched_through_mut: Cell::new(false),
                touched_through_pin: Cell::new(false),
                _pin: PhantomPinned,
            })
        }

        fn new_choice() -> State<SOwned, Self, PinChoiceLeft> {
            State::<SOwned, Self, PinChoiceLeft>::new(Self {
                value: Cell::new(0),
                touched_through_mut: Cell::new(false),
                touched_through_pin: Cell::new(false),
                _pin: PhantomPinned,
            })
        }

        fn set_value(self: Pin<&Self>, value: u32) {
            self.get_ref().value.set(value);
            self.get_ref().touched_through_pin.set(true);
        }

        fn mark_done(self: Pin<&mut Self>) {
            self.as_ref().get_ref().value.set(99);
            self.as_ref().get_ref().touched_through_pin.set(true);
        }

        fn start<S>(self: State<S, Self, PinReady>, value: u32) -> State<S, Self, PinRunning>
        where
            S: SPinMut,
        {
            crate::transition!(pin self, value)
        }

        fn start_unpinned<S>(
            self: State<S, Self, PinReady>,
            value: u32,
        ) -> State<S, Self, PinRunning>
        where
            S: SMut,
        {
            crate::transition!(self, value)
        }

        fn finish<S>(self: State<S, Self, PinRunning>) -> State<S, Self, PinDone>
        where
            S: SPinMut,
        {
            crate::transition!(pin self)
        }

        fn finish_const<S, Current>(self: State<S, Self, Current>) -> State<S, Self, PinChoiceDone>
        where
            S: SPinMut,
            Current: InPinChoice,
        {
            crate::transition!(pin const PinChoice self)
        }

        fn finish_dyn<S, Current>(self: State<S, Self, Current>) -> State<S, Self, PinChoiceDone>
        where
            S: SPinMut,
            Current: InPinChoice,
        {
            crate::transition!(pin dyn PinChoice self)
        }

        fn pinned_value<S>(self: &State<S, Self, PinRunning>) -> u32
        where
            S: SPinRef,
        {
            crate::pin_ref(self).get_ref().value.get()
        }
    }

    #[test]
    fn pin_box_supports_pinned_transitions_for_not_unpin_runtime() {
        let ready = PinnedRuntime::new();
        let ready: SPinBox<PinnedRuntime, PinReady> = SPinBox::new(SBox::new(ready));
        let running = ready.start(41);

        assert_eq!(running.pinned_value(), 41);
        assert!(running.touched_through_pin.get());
        assert!(!running.touched_through_mut.get());

        let done = running.finish();

        assert_eq!(done.value.get(), 99);
        assert!(done.touched_through_pin.get());
    }

    #[test]
    fn same_edge_can_have_normal_and_pinned_effects() {
        let running = PinnedRuntime::new().start_unpinned(40);

        assert_eq!(running.value.get(), 41);
        assert!(running.touched_through_mut.get());
        assert!(!running.touched_through_pin.get());

        let ready = PinnedRuntime::new();
        let ready: SPinBox<PinnedRuntime, PinReady> = SPinBox::new(SBox::new(ready));
        let running = ready.start(40);

        assert_eq!(running.pinned_value(), 40);
        assert!(running.touched_through_pin.get());
        assert!(!running.touched_through_mut.get());
    }

    #[test]
    fn pin_const_and_pin_dyn_support_union_transitions() {
        let left = PinnedRuntime::new_choice();
        let left: SPinBox<PinnedRuntime, PinChoiceLeft> = SPinBox::new(SBox::new(left));
        let done = left.finish_const();

        assert_eq!(done.value.get(), 99);
        assert!(done.touched_through_pin.get());

        let left = PinnedRuntime::new_choice();
        let left: SPinBox<PinnedRuntime, PinChoiceLeft> = SPinBox::new(SBox::new(left));
        let done = left.finish_dyn();

        assert_eq!(done.value.get(), 99);
        assert!(done.touched_through_pin.get());
    }
}

#[test]
#[cfg(feature = "decompose")]
fn matching_decomposed_parts_recompose() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let (state, data) = ready.decompose();
    let recomposed = StateOwned::recompose(state, data);

    assert!(recomposed.is_ok());
}

#[cfg(feature = "decompose")]
fn assert_decompose_backend_round_trip() {
    let ready: StateOwned<_, Ready> = StateOwned::new(SharedRuntime { value: 42 });
    let (state, data) = ready.decompose();

    assert_eq!(state.uid, data.uid);

    let ready = StateOwned::recompose(state, data).expect("matching provenance");

    assert_eq!(ready.value.value, 42);
}

#[test]
#[cfg(all(
    feature = "decompose",
    feature = "decompose-rand",
    not(feature = "nightly-random")
))]
fn decompose_rand_backend_round_trips_state_and_data() {
    assert_decompose_backend_round_trip();
}

#[test]
#[cfg(all(feature = "decompose", feature = "nightly-random"))]
fn decompose_nightly_random_backend_round_trips_state_and_data() {
    assert_decompose_backend_round_trip();
}

#[test]
#[cfg(feature = "decompose")]
fn recomposed_state_can_continue_transitioning() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let (state, data) = ready.decompose();
    let ready = StateOwned::recompose(state, data).expect("matching provenance");
    let _running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());
}

#[test]
#[cfg(feature = "decompose")]
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
    let running: StateOwned<Box<Runtime>, Running> = transition(ready, TransitionToken).call(());

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<Box<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn pinned_box_uses_the_same_contract() {
    let ready: StateOwned<Pin<Box<Runtime>>, Ready> = StateOwned::new(Box::pin(Runtime));
    let running: StateOwned<Pin<Box<Runtime>>, Running> =
        transition(ready, TransitionToken).call(());

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<Pin<Box<Runtime>>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(feature = "unique-rc-arc")]
fn unique_rc_uses_the_same_contract() {
    let ready: StateOwned<UniqueRc<Runtime>, Ready> = StateOwned::new(UniqueRc::new(Runtime));
    let running: StateOwned<UniqueRc<Runtime>, Running> =
        transition(ready, TransitionToken).call(());

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<UniqueRc<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(feature = "unique-rc-arc")]
fn unique_arc_uses_the_same_contract() {
    let ready: StateOwned<UniqueArc<Runtime>, Ready> = StateOwned::new(UniqueArc::new(Runtime));
    let running: StateOwned<UniqueArc<Runtime>, Running> =
        transition(ready, TransitionToken).call(());

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

    let _: StateOwned<Runtime, Running> = transition(first, TransitionToken).call(());
    let _: StateOwned<Runtime, Running> = transition(second, TransitionToken).call(());
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
fn states_macro_forwards_attributes_to_generated_state_structs() {
    use documented_state_markers::{DocAttributeState, DocCommentState};

    fn assert_concrete<T>()
    where
        T: crate::StateMarker<Kind = crate::ConcreteStateKind>,
    {
    }

    assert_concrete::<DocCommentState>();
    assert_concrete::<DocAttributeState>();
    assert_eq!(
        format!("{:?}", DocCommentState::default()),
        "DocCommentState"
    );
    assert_eq!(
        format!("{:?}", DocAttributeState::default()),
        "DocAttributeState"
    );
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
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken).call(());
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
fn explicit_shared_aliases_match_builtin_aliases() {
    let rc: SRc<RefCellStorage, SharedRuntime> = SRc::new::<Ready>(SharedRuntime { value: 11 });
    assert_eq!(rc.borrow::<Ready>().expect("rc ready").value, 11);

    let mutex: SArc<MutexStorage, SharedRuntime> = SArc::new::<Ready>(SharedRuntime { value: 12 });
    assert_eq!(mutex.borrow::<Ready>().expect("mutex ready").value, 12);

    let rwlock: SArc<RwLockStorage, SharedRuntime> =
        SArc::new::<Ready>(SharedRuntime { value: 13 });
    assert_eq!(rwlock.borrow::<Ready>().expect("rwlock ready").value, 13);
}

#[test]
fn mutable_guard_storage_aliases_are_the_actual_backend_types() {
    fn assert_refcell_guard(_: State<SRefCell<'_>, SharedRuntime, Ready>) {}
    fn assert_mutex_guard(_: State<SMutex<'_>, SharedRuntime, Ready>) {}
    fn assert_rwlock_guard(_: State<SRwLock<'_>, SharedRuntime, Ready>) {}

    let refcell = SRcRefCell::new::<Ready>(SharedRuntime { value: 1 });
    assert_refcell_guard(refcell.borrow_mut::<Ready>().expect("refcell guard"));

    let mutex = SArcMutex::new::<Ready>(SharedRuntime { value: 2 });
    assert_mutex_guard(mutex.borrow_mut::<Ready>().expect("mutex guard"));

    let rwlock = SArcRwLock::new::<Ready>(SharedRuntime { value: 3 });
    assert_rwlock_guard(rwlock.borrow_mut::<Ready>().expect("rwlock guard"));
}

#[test]
fn immutable_borrow_returns_read_only_state_view() {
    fn assert_refcell_read_view(_: &SRefView<'_, RefCellStorage, SharedRuntime, Ready>) {}
    fn assert_concrete_storage(
        _: &State<StorageStateRef<'_, RefCellStorage>, SharedRuntime, Ready>,
    ) {
    }
    fn read_with_sref<S>(state: &State<S, SharedRuntime, Ready>) -> u32
    where
        S: crate::SRef,
    {
        state.value
    }

    let shared = SRcRefCell::new::<Ready>(SharedRuntime { value: 31 });
    let ready = shared.borrow::<Ready>().expect("ready state");

    assert_refcell_read_view(&ready);
    assert_concrete_storage(&ready);
    assert_eq!(read_with_sref(&ready), 31);
}

#[test]
fn discriminated_direct_refcell_views_have_no_layout_overhead() {
    type DirectMut<'a> = State<SRefCell<'a>, SharedRuntime, Ready>;
    type DiscriminatedMut<'a> =
        State<SDiscriminated<SRefCell<'a>>, SharedRuntime, StateUnionState<Active>>;
    type DirectRef<'a> = State<StorageStateRef<'a, RefCellStorage>, SharedRuntime, Ready>;
    type DiscriminatedRef<'a> = State<
        SDiscriminated<StorageStateRef<'a, RefCellStorage>>,
        SharedRuntime,
        StateUnionState<Active>,
    >;

    assert_eq!(
        size_of::<DirectMut<'static>>(),
        size_of::<DiscriminatedMut<'static>>()
    );
    assert_eq!(
        size_of::<DirectRef<'static>>(),
        size_of::<DiscriminatedRef<'static>>()
    );
}

#[test]
fn rc_state_weak_handle_upgrades_until_last_strong_handle_drops() {
    let state = SRcRefCell::new::<Ready>(SharedRuntime { value: 7 });
    let weak = state.downgrade();

    let upgraded = weak.upgrade().expect("strong handle is still alive");
    assert_eq!(upgraded.borrow::<Ready>().expect("ready state").value, 7);

    drop(state);
    assert!(weak.upgrade().is_some());

    drop(upgraded);
    assert!(weak.upgrade().is_none());
}

#[test]
fn weak_aliases_upgrade_to_their_matching_strong_aliases() {
    let rc = SRcRefCell::new::<Ready>(SharedRuntime { value: 21 });
    let weak: WeakSRcRefCell<SharedRuntime> = rc.downgrade();
    assert_eq!(
        weak.upgrade()
            .expect("rc alive")
            .borrow::<Ready>()
            .expect("ready")
            .value,
        21
    );

    let mutex = SArcMutex::new::<Ready>(SharedRuntime { value: 22 });
    let weak: WeakSArcMutex<SharedRuntime> = mutex.downgrade();
    assert_eq!(
        weak.upgrade()
            .expect("mutex alive")
            .borrow::<Ready>()
            .expect("ready")
            .value,
        22
    );

    let rwlock = SArcRwLock::new::<Ready>(SharedRuntime { value: 23 });
    let weak: WeakSArcRwLock<SharedRuntime> = rwlock.downgrade();
    assert_eq!(
        weak.upgrade()
            .expect("rwlock alive")
            .borrow::<Ready>()
            .expect("ready")
            .value,
        23
    );
}

#[test]
fn rc_state_borrows_committed_state_through_erased_union() {
    let state = SRcRefCell::new::<Ready>(SharedRuntime { value: 10 });
    let alias = state.clone();

    let guard = state.borrow_mut::<Ready>().expect("initial state");
    let guard = transition_state::<_, _, _, Running>(guard, TransitionToken).call(());
    drop(guard);

    {
        let erased = alias.borrow::<Active>().expect("running is active");
        assert_eq!(erased.value, 10);
    }

    {
        let erased = alias.borrow_mut::<Active>().expect("running is active");
        drop(erased);
    }

    assert_eq!(alias.borrow::<Running>().expect("still concrete").value, 10);
}

#[test]
fn arc_state_guard_commits_transition_on_drop() {
    let state = SArcMutex::new::<Ready>(SharedRuntime { value: 4 });
    let alias = state.clone();

    let guard = state.borrow_mut::<Ready>().expect("initial state");
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken).call(());
    guard.value = 5;
    drop(guard);

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::WrongState(_))
    ));
    assert_eq!(alias.borrow::<Running>().expect("committed state").value, 5);
}

#[test]
fn rwlock_state_allows_shared_reads_and_commits_write_guard_on_drop() {
    let state = SArcRwLock::new::<Ready>(SharedRuntime { value: 14 });
    let alias = state.clone();

    let first_read = state.borrow::<Ready>().expect("first read");
    let second_read = alias.borrow::<Ready>().expect("second read");
    assert_eq!(first_read.value, 14);
    assert_eq!(second_read.value, 14);

    match state.borrow_mut::<Ready>() {
        Err(SharedStateError::Storage(TryLockError::WouldBlock)) => {}
        Err(SharedStateError::Storage(TryLockError::Poisoned(_))) => {
            panic!("rwlock should not be poisoned")
        }
        Err(SharedStateError::WrongState(_)) => panic!("state should still be ready"),
        Ok(_) => panic!("write borrow should not succeed while read guards are alive"),
    }

    drop(first_read);
    drop(second_read);

    let guard = state.borrow_mut::<Ready>().expect("write guard");
    let mut guard = transition_state::<_, _, _, Running>(guard, TransitionToken).call(());
    guard.value = 15;
    drop(guard);

    assert!(matches!(
        alias.borrow::<Ready>(),
        Err(SharedStateError::WrongState(_))
    ));
    assert_eq!(
        alias.borrow::<Running>().expect("committed state").value,
        15
    );
}

#[test]
fn arc_state_weak_handle_upgrades_until_last_strong_handle_drops() {
    let state = SArcMutex::new::<Ready>(SharedRuntime { value: 8 });
    let weak = state.downgrade();

    let upgraded = weak.upgrade().expect("strong handle is still alive");
    assert_eq!(upgraded.borrow::<Ready>().expect("ready state").value, 8);

    drop(state);
    assert!(weak.upgrade().is_some());

    drop(upgraded);
    assert!(weak.upgrade().is_none());
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
    let running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());
    let entry = &running.trace()[0];

    assert!(entry.from().type_name().ends_with("::Ready"));
    assert!(entry.to().type_name().ends_with("::Running"));
    assert_eq!(entry.from().type_id(), TypeId::of::<Ready>());
    assert_eq!(entry.to().type_id(), TypeId::of::<Running>());
    assert_eq!(entry.callsite().file(), file!());
    assert_eq!(entry.callsite().line(), expected_line);
}

#[test]
#[cfg(all(feature = "tracing", not(feature = "dynZST")))]
fn tracing_static_erased_markers_preserve_state_identity() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());
    let entry = running.trace()[0].clone();

    assert_eq!(entry.from().type_id(), TypeId::of::<Ready>());
    assert_eq!(entry.to().type_id(), TypeId::of::<Running>());
    assert!(format!("{entry:?}").contains("Ready"));
}

#[test]
#[cfg(all(feature = "tracing", feature = "dynZST"))]
fn tracing_dynzst_erased_markers_preserve_state_identity() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());
    let entry = running.trace()[0].clone();

    assert_eq!(entry.from().type_id(), TypeId::of::<Ready>());
    assert_eq!(entry.to().type_id(), TypeId::of::<Running>());
    assert!(format!("{entry:?}").contains("Running"));
}

#[test]
#[cfg(all(feature = "tracing", feature = "decompose"))]
fn decomposition_preserves_trace() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());
    let (state, data) = running.decompose();
    let running = StateOwned::recompose(state, data).expect("matching provenance");

    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(feature = "tracing")]
fn cloning_state_clones_erased_markers() {
    let ready: StateOwned<_, Ready> = StateOwned::new(Runtime);
    let running: StateOwned<_, Running> = transition(ready, TransitionToken).call(());
    let cloned = running.clone();

    assert!(cloned.trace()[0].from().type_name().ends_with("::Ready"));
    assert!(cloned.trace()[0].to().type_name().ends_with("::Running"));
    assert_eq!(cloned.trace()[0].from().type_id(), TypeId::of::<Ready>());
    assert_eq!(cloned.trace()[0].to().type_id(), TypeId::of::<Running>());
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
    impl Transition<Connected, Authenticated> for Machine {
        type F = fn(u32);
    }
    impl Transition<Connected, Ready> for Machine {}
    impl Transition<Authenticated, Ready> for Machine {}
    impl Transition<Authenticated, Connected> for Machine {}
    impl Transition<Connected, Stopped> for Machine {}
    impl Transition<Authenticated, Stopped> for Machine {}

    crate::StateUnion!(Online: Connected | Authenticated);

    crate::StateMachineImpl! {
        Runtime: Machine;

        transition Ready => Connected();

        transition Connected => Authenticated(amount: u32) {
            self.value += amount;
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
        let connected = crate::transition!(ready,);
        let authenticated: State<SOwned, _, Authenticated> = crate::transition!(connected, 1,);

        assert_eq!(authenticated.value, 1);
    }

    #[test]
    fn comma_terminated_transition_shares_next_body() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = crate::transition!(ready);
        let ready: State<SOwned, _, Ready> = crate::transition!(connected);

        assert_eq!(ready.value, 10);

        let connected = crate::transition!(ready);
        let authenticated: State<SOwned, _, Authenticated> = crate::transition!(connected, 1);
        let ready: State<SOwned, _, Ready> = crate::transition!(authenticated);

        assert_eq!(ready.value, 21);
    }

    #[test]
    fn erased_union_transition_runs_concrete_body_with_normal_transition() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = crate::transition!(ready);
        let authenticated: State<SOwned, _, Authenticated> = crate::transition!(connected, 1);
        let online = <Authenticated as crate::In<Online>>::into_discriminated(authenticated);
        let ready: State<SOwned, _, Ready> =
            crate::undiscriminate_state(crate::transition!(dyn Online, online,));

        assert_eq!(ready.value, 11);
    }

    #[test]
    fn union_proof_transition_infers_proof_from_receiver_and_target() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = crate::transition!(ready);
        let authenticated: State<SOwned, _, Authenticated> = crate::transition!(connected, 1);
        let ready: State<SOwned, _, Ready> = crate::transition!(const Online authenticated,);

        assert_eq!(ready.value, 11);
    }

    #[test]
    fn dynamic_transition_ident_form_accepts_trailing_comma() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = crate::transition!(ready);
        let online = <Connected as crate::In<Online>>::into_discriminated(connected);
        let stopped: State<SOwned, _, Stopped> =
            crate::undiscriminate_state(crate::transition!(dyn Online online,));

        assert_eq!(stopped.value, 2);
    }

    #[test]
    fn discriminated_union_transition_runs_exact_body_when_bodies_differ() {
        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = crate::transition!(ready);
        let stopped: State<SOwned, _, Stopped> = crate::undiscriminate_state(crate::transition!(
            dyn Online,
            <Connected as crate::In<Online>>::into_discriminated(connected),
        ));

        assert_eq!(stopped.value, 2);

        let ready = State::<SOwned, _, Ready>::new(Runtime { value: 0 });
        let connected = crate::transition!(ready);
        let authenticated: State<SOwned, _, Authenticated> = crate::transition!(connected, 1);
        let stopped: State<SOwned, _, Stopped> = crate::undiscriminate_state(crate::transition!(
            dyn Online,
            <Authenticated as crate::In<Online>>::into_discriminated(authenticated),
        ));

        assert_eq!(stopped.value, 21);
    }
}
