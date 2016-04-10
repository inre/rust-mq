pub mod publish;
pub mod subscribe;

pub use client::command::publish::PublishCommand;
pub use client::command::subscribe::SubscribeCommand;

pub trait Command {
    fn run(&self) -> !;
}
