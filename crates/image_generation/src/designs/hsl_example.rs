use crate::prelude::*;

pub fn draw() -> RgbImage {
    image_from_fn_parallel(360, 64, |x, _| {
        let x = f64::from(x);
        Hsl::new(x, 0.9, 0.6)
    })
}
