use std::str::FromStr;

use crate::prelude::*;
use num_complex::Complex64;

#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct Args {
    /// The width of image
    #[arg(long, default_value_t = 512)]
    width: u32,

    /// The height of image
    #[arg(long, default_value_t = 512)]
    height: u32,

    /// Iterations
    #[arg(long, short, default_value_t = 1000)]
    iterations: u32,

    /// How zoomed in the image should be
    #[arg(long, default_value_t = String::from("0.6"))]
    zoom: String,

    /// Shift the image on the X axis
    #[arg(long, default_value_t = String::from("0"))]
    move_x: String,

    /// Shift the image on the Y axis
    #[arg(long, default_value_t = String::from("0"))]
    move_y: String,

    /// Shift the hue
    #[arg(long, default_value_t = String::from("180"))]
    hue: String,

    #[arg(long, default_value_t = String::from("0.156i-0.8"))]
    c: String,
}

pub fn draw(
    Args {
        width,
        height,
        zoom,
        move_x,
        move_y,
        iterations,
        c,
        hue,
    }: Args,
) -> ImageResult {
    Ok(image_from_fn_parallel(width, height, {
        let zoom = zoom.parse::<f64>()?;
        let move_x: f64 = move_x.parse()?;
        let move_y: f64 = move_y.parse()?;
        let hue: f64 = hue.parse()?;
        let c = Complex64::from_str(&c)?;
        let width = f64::from(width);
        let height = f64::from(height);
        let l = width.max(height);

        move |x, y| {
            let z = Complex64::new(
                2.0f64.mul_add(f64::from(x), -width) / (l * zoom) + move_x,
                2.0f64.mul_add(f64::from(y), -height) / (l * zoom) + move_y,
            );

            let escape_count = escape_count(z, c, iterations);

            if escape_count == iterations {
                Hsl::new(0.0, 0.0, 0.0)
            } else {
                Hsl::new(f64::from(escape_count) + hue, 0.9, 0.5)
            }
        }
    }))
}

fn escape_count(mut z: Complex64, c: Complex64, iterations: u32) -> u32 {
    for iteration in 0..iterations {
        z = z.powu(2) + c;
        if z.norm_sqr() > 4.0 {
            return iteration;
        }
    }

    iterations
}
