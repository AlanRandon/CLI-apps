use crate::prelude::*;

mod hsl_example;
mod julia_set;
mod triangle;
mod waves;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Design {
    /// A simple example where the x of the pixel determines its hue
    HslExample,
    /// The Sierpinski Triangle
    Triangle {
        #[command(flatten)]
        args: triangle::Args,
    },
    // Draw a Julia set
    JuliaSet {
        #[command(flatten)]
        args: julia_set::Args,
    },
    Waves {
        #[command(flatten)]
        args: waves::Args,
    },
}

impl Design {
    pub fn draw(self) -> ImageResult {
        match self {
            Self::HslExample => Ok(hsl_example::draw()),
            Self::Triangle { args } => Ok(triangle::draw(args)),
            Self::JuliaSet { args } => julia_set::draw(args),
            Self::Waves { args } => Ok(waves::draw(&args)),
        }
    }
}
