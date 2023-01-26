use crate::prelude::*;
use num_complex::Complex64;

#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct Args {
    /// How large the image should be
    #[arg(long, short, default_value_t = 512)]
    size: u32,

    /// Iterations
    #[arg(long, short, default_value_t = 100)]
    iterations: u32,

    /// How zoomed in the image should be
    #[arg(long, short, default_value_t = String::from("100.0"))]
    zoom: String,

    /// The start of the image on the X axis
    #[arg(long, short = 'x', default_value_t = String::from("-2"))]
    start_x: String,

    /// The start of the image on the Y axis
    #[arg(long, short = 'y', default_value_t = String::from("1.2"))]
    start_y: String,
}

pub fn draw(
    Args {
        size,
        zoom,
        start_x,
        start_y,
        iterations,
    }: Args,
) -> RgbImage {
    let zoom = zoom.parse().unwrap();
    let start_x = start_x.parse().unwrap();
    let start_y = start_y.parse().unwrap();

    image_from_fn_parallel(size, size, |x, y| {
        let z = Complex64::new(
            f64::from(x).mul_add(zoom, start_x),
            f64::from(y).mul_add(zoom, start_y),
        );

        let escape_count = escape_count(z, iterations);

        if escape_count == iterations {
            Hsl::new(0.0, 0.0, 0.0)
        } else {
            Hsl::new(0.0, 0.0, 1.0 - f64::from(escape_count / iterations))
        }
    })
}

fn escape_count(mut z: Complex64, iterations: u32) -> u32 {
    let c = Complex64::new(-0.7, -0.11);

    for iteration in 0..iterations {
        z = z.powu(2) + c;
        if z.norm_sqr() > 4.0 {
            return iteration;
        }
    }

    iterations
}
