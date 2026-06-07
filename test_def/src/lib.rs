#![feature(negative_impls)]
#![forbid(unsafe_code)]

use statemachines::{Initial, StateClone, StateCopy, StateUnion, Transition};

/// ZST identifying this state-machine contract.
pub struct ConnectionStandin;

/// States owned by the definition crate.
pub mod states {
    pub struct Disconnected;
    pub struct Connected;
    pub struct Authenticated;
}

use states::{Authenticated, Connected, Disconnected};

// Connected values may be cloned but not implicitly copied. Authenticated
// values are linear and cannot be cloned or copied.
impl !StateCopy for Connected {}
impl !StateClone for Authenticated {}

// These are the complete set of legal transitions. An implementation crate
// cannot add more because both the stand-in and states are foreign to it.
impl Initial<Disconnected> for ConnectionStandin {}
impl Transition<Disconnected, Connected> for ConnectionStandin {}
impl Transition<Connected, Disconnected> for ConnectionStandin {}
impl Transition<Connected, Authenticated> for ConnectionStandin {
    type F = fn(String);
}
impl Transition<Authenticated, Connected> for ConnectionStandin {}

StateUnion!(Online: Connected + Authenticated);
