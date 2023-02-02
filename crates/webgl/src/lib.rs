use js_sys::Math::random;
use noise::{utils::NoiseMap, NoiseFn, Perlin};
use num_complex::Complex32;
use std::sync::Arc;
use three_d::{
    prelude::*, AmbientLight, Attenuation, Camera, ClearState, ColorMaterial, CpuMesh, CpuModel,
    FrameInput, FrameOutput, Gm, Mesh, Model, OrbitControl, PhysicalMaterial, SpotLight, Terrain,
    Window, WindowSettings,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).expect("error initializing log");
    log::info!("Hello from Rust!");

    run()?;

    Ok(())
}

fn run() -> Result<(), JsValue> {
    let window = Window::new(WindowSettings {
        title: "WebGL!".to_string(),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();

    let target = vec3(0.0, 0.0, 0.0);
    let scene_radius: f32 = 200.0;
    let mut camera = Camera::new_perspective(
        window.viewport(),
        target + scene_radius * vec3(0.1, 0.0, 0.0).normalize(),
        target,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut controls = OrbitControl::new(target, 100.0, 300.0);

    let noise = Perlin::new((random() * (u32::MAX as f64)) as u32);

    let mut ground = Terrain::new(
        &context,
        PhysicalMaterial {
            albedo: Color::new(230, 225, 250, 255),
            metallic: 0.7,
            roughness: 0.3,
            ..Default::default()
        },
        Arc::new(move |x, y| (noise.get([x as f64 / 100.0, y as f64 / 100.0]) * 100.0) as f32),
        1000.0,
        1.0,
        vec2(0.0, 0.0),
    );

    let model2 = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        ColorMaterial::default(),
    );

    let light = SpotLight::new(
        &context,
        1.0,
        Color::WHITE,
        &vec3(0.0, 50.0, 0.0),
        &vec3(0.0, -1.0, 0.0),
        Rad::full_turn(),
        Attenuation::default(),
    );

    let ambient_light = AmbientLight::new(&context, 0.1, Color::WHITE);

    window.render_loop(move |mut frame_input: FrameInput| {
        camera.set_viewport(frame_input.viewport);
        controls.handle_events(&mut camera, &mut frame_input.events);
        let position = camera.position();
        ground.set_center(vec2(position.x, position.y));

        // let time = (frame_input.accumulated_time / 5000.0) as f32;

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
            .render(&camera, &ground, &[&light, &ambient_light])
            .render(&camera, &model2, &[&light]);

        FrameOutput {
            ..Default::default()
        }
    });

    Ok(())
}
