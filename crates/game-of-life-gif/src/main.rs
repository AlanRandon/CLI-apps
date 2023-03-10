#![warn(clippy::pedantic)]

use clap::Parser;
use game_of_life_core::prelude::*;
use gif::{Encoder, Frame};
use std::{fs::File, path::PathBuf};
use thiserror::Error;

#[cfg(debug_assertions)]
pub const DEFAULT_OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/output.gif");

#[cfg(not(debug_assertions))]
pub const DEFAULT_OUTPUT_PATH: &str = "output.gif";

struct GifBackend {
    width: u16,
    height: u16,
    frame_delay: u16,
    encoder: Encoder<File>,
}

#[derive(Parser, Debug)]
struct GifBackendConfig {
    #[clap(short, long, default_value = DEFAULT_OUTPUT_PATH)]
    path: PathBuf,
    #[clap(long, default_value_t = 256)]
    width: u16,
    #[clap(long, default_value_t = 256)]
    height: u16,
    #[clap(short = 'd', long, default_value_t = 10)]
    frame_delay: u16,
    #[clap(short = 'c', long, default_value_t = 512)]
    frame_count: usize,
    #[clap(long, value_parser = parse_hex_color, default_value = "ffffff")]
    alive_color: [u8; 3],
    #[clap(long, value_parser = parse_hex_color, default_value = "000000")]
    dead_color: [u8; 3],
}

#[derive(Error, Debug)]
enum ParseHexColorError {
    #[error("insufficient length of input")]
    Length,
    #[error("failed encoding")]
    FromStrRadix(#[from] std::num::ParseIntError),
}

fn parse_hex_color(input: &str) -> Result<[u8; 3], ParseHexColorError> {
    if input.len() != 6 {
        return Err(ParseHexColorError::Length);
    };

    let bytes = u32::from_str_radix(input, 16)?.to_le_bytes();

    Ok([bytes[2], bytes[1], bytes[0]])
}

#[derive(Error, Debug)]
enum GifBackendError {
    #[error("failed to write to output")]
    IO(#[from] std::io::Error),
    #[error("failed encoding")]
    Encoding(#[from] gif::EncodingError),
}

impl GifBackend {
    fn new(
        GifBackendConfig {
            path,
            width,
            height,
            frame_delay,
            alive_color,
            dead_color,
            ..
        }: GifBackendConfig,
    ) -> Result<Self, GifBackendError> {
        log::info!("Opening gif file");
        let file = File::create(path).unwrap();
        log::info!("Creating gif encoder");
        let mut encoder = Encoder::new(file, width, height, &[dead_color, alive_color].concat())?;
        encoder.set_repeat(gif::Repeat::Infinite)?;

        Ok(Self {
            width,
            height,
            frame_delay,
            encoder,
        })
    }
}

impl RendererBackend<GifBackendError> for GifBackend {
    type Config = GifBackendConfig;

    fn render(&mut self, state: state::Frame) -> Result<(), GifBackendError> {
        let mut frame = Frame::from_indexed_pixels(
            self.width,
            self.height,
            &state.to_buffer(|state| u8::from(state == CellState::Alive)),
            None,
        );

        frame.delay = self.frame_delay;
        frame.make_lzw_pre_encoded();

        self.encoder.write_lzw_pre_encoded_frame(&frame)?;

        Ok(())
    }

    fn renderer(config: Self::Config) -> Result<Renderer<Self, GifBackendError>, GifBackendError> {
        Ok(Renderer::new(
            State::new(config.width.into(), config.height.into()),
            Self::new(config)?,
        ))
    }
}

fn main() -> Result<(), GifBackendError> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Parsing config from CLI args");
    let config = GifBackendConfig::parse();

    let frame_count = config.frame_count;

    let renderer = GifBackend::renderer(config)?;

    log::info!("Rendering frames");

    renderer
        .take(frame_count)
        .collect::<Result<(), GifBackendError>>()?;

    log::info!("Complete");

    Ok(())
}
