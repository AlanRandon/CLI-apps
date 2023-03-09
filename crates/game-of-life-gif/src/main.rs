#![warn(clippy::pedantic)]

use game_of_life_core::prelude::*;
use gif::{Encoder, Frame};
use std::{fs::File, path::PathBuf};
use thiserror::Error;

#[cfg(debug_assertions)]
pub const OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/output.gif");

#[cfg(not(debug_assertions))]
pub const OUTPUT_PATH: &str = "output.gif";

struct GifBackend {
    width: u16,
    height: u16,
    frame_delay: u16,
    encoder: Encoder<File>,
}

struct GifBackendConfig {
    path: PathBuf,
    width: u16,
    height: u16,
    frame_delay: u16,
    alive_color: [u8; 3],
    dead_color: [u8; 3],
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
        }: GifBackendConfig,
    ) -> Result<Self, GifBackendError> {
        let file = File::create(path).unwrap();
        let encoder = Encoder::new(file, width, height, &[dead_color, alive_color].concat())?;
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

    fn render<I>(&mut self, state: I) -> Result<(), GifBackendError>
    where
        I: Iterator<Item = CellRenderInfo>,
    {
        let mut pixels = vec![0; (self.width * self.height).into()];

        let height = usize::from(self.height);
        for CellRenderInfo {
            state,
            coordinates: Coordinates { y, x },
            ..
        } in state
        {
            #[allow(clippy::cast_sign_loss)]
            let index = y as usize * height + x as usize;

            pixels[index] = u8::from(state == CellState::Alive);
        }

        let mut frame = Frame::from_indexed_pixels(self.width, self.height, &pixels, None);

        frame.delay = self.frame_delay;

        self.encoder.set_repeat(gif::Repeat::Infinite)?;

        self.encoder.write_frame(&frame)?;

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
    // TODO: accept config as CLI args
    let mut renderer = GifBackend::renderer(GifBackendConfig {
        path: OUTPUT_PATH.into(),
        width: 100,
        height: 100,
        frame_delay: 10,
        alive_color: [0xff, 0xff, 0xff],
        dead_color: [0, 0, 0],
    })?;

    for _ in 0..300 {
        renderer.render_next_state()?;
    }

    Ok(())
}
