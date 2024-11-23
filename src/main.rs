use clap::Parser;
use clap_verbosity_flag::Verbosity;

// Maybe the encoder needs a control structure
// where
#[derive(Parser)]
enum Direction {
    // archive, compress, files.
    // update manifest with map<path, sha>
    // this is what git does...
    Encode {},
    Decode {},
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    direction: Direction,

    #[command(flatten)]
    verbosity: Verbosity,
}

fn main() {
    println!("Hello, world!");
}
