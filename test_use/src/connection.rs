use statemachines::{State, StateStorageDeref, StateStorageDerefMut, StorageStateOwned};
use test_def::{
    ConnectionStandin, Online,
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
    pub(crate) fn disconnected(
        endpoint: impl Into<String>,
    ) -> State<StorageStateOwned, Self, Disconnected> {
        State::new(Self::new(endpoint))
    }

    #[must_use]
    pub(crate) fn connect<S>(
        self: State<S, Self, Disconnected>,
    ) -> State<S, Self, Connected>
    where
        S: StateStorageDeref<Self>,
    {
        self.transition()()
    }

    #[must_use]
    pub(crate) fn authenticate<S>(
        mut self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: StateStorageDerefMut<Self>,
    {
        let user = user.into();
        self.user = Some(user.clone());
        self.transition()(user)
    }

    #[must_use]
    pub(crate) fn disconnect<S>(
        mut self: State<S, Self, Connected>,
    ) -> State<S, Self, Disconnected>
    where
        S: StateStorageDerefMut<Self>,
    {
        self.user = None;
        self.transition()()
    }

    #[must_use]
    pub(crate) fn logout<S>(
        mut self: State<S, Self, Authenticated>,
    ) -> State<S, Self, Connected>
    where
        S: StateStorageDerefMut<Self>,
    {
        self.user = None;
        self.transition()()
    }

    pub(crate) fn endpoint<S>(self: &State<S, Self, impl Online>) -> &str
    where
        S: StateStorageDeref<Self>,
    {
        &self.endpoint
    }

    pub(crate) fn raw_endpoint(&self) -> &str {
        &self.endpoint
    }

    pub(crate) fn user<S>(self: &State<S, Self, Authenticated>) -> &str
    where
        S: StateStorageDeref<Self>,
    {
        self.user
            .as_deref()
            .expect("authenticated state always has a user")
    }
}
