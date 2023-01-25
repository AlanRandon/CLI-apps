use crate::prelude::*;
use image::Rgb;
use imageproc::{drawing::draw_polygon_mut, point::Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::Args)]
pub struct Args {
    /// How large the output image should be (in pxiels)
    #[arg(long, short, default_value_t = 1024)]
    size: u32,
    /// How much padding should be around the triangle (in pixels)
    #[arg(long, short, default_value_t = 50)]
    padding: u32,
    /// How many times the triangle should recurse
    #[arg(long, short, default_value_t = 7)]
    iterations: u32,
}

fn draw_triangles(image: &mut RgbImage, vertices: [Point<f64>; 3], iterations: u32) {
    if iterations == 0 {
        #[allow(clippy::cast_possible_truncation)]
        let vertices = vertices.map(|point| Point::new(point.x as i32, point.y as i32));

        draw_polygon_mut(image, &vertices, Rgb::from([255, 255, 255]));
    } else {
        for index in 0..3 {
            let mut vertices = vertices.to_vec();
            let outer_vertex = vertices.swap_remove(index);
            for vertex in &mut vertices {
                *vertex = midpoint(*vertex, outer_vertex);
            }

            draw_triangles(
                image,
                [outer_vertex, vertices[1], vertices[0]],
                iterations - 1,
            );
        }
    }
}

fn midpoint(a: Point<f64>, b: Point<f64>) -> Point<f64> {
    Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

pub fn draw(
    Args {
        size,
        padding,
        iterations,
    }: Args,
) -> RgbImage {
    let side_length = f64::from(size - padding * 2);
    let padding = f64::from(padding);
    let base_height = f64::from(size) - padding;

    let a = Point::new(padding, base_height);
    let b = Point::new(side_length + padding, base_height);
    let c = Point::new(
        (a.x + b.x) / 2.0,
        side_length.mul_add(-(3f64.sqrt()) / 2.0, a.y),
    );

    let mut image = RgbImage::from_pixel(size, size, to_rgb(Hsl::new(230.0, 0.6, 0.2)));

    draw_triangles(&mut image, [a, b, c], iterations);

    image
}
