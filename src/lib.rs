#![cfg_attr(
    all(feature = "decompose", feature = "nightly-random"),
    feature(random)
)]
#![cfg_attr(feature = "unique-rc-arc", feature(unique_rc_arc))]
#![cfg_attr(not(feature = "gen_no_unsafe"), feature(allow_internal_unsafe))]
#![feature(
    arbitrary_self_types,
    associated_type_defaults,
    auto_traits,
    negative_impls
)]
#![cfg_attr(not(feature = "gen_no_unsafe"), allow(internal_features))]
#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Ergonomic typestate wrappers for compiler-enforced state machines.
//!
//! This crate requires nightly Rust for `arbitrary_self_types`.
//!
//! MagicStateMachines lets a state-machine contract live separately from the
//! runtime type that implements it. A definition crate owns the stand-in type,
//! state marker ZSTs, initial states, legal transitions, and state unions. An
//! implementation crate connects a runtime type to that contract with
//! [`StateMachineImpl!`](macro@crate::StateMachineImpl) and exposes ordinary
//! inherent methods whose receiver type carries the current state.
//!
//! The core receiver type is [`State<Storage, T, S>`], where `T` is the runtime
//! implementation, `S` is the current state marker, and `Storage` selects how
//! the runtime value is held. Methods usually constrain storage by capability:
//! [`SRef`] for read-only access, [`SMut`] for mutable transitions, [`SPinMut`]
//! for pinned transitions, and [`SMove`] when storage must move by value. This
//! allows the same state-machine methods to work for owned values, boxes,
//! pinned boxes, shared guard views, and custom storage backends.
//!
//! In the default configuration this crate denies unsafe code. The `dynZST`
//! feature uses the external `dynzst` crate for thin erased ZST state markers.
//! Without tracing, directly owned state wrappers are layout-transparent over
//! the runtime data, and concrete-state transitions are statically dispatched.
//! When state crosses a boundary the compiler cannot prove, such as shared
//! storage behind `Rc`, `Arc`, `RefCell`, `Mutex`, or `RwLock`, the committed
//! erased state is checked at the boundary before returning a typed view.
//!
//! State unions support methods over a set of states. A union such as
//! `Online` generates a sealed membership trait such as `InOnline`, a
//! discriminated state representation, and an enum such as `OnlineEnum`.
//! Static union transitions use [`transition!`](macro@crate::transition) with
//! the `const` form when all members share the same transition body. Dynamic
//! union transitions use the `dyn` form when the concrete member must be
//! discriminated first.

mod contract;
#[cfg(feature = "decompose")]
mod decomposed;
mod kind;
mod macros;
mod policy;
mod proof;
mod shared;
mod state;
mod state_trait;
#[cfg(feature = "tracing")]
/// Transition tracing support.
///
/// This module is available with the `tracing` feature. It contains the
/// [`TraceEntry`] type stored by state wrappers when tracing is enabled.
pub mod tracing;
mod union;
mod util;

pub use contract::{Initial, StateMachineImpl, Transition, TransitionSignature};
#[cfg(feature = "decompose")]
pub use decomposed::{DecomposedData, DecomposedState, RecomposeError};
pub use kind::{
    ConcreteStateKind, RuntimeStateMarker, StateKind, StateMarker, StateRuntimeMarkerFor,
    UnionStateKind,
};
pub use policy::{StateClone, StateCopy};
#[doc(hidden)]
pub use proof::StateUnionTransitionProof;
#[doc(hidden)]
pub use proof::UnionTransitionProof;
pub use proof::{
    StateConcreteProvenState, StateConcreteTransitionProof, StateProofTransition,
    StateTransitionProofBind, StateUnionProvenState, StateWithProof, TransitionProof,
};
pub use shared::{
    MutexStorage, RefCellStorage, RwLockStorage, SArc, SArcMutex, SArcRwLock, SMutView, SMutex,
    SRc, SRcRefCell, SRefCell, SRefView, SRwLock, SharedBorrowState, SharedState, SharedStateError,
    SharedStorage, SharedValue, StateMut, StateMutTransitionCall, StateRef, StorageStateMut,
    StorageStateRef, WeakSArc, WeakSArcMutex, WeakSArcRwLock, WeakSRc, WeakSRcRefCell,
    WrongStateError, transition_mut,
};
pub use state::{
    ConcreteProofTransitionCall, ConcreteStated, DiscriminatedTransitionCall, EffectTransitionCall,
    InferenceKind, InnerInference, InnerStateInference, KindProofTransitionCall, MayTransition,
    OuterInference, PinnedDiscriminatedTransitionCall, PinnedEffectTransitionCall,
    PinnedStateUnionProofTransitionCall, PinnedTransitionEffect, PinnedTransitionEffectSelector,
    SBox, SMapRuntime, SMove, SMut, SOwned, SPin, SPinBox, SPinMut, SPinRef, SRef, SResult, State,
    StateInference, StateOwned, StateStorage, StateStorageNew, StateTransitionCall,
    StateUnionProofTransitionCall, StorageStateOwned, StorageStateOwnedBox,
    StorageStateOwnedPinBox, TransitionCall, TransitionCallsite, TransitionEffect,
    TransitionEffectSelector, complete_transition_after_effect, pin_mut, pin_ref, proven_state,
    proven_union_state, transition, transition_callsite, transition_concrete_after_effect,
    transition_concrete_after_pinned_effect, transition_discriminated_state,
    transition_discriminated_state_pinned, transition_state,
    transition_state_with_concrete_kind_proof, transition_state_with_concrete_proof,
    transition_state_with_concrete_transition_proof, transition_state_with_effect,
    transition_state_with_erased_transition_proof, transition_state_with_kind_proof,
    transition_state_with_pinned_effect, transition_state_with_static_union_pinned_proof,
    transition_state_with_static_union_proof, transition_state_with_union_proof,
    transition_state_with_union_transition_proof,
};
#[cfg(feature = "unique-rc-arc")]
pub use state::{StorageStateOwnedUniqueArc, StorageStateOwnedUniqueRc};
#[doc(hidden)]
pub use state_trait::ConcreteStateTrait;
pub use state_trait::StateTrait;
#[doc(hidden)]
pub use state_trait::{ErasedState, clone_erased as clone_erased_state};
#[cfg(feature = "tracing")]
pub use tracing::TraceEntry;
#[doc(hidden)]
pub use union::StateUnionConcreteState;
#[doc(hidden)]
pub use union::StateUnionDiscriminatedPinnedTransition;
#[doc(hidden)]
pub use union::StateUnionDiscriminatedTransition;
#[doc(hidden)]
pub use union::{
    DiscriminatedInner, SDiscriminated, StateUnionErased, StateUnionMember,
    StateUnionProofMembership, StateUnionProofTarget, StateUnionRuntime, StateUnionSharedEffect,
    StateUnionSharedPinnedEffect, StateUnionSharedPinnedTransitionEffect,
    StateUnionSharedTransitionEffect, StateUnionState, StateUnionTransition,
    concretize_discriminated_state, discriminate_state, discriminated_state_marker,
    erased_state_type_id, rediscriminate_union_state, state_union_marker, undiscriminate_state,
};
pub use union::{DiscriminatedState, In, StateUnionDiscriminant};
pub use util::EnumExt;

#[doc(hidden)]
pub mod __private {
    pub use crate::state::concrete_stated_new;
    pub use paste::paste;
}

#[cfg(test)]
mod tests;
