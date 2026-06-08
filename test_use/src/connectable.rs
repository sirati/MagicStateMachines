use statemachines::{DiscriminatedState, SMut, SOwned, SRef, State, StateMachineImpl};
use test_def::{
    ConnectionStandin, InOnline, Online,
    states::{Authenticated, Connected, Disconnected},
};

/// Trait-shaped API over the connection contract.
///
/// Transition methods are intentionally required methods. The trait can name
/// the checked surface, but only the concrete implementation module owns the
/// private transition helpers and can perform transitions.
pub(crate) trait Connectable:
    StateMachineImpl<Standin = ConnectionStandin, Impl = Self> + Sized
{
    #[must_use]
    fn new(endpoint: impl Into<String>) -> State<SOwned, Self, Disconnected>;

    #[must_use]
    fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
    where
        S: SMut;

    #[must_use]
    fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut;

    #[must_use]
    fn disconnect<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
    where
        S: SMut;

    #[must_use]
    fn logout<S>(self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
    where
        S: SMut;

    #[must_use]
    fn authenticate_if<S>(
        self: State<S, Self, Connected>,
        user: Option<String>,
    ) -> DiscriminatedState<S, Self, Online>
    where
        S: SMut,
    {
        match user {
            Some(user) => <Authenticated as InOnline>::into_enum(self.authenticate(user)),
            None => <Connected as InOnline>::into_enum(self),
        }
    }

    #[must_use]
    fn as_online_enum<S>(self: State<S, Self, impl InOnline>) -> DiscriminatedState<S, Self, Online>
    where
        S: SRef,
    {
        <_ as InOnline>::into_enum(self)
    }

    fn endpoint(self: &State<impl SRef, Self, impl InOnline>) -> &str;

    fn raw_endpoint(&self) -> &str;

    fn user(self: &State<impl SRef, Self, Authenticated>) -> &str;
}

#[derive(Debug)]
pub(crate) struct ConnectionViaTrait {
    endpoint: String,
    user: Option<String>,
}

statemachines::StateMachineImpl! {
    ConnectionViaTrait: ConnectionStandin;

    transition Disconnected => Connected();

    transition Connected => Authenticated(user: String) {
        self.user = Some(user);
    }

    transition Connected | Authenticated => Disconnected(),
    transition Authenticated => Connected() {
        self.user = None;
    }
}

impl Connectable for ConnectionViaTrait {
    fn new(endpoint: impl Into<String>) -> State<SOwned, Self, Disconnected> {
        State::<SOwned, Self, Disconnected>::new(Self {
            endpoint: endpoint.into(),
            user: None,
        })
    }

    fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        self.transition()()
    }

    fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        self.transition()(user.into())
    }

    fn disconnect<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        statemachines::undiscriminate_state(<_ as InOnline>::into_enum(self)).transition()()
    }

    fn logout<S>(self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        self.transition()()
    }

    fn endpoint(self: &State<impl SRef, Self, impl InOnline>) -> &str {
        &self.endpoint
    }

    fn raw_endpoint(&self) -> &str {
        &self.endpoint
    }

    fn user(self: &State<impl SRef, Self, Authenticated>) -> &str {
        self.user
            .as_deref()
            .expect("authenticated state always has a user")
    }
}

pub(crate) fn run() {
    let authenticated = ConnectionViaTrait::new("localhost:8087")
        .connect()
        .authenticate("carol");

    println!(
        "{} is trait-authenticated as {}",
        authenticated.endpoint(),
        authenticated.user()
    );

    let online = authenticated.logout().as_online_enum();
    println!("{} is trait-online", online.endpoint());

    let disconnected = online.disconnect();
    println!("{} is trait-disconnected", disconnected.raw_endpoint());

    let online = ConnectionViaTrait::new("localhost:8088")
        .connect()
        .authenticate_if(None);
    println!("{} is trait-online through a branch", online.endpoint());
}
