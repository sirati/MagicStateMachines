use crate::connection::Connection;
use statemachines::{State, StorageStateOwnedBox};
use test_def::{OnlineEnum, states::Disconnected};

pub(crate) fn run() {
    let connection = Connection::disconnected("localhost:8080");
    let connection = match connection.try_connect(true) {
        Ok(connection) => connection,
        Err(_) => return,
    };
    let connection = match connection.as_online_enum() {
        OnlineEnum::Connected(connection) => connection.into_state(),
        OnlineEnum::Authenticated(_) => return,
    };
    let online = connection.authenticate_if(Some("alice".into()));
    println!("{} is online", online.endpoint());

    let connection = match online {
        OnlineEnum::Authenticated(connection) => connection.into_state(),
        OnlineEnum::Connected(_) => return,
    };

    println!(
        "{} is authenticated as {}",
        connection.endpoint(),
        connection.user()
    );

    let connection = connection.logout();
    println!("{} is still online", connection.endpoint());

    let _connection = connection.disconnect();

    let online = Connection::disconnected("localhost:8081")
        .connect()
        .authenticate_if(None);
    let _disconnected = online.into_erased().disconnect_online();

    let boxed: State<StorageStateOwnedBox, Connection, Disconnected> =
        State::new(Connection::new("localhost:9090"));
    let _boxed = boxed.connect();
}
