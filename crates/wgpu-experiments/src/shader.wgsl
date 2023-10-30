struct VertexInput {
	@location(0) position: vec2<f32>,
};

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
	in: VertexInput,
	@builtin(instance_index) instance: u32,
) -> VertexOutput {
	var out: VertexOutput;
	var color = f32(instance) / 10.;
	out.color = vec3(color, color, color);
	out.clip_position = vec4(in.position + vec2((f32(instance) - 5.) / 10., 0.), 0., 1.);
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4(in.color, 1.0);
}
