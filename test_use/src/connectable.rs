use magicstatemachines::{
    DiscriminatedState, In, SMut, SOwned, SRef, State, StateMachineImpl, transition,
};
use test_def::{
    ConnectionStandin, Online,
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
    fn disconnect<S>(self: State<S, Self, impl In<Online>>) -> State<S, Self, Disconnected>
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
            Some(user) => <Authenticated as In<Online>>::into_enum(self.authenticate(user)),
            None => <Connected as In<Online>>::into_enum(self),
        }
    }

    #[must_use]
    fn as_online_enum<S>(
        self: State<S, Self, impl In<Online>>,
    ) -> DiscriminatedState<S, Self, Online>
    where
        S: SRef,
    {
        <_>::into_enum(self)
    }

    fn endpoint(self: &State<impl SRef, Self, impl In<Online>>) -> &str;

    fn raw_endpoint(&self) -> &str;

    fn user(self: &State<impl SRef, Self, Authenticated>) -> &str;
}

#[derive(Debug)]
pub(crate) struct ConnectionViaTrait {
    endpoint: String,
    user: Option<String>,
}

magicstatemachines::StateMachineImpl! {
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
        transition!(self)
    }

    fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        transition!(self, user.into())
    }

    fn disconnect<S>(self: State<S, Self, impl In<Online>>) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        transition!(dyn Online self)
    }

    fn logout<S>(self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        transition!(self)
    }

    fn endpoint(self: &State<impl SRef, Self, impl In<Online>>) -> &str {
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
