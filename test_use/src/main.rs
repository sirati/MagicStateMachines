#![feature(arbitrary_self_types)]
#![forbid(unsafe_code)]

mod connectable;
mod connection;
mod connection_async;
mod connection_async_usage;
mod custom_backend;
mod owned;

fn main() {
    owned::run();
    connection_async_usage::run();
    connectable::run();
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
    use test_def::{InOnline, Online, OnlineDiscriminator, OnlineEnum};

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
        fn assert_in_online<T: InOnline>() {}

        assert_in_online::<test_def::states::Connected>();
        assert_in_online::<test_def::states::Authenticated>();
        assert_in_online::<statemachines::StateUnionState<Online>>();
    }

    #[test]
    fn online_enum_deref_carries_discriminator_in_storage() {
        fn expect_discriminated_storage(
            _state: &statemachines::State<
                statemachines::SDiscriminated<statemachines::SOwned, OnlineDiscriminator>,
                Connection,
                statemachines::StateUnionState<Online>,
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
