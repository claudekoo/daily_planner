#![windows_subsystem = "windows"]

mod ui;
mod structs;
mod color_palette;

fn main() {
    println!("Starting UI...");
    let app = ui::PlannerApp::new().expect("Failed to create PlannerApp");
    
    println!("Showing UI...");
    ui::show_ui(app).expect("Failed to show UI");
}
