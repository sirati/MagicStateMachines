#![feature(arbitrary_self_types)]
#![forbid(unsafe_code)]

mod connectable;
mod connection;
mod connection_async;
mod connection_async_usage;
mod custom_backend;
mod job;
mod owned;

fn main() {
    owned::run();
    connection_async_usage::run();
    connectable::run();
    job::run();
    custom_backend::run();
}

#[cfg(test)]
mod tests {
    use core::{
        future::Future,
        pin::pin,
        task::{Context, Poll, Waker},
    };

    use super::connectable::{Connectable, ConnectionViaTrait};
    use super::connection::Connection;
    use super::connection_async::ConnectionAsync;
    use super::job::Job;
    use magicstatemachines::{In, SOwned, State, StateUnionDiscriminant};
    use test_def::states::{Authenticated, Disconnected};
    use test_def::{Online, OnlineEnum};

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

    #[test]
    fn online_enum_calls_online_restricted_endpoint() {
        let online = Connection::new("localhost:8080").connect().as_online_enum();

        assert_eq!(online.endpoint(), "localhost:8080");
    }

    #[test]
    fn anonymous_online_state_converts_through_union_trait() {
        let online = Connection::new("localhost:8081")
            .connect()
            .authenticate("alice")
            .as_online_enum();

        assert_eq!(online.endpoint(), "localhost:8081");
    }

    #[test]
    fn anonymous_online_state_disconnects_through_erased_transition() {
        let disconnected = Connection::new("localhost:8082")
            .connect()
            .authenticate("alice")
            .disconnect();

        assert_eq!(disconnected.raw_endpoint(), "localhost:8082");
    }

    #[test]
    fn async_connection_uses_the_same_contract() {
        let authenticated = block_on(async {
            ConnectionAsync::new("localhost:8083")
                .connect()
                .await
                .authenticate("alice")
                .await
        });

        assert_eq!(authenticated.endpoint(), "localhost:8083");
        assert_eq!(authenticated.user(), "alice");

        let disconnected = block_on(authenticated.disconnect());
        assert_eq!(disconnected.raw_endpoint(), "localhost:8083");
    }

    #[test]
    fn async_connection_can_return_online_enum() {
        let online = block_on(async {
            ConnectionAsync::new("localhost:8084")
                .connect()
                .await
                .authenticate_if(None)
                .await
        });

        assert_eq!(online.endpoint(), "localhost:8084");
    }

    #[test]
    fn sync_connection_state_can_convert_to_async_connection_state() {
        let converted: State<SOwned, ConnectionAsync, Authenticated> =
            ConnectionAsync::from_authenticated_connection(
                Connection::new("localhost:8088")
                    .connect()
                    .authenticate("alice"),
            );

        assert_eq!(converted.endpoint(), "localhost:8088");
        assert_eq!(converted.user(), "alice");

        let connected = block_on(converted.logout());
        assert_eq!(connected.endpoint(), "localhost:8088");
    }

    #[test]
    fn authenticated_connection_can_start_overlapping_job_state_machine() {
        let job: State<SOwned, Job, Authenticated> = Job::from_authenticated_connection(
            Connection::new("localhost:8095")
                .connect()
                .authenticate("dana"),
        );

        assert_eq!(job.endpoint(), "localhost:8095");
        assert_eq!(job.user(), "dana");

        let job = match job.authorise(true) {
            Ok(job) => job,
            Err(_) => panic!("job authorised"),
        };
        assert_eq!(job.endpoint(), "localhost:8095");

        let job = match job.dispatch(true) {
            Ok(job) => job,
            Err(_) => panic!("job dispatched"),
        };
        assert_eq!(job.attempts(), 1);

        let done = match job.complete(true) {
            Ok(done) => done,
            Err(_) => panic!("job done"),
        };
        let disconnected: State<SOwned, Job, Disconnected> = done.disconnect();
        assert_eq!(disconnected.raw_endpoint(), "localhost:8095");
    }

    #[test]
    fn overlapping_job_can_fail_and_disconnect() {
        let job: State<SOwned, Job, Authenticated> = Job::from_authenticated_connection(
            Connection::new("localhost:8096")
                .connect()
                .authenticate("frank"),
        );

        let failed = match job.authorise(false) {
            Ok(_) => panic!("job rejected"),
            Err(failed) => failed,
        };
        let disconnected = failed.disconnect();

        assert_eq!(disconnected.raw_endpoint(), "localhost:8096");
    }

    #[test]
    fn connectable_trait_surface_uses_the_same_contract() {
        let authenticated = ConnectionViaTrait::new("localhost:8089")
            .connect()
            .authenticate("carol");

        assert_eq!(authenticated.endpoint(), "localhost:8089");
        assert_eq!(authenticated.user(), "carol");

        let disconnected = authenticated.disconnect();
        assert_eq!(disconnected.raw_endpoint(), "localhost:8089");
    }

    #[test]
    fn connectable_trait_can_return_online_enum() {
        let online = ConnectionViaTrait::new("localhost:8090")
            .connect()
            .authenticate_if(None);

        assert_eq!(online.endpoint(), "localhost:8090");
    }

    #[test]
    fn online_members_are_in_online() {
        fn assert_in_online<T: In<Online>>() {}

        assert_in_online::<test_def::states::Connected>();
        assert_in_online::<test_def::states::Authenticated>();
        assert_in_online::<magicstatemachines::StateUnionState<Online>>();
    }

    #[test]
    fn online_marker_names_its_enum_type() {
        let marker_enum: Option<
            <Online as StateUnionDiscriminant>::Enum<magicstatemachines::SOwned, Connection>,
        > = None;
        let _: Option<OnlineEnum<magicstatemachines::SOwned, Connection>> = marker_enum;
    }

    #[test]
    fn online_enum_deref_carries_inferred_state_in_storage() {
        fn expect_discriminated_storage(
            _state: &magicstatemachines::State<
                magicstatemachines::SDiscriminated<magicstatemachines::SOwned>,
                Connection,
                magicstatemachines::StateUnionState<Online>,
            >,
        ) {
        }

        let online = Connection::new("localhost:8092")
            .connect()
            .authenticate_if(None);

        expect_discriminated_storage(&online);
    }

    #[test]
    fn erased_online_state_recovers_discriminated_variant() {
        let connected = Connection::new("localhost:8093")
            .connect()
            .authenticate_if(None)
            .discriminate();

        assert!(matches!(connected, OnlineEnum::Connected(_)));

        let authenticated = Connection::new("localhost:8094")
            .connect()
            .authenticate_if(Some("alice".to_owned()))
            .discriminate();

        assert!(matches!(authenticated, OnlineEnum::Authenticated(_)));
    }
}
