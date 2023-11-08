struct Globals {
	size: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> globals: Globals;

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
}

struct VertexInput {
	@location(0) position: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4(in.position / globals.size * 2. - vec2(1., 1.), 0., 1.);
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4(1.0, 1.0, 1.0, 1.0);
}
