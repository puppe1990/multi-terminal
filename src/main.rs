use clap::Parser;
use multi_terminal::{run, Args};

fn main() {
    let args = Args::parse();
    run(args);
}
