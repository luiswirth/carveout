struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
  in: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;

  out.clip_position = vec4<f32>(in.position * vec2<f32>(1.0, -1.0), 0.0, 1.0);
  out.tex_coords = in.tex_coords;
  return out;
}


@group(0) @binding(0)
var tex: texture_2d<f32>;
@group(0)@binding(1)
var samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.tex_coords);
}
