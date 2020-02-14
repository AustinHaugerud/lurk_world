use clap::{App, Arg, Clap};

#[derive(Clap)]
#[clap(version = "1.0",  author = "Austin Jenkins")]
pub struct Args {
    #[clap(short = "m", long = "module")]
    pub module: String,

    #[clap(short = "p", long = "port")]
    pub port: u16,
}
