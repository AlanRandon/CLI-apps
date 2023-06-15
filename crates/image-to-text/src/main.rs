use clap::Parser;
use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use image::{imageops::FilterType, DynamicImage, Luma};
use itertools::Itertools;
use std::{
    io::{self, Stdout, Write},
    iter,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, short)]
    input: String,
}

trait Render {
    type Err;

    fn render(self, image: DynamicImage, stdout: &mut Stdout) -> Result<(), Self::Err>;
}

trait IterExt: Iterator + Sized {
    fn boxed<'a>(self) -> Box<dyn Iterator<Item = Self::Item> + 'a>
    where
        Self: 'a,
    {
        Box::new(self) as Box<dyn Iterator<Item = Self::Item>>
    }
}

impl<T> IterExt for T where T: Iterator {}

struct ColorRenderer;

impl Render for ColorRenderer {
    type Err = crossterm::ErrorKind;

    fn render(self, image: DynamicImage, stdout: &mut Stdout) -> Result<(), Self::Err> {
        let (width, height) = terminal::size()?;
        let image = image.resize(width as u32, height as u32, FilterType::Triangle);
        let image = image.to_rgb8();

        for row in image.rows() {
            for pixel in row {
                let [r, g, b] = pixel.0;
                let color = Color::Rgb { r, g, b };
                queue!(
                    stdout,
                    SetBackgroundColor(color),
                    SetForegroundColor(color),
                    Print("0")
                )?;
            }
            queue!(stdout, ResetColor, Print("\n"))?;
        }

        queue!(stdout, ResetColor)?;

        stdout.flush()?;
        Ok(())
    }
}

struct BlockElementRenderer;

impl Render for BlockElementRenderer {
    type Err = crossterm::ErrorKind;

    fn render(self, image: DynamicImage, stdout: &mut Stdout) -> Result<(), Self::Err> {
        let (width, height) = terminal::size()?;
        let image = image.resize(width as u32 * 2, height as u32 * 2, FilterType::Triangle);
        let image = image.to_luma8();

        let empty_pixels = iter::repeat(0).map(|pixel| Luma([pixel]));

        let characters = image
            .rows()
            .map(|row| row.map(Clone::clone))
            .chunks(2)
            .into_iter()
            .flat_map(|mut row| {
                let mut subrow = || {
                    row.next()
                        .map(IterExt::boxed)
                        .unwrap_or_else(|| empty_pixels.clone().boxed())
                };
                iter::zip(subrow(), subrow())
            });

        // for row in image.rows() {
        //     for pixel in row {
        //         let [r, g, b] = pixel.0;
        //         let color = Color::Rgb { r, g, b };
        //         queue!(
        //             stdout,
        //             SetBackgroundColor(color),
        //             SetForegroundColor(color),
        //             Print("0")
        //         )?;
        //     }
        //     queue!(stdout, ResetColor, Print("\n"))?;
        // }

        queue!(stdout, ResetColor)?;

        stdout.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { input } = Args::parse();
    let mut stdout = io::stdout();
    let image = image::io::Reader::open(input)?.decode()?;
    ColorRenderer.render(image, &mut stdout)?;

    Ok(())
}
