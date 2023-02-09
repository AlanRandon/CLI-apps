use rand::thread_rng;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct Args {
    #[clap(long, default_value_t = 100)]
    width: u32,
    #[clap(long, default_value_t = 100)]
    height: u32,
}

pub fn draw(args: &Args) -> RgbImage {
    let Args { width, height } = *args;
    // let mut rng = thread_rng();

    let wave_count = 3;

    let wave_fns = (0..wave_count)
        .map(|wave| {
            let min_distance = (height / wave_count) * wave;
            let max_distance = (height / wave_count) * (wave + 1);

            let color = Hsl::new((height / wave_count * wave * 60) as f32, 0.8, 0.8);

            move |x, y| {
                (min_distance..(max_distance + ((y as f32).sin() * 10.0).abs() as u32))
                    .contains(&y)
                    .then_some(color)
            }
        })
        .collect::<Vec<_>>();

    image_from_fn_parallel(width, height, move |x, y| {
        wave_fns
            .par_iter()
            .map(|wave_fn| wave_fn(x, y))
            .find_first(Option::is_some)
            .unwrap_or_else(|| Some(Hsl::new(0.0, 0.0, 0.0)))
            .unwrap()
    })
}
