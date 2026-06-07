use statemachines::{SMut, SOwned, SRef, SResult, State};
use test_def::{
    ConnectionStandin, InOnline, OnlineEnum, OnlineIntoEnum,
    states::{Authenticated, Connected, Disconnected},
};

/// Runtime data and all behavior live in the implementation crate.
#[derive(Debug)]
pub(crate) struct Connection {
    endpoint: String,
    user: Option<String>,
}

statemachines::StateMachineImpl! {
    Connection: ConnectionStandin;

    transition Disconnected => Connected();

    transition Connected => Authenticated(user: String) {
        self.user = Some(user);
    }

    transition Connected => Disconnected() {
        self.user = None;
    }

    transition Authenticated => Disconnected() {
        self.user = None;
    }

    transition Authenticated => Connected() {
        self.user = None;
    }
}

impl Connection {
    pub(crate) fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            user: None,
        }
    }

    #[must_use]
    pub(crate) fn disconnected(endpoint: impl Into<String>) -> State<SOwned, Self, Disconnected> {
        State::new(Self::new(endpoint))
    }

    #[must_use]
    pub(crate) fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        self.transition()()
    }

    pub(crate) fn try_connect<S>(
        self: State<S, Self, Disconnected>,
        available: bool,
    ) -> SResult<S, Self, Connected, Disconnected>
    where
        S: SMut,
    {
        if available {
            Ok(self.connect())
        } else {
            Err(self)
        }
    }

    #[must_use]
    pub(crate) fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        self.transition()(user.into())
    }

    #[must_use]
    pub(crate) fn authenticate_if<S>(
        self: State<S, Self, Connected>,
        user: Option<String>,
    ) -> OnlineEnum<S, Self>
    where
        S: SMut,
    {
        match user {
            Some(user) => self.authenticate(user).into(),
            None => self.into(),
        }
    }

    #[must_use]
    pub(crate) fn as_online_enum<S>(
        self: State<S, Self, impl OnlineIntoEnum>,
    ) -> OnlineEnum<S, Self>
    where
        S: SRef,
    {
        <_ as OnlineIntoEnum>::into_enum(self)
    }

    #[must_use]
    pub(crate) fn disconnect_online<S, Current>(
        self: State<S, Self, Current>,
    ) -> State<S, Self, Disconnected>
    where
        S: SMut,
        Current: InOnline,
        ConnectionStandin: statemachines::Transition<Current, Disconnected, F = fn()>,
    {
        <_ as InOnline>::into_erased(self).transition()()
    }

    #[must_use]
    pub(crate) fn disconnect<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        <_ as InOnline>::into_erased(self).transition()()
    }

    #[must_use]
    pub(crate) fn logout<S>(self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        self.transition()()
    }

    pub(crate) fn endpoint(self: &State<impl SRef, Self, impl InOnline>) -> &str {
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
