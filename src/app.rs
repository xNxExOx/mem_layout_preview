use egui::{Align2, Color32, FontId, Pos2, RichText, Sense, Vec2};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {

    //#[serde(skip)] // This how you opt-out of serialization of a field
    //value: f32,
    fields: Vec<FieldSize>,
}

#[derive(serde::Deserialize, serde::Serialize, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
enum FieldSize {
    U8, U16, U32, U64, U128
}

impl FieldSize {
    fn to_str(&self) -> &'static str {
        match self {
            FieldSize::U8 => "u8",
            FieldSize::U16 => "u16",
            FieldSize::U32 => "u32",
            FieldSize::U64 => "u64",
            FieldSize::U128 => "u128",
        }
    }
    fn all() -> &'static [FieldSize] {
        &[FieldSize::U8, FieldSize::U16, FieldSize::U32, FieldSize::U64, FieldSize::U128]
    }

    fn size(&self) -> usize {
        match self {
            FieldSize::U8 => 1,
            FieldSize::U16 => 2,
            FieldSize::U32 => 4,
            FieldSize::U64 => 8,
            FieldSize::U128 => 16,
        }
    }

    fn total_struct_size(fields: &[FieldSize]) -> usize {
        let mut max_size = 0;
        let mut offset = 0;
        for f in fields.iter() {
            max_size = max_size.max(f.size());
            let align = offset % f.size();
            if align != 0 {
                offset += f.size() - align;
            }
            offset += f.size();
        }

        let align = offset % max_size;
        if align == 0 {
            offset
        } else {
            offset + max_size - align
        }
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            fields: vec![FieldSize::U8, FieldSize::U128],
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            const LEVELS : &[&str] = &[
                "u8",
                "u16",
                "u32",
                "u64",
                "u128",
            ];
            const CELL_WIDTH: f32 = 50.0;
            let c1 =  ui.style().visuals.widgets.active.bg_fill;
            let c2 =  ui.style().visuals.widgets.noninteractive.bg_fill;
            let text_color = ui.style().visuals.widgets.active.fg_stroke.color;

            egui::scroll_area::ScrollArea::horizontal().show_viewport(ui, |ui, rect|{
                ui.allocate_space(Vec2::new((1 << 20) as f32 * CELL_WIDTH, (LEVELS.len() + 2) as f32 * CELL_WIDTH));
                let start = rect.min.x;
                let end = rect.max.x - rect.min.x;

                for (l, level) in LEVELS.iter().enumerate() {
                    let s = 1 << l;
                    let cw = s as f32 * CELL_WIDTH;
                    let i = ((start - cw) / cw).round();
                    let mut start = i * cw - start;
                    let y = l as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
                    let y2 = (l+1) as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
                    let mut i = i as i32;
                    loop {
                        let x = start;
                        start += cw;
                        let x2 = start;
                        let rect = egui::Rect{
                            min: Pos2::new(x, y), max: Pos2::new(x2, y2) };
                        let painter = ui.painter_at(rect);

                        let color = if i & 1 == 0 {
                            c1
                        } else {
                            c2
                        };
                        painter.rect(rect, 0., color, egui::Stroke::NONE, egui::StrokeKind::Middle);
                        let text = format!("{level} {i}");
                        let size = if i < 1000 || l > 0 {
                            12.
                        } else if i < 10000 {
                            11.
                        } else if i < 100000 {
                            10.
                        } else if i < 1000000 {
                            9.5
                        } else {
                            9.
                        };
                        painter.text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(size), text_color);
                        ui.allocate_rect(rect, Sense::hover()).on_hover_ui(|ui| {
                            ui.label(format!("address: {}", i * s));
                        });
                        i += 1;
                        if x2 > end {
                            break;
                        }
                    }
                }

                let struct_size = FieldSize::total_struct_size(&self.fields);
                let s = struct_size as i32;
                let cw = s as f32 * CELL_WIDTH;
                let i = ((start - cw) / cw).round();
                let mut start = i * cw - start;
                let y = LEVELS.len() as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
                let y2 = (LEVELS.len()+1) as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
                let y3 = (LEVELS.len()+2) as f32 * CELL_WIDTH + CELL_WIDTH / 2.0;
                let mut i = i as i32;
                loop {
                    let x = start;
                    start += cw;
                    let x2 = start;


                    let color = if i & 1 == 0 {
                        c1
                    } else {
                        c2
                    };
                    {
                        let rect = egui::Rect{
                            min: Pos2::new(x, y), max: Pos2::new(x2, y2) };
                        let painter = ui.painter_at(rect);
                        painter.rect(rect, 0., color, egui::Stroke::NONE, egui::StrokeKind::Middle);
                        let text = format!("MyStruct {i}");
                        painter.text(rect.center(), Align2::CENTER_CENTER, text, FontId::proportional(12.), text_color);
                        ui.allocate_rect(rect, Sense::hover()).on_hover_ui(|ui| {
                            ui.label(format!("address: {}", i * s));
                        });
                    }

                    let mut max_size = 0;
                    let mut offset = 0;

                    for (fi, f) in self.fields.iter_mut().enumerate() {
                        max_size = max_size.max(f.size());
                        let align = offset % f.size();
                        if align != 0 {
                            let rect = egui::Rect{
                                min: Pos2::new(x + CELL_WIDTH * offset as f32, y2),
                                max: Pos2::new(x + CELL_WIDTH * (offset + f.size() - align) as f32, y3) };
                            let painter = ui.painter_at(rect);
                            painter.rect(rect, 0., Color32::RED, egui::Stroke::NONE, egui::StrokeKind::Middle);
                            offset += f.size() - align;
                        }

                        let mut _backing_field = [0; 4];
                        let name = &*(('a' as u8 + fi as u8) as char).encode_utf8(&mut _backing_field);

                        {
                            let rect = egui::Rect{
                                min: Pos2::new(x + CELL_WIDTH * offset as f32, y2),
                                max: Pos2::new(x + CELL_WIDTH * (offset + f.size()) as f32, y3) };
                            let painter = ui.painter_at(rect);
                            painter.rect(rect, 0., color, egui::Stroke::new(1., Color32::BLUE), egui::StrokeKind::Middle);
                            painter.text(rect.center(), Align2::CENTER_CENTER, name, FontId::proportional(12.), text_color);
                            ui.allocate_rect(rect, Sense::hover()).on_hover_ui(|ui| {
                                ui.label(format!("address: {}", i * s + offset as i32));
                            });
                        }
                        offset += f.size();
                    }
                    let align = offset % max_size;
                    if align != 0 {
                        let rect = egui::Rect{
                            min: Pos2::new(x + CELL_WIDTH * offset as f32, y2),
                            max: Pos2::new(x + CELL_WIDTH * (offset + max_size - align) as f32, y3) };
                        let painter = ui.painter_at(rect);
                        painter.rect(rect, 0., Color32::RED, egui::Stroke::NONE, egui::StrokeKind::Middle);
                    }

                    i += 1;
                    if x2 > end {
                        break;
                    }
                }

            });

            ui.group(|ui| {
                ui.label("#[repr(C)]");
                ui.label("struct MyStruct {");
                ui.indent(0, |ui|{
                    let mut max_size = 0;
                    let mut offset = 0;
                    let mut remove = None;
                    for (i, f) in self.fields.iter_mut().enumerate() {
                        max_size = max_size.max(f.size());
                        let align = offset % f.size();
                        if align != 0 {
                            ui.label(format!("_ : [u8;{}], // necessary padding for alignment", f.size() - align));
                            offset += f.size() - align;
                        }
                        offset += f.size();
                        ui.horizontal(|ui| {
                            let mut _backing_field = [0; 4];
                            let name = &*(('a' as u8 + i as u8) as char).encode_utf8(&mut _backing_field);
                            ui.label(name);
                            ui.label(": ");
                            egui::ComboBox::new(name, "")
                                .selected_text(f.to_str())
                                .show_ui(ui, |ui| {
                                    for n in FieldSize::all().iter().cloned() {
                                        ui.selectable_value(f, n, n.to_str());
                                    }
                                });
                            ui.label(", // ");
                            if ui.button(RichText::new("-").color(Color32::RED)).clicked() {
                                remove = Some(i);
                            }
                        });
                    }
                    let align = offset % max_size;
                    if align != 0 {
                        ui.label(format!("_ : [u8;{}], // necessary padding for alignment", max_size - align));
                    }

                    if ui.button(RichText::new("+").color(Color32::GREEN)).clicked() {
                        self.fields.push(FieldSize::U8);
                    }
                    if let Some(r) = remove {
                        self.fields.remove(r);
                    }
                });
                ui.label("}");
            });


            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
