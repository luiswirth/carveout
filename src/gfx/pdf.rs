use crate::{
  math::Rect,
  natrans,
  pdf::PdfManager,
  spaces::{Camera, Space, SpaceManager},
};

use pdfium_render::prelude::*;
use std::mem;
use wgpu::util::DeviceExt;

// from `PdfPagePaperStandardSize::A4.width()`
const A4_WIDTH_PDF_POINTS: f32 = 210.0;
const CANVAS_UNITS_PER_PDF_POINT: f32 = 2.0 / A4_WIDTH_PDF_POINTS;

pub struct PdfRenderer {
  pipeline: wgpu::RenderPipeline,
  bind_group_layout: wgpu::BindGroupLayout,
  bind_group: Option<wgpu::BindGroup>,
  index_buffer: wgpu::Buffer,
  vertex_buffer: wgpu::Buffer,

  prev_camera: Camera,
  prev_screen_rect: Rect,
  static_since_nframes: u32,
}

impl PdfRenderer {
  pub fn init(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
    // a rectangle consiting of two triangles
    const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("pdf_vertex_buffer"),
      contents: bytemuck::cast_slice(&INDICES),
      usage: wgpu::BufferUsages::INDEX,
    });

    // a screen norm rectangle constiting of two triangles
    const VERTICES: [PdfVertex; 4] = [
      PdfVertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 0.0],
      },
      PdfVertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 1.0],
      },
      PdfVertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 1.0],
      },
      PdfVertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 0.0],
      },
    ];
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("pdf_vertex_buffer"),
      contents: bytemuck::cast_slice(&VERTICES),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("pdf_renderer_bind_group_layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let bind_group = None;

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("pdf_renderer_pipeline_layout"),
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let fragment_targets = &[Some(wgpu::ColorTargetState {
      format,
      blend: Some(wgpu::BlendState::ALPHA_BLENDING),
      write_mask: wgpu::ColorWrites::ALL,
    })];

    let shader = device.create_shader_module(wgpu::include_wgsl!("pdf/shader.wgsl"));

    let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
      label: Some("pdf_render_pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[PdfVertex::vertex_buffer_layout()],
      },
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: wgpu::PolygonMode::Fill,
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: fragment_targets,
      }),
      multiview: None,
    };
    let pipeline = device.create_render_pipeline(&pipeline_descriptor);

    let prev_camera = Camera::default();
    let prev_screen_rect = Rect::default();
    let static_since_nframes = 0;

    Self {
      pipeline,
      bind_group_layout,
      bind_group,
      index_buffer,
      vertex_buffer,

      prev_camera,
      prev_screen_rect,
      static_since_nframes,
    }
  }

  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    spaces: &SpaceManager,
    pdf_manager: &PdfManager,
  ) {
    let screen_size_physical = spaces.transform_vector(
      spaces.screen_rect_window_logical().size(),
      Space::WindowLogical,
      Space::WindowPhysical,
    );

    let is_camera_and_screen_static = self.prev_camera == *spaces.camera()
      && self.prev_screen_rect == spaces.screen_rect_window_logical();
    if is_camera_and_screen_static {
      self.static_since_nframes += 1;
    } else {
      self.static_since_nframes = 0;
    }

    const NIMPROVEMENTS: u32 = 4;
    const NFRAMES_BETWEEN_IMPROVEMENTS: u32 = 5;
    const NFRAMES_ALL_IMPROVEMENTS: u32 = (NIMPROVEMENTS - 1) * NFRAMES_BETWEEN_IMPROVEMENTS;
    const MIN_RESOLUTION: f32 = 0.25;
    const RESOLUTION_STEP_SIZE: f32 = (1.0 - MIN_RESOLUTION) / NIMPROVEMENTS as f32;

    let (resolution_factor, has_resolution_increased) =
      match self.static_since_nframes % NFRAMES_BETWEEN_IMPROVEMENTS == 0
        && self.static_since_nframes <= NFRAMES_ALL_IMPROVEMENTS
      {
        true => (
          (self.static_since_nframes / NFRAMES_BETWEEN_IMPROVEMENTS + 1) as f32
            * RESOLUTION_STEP_SIZE,
          true,
        ),
        false => (1.0, false),
      };

    let texture_size_physical = screen_size_physical * resolution_factor;
    let texture_width_u16 = texture_size_physical.x as u16;
    let texture_height_u16 = texture_size_physical.y as u16;
    let texture_width_u32 = texture_width_u16 as u32;
    let texture_height_u32 = texture_height_u16 as u32;

    let was_transform_updated = !is_camera_and_screen_static || has_resolution_increased;
    self.prev_camera = spaces.camera().clone();
    self.prev_screen_rect = spaces.screen_rect_window_logical();

    let screen_rect_canvas = spaces.transform_rect(
      spaces.screen_rect_window_logical(),
      Space::WindowLogical,
      Space::Canvas,
    );

    let texture_nbytes = 4 * texture_width_u32 as usize * texture_height_u32 as usize;
    let mut texture_data = vec![0u8; texture_nbytes];
    let mut page_center_canvas = na::Point2::origin();
    let mut was_a_page_rendered = false;
    pdf_manager.page_slice().iter().for_each(|page| {
      let page_size_canvas = na::vector![
        page.width().value * CANVAS_UNITS_PER_PDF_POINT,
        page.height().value * CANVAS_UNITS_PER_PDF_POINT
      ];
      let page_rect_canvas = Rect::from_size_center(page_size_canvas, page_center_canvas);

      let is_visible = parry2d::query::intersection_test(
        &page_rect_canvas.isometry(),
        &page_rect_canvas.shape(),
        &screen_rect_canvas.isometry(),
        &screen_rect_canvas.shape(),
      )
      .unwrap();
      let is_visible = true;

      let should_render = is_visible && (was_transform_updated || has_resolution_increased);
      if should_render {
        was_a_page_rendered = true;

        let transform = page_to_texture_renderer_transform(
          &page,
          page_center_canvas,
          texture_size_physical,
          spaces,
        );
        page_center_canvas.y += page_size_canvas.y * 1.05;

        let render_config = PdfRenderConfig::default()
          .clear_before_rendering(false)
          .set_target_size(texture_width_u16, texture_height_u16)
          .set_maximum_width(texture_width_u16)
          .set_maximum_height(texture_height_u16)
          .transform(
            transform.m11,
            transform.m21,
            transform.m12,
            transform.m22,
            transform.m13,
            transform.m23,
          )
          .unwrap();

        // TODO: stop recreating bitmap
        let mut bitmap = PdfBitmap::empty(
          texture_width_u16,
          texture_height_u16,
          PdfBitmapFormat::default(),
          page.get_bindings(),
        )
        .unwrap();
        page
          .render_into_bitmap_with_config(&mut bitmap, &render_config)
          .unwrap();

        texture_data
          .iter_mut()
          .zip(bitmap.as_bytes())
          .for_each(|(store, new)| *store = store.saturating_add(*new));

        assert_eq!(texture_data.len(), bitmap.as_bytes().len());
      }
    });

    if was_a_page_rendered {
      let size = wgpu::Extent3d {
        width: texture_width_u32,
        height: texture_height_u32,
        depth_or_array_layers: 1,
      };
      let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pdf_page"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
      });
      queue.write_texture(
        texture.as_image_copy(),
        &texture_data,
        wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: std::num::NonZeroU32::new(4 * size.width),
          rows_per_image: std::num::NonZeroU32::new(size.height),
        },
        size,
      );

      let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
      let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

      self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pdf_renderer_bind_group"),
        layout: &self.bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&sampler),
          },
        ],
      }));
    }
  }

  pub fn render<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
    if let Some(bind_group) = self.bind_group.as_ref() {
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

      render_pass.set_bind_group(0, bind_group, &[]);
      render_pass.draw_indexed(0..6, 0, 0..1);
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PdfVertex {
  pub position: [f32; 2],
  pub tex_coords: [f32; 2],
}
impl PdfVertex {
  const LAYOUT_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

  fn vertex_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &Self::LAYOUT_ATTRIBUTES,
    }
  }
}

