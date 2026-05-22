use crate::components;
use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Vec2, pos2, vec2};

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize)]
struct Terminal {
    offset: Vec2
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ElectricComponent {
    rect: Rect,
    terminals: [Terminal; 2]
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Camera {
    pan: Vec2,
    zoom: f32, // 1.0 = 1 px/pt no seu mundo
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
enum InteractionState {
    Idle,
    DraggingCanvas,
    PlacingComponent {
        start_world: Pos2,
        current_world: Pos2
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RustadApplication {
    label: String,
    camera: Camera,
    interaction: InteractionState,
    components: Vec<ElectricComponent>,

    #[serde(skip)]
    value: f32,
}

impl Default for RustadApplication {
    fn default() -> Self {
        Self {
            label: "Rustad".to_owned(),
            value: 2.7,
            camera: Camera { pan: vec2(0.0, 0.0), zoom: 1.0},
            interaction: InteractionState::Idle,
            components: vec![]
        }
    }
}

impl RustadApplication {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()

        /* TODO: add storage here with {features = ['persistence']}
        if let Some(storage)  = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
        */
    }
}


// TODO: move this to a camera module
impl Camera {
    fn world_to_screen(&self, world: Pos2, vp: Rect) -> Pos2 {
        let c = vp.center();
        pos2(c.x + (world.x - self.pan.x) * self.zoom, c.y + (world.y - self.pan.y) * self.zoom)
    }

    fn screen_to_world(&self, screen: Pos2, vp: Rect) -> Pos2 {
        let c = vp.center();
        pos2(self.pan.x + (screen.x - c.x) / self.zoom, self.pan.y + (screen.y - c.y) / self.zoom)
    }

    fn zoom_at(&mut self, vp: Rect, cursor: Pos2, zoom_factor: f32) {
        let before = self.screen_to_world(cursor, vp);
        self.zoom = (self.zoom * zoom_factor).clamp(0.05, 50.0);
        self.pan = vec2(
            before.x - (cursor.x - vp.center().x) / self.zoom,
            before.y - (cursor.y - vp.center().y) / self.zoom,
        );
    }
}

fn nice_step(zoom: f32) -> f32 {
    let target_px = 64.0;
    let world_step = target_px / zoom;
    let pow10 = 10.0_f32.powf(world_step.log10().floor());
    for m in [1.0, 2.0, 5.0, 10.0] {
        let s = m * pow10;
        if world_step <= s {
            return s;
        }
    }
    10.0 * pow10
}

fn draw_grid(painter: &egui::Painter, vp: Rect, cam: &Camera) {
    painter.rect_filled(vp, 0.0, Color32::from_gray(18));

    let world_min = cam.screen_to_world(vp.left_top(), vp);
    let world_max = cam.screen_to_world(vp.right_bottom(), vp);

    let step = nice_step(cam.zoom);
    let major_every = 5.0 * step;

    let mut x = (world_min.x / step).floor() * step;
    while x <= world_max.x {
        let sx = cam.world_to_screen(pos2(x, 0.0), vp).x;
        let is_axis = x.abs() < step * 0.5;
        let is_major = (x / major_every).fract().abs() < 0.001;

        let stroke = if is_axis {
            Stroke::new(1.5, Color32::LIGHT_GRAY)
        } else if is_major {
            Stroke::new(1.0, Color32::from_gray(90))
        } else {
            Stroke::new(1.0, Color32::from_gray(45))
        };

        painter.line_segment(
            [pos2(sx, vp.top()), pos2(sx, vp.bottom())],
            stroke
        );
        x += step;
    }

    let mut y = (world_min.y / step).floor() * step;
    while y <= world_max.y {
        let sy = cam.world_to_screen(pos2(y, 0.0), vp).y;
        let is_axis = y.abs() < step * 0.5;
        let is_major = (y / major_every).fract().abs() < 0.001;

        let stroke = if is_axis {
            Stroke::new(1.5, Color32::LIGHT_GRAY)
        } else if is_major {
            Stroke::new(1.0, Color32::from_gray(90))
        } else {
            Stroke::new(1.0, Color32::from_gray(45))
        };

        painter.line_segment(
            [pos2(vp.left(), sy), pos2(vp.right(), sy)],
            stroke
        );
        y += step;
    }
}

// end camera module

impl eframe::App for RustadApplication {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // here will be the widgets
        components::panel::make_panel(ui);

        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);


        let zoom_factor = ui.ctx().input(|i| i.zoom_delta());
        if (zoom_factor - 1.0).abs() > f32::EPSILON {
            let cursor = ui.ctx()
                .input(|i| i.pointer.hover_pos())
                .unwrap_or(rect.center());
            self.camera.zoom_at(rect, cursor, zoom_factor);
        }

        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta);

        if scroll != Vec2::ZERO {
            self.camera.pan -= scroll / self.camera.zoom;
        }

        /*
        if response.dragged() {
            self.camera.pan -= response.drag_delta() / self.camera.zoom;
        }
        */


        // INFO: this is my right click menu
        response.context_menu(|ui| {
            // each button is my option
            if ui.button("add generic component").clicked() {
                if let Some(mouse) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                    let world = self.camera.screen_to_world(mouse, rect);
                    self.interaction = InteractionState::PlacingComponent {
                        start_world: world,
                        current_world: world
                    };
                }
                ui.close();
            }
        });

        match &mut self.interaction {
            InteractionState::Idle => {
                if response.drag_started_by(egui::PointerButton::Middle) {
                    self.interaction = InteractionState::DraggingCanvas;
                }
                if response.dragged_by(egui::PointerButton::Middle) {
                    self.camera.pan -= response.drag_delta() / self.camera.zoom;
                }
                if response.drag_stopped_by(egui::PointerButton::Middle) {
                    self.interaction = InteractionState::Idle;
                }
            },

            InteractionState::DraggingCanvas => {
                if response.dragged_by(egui::PointerButton::Middle) {
                    self.camera.pan -= response.drag_delta() / self.camera.zoom;
                }
                if response.drag_stopped_by(egui::PointerButton::Middle) {
                    self.interaction = InteractionState::Idle;
                }
            },
            InteractionState::PlacingComponent { start_world, current_world } => {
                if response.drag_started_by(egui::PointerButton::Primary) {
                    if let Some(mouse) = ui.ctx().input(|i| i.pointer.press_origin()) {
                        let world = self.camera.screen_to_world(mouse, rect);
                        *start_world = world;
                        *current_world = world;
                    }
                }
                if response.dragged_by(egui::PointerButton::Primary) {
                    if let Some(mouse) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                        *current_world = self.camera.screen_to_world(mouse, rect);
                    }
                }

                if response.drag_stopped_by(egui::PointerButton::Primary) {
                    let min = pos2(
                        start_world.x.min(current_world.x),
                        start_world.y.min(current_world.y)
                    );

                    let max = pos2(
                        start_world.x.max(current_world.x),
                        start_world.y.max(current_world.y)
                    );

                    let placed = Rect::from_min_max(min, max);

                    if placed.width() > 5.0 && placed.height() >5.0 {
                        self.components.push(ElectricComponent {
                            rect: placed,
                            terminals: [
                                Terminal { offset: vec2(0.0, placed.height() * 0.5) },
                                Terminal { offset: vec2(placed.width(), placed.height() * 0.5) },
                            ]
                        })
                    }
                    self.interaction = InteractionState::Idle;
                }
            }
        }

