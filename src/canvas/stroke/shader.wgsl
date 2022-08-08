struct VertexOutput {
  @location(0) color: vec4<f32>,
  @builtin(position) position: vec4<f32>,
};

struct Globals {
  canvas_to_portal: mat3x2<f32>,
};
@group(0) @binding(0) var<uniform> r_globals: Globals;

@vertex
fn vs_main(
  @location(0) a_pos: vec2<f32>,
  @location(1) a_normal: vec2<f32>,
  @location(2) a_stroke_width: f32,
  @location(3) a_color: vec4<f32>,
) -> VertexOutput {
  var out: VertexOutput;

  let canvas_pos = a_pos + a_normal * a_stroke_width;
  let portal_pos = (r_globals.canvas_to_portal * vec3<f32>(canvas_pos, 1.0)).xy;
  var correction: vec2<f32> = vec2<f32>(2.0, -2.0);
  let portal_pos = correction * portal_pos;

  out.position = vec4<f32>(portal_pos, 0.0, 1.0);
  out.color = a_color;

  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}