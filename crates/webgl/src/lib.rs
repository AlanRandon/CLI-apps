use std::f32::consts::TAU;

use three_d::{
    prelude::*, AmbientLight, Attenuation, Camera, ClearState, CpuMesh, FrameInput, FrameOutput,
    Gm, Mesh, OrbitControl, PhysicalMaterial, Positions, SpotLight, Window, WindowSettings,
};
use wasm_bindgen::prelude::*;

mod positions;

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
    let scene_radius: f32 = 50.0;
    let mut camera = Camera::new_perspective(
        window.viewport(),
        target + scene_radius * vec3(0.1, 0.0, 0.0).normalize(),
        target,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut controls = OrbitControl::new(target, 10.0, 300.0);

    let mut mesh = CpuMesh {
        name: String::from("model"),
        positions: positions::ConeBuilder::new()
            .with_height(10.0)
            .with_radius(3.0)
            .with_sector_count(10)
            .build(),
        ..Default::default()
    };

    mesh.compute_normals();

    let model = Gm::new(
        Mesh::new(&context, &mesh),
        PhysicalMaterial {
            albedo: Color::new(230, 225, 250, 255),
            metallic: 0.5,
            roughness: 0.7,
            ..Default::default()
        },
    );

    let mut light = SpotLight::new(
        &context,
        1.0,
        Color::WHITE,
        &vec3(10.0, 30.0, 10.0),
        &vec3(0.0, -1.0, 0.0),
        Rad::full_turn(),
        Attenuation::default(),
    );

    let ambient_light = AmbientLight::new(&context, 0.1, Color::WHITE);

    window.render_loop(move |mut frame_input: FrameInput| {
        camera.set_viewport(frame_input.viewport);
        controls.handle_events(&mut camera, &mut frame_input.events);

        light.position = vec3(
            (frame_input.accumulated_time / 100.0).sin() as f32 * 15.0,
            30.0,
            (frame_input.accumulated_time / 100.0).cos() as f32 * 15.0,
        );

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
            .render(&camera, &model, &[&light, &ambient_light]);

        FrameOutput {
            ..Default::default()
        }
    });

    Ok(())
}
