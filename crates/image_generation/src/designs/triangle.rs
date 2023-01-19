use crate::prelude::*;
use image::Rgb;
use imageproc::{drawing::draw_polygon_mut, point::Point};
use std::f64::consts::TAU;

const SIZE: i32 = 300;

fn draw_triangles(image: &mut RgbImage, verticies: [Point<f64>; 3], depth: u32, max_depth: u32) {
    if depth == max_depth {
        draw_polygon_mut(
            image,
            &verticies.map(|point| Point::new(point.x as i32, point.y as i32)),
            Rgb::from([255, 255, 255]),
        )
    } else {
        let middle_side_1 = middle(verticies[0], verticies[1]);
        let middle_side_2 = middle(verticies[0], verticies[2]);
        let middle_side_3 = middle(verticies[2], verticies[1]);

        let depth = depth - 1;

        draw_triangles(
            image,
            [middle_side_1, middle_side_2, center],
            depth,
            max_depth,
        )
    }
}

fn middle(a: Point<f64>, b: Point<f64>) -> Point<f64> {
    Point::new(a.x + b.x / 2.0, a.y + b.y / 2.0)
}

fn sierpinski(image: &mut RgbImage) {}

pub fn draw() -> Result<(), Box<dyn std::error::Error>> {
    let mut image = RgbImage::new(SIZE as u32, SIZE as u32);

    sierpinski(&mut image);

    image.save(OUTPUT_PATH)?;

    Ok(())
}
