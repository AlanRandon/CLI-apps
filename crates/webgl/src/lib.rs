use js_sys::Float32Array;
use kiss3d::{
    light::Light,
    nalgebra::{UnitQuaternion, Vector3},
    window::Window,
};
use wasm_bindgen::{prelude::*, JsCast};

struct State {}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        console_log::init_with_level(log::Level::Trace).expect("error initializing log");
        log::info!("Hello from Rust!");
    }

    let mut window = Window::new("Kiss3d: wasm example");
    window.set_background_color(1.0, 1.0, 1.0);
    let mut c = window.add_cube(0.1, 0.1, 0.1);

    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    // Generate the widget identifiers.
    let ids = Ids::new(window.conrod_ui_mut().widget_id_generator());

    let state = State {};

    window.render_loop(state);

    Ok(())
}
