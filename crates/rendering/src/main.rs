use euc::{buffer::Buffer2d, rasterizer, Interpolate, Pipeline};
use image::Rgb;
use rand::{distributions::uniform::Uniform, prelude::*};

#[derive(Debug, Clone, Copy)]
struct Point([f32; 3]);

impl Point {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }

    #[inline(always)]
    fn x(&self) -> f32 {
        self.0[0]
    }

    #[inline(always)]
    fn y(&self) -> f32 {
        self.0[1]
    }

    #[inline(always)]
    fn z(&self) -> f32 {
        self.0[2]
    }
}

impl From<[f32; 3]> for Point {
    fn from(value: [f32; 3]) -> Self {
        Self(value)
    }
}

impl Interpolate for Point {
    #[inline(always)]
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
        //a * x + b * y
        [0, 1, 2].map(|i| a.0[i].mul_add(x, b.0[i] * y)).into()
    }

    #[inline(always)]
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        //a * x + b * y + c * z
        [0, 1, 2]
            .map(|i| a.0[i].mul_add(x, b.0[i] * y) + c.0[i] * z)
            .into()
    }
}

struct Triangle;

impl Pipeline for Triangle {
    type Vertex = [f32; 3];
    type VsOut = Point;
    type Pixel = Rgb<u8>;

    #[inline(always)]
    fn vert(&self, pos: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        (
            [pos[0], pos[1], 0.0, pos[2]],
            Self::VsOut::new(pos[0], pos[1], pos[2]),
        )
    }

    #[inline(always)]
    fn frag(&self, pos: &Self::VsOut) -> Self::Pixel {
        Rgb([
            (pos.z() * 255.) as u8,
            (pos.z() * 255.) as u8,
            (pos.z() * 255.) as u8,
        ])
    }
}

#[cfg(debug_assertions)]
const OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/output.png");

#[cfg(not(debug_assertions))]
const OUTPUT_PATH: &str = "output.png";

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Buffer2d::new([WIDTH as _, HEIGHT as _], Rgb([0, 0, 0]));
    let mut rng = rand::thread_rng();

    let mut verticies = std::iter::from_fn({
        let xy_dist = Uniform::new(-1.0, 1.0);
        let z_dist = Uniform::new(0.5, 1.0);
        move || Some([rng.sample(xy_dist), rng.sample(xy_dist), rng.sample(z_dist)])
    });

    let triangles = std::iter::successors(
        Some([
            verticies.next().unwrap(),
            verticies.next().unwrap(),
            verticies.next().unwrap(),
        ]),
        |[_, b, c]| Some([*b, *c, verticies.next().unwrap()]),
    );

    Triangle.draw::<rasterizer::Triangles<(f32,)>, _>(
        triangles.take(10).flatten().collect::<Vec<_>>().as_ref(),
        &mut buffer,
        None,
    );

    let buffer = image::RgbImage::from_raw(
        WIDTH,
        HEIGHT,
        buffer
            .as_ref()
            .iter()
            .flat_map(|Rgb(data)| data)
            .cloned()
            .collect::<Vec<_>>(),
    )
    .unwrap();

    buffer.save(OUTPUT_PATH)?;
    Ok(())
}
