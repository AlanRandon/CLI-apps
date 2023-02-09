#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::cast_sign_loss
)]

use crate::prelude::*;
use clap::Parser;

mod designs;
mod prelude;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[command(subcommand)]
    design: designs::Design,

    /// The file path where the output should be saved
    #[arg(long, short, default_value_t = DEFAULT_OUTPUT_PATH.to_string())]
    output_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args {
        design,
        output_path,
    } = Args::parse();

    design.draw()?.save(output_path)?;

    Ok(())
}
