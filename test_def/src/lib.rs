#![feature(negative_impls)]
#![forbid(unsafe_code)]

use magicstatemachines::{StateClone, StateCopy, StateMachineDefinition};

/// States owned by the definition crate.
pub mod states {
    use magicstatemachines::States;

    States! {
        /// this is a doc string
        Disconnected;
        Connected;
        Authenticated;
        Authorised;
        Dispatched;
        Done;
        Failed;
        Invalid;
    }
}

use states::*;

// Connected values may be cloned but not implicitly copied. Authenticated
// values are linear and cannot be cloned or copied.
impl !StateCopy for Connected {}
impl !StateClone for Authenticated {}

/// ZST identifying this state-machine contract.
pub struct ConnectionStandin;

// This is the complete set of legal transitions. An implementation crate
// cannot add more because both the stand-in and states are foreign to it.
StateMachineDefinition! {
    for ConnectionStandin;

    pub Initial: Disconnected;

    transition Disconnected => Connected();
    transition Connected => Authenticated(user: String);
    transition Connected => Disconnected();
    transition Authenticated => Connected | Disconnected();

    union DisconnectedMarker: AllMarker, Disconnected;
    union AllMarker: Disconnected | Connected | Authenticated;
    union Online: AllMarker, Connected | Authenticated;
}

pub struct JobStandin;
StateMachineDefinition! {
    for JobStandin;

    transition Authenticated => Authorised | Failed();
    transition Authorised => Dispatched | Failed();
    transition Dispatched => Done | Failed();
    transition Done | Failed => Disconnected();
    union Healthy: Authenticated | Authorised | Dispatched | Done;
    union Terminal: Failed | Done;

}

// // Trait unions can inherit one or more previously defined union traits.
// StateUnion!(OnlineMarker: AllMarker, Connected | Authenticated);
// StateUnion!(DisconnectedMarker: AllMarker, Disconnected);
// StateUnion!(
//     All2Marker: AllMarker + OnlineMarker,
//     Connected | Authenticated
// );

// // The enum-only form remains independent of marker traits
// StateUnion!(enum OnlineValue: Connected | Authenticated);
