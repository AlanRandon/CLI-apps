use clap::Parser;
use crossterm::{
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use image::{imageops::FilterType, DynamicImage};
use itertools::Itertools;
use std::{
    convert::Infallible,
    io::{self, BufWriter, Stdout, Write},
    iter,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, short)]
    input: String,
    #[arg(long, short, default_value = "block-element-renderer")]
    renderer: Renderer,
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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Renderer {
    ColorRenderer,
    BlockElementRenderer,
    AsciiRenderer,
}

impl Render for Renderer {
    type Err = crossterm::ErrorKind;

    fn render(self, image: DynamicImage, stdout: &mut Stdout) -> Result<(), Self::Err> {
        match self {
            Self::ColorRenderer => ColorRenderer.render(image, stdout),
            Self::BlockElementRenderer => BlockElementRenderer.render(image, stdout),
            Self::AsciiRenderer => AsciiRenderer.render(image, stdout),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
struct AsciiRenderer;

impl Render for AsciiRenderer {
    type Err = crossterm::ErrorKind;

    fn render(self, image: DynamicImage, stdout: &mut Stdout) -> Result<(), Self::Err> {
        const PALLETTE: &[u8] = " '.-~+=![{#$0".as_bytes();
        const PALLETTE_SECTION_SIZE: f32 = PALLETTE.len() as f32 / 255.0;

        let (width, height) = terminal::size()?;
        let image = image.resize(width as u32, height as u32, FilterType::Triangle);
        let image = image.to_luma8();
        let mut stdout = BufWriter::new(stdout);

        for row in image.rows() {
            for pixel in row {
                let luma = pixel.0[0];
                write!(
                    stdout,
                    "{}",
                    PALLETTE[(luma as f32 * PALLETTE_SECTION_SIZE).clamp(0.0, PALLETTE.len() as f32)
                        as usize] as char
                )?;
            }
            writeln!(stdout)?;
        }

        stdout.flush()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct BlockElementRenderer;

struct BlockElement {
    top: image::Rgb<u8>,
    bottom: image::Rgb<u8>,
}

impl BlockElement {
    fn print(self, stdout: &mut Stdout) -> crossterm::Result<()> {
        let [r, g, b] = self.top.0;
        let top_color = Color::Rgb { r, g, b };

        let [r, g, b] = self.bottom.0;
        let bottom_color = Color::Rgb { r, g, b };

        queue!(
            stdout,
            SetBackgroundColor(top_color),
            SetForegroundColor(bottom_color),
            Print("\u{2584}")
        )
    }
}

impl Render for BlockElementRenderer {
    type Err = crossterm::ErrorKind;

    fn render(self, image: DynamicImage, stdout: &mut Stdout) -> Result<(), Self::Err> {
        let (width, height) = terminal::size()?;
        let image = image.resize(width as u32, height as u32 * 2, FilterType::Triangle);
        let image = image.to_rgb8();

        let empty_pixel = image::Rgb([0, 0, 0]);

        let rows = image
            .rows()
            .map(|row| row.cloned())
            .batching(|rows| {
                Some(iter::zip(
                    rows.next()?,
                    rows.next()
                        .map(IterExt::boxed)
                        .unwrap_or_else(|| iter::repeat_with(|| empty_pixel).boxed()),
                ))
            })
            .map(|row| row.map(|(top, bottom)| BlockElement { top, bottom }));

        for row in rows {
            for block in row {
                block.print(stdout)?;
            }
            queue!(stdout, ResetColor, Print("\n"))?;
        }

        queue!(stdout, ResetColor)?;

        stdout.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Args { input, renderer } = Args::parse();
    let mut stdout = io::stdout();
    let image = image::io::Reader::open(input)?.decode()?;
    renderer.render(image, &mut stdout)?;

    Ok(())
}
