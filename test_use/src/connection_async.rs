use magicstatemachines::{DiscriminatedState, In, SMut, SOwned, SRef, State, transition};
use test_def::{
    ConnectionStandin, InOnline, Online,
    states::{Authenticated, Connected, Disconnected},
};

use crate::connection::Connection;

/// A second implementation of the same definition-crate state-machine contract.
#[derive(Debug)]
pub(crate) struct ConnectionAsync {
    endpoint: String,
    user: Option<String>,
}

magicstatemachines::StateMachineImpl! {
    ConnectionAsync: ConnectionStandin;

    priv Initial: Authenticated;

    transition Disconnected => Connected();

    transition Connected => Authenticated(user: String) {
        self.user = Some(user);
    }

    transition Connected | Authenticated => Disconnected(),
    transition Authenticated => Connected() {
        self.user = None;
    }
}

impl ConnectionAsync {
    #[must_use]
    pub(crate) fn new(endpoint: impl Into<String>) -> State<SOwned, Self, Disconnected> {
        State::<SOwned, Self, Disconnected>::new(Self {
            endpoint: endpoint.into(),
            user: None,
        })
    }

    #[must_use]
    pub(crate) fn from_authenticated_connection(
        source: State<SOwned, Connection, Authenticated>,
    ) -> State<SOwned, Self, Authenticated> {
        let source = State::into_concrete(source).into_raw();
        let (endpoint, user) = source.into_parts();
        State::from_concrete(Self::with_state_priv::<Authenticated>(Self {
            endpoint,
            user,
        }))
    }

    pub(crate) async fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        transition!(self)
    }

    pub(crate) async fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        transition!(self, user.into())
    }

    pub(crate) async fn authenticate_if<S>(
        self: State<S, Self, Connected>,
        user: Option<String>,
    ) -> DiscriminatedState<S, Self, Online>
    where
        S: SMut,
    {
        match user {
            Some(user) => {
                <Authenticated as In<Online>>::into_discriminated(self.authenticate(user).await)
            }
            None => <Connected as In<Online>>::into_discriminated(self),
        }
    }

    #[must_use]
    pub(crate) fn as_online_enum<S>(
        self: State<S, Self, impl In<Online>>,
    ) -> DiscriminatedState<S, Self, Online>
    where
        S: SRef,
    {
        <_ as In<Online>>::into_discriminated(self)
    }

    pub(crate) async fn disconnect<S>(
        self: State<S, Self, impl InOnline>,
    ) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        transition!(const Online self)
    }

    pub(crate) async fn logout<S>(self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        transition!(self)
    }

    pub(crate) fn endpoint(self: &State<impl SRef, Self, impl In<Online>>) -> &str {
        &self.endpoint
    }

    pub(crate) fn raw_endpoint(&self) -> &str {
        &self.endpoint
    }

    pub(crate) fn user(self: &State<impl SRef, Self, Authenticated>) -> &str {
        self.user
            .as_deref()
            .expect("authenticated state always has a user")
    }
}
