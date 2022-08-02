mod log;
pub mod space;

pub use log::init_log;

pub fn tuple2array4<T>(t: (T, T, T, T)) -> [T; 4] {
  [t.0, t.1, t.2, t.3]
}

#[allow(dead_code)]
pub fn enum_variant_eq<T>(a: &T, b: &T) -> bool {
  std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[allow(dead_code)]
pub fn rgba_palette2egui(palette: palette::LinSrgba) -> egui::color::Rgba {
  let palette = palette::Blend::into_premultiplied(palette);
  egui::color::Rgba::from_rgba_premultiplied(
    palette.red,
    palette.green,
    palette.blue,
    palette.alpha,
  )
}

#[allow(dead_code)]
pub fn rgba_egui2palette(egui: egui::Rgba) -> palette::LinSrgba {
  palette::blend::PreAlpha::from(palette::LinSrgba::new(
    egui.r(),
    egui.g(),
    egui.b(),
    egui.a(),
  ))
  .into()
}

#[allow(dead_code)]
pub fn hsva_palette2egui(palette: palette::Hsva) -> egui::color::Hsva {
  egui::color::Hsva::new(
    palette.hue.to_positive_degrees() / 360.0,
    palette.saturation,
    palette.value,
    palette.alpha,
  )
}

#[allow(dead_code)]
pub fn hsva_egui2palette(egui: egui::color::Hsva) -> palette::Hsva {
  palette::Hsva::new(egui.h * 360.0, egui.s, egui.v, egui.a)
}
