use core::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
use statemachines::{SMut, SOwned, SRef, State};
use test_def::{
    ConnectionStandin, InOnline, Online, OnlineEnum, OnlineIntoEnum,
    states::{Authenticated, Connected, Disconnected},
};

/// A second implementation of the same definition-crate state-machine contract.
#[derive(Debug)]
pub(crate) struct ConnectionAsync {
    endpoint: String,
    user: Option<String>,
}

fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => core::hint::spin_loop(),
        }
    }
}

statemachines::StateMachineImpl! {
    ConnectionAsync: ConnectionStandin;

    transition Disconnected => Connected();

    transition Connected => Authenticated(user: String) {
        self.user = Some(user);
    }

    transition Connected | Authenticated => Disconnected(),
    transition Authenticated => Connected() {
        self.user = None;
    }
}

pub(crate) fn run() {
    let authenticated = block_on(async {
        ConnectionAsync::new("localhost:8085")
            .connect()
            .await
            .authenticate("bob")
            .await
    });

    println!(
        "{} is asynchronously authenticated as {}",
        authenticated.endpoint(),
        authenticated.user()
    );

    let online = block_on(async { authenticated.logout().await.authenticate_if(None).await });
    println!("{} is asynchronously online", online.endpoint());

    let disconnected = block_on(online.into_erased().disconnect());
    println!(
        "{} is asynchronously disconnected",
        disconnected.raw_endpoint()
    );

    let online = block_on(ConnectionAsync::new("localhost:8086").connect()).as_online_enum();
    println!("{} is asynchronously online via enum", online.endpoint());
}

impl ConnectionAsync {
    #[must_use]
    pub(crate) fn new(endpoint: impl Into<String>) -> State<SOwned, Self, Disconnected> {
        State::<SOwned, Self, Disconnected>::new(Self {
            endpoint: endpoint.into(),
            user: None,
        })
    }

    pub(crate) async fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        self.transition()()
    }

    pub(crate) async fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        self.transition()(user.into())
    }

    pub(crate) async fn authenticate_if<S>(
        self: State<S, Self, Connected>,
        user: Option<String>,
    ) -> OnlineEnum<S, Self>
    where
        S: SMut,
    {
        match user {
            Some(user) => self.authenticate(user).await.into(),
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

    pub(crate) async fn disconnect<S>(
        self: State<S, Self, impl InOnline>,
    ) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        self.transition_erased::<Online, _>()()
    }

    pub(crate) async fn logout<S>(self: State<S, Self, Authenticated>) -> State<S, Self, Connected>
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
