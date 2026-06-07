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
mod state;
#[allow(unsafe_code)]
#[cfg(feature = "tracing")]
pub mod tracing;

pub use contract::{Initial, StateMachineImpl, Transition};
pub use decomposed::{DecomposedData, DecomposedState, RecomposeError};
pub use policy::{StateClone, StateCopy};
pub use state::{State, TransitionCall};
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

#[cfg(test)]
mod tests;
