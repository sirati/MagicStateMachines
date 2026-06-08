#![feature(arbitrary_self_types)]
#![forbid(unsafe_code)]

mod connection;
mod connection_async;
mod custom_backend;
mod owned;

fn main() {
    owned::run();
    connection_async::run();
    custom_backend::run();
}

#[cfg(test)]
mod tests {
    use core::{
        future::Future,
        pin::pin,
        task::{Context, Poll, Waker},
    };

    use super::connection::Connection;
    use super::connection_async::ConnectionAsync;

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
}
