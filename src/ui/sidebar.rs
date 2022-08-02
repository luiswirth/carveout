use palette::{FromColor, Hsv, IntoColor};

use crate::canvas::tool::{ToolConfig, ToolEnum};

pub struct SidebarUi {
  rainbow_mode: bool,
}

impl SidebarUi {
  pub fn init() -> Self {
    Self {
      rainbow_mode: false,
    }
  }

  pub fn build_ui(&mut self, ctx: &egui::Context, tool_config: &mut ToolConfig) {
    egui::SidePanel::left("toolbox_panel").show(ctx, |ui| {
      ui.add_space(10.0);
      ui.add(egui::Label::new(
        egui::RichText::new("📦 Toolbox").text_style(egui::TextStyle::Heading),
      ));
      ui.add_space(20.0);

      ui.group(|ui| {
        ui.label("Tools");

        ui.horizontal_wrapped(|ui| {
          selectable_tool(ui, &mut tool_config.selected, ToolEnum::Pen, "✏");
          selectable_tool(ui, &mut tool_config.selected, ToolEnum::Translate, "✋");
          selectable_tool(ui, &mut tool_config.selected, ToolEnum::Scale, "🔍");
        });

        ui.separator();

        match tool_config.selected {
          ToolEnum::Pen => {
            let mut pen = &mut tool_config.pen;

            ui.label("Pen color");
            let color = pen.color.into_components();
            let mut color = [color.0, color.1, color.2];
            ui.color_edit_button_rgb(&mut color);
            pen.color = palette::LinSrgb::new(color[0], color[1], color[2]);

            ui.checkbox(&mut self.rainbow_mode, "Rainbow mode");
            if self.rainbow_mode {
              let mut hsv = Hsv::from_color(pen.color);
              hsv.hue += 2.0;
              hsv.saturation = 1.0;
              hsv.value = 1.0;
              pen.color = hsv.into_color();
            }

            ui.label("Pen width");
            ui.add(egui::Slider::new(&mut pen.width, 0.1..=10.0));
          }
          ToolEnum::Scale => {
            ui.label("Scale options");
            //let mut scale = CONFIG.lock().unwrap().canvas_viewport.scale;
            //const SPEED_MUL: f32 = 0.003;
            //let speed = scale * SPEED_MUL;
            //ui.add(
            //  egui::DragValue::new(&mut scale)
            //    .clamp_range(0.1..=10.0)
            //    .speed(speed),
            //);
            //CONFIG.lock().unwrap().canvas_viewport.scale = scale;
          }
          ToolEnum::Translate => {
            ui.label("Translate option");
            //let mut translate = CONFIG.lock().unwrap().canvas_viewport.translate;
            //ui.horizontal(|ui| {
            //  const SPEED: f32 = 0.001;
            //  ui.colored_label(egui::Color32::RED, "X:");
            //  ui.add(egui::DragValue::new(&mut translate[0]).speed(SPEED));
            //  ui.colored_label(egui::Color32::BLUE, "Y:");
            //  ui.add(egui::DragValue::new(&mut translate[1]).speed(SPEED));
            //});
            //CONFIG.lock().unwrap().canvas_viewport.translate = translate;
          }
        }
      });
    });
  }
}

fn selectable_tool(ui: &mut egui::Ui, selected: &mut ToolEnum, selectable: ToolEnum, text: &str) {
  if ui
    .add(egui::SelectableLabel::new(*selected == selectable, text))
    .clicked()
  {
    *selected = selectable;
  }
}
