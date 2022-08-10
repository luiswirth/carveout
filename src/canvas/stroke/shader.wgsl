struct VertexOutput {
  @location(0) color: vec4<f32>,
  @builtin(position) clip_position: vec4<f32>,
};

struct PortalUniform {
  canvas_to_portal: mat3x2<f32>,
};

@group(0) @binding(0)
var<uniform> u_portal: PortalUniform;

@vertex
fn vs_main(
  @location(0) a_pos: vec2<f32>,
  @location(1) a_normal: vec2<f32>,
  @location(2) a_stroke_width: f32,
  @location(3) a_color: vec4<f32>,
) -> VertexOutput {
  var out: VertexOutput;

  let canvas_pos = a_pos + a_normal * a_stroke_width;
  let portal_pos = (u_portal.canvas_to_portal * vec3<f32>(canvas_pos, 1.0)).xy;
  // TODO: replace this with OPENGL_TO_WGPU_MATRIX
  var correction: vec2<f32> = vec2<f32>(2.0, -2.0);
  let portal_pos = correction * portal_pos;

  out.clip_position = vec4<f32>(portal_pos, 0.0, 1.0);
  out.color = a_color;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}