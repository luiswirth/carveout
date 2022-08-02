/// Owned [`wgpu::Label`].
pub type OwnedLabel = Option<String>;

/// [`wgpu::util::BufferInitDescriptor`] but with an additional `size` field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferInitDescriptor<'a> {
  /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
  pub label: wgpu::Label<'a>,
  /// Contents of a buffer on creation.
  pub contents: &'a [u8],
  /// Size of the buffer. Must be at least size of `contents`. If unspecified, the size of `contents` is used.
  pub size: Option<wgpu::BufferAddress>,
  /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
  /// will panic.
  pub usage: wgpu::BufferUsages,
}

/// Extension trait for [`wgpu::Device`].
pub trait DeviceExt {
  /// [`wgpu::util::DeviceExt::create_buffer_init`] but for this [`BufferInitDescriptor`]
  /// (includes size field).
  fn create_buffer_init(&self, desc: &BufferInitDescriptor<'_>) -> wgpu::Buffer;
}

impl DeviceExt for wgpu::Device {
  fn create_buffer_init(&self, descriptor: &BufferInitDescriptor<'_>) -> wgpu::Buffer {
    let contents_size = descriptor.contents.len() as wgpu::BufferAddress;
    let unpadded_size = match descriptor.size {
      None => contents_size,
      Some(specified_size) => {
        assert!(
          specified_size >= contents_size,
          "specified size must at least be size of contents"
        );
        specified_size
      }
    };

    if unpadded_size == 0 {
      let wgt_descriptor = wgpu::BufferDescriptor {
        label: descriptor.label,
        size: 0,
        usage: descriptor.usage,
        mapped_at_creation: false,
      };

      self.create_buffer(&wgt_descriptor)
    } else {
      // Valid vulkan usage is
      // 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
      // 2. buffer size must be greater than 0.
      // Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
      let align_mask = wgpu::COPY_BUFFER_ALIGNMENT - 1;
      let padded_size =
        ((unpadded_size + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT);

      let wgt_descriptor = wgpu::BufferDescriptor {
        label: descriptor.label,
        size: padded_size,
        usage: descriptor.usage,
        mapped_at_creation: true,
      };

      let buffer = self.create_buffer(&wgt_descriptor);

      buffer.slice(..).get_mapped_range_mut()[..contents_size as usize]
        .copy_from_slice(descriptor.contents);
      buffer.unmap();

      buffer
    }
  }
}

/// A [`wgpu::Buffer`] which dynamically grows based on the contents.
#[derive(Debug)]
pub struct DynamicBuffer {
  raw: wgpu::Buffer,

  label: OwnedLabel,
  size: wgpu::BufferAddress,
  usage: wgpu::BufferUsages,
}

impl DynamicBuffer {
  /// Create a new empty minimal buffer.
  pub fn new(device: &wgpu::Device, descriptor: &wgpu::BufferDescriptor) -> Self {
    let raw = device.create_buffer(descriptor);

    Self {
      raw,
      label: descriptor.label.map(|l| l.to_owned()),
      size: descriptor.size,
      usage: descriptor.usage,
    }
  }

  /// Uploads `contents` and resizes the buffer if needed.
  ///
  /// If `contents` fits, uploads using [`wgpu::Queue`], otherwise reallocates and uploads using
  /// [`wgpu::Device`].
  pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, contents: &[u8]) {
    if self.try_upload(queue, contents).is_err() {
      self.upload_by_init(device, contents)
    }
  }

  /// Uploades `data` using [`wgpu::Queue`] without resizing.
  /// Fails if `data` doesn't fit in buffers and returns the size difference.
  pub fn try_upload(
    &mut self,
    queue: &wgpu::Queue,
    contents: &[u8],
  ) -> Result<(), wgpu::BufferAddress> {
    if contents.is_empty() {
      return Ok(());
    }
    let contents_size = contents.len() as wgpu::BufferAddress;
    if contents_size < self.size {
      queue.write_buffer(&self.raw, 0, contents);
      Ok(())
    } else {
      Err(contents_size - self.size)
    }
  }

  /// Allocates a new buffer, replaces the old one and uploades the contents using
  /// [`wgpu::Device`].
  pub fn upload_by_init(&mut self, device: &wgpu::Device, contents: &[u8]) {
    while self.size < contents.len() as u64 {
      self.size *= 2;
    }
    self.raw = device.create_buffer_init(&BufferInitDescriptor {
      label: self.label.as_deref(),
      contents,
      usage: self.usage,
      size: Some(self.size),
    });
  }

  /// Get a reference to the raw buffer.
  pub fn raw(&self) -> &wgpu::Buffer {
    &self.raw
  }
}
