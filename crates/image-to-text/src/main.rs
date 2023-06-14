use clap::Parser;
use crossterm::terminal;
use image::imageops::FilterType;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, short)]
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { input } = Args::parse();
    let (width, height) = terminal::size()?;
    let image = image::io::Reader::open(input)?.decode()?;
    let image = image.resize(width as u32, height as u32, FilterType::Triangle);
    let image = image.to_rgb8();
    todo!("output image to terminal using crossterm, or as plain ascii");
    Ok(())
}