        draw_grid(&painter, rect, &self.camera);

        for c in &self.components {
            let min = self.camera.world_to_screen(c.rect.min, rect);
            let max = self.camera.world_to_screen(c.rect.max, rect);
            let screen_rect = Rect::from_min_max(min, max);

            painter.rect_filled(screen_rect, CornerRadius::same(4), Color32::from_gray(55));
            painter.rect_stroke(
                screen_rect,
                CornerRadius::same(4),
                Stroke::new(1.0, Color32::WHITE),
                egui::StrokeKind::Outside
            );

            for t in c.terminals {
                let p_world = pos2(c.rect.min.x + t.offset.x, c.rect.min.y + t.offset.y);
                let p_screen = self.camera.world_to_screen(p_world, rect);
                painter.circle_filled(p_screen, 4.0, Color32::YELLOW);
            }
        }

        if let InteractionState::PlacingComponent { start_world, current_world } = &self.interaction {
            
            let min = pos2(
                start_world.x.min(current_world.x),
                start_world.y.min(current_world.y),
            );
            let max = pos2(
                start_world.x.max(current_world.x),
                start_world.y.max(current_world.y),
            );

            let preview = Rect::from_min_max(min, max);
            let min_s = self.camera.world_to_screen(preview.min, rect);
            let max_s = self.camera.world_to_screen(preview.max, rect);
            let preview_screen = Rect::from_min_max(min_s, max_s);

            painter.rect_stroke(
                preview_screen,
                CornerRadius::same(4),
                Stroke::new(1.0, Color32::LIGHT_GREEN),
                egui::StrokeKind::Outside,
            );
        }
    }
}

