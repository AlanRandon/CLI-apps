use crate::prelude::*;
use image::Rgb;
use imageproc::{drawing::draw_polygon_mut, point::Point};

const SIZE: u32 = 1024;
const PADDING: u32 = 50;
const ITERATIONS: u32 = 7;

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

pub fn draw() -> Result<(), Box<dyn std::error::Error>> {
    let padding = f64::from(PADDING);
    let size = f64::from(SIZE);
    let side_length = f64::from(SIZE - PADDING * 2);
    let base_height = size - padding;

    let a = Point::new(padding, base_height);
    let b = Point::new(side_length + padding, base_height);
    let c = Point::new(
        (a.x + b.x) / 2.0,
        side_length.mul_add(-(3f64.sqrt()) / 2.0, a.y),
    );

    let mut image = RgbImage::from_pixel(SIZE, SIZE, to_rgb(Hsl::new(230.0, 0.6, 0.2)));

    draw_triangles(&mut image, [a, b, c], ITERATIONS);

    image.save(OUTPUT_PATH)?;

    Ok(())
}
