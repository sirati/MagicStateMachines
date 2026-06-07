#![feature(arbitrary_self_types)]
#![forbid(unsafe_code)]

mod connection;
mod custom_backend;
mod owned;

fn main() {
    owned::run();
    custom_backend::run();
}

#[cfg(test)]
mod tests {
    use super::connection::Connection;

    #[test]
    fn online_enum_calls_online_restricted_endpoint() {
        let online = Connection::disconnected("localhost:8080")
            .connect()
            .as_online_enum();

        assert_eq!(online.endpoint(), "localhost:8080");
    }

    #[test]
    fn anonymous_online_state_converts_through_union_trait() {
        let online = Connection::disconnected("localhost:8081")
            .connect()
            .authenticate("alice")
            .as_online_enum();

        assert_eq!(online.endpoint(), "localhost:8081");
    }
}
