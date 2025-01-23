// use super::color_palette::*;
use eframe::egui;
use serde_json;
use super::color_palette::*;
use super::structs::*;

pub fn show_ui(app: PlannerApp) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            window_level: Some(egui::viewport::WindowLevel::AlwaysOnTop),
            maximize_button: Some(false),
            resizable: Some(false),
            inner_size: Some(egui::vec2(257.0, 832.0)),
            ..Default::default()
        },
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Daily Planner",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

pub struct PlannerApp {
    plans: Vec<Plan>,
    last_update: SimpleTime,
    add_plan_window_open: bool,
    close_add_plan_window: bool,
    new_plan_name: String,
    new_plan_start_time: (u8, u8),
    new_plan_end_time: (u8, u8),
    selected_plan_id_for_delete: Option<u32>,
    confirm_delete_plan_window_open: bool,
    close_update_plan_window: bool,
}

impl PlannerApp {
    pub fn new() -> std::io::Result<Self> {

        // Load plans from file
        let mut plans: Vec<Plan> = if let Ok(plans_json) = std::fs::read_to_string("plans.json") {
            if let Ok(plans) = serde_json::from_str(&plans_json) {
                plans
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for plan in &mut plans {
            plan.update_now();
        }

        Ok(Self {
            plans,
            last_update: SimpleTime::from_now(),
            add_plan_window_open: false,
            close_add_plan_window: false,
            new_plan_name: "".to_string(),
            new_plan_start_time: (0, 0),
            new_plan_end_time: (0, 0),
            selected_plan_id_for_delete: None,
            confirm_delete_plan_window_open: false,
            close_update_plan_window: false,
        })
    }

    fn update_plans_every_ten_seconds(&mut self) {
        // Update the plans after 10 seconds from last update
        let now = SimpleTime::from_now();
        if now.as_seconds() - self.last_update.as_seconds() >= 10 {
            self.last_update = now;
            for plan in &mut self.plans {
                plan.update_now();
            }
        }
    }
}

impl eframe::App for PlannerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_plans_every_ten_seconds();

        let visuals = egui::Visuals {
            panel_fill: DARK_GREY,
            window_fill: DARK_GREY,
            ..Default::default()
        };
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            let spacing = ui.spacing_mut();
            spacing.item_spacing = egui::vec2(10.0, 10.0);
            spacing.button_padding = egui::vec2(10.0, 5.0);

            // Add Plan / Save Plans buttons
            ui.horizontal(|ui| {
                if ui.button("New Plan").clicked() {
                    self.add_plan_window_open = true;
                }
                if ui.button("Save Plans").clicked() {
                    let plans_json = serde_json::to_string(&self.plans).unwrap();
                    std::fs::write("plans.json", plans_json).expect("Failed to save plans");
                }
                if ui.button("Delete All").clicked() {
                    // just delete all plans and delete the file too
                    self.plans = vec![];
                    _ = std::fs::remove_file("plans.json");
                }
            });

            if self.close_add_plan_window {
                self.add_plan_window_open = false;
                self.close_add_plan_window = false;
            }

            if self.add_plan_window_open {
                egui::Window::new("New Plan")
                    .default_size(egui::vec2(140.0, 70.0))
                    .title_bar(false)
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut self.add_plan_window_open)
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_plan_name);
                        });
                        ui.horizontal(|ui| {
                            time_picker(ui, "Start:", &mut self.new_plan_start_time, "new_plan_start_time");
                        });
                        ui.horizontal(|ui| {
                            time_picker(ui, "End:", &mut self.new_plan_end_time, "new_plan_end_time");
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("Add").clicked() {
                                self.plans.push(Plan::new(
                                    self.plans.len() as u32,
                                    self.new_plan_name.to_ascii_uppercase(),
                                    SimpleTime::new(self.new_plan_start_time.0, self.new_plan_start_time.1, 0),
                                    SimpleTime::new(self.new_plan_end_time.0, self.new_plan_end_time.1, 0),
                                ));
                                self.new_plan_name = "".to_string();
                                self.new_plan_end_time = (0, 0);
                                self.new_plan_start_time = (0, 0);
                                self.close_add_plan_window = true;
                            }
                            if ui.button("Cancel").clicked() {
                                self.close_add_plan_window = true;
                            }
                        });

                    });
            }

            // First, draw the hours as rows of rectangles
            ui.vertical(|ui| {
                for h in 0..=23 {
                    ui.allocate_ui_with_layout(
                        egui::vec2(100.0, 40.0),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            if SimpleTime::from_now().hour() == h {
                                ui.label(
                                    egui::RichText::new(format!("{:02}:00", h))
                                        .size(20.0)
                                        .color(WHITE),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(format!("{:02}:00", h))
                                        .size(20.0)
                                        .color(GREY),
                                );
                            }

                        },
                    );
                }
            });

            // Draw the plans

            for plan in &self.plans {
                let plan_color = if plan.is_now {
                    LIGHT_GREEN
                } else {
                    LIGHT_GREY
                };

                let plan_font_color = if plan.is_now {
                    DARK_GREEN
                } else {
                    WHITE
                };

                let fixed_pos = egui::pos2(
                    65.0,
                    40.0 + 33.0 * (plan.start_time.hour() as f32 + plan.start_time.minute() as f32 / 60.0),
                );

                let fixed_size = egui::vec2(
                    185.0,
                    33.0 * (plan.end_time.hour() as f32 + plan.end_time.minute() as f32 / 60.0)
                        - 33.0 * (plan.start_time.hour() as f32 + plan.start_time.minute() as f32 / 60.0),
                );

                let rect = egui::Rect::from_min_size(fixed_pos, fixed_size);
                
                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.painter().rect_filled(rect, 3.0, plan_color);
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new(&plan.name)
                        .color(plan_font_color)
                        );
                    });
                });
            }

            // Draw a horizontal line that marks the current time
            let current_time = SimpleTime::from_now();
            let current_time_y = 40.0 + 33.0 * (current_time.hour() as f32 + current_time.minute() as f32 / 60.0);
            ui.allocate_ui_with_layout(
                egui::vec2(250.0, 2.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.painter().line_segment(
                        [egui::pos2(0.0, current_time_y), egui::pos2(250.0, current_time_y)],
                        (1.0, RED),
                    );
                },
            );

        });
    }
}

fn time_picker(ui: &mut egui::Ui, label: &str, time: &mut (u8, u8), id_prefix: &str) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.add_space(20.0);

            egui::ComboBox::from_id_source(format!("{}_minute", id_prefix))
                .width(47.0)
                .selected_text(
                    format!("{} min", time.1)
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut time.1, 0, "Minute".to_string());
                    for m in 0..=59 {
                        ui.selectable_value(&mut time.1, m, m.to_string());
                    }
                });

            egui::ComboBox::from_id_source(format!("{}_hour", id_prefix))
                .width(47.0)
                .selected_text(
                    format!("{} hs", time.0)
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut time.0, 0, "Hour".to_string());
                    for h in 0..=23 {
                        ui.selectable_value(&mut time.0, h, h.to_string());
                    }
                });

            ui.allocate_ui_with_layout(
                egui::Vec2::new(38.0, 20.0),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.label(label);
                },
            );

    });
}
