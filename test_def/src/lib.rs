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
impl Transition<Authenticated, Disconnected> for ConnectionStandin {}

StateUnion!(AllMarker: Disconnected | Connected | Authenticated);
StateUnion!(Online: AllMarker, Connected | Authenticated);

// // Trait unions can inherit one or more previously defined union traits.
// StateUnion!(OnlineMarker: AllMarker, Connected | Authenticated);
// StateUnion!(DisconnectedMarker: AllMarker, Disconnected);
// StateUnion!(
//     All2Marker: AllMarker + OnlineMarker,
//     Connected | Authenticated
// );

// // The enum-only form remains independent of marker traits.
// StateUnion!(enum OnlineValue: Connected | Authenticated);
