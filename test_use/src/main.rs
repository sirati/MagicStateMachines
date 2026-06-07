#![feature(arbitrary_self_types)]
#![forbid(unsafe_code)]

use statemachines::{State, StateMachineImpl};
use test_def::{
    ConnectionStandin, Online,
    states::{Authenticated, Connected, Disconnected},
};

/// Runtime data and all behavior live in the implementation crate.
#[derive(Debug)]
struct Connection {
    endpoint: String,
    user: Option<String>,
}

impl StateMachineImpl for Connection {
    type Standin = ConnectionStandin;
    type Impl = Self;
}

impl Connection {
    #[must_use]
    fn disconnected(endpoint: impl Into<String>) -> State<Self, Disconnected> {
        State::new(Self {
            endpoint: endpoint.into(),
            user: None,
        })
    }

    #[must_use]
    fn connect(self: State<Self, Disconnected>) -> State<Self, Connected> {
        self.transition()()
    }

    #[must_use]
    fn authenticate(
        mut self: State<Self, Connected>,
        user: impl Into<String>,
    ) -> State<Self, Authenticated> {
        let user = user.into();
        self.user = Some(user.clone());
        self.transition()(user)
    }

    #[must_use]
    fn disconnect(mut self: State<Self, Connected>) -> State<Self, Disconnected> {
        self.user = None;
        self.transition()()
    }

    #[must_use]
    fn logout(mut self: State<Self, Authenticated>) -> State<Self, Connected> {
        self.user = None;
        self.transition()()
    }

    fn endpoint<S>(self: &State<Self, S>) -> &str
    where
        S: Online,
    {
        &self.endpoint
    }

    fn user(self: &State<Self, Authenticated>) -> &str {
        self.user
            .as_deref()
            .expect("authenticated state always has a user")
    }

    #[must_use]
    fn connect_boxed(self: State<Box<Self>, Disconnected>) -> State<Box<Self>, Connected> {
        self.transition()()
    }
}

fn main() {
    let connection = Connection::disconnected("localhost:8080");
    let connection = connection.connect();
    let connection = connection.authenticate("alice");
    let (state, data) = connection.decompose();
    let connection = State::recompose(state, data).expect("tokens came from the same state");

    println!(
        "{} is authenticated as {}",
        connection.endpoint(),
        connection.user()
    );

    let connection = connection.logout();
    println!("{} is still online", connection.endpoint());

    let _connection = connection.disconnect();

    let boxed: State<Box<Connection>, Disconnected> = State::new(Box::new(Connection {
        endpoint: "localhost:9090".into(),
        user: None,
    }));
    let _boxed = boxed.connect_boxed();
}
