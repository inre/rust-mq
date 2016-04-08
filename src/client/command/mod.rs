pub mod subscribe;

pub use client::command::subscribe::{
    SubscribeCommand
};

pub trait Command {
    fn run(&self) -> !;
}
