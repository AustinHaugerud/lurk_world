#[macro_use]
extern crate clap;
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate lurk_macros;
extern crate rlua;

mod cli;
mod client;
mod protocol;
mod lua;
mod read;
mod read_buffer;
mod server;
mod write;

fn main() {
    use clap::Clap;
    use cli::Args;

    let args: Args = Args::parse();

    server::server(&args);
}
