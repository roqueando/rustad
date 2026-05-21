fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rustad",
        native_options,
        Box::new(|cc| Ok(Box::new(rustad::RustadApplication::new(cc)))),
    )
}
