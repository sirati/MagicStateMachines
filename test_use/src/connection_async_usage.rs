use crate::connection_async::ConnectionAsync;
use core::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

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

    let disconnected = block_on(online.disconnect());
    println!(
        "{} is asynchronously disconnected",
        disconnected.raw_endpoint()
    );

    let online = block_on(async {
        let c = ConnectionAsync::new("localhost:8086");
        let c = c.connect().await;
        let c = c.as_online_enum();
        let c = c.as_online_enum();
        c
    });
    println!("{} is asynchronously online via enum", online.endpoint());
}
