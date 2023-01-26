use image::RgbImage;

mod hsl_example;
mod julia_set;
mod triangle;

#[derive(Debug, Clone, PartialEq, Eq, clap::Subcommand)]
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
}

impl Design {
    pub fn draw(self) -> RgbImage {
        match self {
            Self::HslExample => hsl_example::draw(),
            Self::Triangle { args } => triangle::draw(args),
            Self::JuliaSet { args } => julia_set::draw(args),
        }
    }
}
