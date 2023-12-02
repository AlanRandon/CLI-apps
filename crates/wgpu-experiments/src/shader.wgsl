struct Globals {
	size: vec2<f32>,
	window_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coords: vec2<f32>,
}

struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let win_size = globals.window_size;
    let min_axis = max(win_size.x, win_size.y);
    let size = vec2(win_size.y / min_axis, win_size.x / min_axis) * 2.;
    let pos = in.position / globals.size * size - vec2(1.);
    let padding = (vec2(2.) - size) / 2.;

    var out: VertexOutput;
    out.clip_position = vec4(pos + padding, 0., 1.);
    out.tex_coords = in.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.tex_coords);
}
