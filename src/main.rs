use goap_mission_designer::application::GOAPApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("GOAP Mission Designer")
            .with_inner_size([1600.0, 950.0])
            .with_min_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "GOAP Mission Designer",
        options,
        Box::new(|_cc| Ok(Box::new(GOAPApp::default()))),
    )
}
