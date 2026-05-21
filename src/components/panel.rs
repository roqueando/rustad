use egui::InnerResponse;

pub fn make_panel(ui: &mut egui::Ui) -> InnerResponse<()> {
    egui::Panel::top("top_panel").show_inside(ui, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            menu_file_btn(ui);
            ui.add_space(16.0);

            ui.menu_button("edit", |_ui| {
            });

            ui.menu_button("draw", |_ui| {
            });

            ui.menu_button("scopes", |_ui| {
            });

            ui.menu_button("options", |_ui| {
            });

            ui.menu_button("tools", |_ui| {
            });
        });
    })
} 

fn menu_file_btn(ui: &mut egui::Ui) -> InnerResponse<Option<()>> {
    ui.menu_button("file", |ui| {
        egui::widgets::global_theme_preference_buttons(ui);
        if ui.button("quit").clicked() {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    })
}