fn page_to_texture_renderer_transform(
  page: &PdfPage,
  page_center_canvas: na::Point2<f32>,
  texture_size_physical: na::Vector2<f32>,
  spaces: &SpaceManager,
) -> na::Matrix3<f32> {
  let page_anchor = na::Point2::new(page.width().value / 2.0, page.height().value / 2.0);
  let page_to_canvas = natrans!(na::Translation2::from(page_center_canvas))
    * natrans!(na::Scale2::new(
      CANVAS_UNITS_PER_PDF_POINT,
      CANVAS_UNITS_PER_PDF_POINT
    ))
    * natrans!(na::Translation::from(-page_anchor));

  let screen_norm_to_texture = {
    let translation = na::Translation2::new(1.0, 1.0);
    let scale = na::Scale2::new(texture_size_physical.x / 2.0, texture_size_physical.y / 2.0);
    natrans!(scale) * natrans!(translation)
  };

  // the transformation we want to do from page to the texture
  let page_to_texture = screen_norm_to_texture
    * natrans!(spaces.canvas_view_to_screen_norm())
    * natrans!(spaces.canvas_to_view())
    * page_to_canvas;

  // for reverting what the pdf renderer is going to do automatically
  let page_to_texture_scale = natrans!(na::Scale2::new(
    page.width().value / texture_size_physical.x,
    page.height().value / texture_size_physical.y
  ));
  // the transform to give to the renderer
  let transform = page_to_texture_scale * page_to_texture;
  transform.to_homogeneous()
}
