use statemachines::{SMut, SOwned, SRef, SResult, State};
use test_def::{
    ConnectionStandin, Online, OnlineEnum,
    states::{Authenticated, Connected, Disconnected},
};

/// Runtime data and all behavior live in the implementation crate.
#[derive(Debug)]
pub(crate) struct Connection {
    endpoint: String,
    user: Option<String>,
}

statemachines::StateMachineImpl!(Connection: ConnectionStandin);

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
        S: SRef,
    {
        self.transition()()
    }

    pub(crate) fn try_connect<S>(
        self: State<S, Self, Disconnected>,
        available: bool,
    ) -> SResult<S, Self, Connected, Disconnected>
    where
        S: SRef,
    {
        if available {
            Ok(self.connect())
        } else {
            Err(self)
        }
    }

    #[must_use]
    pub(crate) fn authenticate<S>(
        mut self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        let user = user.into();
        self.user = Some(user.clone());
        self.transition()(user)
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
    pub(crate) fn disconnect<S>(
        mut self: State<S, Self, impl Online>,
    ) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        self.user = None;
        <_ as Online>::into_erased(self).transition()()
    }

    #[must_use]
    pub(crate) fn logout<S>(mut self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        self.user = None;
        self.transition()()
    }

    pub(crate) fn endpoint(self: &State<impl SRef, Self, impl Online>) -> &str {
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
