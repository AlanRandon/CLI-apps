use three_d::{
    prelude::*, AmbientLight, Attenuation, Camera, ClearState, CpuMesh, FrameInput, FrameOutput,
    Gm, Mesh, PhysicalMaterial, SpotLight, Window, WindowSettings,
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
    let scene_radius: f32 = 6.0;
    let mut camera = Camera::new_perspective(
        window.viewport(),
        target + scene_radius * vec3(0.1, 0.1, 0.1).normalize(),
        target,
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );

    let cpu_mesh = CpuMesh::cube();

    let mut model = Gm::new(
        Mesh::new(&context, &cpu_mesh),
        PhysicalMaterial {
            albedo: Color::new(0, 200, 150, 255),
            ..Default::default()
        },
    );

    let light = SpotLight::new(
        &context,
        0.7,
        Color::WHITE,
        &vec3(0.0, 1.5, 0.0),
        &vec3(0.0, -1.0, 0.0),
        Rad::full_turn(),
        Attenuation::default(),
    );

    let ambient_light = AmbientLight::new(&context, 0.1, Color::WHITE);

    let turn = Rad::full_turn();

    window.render_loop(move |frame_input: FrameInput| {
        camera.set_viewport(frame_input.viewport);

        let time = (frame_input.accumulated_time / 5000.0) as f32;

        let turn = Matrix4::from_angle_x(turn * time)
            * Matrix4::from_angle_y(turn * time)
            * Matrix4::from_angle_z(turn * time);

        model.set_transformation(turn);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
            .render(&camera, &model, &[&light, &ambient_light]);

        FrameOutput {
            ..Default::default()
        }
    });

    Ok(())
}
