use magicstatemachines::{DiscriminatedState, In, SMut, SOwned, SRef, SResult, State, transition};
use test_def::{
    Healthy, InHealthy, InTerminal, JobStandin, Terminal,
    states::{Authenticated, Authorised, Disconnected, Dispatched, Done, Failed},
};

use crate::connection::Connection;

#[derive(Debug)]
pub(crate) struct Job {
    endpoint: String,
    user: Option<String>,
    attempts: u32,
}

magicstatemachines::StateMachineImpl! {
    Job: JobStandin;

    priv Initial: Authenticated;

    transition Authenticated => Authorised();

    transition Authorised => Dispatched() {
        self.attempts += 1;
    }

    transition Dispatched => Done();
    transition Dispatched | Authenticated | Authorised => Failed();

    transition Done | Failed => Disconnected();
}

impl Job {
    #[must_use]
    pub(crate) fn from_authenticated_connection(
        source: State<SOwned, Connection, Authenticated>,
    ) -> State<SOwned, Self, Authenticated> {
        let source = State::into_concrete(source).into_raw();
        let (endpoint, user) = source.into_parts();
        State::from_concrete(Self::with_state_priv::<Authenticated>(Self {
            endpoint,
            user,
            attempts: 0,
        }))
    }

    #[must_use]
    pub(crate) fn authorise<S>(
        self: State<S, Self, Authenticated>,
        allowed: bool,
    ) -> SResult<S, Self, Authorised, Failed>
    where
        S: SMut,
    {
        if allowed {
            let authorised: State<S, Self, Authorised> = transition!(self);
            Ok(authorised)
        } else {
            let failed: State<S, Self, Failed> = transition!(self);
            Err(failed)
        }
    }

    #[must_use]
    pub(crate) fn dispatch<S>(
        self: State<S, Self, Authorised>,
        available: bool,
    ) -> SResult<S, Self, Dispatched, Failed>
    where
        S: SMut,
    {
        if available {
            let dispatched: State<S, Self, Dispatched> = transition!(self);
            Ok(dispatched)
        } else {
            let failed: State<S, Self, Failed> = transition!(self);
            Err(failed)
        }
    }

    #[must_use]
    pub(crate) fn complete<S>(
        self: State<S, Self, Dispatched>,
        succeeded: bool,
    ) -> SResult<S, Self, Done, Failed>
    where
        S: SMut,
    {
        if succeeded {
            let done: State<S, Self, Done> = transition!(self);
            Ok(done)
        } else {
            let failed: State<S, Self, Failed> = transition!(self);
            Err(failed)
        }
    }

    #[must_use]
    pub(crate) fn disconnect<S>(
        self: State<S, Self, impl InTerminal>,
    ) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        transition!(const Terminal self)
    }

    #[must_use]
    pub(crate) fn as_healthy<S>(
        self: State<S, Self, impl In<Healthy>>,
    ) -> DiscriminatedState<S, Self, Healthy>
    where
        S: SRef,
    {
        <_>::into_discriminated(self)
    }

    pub(crate) fn endpoint(self: &State<impl SRef, Self, impl InHealthy>) -> &str {
        &self.endpoint
    }

    pub(crate) fn raw_endpoint(&self) -> &str {
        &self.endpoint
    }

    pub(crate) fn user(self: &State<impl SRef, Self, Authenticated>) -> &str {
        self.user
            .as_deref()
            .expect("authenticated job always has a user")
    }

    pub(crate) fn attempts(self: &State<impl SRef, Self, impl InHealthy>) -> u32 {
        self.attempts
    }
}

pub(crate) fn run() {
    let job = crate::connection::Connection::new("localhost:9091")
        .connect()
        .authenticate("erin");
    let job = Job::from_authenticated_connection(job);

    let job = match job.authorise(true) {
        Ok(job) => job,
        Err(_) => panic!("job authorised"),
    };
    let job = match job.dispatch(true) {
        Ok(job) => job,
        Err(_) => panic!("job dispatched"),
    };
    let job = match job.complete(true) {
        Ok(job) => job,
        Err(_) => panic!("job completed"),
    };
    let job = job.as_healthy();

    println!(
        "{} completed a job after {} dispatch attempt(s)",
        job.endpoint(),
        job.attempts()
    );
}
