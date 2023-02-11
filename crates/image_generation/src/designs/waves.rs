use crate::prelude::*;
use palette::{gradient::Gradient, rgb::Rgb, IntoColor};
use rand::{thread_rng, Rng};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    #[clap(long, default_value_t = 256)]
    width: u32,
    #[clap(long, default_value_t = 100)]
    height: u32,
    #[clap(long, default_value_t = 7)]
    wave_count: u32,
    #[clap(long, value_parser = parse_gradient, default_value  = "#00c6ff,#00274a")]
    gradient: Gradient<Hsl>,
    #[clap(long, default_value_t = 50.)]
    min_wavelength: f32,
    #[clap(long, default_value_t = 200.)]
    max_wavelength: f32,
}

impl Args {
    fn get_wave_fns(&self) -> Vec<impl Fn(u32, u32) -> Option<Hsl>> {
        let Self {
            height,
            wave_count,
            gradient,
            min_wavelength,
            max_wavelength,
            ..
        } = self;

        (0..*wave_count)
            .into_par_iter()
            .map(|wave| {
                let min_distance = (height / wave_count) * wave;
                let max_distance = if wave == wave_count - 1 {
                    *height
                } else {
                    (height / wave_count) * (wave + 1)
                };

                let color = gradient.get(wave as f32 / (wave_count - 1) as f32);
                let mut rng = thread_rng();
                let wavelength = rng.gen_range(*min_wavelength..*max_wavelength);
                let wave_start = rng.gen_range(0.0..wavelength);

                move |x, y| {
                    (min_distance
                        ..(max_distance
                            + ((x as f32 / wavelength + wave_start).sin() * 10.0).abs() as u32))
                        .contains(&y)
                        .then_some(color)
                }
            })
            .collect::<Vec<_>>()
    }
}

fn parse_gradient(input: &str) -> Result<Gradient<Hsl>, palette::rgb::FromHexError> {
    input
        .split(',')
        .map(|input| {
            input
                .parse::<Rgb<_, u8>>()
                .map(|color| color.into_format().into_color())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Gradient::new)
}

pub fn draw(args: &Args) -> RgbImage {
    let wave_fns = args.get_wave_fns();
    let Args { width, height, .. } = args;

    image_from_fn_parallel(*width, *height, move |x, y| {
        wave_fns
            .par_iter()
            .map(|wave_fn| wave_fn(x, y))
            .find_first(Option::is_some)
            .unwrap_or_else(|| Some(Hsl::new(0., 0., 0.)))
            .unwrap()
    })
}
