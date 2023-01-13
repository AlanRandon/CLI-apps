use image::ImageBuffer;
use palette::{FromColor, Pixel, Srgb};
use rayon::prelude::*;
use std::sync::Arc;

pub fn image_from_fn_parallel<F, P, T>(
    width: u32,
    height: u32,
    generate: F,
) -> ImageBuffer<image::Rgb<u8>, Vec<u8>>
where
    F: Fn(u32, u32) -> P + Send + Sync,
    Srgb<T>: FromColor<P>,
    T: palette::Component + palette::IntoComponent<u8>,
{
    let generate = Arc::new(generate);

    ImageBuffer::from_vec(
        width,
        height,
        (0..height)
            .into_par_iter()
            .flat_map(|y| {
                (0..width).into_par_iter().flat_map({
                    let generate = Arc::clone(&generate);
                    move |x| {
                        Srgb::from_color(generate(x, y))
                            .into_format()
                            .into_raw::<[_; 3]>()
                    }
                })
            })
            .collect(),
    )
    .unwrap()
}
