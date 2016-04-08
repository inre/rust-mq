extern crate rustmq;

use std::env;
use rustmq::client::CLI;

fn main() {
    let args: Vec<String> = env::args().collect();
    let client = CLI::new(args);
    let cmd = client.parse();
    cmd.run();
}
