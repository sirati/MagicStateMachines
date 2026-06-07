#![feature(arbitrary_self_types)]
#![forbid(unsafe_code)]

mod connection;
mod custom_backend;
mod owned;

fn main() {
    owned::run();
    custom_backend::run();
}
