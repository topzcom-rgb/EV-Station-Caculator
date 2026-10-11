use eframe::egui;
use ev_station_calculator::gui::{setup_custom_fonts, EvStationApp};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("EV Station Electrical Sizing & Diagram Generator"),
        ..Default::default()
    };

    eframe::run_native(
        "EV Station Electrical Sizing & Diagram Generator",
        native_options,
        Box::new(|cc| {
            // ติดตั้งฟอนต์ภาษาไทยและขนาดตัวอักษร
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(EvStationApp::new()))
        }),
    )
}
