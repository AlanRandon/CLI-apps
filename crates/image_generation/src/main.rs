#![warn(clippy::pedantic, clippy::nursery)]

use clap::Parser;

mod designs;
mod prelude;

/// A simple program to test the primality of a number
#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    // The primality test to use
    #[arg(long, short, value_enum)]
    design: designs::Design,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    args.design.draw()?;

    Ok(())
}
