#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc
)]

#[cfg(target_arch = "wasm32")]
pub fn main() -> Result<(), wasm_bindgen::JsValue> {
    main::main()
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::missing_panics_doc)]
pub fn main() {
    panic!("Please run me on the web, I do not like this.")
}

#[cfg(target_arch = "wasm32")]
mod positions;

#[cfg(target_arch = "wasm32")]
mod main {
    use super::positions;
    use rand::prelude::*;
    use three_d::{
        prelude::*, AmbientLight, Attenuation, Camera, ClearState, CpuMesh, FrameInput,
        FrameOutput, Gm, Mesh, OrbitControl, PhysicalMaterial, SpotLight, Window, WindowSettings,
    };
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn main() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Trace).expect("error initializing log");
        log::info!("Hello from Rust!");

        run();

        Ok(())
    }

    fn run() {
        let window = Window::new(WindowSettings {
            title: "WebGL!".to_string(),
            ..Default::default()
        })
        .unwrap();

        let context = window.gl();

        let mut camera = Camera::new_perspective(
            window.viewport(),
            vec3(30.0, 0.0, -30.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let mut controls = OrbitControl::new(vec3(0.0, 0.0, 0.0), 10.0, 300.0);

        let material = PhysicalMaterial {
            albedo: Color::new(230, 225, 250, 255),
            metallic: 0.5,
            roughness: 0.7,
            ..Default::default()
        };

        let mut rng = thread_rng();

        let models = (0..200)
            .map(|_| {
                let direction = vec3::<f32>(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                );

                let height = 50.0;

                let mut mesh = CpuMesh {
                    name: String::from("model"),
                    positions: positions::ConeBuilder::new()
                        .with_height(height - 0.0001)
                        .with_radius(1.0)
                        .with_sector_count(10)
                        .with_origin(
                            vec3(-direction.x, -direction.y, -direction.z).normalize() * height,
                        )
                        .with_direction(direction)
                        .build(),
                    ..Default::default()
                };

                mesh.compute_normals();

                Gm::new(Mesh::new(&context, &mesh), material.clone())
            })
            .collect::<Vec<_>>();

        // let axes = Axes::new(&context, 0.5, 10.0);

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
                .render(&camera, &models, &[&light, &ambient_light]);
            // .render(&camera, &axes, &[]);

            FrameOutput {
                ..Default::default()
            }
        });
    }
}
