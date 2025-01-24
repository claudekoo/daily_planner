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
            taskbar: Some(false),
            resizable: Some(false),
            inner_size: Some(egui::vec2(273.0, 838.0)),
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
    activities: Vec<Activity>,
    last_update: SimpleTime,
    add_activity_window_open: bool,
    close_add_activity_window: bool,
    new_activity_name: String,
    new_activity_start_time: (u8, u8),
    new_activity_end_time: (u8, u8),
    selected_activity_id_for_update: Option<u32>,
    selected_activity_new_name: String,
    selected_activity_new_start_time: (u8, u8),
    selected_activity_new_end_time: (u8, u8),
    update_activity_window_open: bool,
    close_update_activity_window: bool,
}

impl PlannerApp {
    pub fn new() -> std::io::Result<Self> {

        // Load activities from file
        let mut activities: Vec<Activity> = if let Ok(activities_json) = std::fs::read_to_string("plan.json") {
            if let Ok(activities) = serde_json::from_str(&activities_json) {
                activities
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        for activity in &mut activities {
            activity.update_now();
        }

        Ok(Self {
            activities,
            last_update: SimpleTime::from_now(),
            add_activity_window_open: false,
            close_add_activity_window: false,
            new_activity_name: "".to_string(),
            new_activity_start_time: (0, 0),
            new_activity_end_time: (0, 0),
            selected_activity_id_for_update: None,
            selected_activity_new_name: "".to_string(),
            selected_activity_new_start_time: (0, 0),
            selected_activity_new_end_time: (0, 0),
            update_activity_window_open: false,
            close_update_activity_window: false,
        })
    }

    fn update_activities_every_ten_seconds(&mut self) {
        // Update the activities after 10 seconds from last update
        let now = SimpleTime::from_now();
        if now.as_seconds() - self.last_update.as_seconds() >= 10 {
            self.last_update = now;
            for activity in &mut self.activities {
                activity.update_now();
            }
        }
    }
}

impl eframe::App for PlannerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_activities_every_ten_seconds();

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

            // Add Activity / Delete All buttons
            ui.horizontal(|ui| {
                if ui.button("New Activity").clicked() {
                    self.add_activity_window_open = true;
                }
                if ui.button("Delete All").clicked() {
                    // just delete all activities
                    self.activities = vec![];
                }
                if ui.button("Save Plan").clicked() {
                    let activities_json = serde_json::to_string(&self.activities).unwrap();
                    std::fs::write("plan.json", activities_json).expect("Failed to save plan");
                }
            });

            // ui.horizontal(|ui| {
            //     if ui.button("Save Plan").clicked() {
            //         let activities_json = serde_json::to_string(&self.activities).unwrap();
            //         std::fs::write("plan.json", activities_json).expect("Failed to save plan");
            //     }
            //     if ui.button("Delete Save").clicked() {
            //         // just delete all activities and delete the file too
            //         self.activities = vec![];
            //         _ = std::fs::remove_file("plan.json");
            //     }
            // });

            if self.close_add_activity_window {
                self.add_activity_window_open = false;
                self.close_add_activity_window = false;
            }

            if self.add_activity_window_open {
                egui::Window::new("New Activity")
                    .default_size(egui::vec2(140.0, 70.0))
                    .title_bar(false)
                    .collapsible(false)
                    .resizable(false)
                    .open(&mut self.add_activity_window_open)
                    .show(ui.ctx(), |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_activity_name);

                        ui.label("Start Time:");
                        time_picker(ui, &mut self.new_activity_start_time, "new_activity_start_time");

                        ui.label("End Time:");
                        time_picker(ui, &mut self.new_activity_end_time, "new_activity_end_time");
                        
                        ui.add_space(5.0);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("Add").clicked() {
                                self.activities.push(Activity::new(
                                    self.activities.len() as u32,
                                    self.new_activity_name.to_ascii_uppercase(),
                                    SimpleTime::new(self.new_activity_start_time.0, self.new_activity_start_time.1, 0),
                                    SimpleTime::new(self.new_activity_end_time.0, self.new_activity_end_time.1, 0),
                                ));
                                self.new_activity_name = "".to_string();
                                self.new_activity_end_time = (0, 0);
                                self.new_activity_start_time = (0, 0);
                                self.close_add_activity_window = true;
                            }
                            if ui.button("Cancel").clicked() {
                                self.close_add_activity_window = true;
                            }
                        });

                    });
            }

            if self.close_update_activity_window {
                self.update_activity_window_open = false;
                self.close_update_activity_window = false;
            }

            if self.update_activity_window_open {
                if let Some(activity_id) = self.selected_activity_id_for_update {
                    if let Some(activity) = self.activities.iter_mut().find(|p| p.id == activity_id) {
                        egui::Window::new("Update Activity")
                            .default_size(egui::vec2(140.0, 70.0))
                            .title_bar(false)
                            .collapsible(false)
                            .resizable(false)
                            .open(&mut self.update_activity_window_open)
                            .show(ui.ctx(), |ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.selected_activity_new_name);

                                ui.label("Start Time:");
                                time_picker(ui, &mut self.selected_activity_new_start_time, "update_activity_start_time");

                                ui.label("End Time:");
                                time_picker(ui, &mut self.selected_activity_new_end_time, "update_activity_end_time");
                                
                                ui.add_space(5.0);

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                    if ui.button("Update").clicked() {
                                        activity.name = self.selected_activity_new_name.to_ascii_uppercase();
                                        activity.start_time = SimpleTime::new(self.selected_activity_new_start_time.0, self.selected_activity_new_start_time.1, 0);
                                        activity.end_time = SimpleTime::new(self.selected_activity_new_end_time.0, self.selected_activity_new_end_time.1, 0);
                                        self.close_update_activity_window = true;
                                    }
                                    if ui.button("Cancel").clicked() {
                                        self.close_update_activity_window = true;
                                    }
                                });

                            });
                    }
                }
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

            // Draw the activities

            for activity in &self.activities {
                let activity_color = if activity.is_now { LIGHT_GREEN } else { LIGHT_GREY };
                let activity_font_color = if activity.is_now { DARK_GREEN } else { WHITE };

                let fixed_pos = egui::pos2(
                    65.0,
                    40.0 + 33.0 * (activity.start_time.hour() as f32 + activity.start_time.minute() as f32 / 60.0),
                );
                let fixed_size = egui::vec2(
                    200.0,
                    33.0 * (activity.end_time.hour() as f32 + activity.end_time.minute() as f32 / 60.0)
                        - 33.0 * (activity.start_time.hour() as f32 + activity.start_time.minute() as f32 / 60.0),
                );

                let rect = egui::Rect::from_min_size(fixed_pos, fixed_size);

                ui.allocate_ui_at_rect(rect, |ui| {
                    ui.painter().rect_filled(rect, 3.0, activity_color);
                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                        if ui.add(egui::Label::new(
                            egui::RichText::new(&activity.name)
                                .color(activity_font_color)
                        ).sense(egui::Sense::click())).clicked() {
                            self.selected_activity_id_for_update = Some(activity.id);
                            self.selected_activity_new_name = activity.name().to_string();
                            self.selected_activity_new_start_time = (activity.start_time().hour(), activity.start_time().minute());
                            self.selected_activity_new_end_time = (activity.end_time().hour(), activity.end_time().minute());
                            self.update_activity_window_open = true;
                        }
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

fn time_picker(ui: &mut egui::Ui, time: &mut (u8, u8), id_prefix: &str) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            ui.add_space(20.0);

            egui::ComboBox::from_id_salt(format!("{}_minute", id_prefix))
                .width(64.0)
                .selected_text(
                    format!("{} min", time.1)
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut time.1, 0, "Minute".to_string());
                    for m in 0..=59 {
                        ui.selectable_value(&mut time.1, m, m.to_string());
                    }
                });

            egui::ComboBox::from_id_salt(format!("{}_hour", id_prefix))
                .width(64.0)
                .selected_text(
                    format!("{} hs", time.0)
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut time.0, 0, "Hour".to_string());
                    for h in 0..=23 {
                        ui.selectable_value(&mut time.0, h, h.to_string());
                    }
                });

    });
}
