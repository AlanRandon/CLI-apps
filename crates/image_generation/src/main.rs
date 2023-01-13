use palette::Hsl;

mod utilities;

#[cfg(debug_assertions)]
const OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/output.png");

#[cfg(not(debug_assertions))]
const OUTPUT_PATH: &str = "output.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    utilities::image_from_fn_parallel(360, 64, |x, _| {
        let x = f64::from(x);
        Hsl::new(x, 0.9, 0.6)
    })
    .save(OUTPUT_PATH)?;

    Ok(())
}
