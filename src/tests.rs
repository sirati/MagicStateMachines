use crate::{Initial, State, StateCopy, StateMachineImpl, Transition};
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

impl !StateCopy for Running {}

impl Initial<Ready> for Machine {}
impl Transition<Ready, Running> for Machine {}

impl StateMachineImpl for Runtime {
    type Standin = Machine;
    type Impl = Self;
}

#[test]
#[cfg(not(feature = "tracing"))]
fn state_marker_has_no_layout_cost() {
    assert_eq!(size_of::<State<[u8; 8], Ready>>(), size_of::<[u8; 8]>());
    assert_eq!(align_of::<State<u64, Ready>>(), align_of::<u64>());
}

#[test]
fn declared_transition_changes_only_the_type() {
    let ready: State<_, Ready> = State::new(Runtime);
    let running: State<_, Running> = ready.transition()();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), 0);
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn matching_decomposed_parts_recompose() {
    let ready: State<_, Ready> = State::new(Runtime);
    let (state, data) = ready.decompose();
    let recomposed = State::recompose(state, data);

    assert!(recomposed.is_ok());
}

#[test]
fn mismatched_decomposed_parts_do_not_recompose() {
    let first: State<_, Ready> = State::new(Runtime);
    let (first_state, _) = first.decompose();
    let second_data = loop {
        let second: State<_, Ready> = State::new(Runtime);
        let (_, data) = second.decompose();

        if first_state.uid != data.uid {
            break data;
        }
    };

    assert!(State::recompose(first_state, second_data).is_err());
}

#[test]
fn boxed_implementation_uses_the_same_contract() {
    let ready: State<Box<Runtime>, Ready> = State::new(Box::new(Runtime));
    let running: State<Box<Runtime>, Running> = ready.transition()();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<Box<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn pinned_box_uses_the_same_contract() {
    let ready: State<Pin<Box<Runtime>>, Ready> = State::new(Box::pin(Runtime));
    let running: State<Pin<Box<Runtime>>, Running> = ready.transition()();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<Pin<Box<Runtime>>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn mutable_reference_uses_the_same_contract() {
    let mut runtime = Runtime;
    let ready: State<&mut Runtime, Ready> = State::new(&mut runtime);
    let running: State<&mut Runtime, Running> = ready.transition()();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<&mut Runtime>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn unique_rc_uses_the_same_contract() {
    let ready: State<UniqueRc<Runtime>, Ready> = State::new(UniqueRc::new(Runtime));
    let running: State<UniqueRc<Runtime>, Running> = ready.transition()();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<UniqueRc<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
fn unique_arc_uses_the_same_contract() {
    let ready: State<UniqueArc<Runtime>, Ready> = State::new(UniqueArc::new(Runtime));
    let running: State<UniqueArc<Runtime>, Running> = ready.transition()();

    #[cfg(not(feature = "tracing"))]
    assert_eq!(size_of_val(&running), size_of::<UniqueArc<Runtime>>());
    #[cfg(feature = "tracing")]
    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(not(feature = "tracing"))]
fn copying_state_copies_the_runtime_value() {
    let first: State<Runtime, Ready> = State::new(Runtime);
    let second = first;

    let _: State<Runtime, Running> = first.transition()();
    let _: State<Runtime, Running> = second.transition()();
}

#[test]
fn clone_policy_can_allow_clone_without_copy() {
    let first = State::<Runtime, Running> {
        value: Runtime,
        state: PhantomData,
        #[cfg(feature = "tracing")]
        trace: Vec::new(),
    };
    let _second = first.clone();
}

#[test]
#[cfg(feature = "tracing")]
fn tracing_records_transition_and_callsite() {
    let ready: State<_, Ready> = State::new(Runtime);
    let expected_line = line!() + 1;
    let running: State<_, Running> = ready.transition()();
    let entry = &running.trace()[0];

    assert!(entry.from().type_name().ends_with("::Ready"));
    assert!(entry.to().type_name().ends_with("::Running"));
    assert_eq!(entry.callsite().file(), file!());
    assert_eq!(entry.callsite().line(), expected_line);
}

#[test]
#[cfg(feature = "tracing")]
fn decomposition_preserves_trace() {
    let ready: State<_, Ready> = State::new(Runtime);
    let running: State<_, Running> = ready.transition()();
    let (state, data) = running.decompose();
    let running = State::recompose(state, data).expect("matching provenance");

    assert_eq!(running.trace().len(), 1);
}

#[test]
#[cfg(feature = "tracing")]
fn cloning_state_clones_erased_markers() {
    let ready: State<_, Ready> = State::new(Runtime);
    let running: State<_, Running> = ready.transition()();
    let cloned = running.clone();

    assert!(cloned.trace()[0].from().type_name().ends_with("::Ready"));
    assert!(cloned.trace()[0].to().type_name().ends_with("::Running"));
}
