struct VertexOutput {
  @location(0) color: vec4<f32>,
  @builtin(position) clip_position: vec4<f32>,
};

struct CameraUniform {
  view_projection: mat3x3<f32>,
};

@group(0) @binding(0)
var<uniform> u_camera: CameraUniform;

@vertex
fn vs_main(
  @location(0) a_pos: vec2<f32>,
  @location(1) a_normal: vec2<f32>,
  @location(2) a_stroke_width: f32,
  @location(3) a_color: vec4<f32>,
) -> VertexOutput {
  var out: VertexOutput;

  let canvas_pos = a_pos + a_normal * a_stroke_width;
  let clip_pos_a = (u_camera.view_projection * vec3<f32>(canvas_pos, 1.0)).xy;
  let clip_pos = vec2<f32>(clip_pos_a.x, -clip_pos_a.y);

  out.clip_position = vec4<f32>(clip_pos, 0.0, 1.0);
  out.color = a_color;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}