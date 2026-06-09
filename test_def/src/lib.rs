#![feature(negative_impls)]
#![forbid(unsafe_code)]

use magicstatemachines::{StateClone, StateCopy, StateMachineDefinition};

/// ZST identifying this state-machine contract.
pub struct ConnectionStandin;

/// States owned by the definition crate.
pub mod states {
    use magicstatemachines::States;
    
    States! {
        Disconnected;
        Connected;
        Authenticated;
    }
}

use states::{Authenticated, Connected, Disconnected};

// Connected values may be cloned but not implicitly copied. Authenticated
// values are linear and cannot be cloned or copied.
impl !StateCopy for Connected {}
impl !StateClone for Authenticated {}

// This is the complete set of legal transitions. An implementation crate
// cannot add more because both the stand-in and states are foreign to it.
StateMachineDefinition! {
    for ConnectionStandin;

    Initial: Disconnected;

    transition Disconnected => Connected();
    transition Connected => Authenticated(user: String);
    transition Connected => Disconnected();
    transition Authenticated => Connected | Disconnected();

    union DisconnectedMarker: AllMarker, Disconnected;
    union AllMarker: Disconnected | Connected | Authenticated;
    union Online: AllMarker, Connected | Authenticated;
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

//impl<T, U> UnionTransition<Online, U> for T where T: Transition<Connected, U> +   Transition<Authenticated, U> {...}
