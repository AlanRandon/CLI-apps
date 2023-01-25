use image::RgbImage;

mod hsl_example;
mod triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::Subcommand)]
pub enum Design {
    /// A simple example where the x of the pixel determines its hue
    HslExample,
    /// The Sierpinski Triangle
    Triangle {
        #[command(flatten)]
        args: triangle::Args,
    },
}

impl Design {
    pub fn draw(self) -> RgbImage {
        match self {
            Self::HslExample => hsl_example::draw(),
            Self::Triangle { args } => triangle::draw(args),
        }
    }
}
