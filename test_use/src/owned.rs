use crate::connection::Connection;
use statemachines::{State, StorageStateOwnedBox};
use test_def::states::Disconnected;

pub(crate) fn run() {
    let connection = Connection::disconnected("localhost:8080");
    let connection = connection.connect();
    let connection = connection.authenticate("alice");

    println!(
        "{} is authenticated as {}",
        connection.endpoint(),
        connection.user()
    );

    let connection = connection.logout();
    println!("{} is still online", connection.endpoint());

    let _connection = connection.disconnect();

    let boxed: State<StorageStateOwnedBox, Connection, Disconnected> =
        State::new(Connection::new("localhost:9090"));
    let _boxed = boxed.connect();
}
