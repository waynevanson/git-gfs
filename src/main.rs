use clap::Parser;
use obfuscat::*;

fn main() {
    println!("Hello, world!");

    let args = Args::parse();

    args.command.call().unwrap();
}
