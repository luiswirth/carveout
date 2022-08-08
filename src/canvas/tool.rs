use palette::LinSrgb;

#[derive(Default)]
pub struct ToolConfig {
  pub selected: ToolEnum,
  pub pen: PenConfig,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ToolEnum {
  #[default]
  Pen,
  Translate,
  Rotate,
  Scale,
}

#[derive(Clone)]
pub struct PenConfig {
  pub width: f32,
  pub color: LinSrgb,
}
impl Default for PenConfig {
  fn default() -> Self {
    Self {
      width: 1.0,
      color: palette::named::WHITE.into_format().into_linear(),
    }
  }
}
