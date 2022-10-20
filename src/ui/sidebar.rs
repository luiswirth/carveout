use super::UiAccess;

use crate::{content::protocol::ProtocolUi, file, pdf::PdfManager, tools::ToolEnum, util};

use egui_file::FileDialog;
use palette::{FromColor, Hsv, IntoColor};

#[derive(Default)]
pub struct SidebarUi {
  rainbow_mode: bool,
  protocol_ui: ProtocolUi,
  protocol_tree_enabled: bool,
  project_file_dialog: Option<FileDialog>,
  pdf_file_dialog: Option<FileDialog>,
}

impl SidebarUi {
  pub fn ui(&mut self, ctx: &egui::Context, ui_access: &mut UiAccess) {
    if let Some(file_dialog) = &mut self.project_file_dialog {
      file_dialog.show(ctx);
      if file_dialog.selected() {
        let file_path = file_dialog.path().unwrap();
        match file_dialog.dialog_type() {
          egui_file::DialogType::OpenFile => {
            let savefile = file::load(&file_path);
            ui_access
              .content_manager
              .replace(savefile.content, savefile.protocol);
          }
          egui_file::DialogType::SaveFile => {
            let (content, protocol) = ui_access.content_manager.clone();
            let savefile = file::Savefile { content, protocol };
            file::save(&savefile, file_path);
          }
          _ => unreachable!(),
        }
      }
    }

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(file_dialog) = &mut self.pdf_file_dialog {
      file_dialog.show(ctx);
      if file_dialog.selected() {
        let file_path = file_dialog.path().unwrap();
        match file_dialog.dialog_type() {
          egui_file::DialogType::OpenFile => {
            *ui_access.pdf_manager = Some(PdfManager::load_document(file_path));
          }
          _ => unreachable!(),
        }
      }
    }

    egui::SidePanel::left("toolbox_panel").show(ctx, |ui| {
      ui.add_space(10.0);
      ui.add(egui::Label::new(
        egui::RichText::new("📦 Toolbox").text_style(egui::TextStyle::Heading),
      ));
      ui.add_space(10.0);

      ui.group(|ui| {
        ui.label("Project File");
        ui.horizontal_wrapped(|ui| {
          if ui.button("📂").clicked() {
            let mut file_dialog =
              FileDialog::open_file(Some(util::USER_DIRS.home_dir().to_owned()));
            file_dialog.open();
            self.project_file_dialog = Some(file_dialog);
          }
          if ui.button("🗄").clicked() {
            let mut file_dialog =
              FileDialog::save_file(Some(util::USER_DIRS.home_dir().to_owned()));
            file_dialog.open();
            self.project_file_dialog = Some(file_dialog);
          }
        });

        ui.separator();

        ui.label("PDF File");
        ui.horizontal_wrapped(|ui| {
          if ui.button("📂").clicked() {
            let mut file_dialog =
              FileDialog::open_file(Some(util::USER_DIRS.home_dir().to_owned()));
            file_dialog.open();
            self.pdf_file_dialog = Some(file_dialog);
          }
        });
      });

      ui.group(|ui| {
        ui.label("Undo");
        let content = &mut ui_access.content_manager;
        ui.horizontal_wrapped(|ui| {
          let undoable = content.undoable();
          let button = egui::Button::new("⮪");
          let response = ui.add_enabled(undoable, button);
          if undoable && response.clicked() {
            content.undo_cmd();
          }

          let redoable = content.redoable();
          let button = egui::Button::new("⮫");
          let response = ui.add_enabled(redoable, button);
          if redoable && response.clicked() {
            content.redo_cmd();
          }
        });

        ui.checkbox(&mut self.protocol_tree_enabled, "Show Protocol");
        if self.protocol_tree_enabled {
          egui::Window::new("Protocol")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| self.protocol_ui.ui(ui, content));
        }
      });

      ui.group(|ui| {
        ui.label("Tools");
        let selected = &mut ui_access.tool_manager.selected;

        ui.horizontal_wrapped(|ui| {
          selectable_tool(ui, selected, ToolEnum::Pen, "✏");
          selectable_tool(ui, selected, ToolEnum::Eraser, "📙");
          selectable_tool(ui, selected, ToolEnum::SelectLoop, "➰");
          selectable_tool(ui, selected, ToolEnum::Translate, "✋");
          selectable_tool(ui, selected, ToolEnum::Rotate, "🔄");
          selectable_tool(ui, selected, ToolEnum::Zoom, "🔍");
        });

        ui.separator();
        match selected {
          ToolEnum::Pen => {
            let mut pen = &mut ui_access.tool_manager.configs.pen;

            ui.label("Pen color");
            let color = pen.color.into_components();
            let mut color = [color.0, color.1, color.2, color.3];
            ui.color_edit_button_rgba_unmultiplied(&mut color);
            pen.color = palette::LinSrgba::new(color[0], color[1], color[2], color[3]);

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
          ToolEnum::Eraser => {}
          ToolEnum::SelectLoop => {}
          ToolEnum::Translate => {
            ui.label("Translate options");
            let position = &mut ui_access.spaces.camera_mut().position_canvas;
            ui.vertical(|ui| {
              const SPEED: f32 = 0.001;
              ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, "X:");
                ui.add(egui::DragValue::new(&mut position.x).speed(SPEED));
              });
              ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::BLUE, "Y:");
                ui.add(egui::DragValue::new(&mut position.y).speed(SPEED));
              });
            });
          }
          ToolEnum::Rotate => {
            ui.label("Rotate options");
            let rotation = &mut ui_access.spaces.camera_mut().angle;
            ui.add(egui::Slider::new(rotation, 0.0..=std::f32::consts::TAU));
          }
          ToolEnum::Zoom => {
            ui.label("Scale options");
            let zoom = &mut ui_access.spaces.camera_mut().zoom;
            const SPEED_MUL: f32 = 0.003;
            let speed = *zoom * SPEED_MUL;
            ui.add(
              egui::DragValue::new(zoom)
                .clamp_range(0.1..=10.0)
                .speed(speed),
            );
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
