use egui::{Context, Sense};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::mod_day12::*;

pub struct AppDay12 {
    grid_channel: (Sender<Grid>, Receiver<Grid>),
    grid: Option<Grid>,
    bfs: Bfs,
    goal_path: Vec<(usize, usize)>,
    paused: bool,
    speed: i32,
    big_grid: bool,
    last_step_ts: chrono::DateTime<chrono::Utc>,
    b_step_up: bool,
}

impl Default for AppDay12 {
    fn default() -> Self {
        Self {
            grid_channel: channel(),
            grid: Some(Grid::default()),
            bfs: Default::default(),
            goal_path: Default::default(),
            paused: true,
            speed: 1,
            big_grid: false,
            last_step_ts: chrono::Utc::now(),
            b_step_up: false,
        }
    }
}

impl AppDay12 {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        AppDay12::default()
    }
}

impl eframe::App for AppDay12 {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // assign grid once it comes in
        if let Ok(f) = self.grid_channel.1.try_recv() {
            self.paused = true;
            self.grid = Some(f);
            self.bfs.reset();
            self.goal_path.clear();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::Ui::horizontal(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);

                ui.separator();

                let is_big_grid = self.big_grid;
                let big_grid_resp = ui.toggle_value(
                    &mut self.big_grid,
                    if is_big_grid {
                        "New Small grid"
                    } else {
                        "New Big Grid"
                    },
                );
                if big_grid_resp.changed() {
                    self.paused = true;
                    match is_big_grid {
                        true => self.grid = Some(Grid::new_small_grid()),
                        false => self.grid = Some(Grid::new_big_grid()),
                    };
                    self.bfs.reset();
                    self.goal_path.clear();
                };

                if ui.button("Open grid input.txt file").clicked() {
                    let sender = self.grid_channel.0.clone();
                    let task = rfd::AsyncFileDialog::new().pick_file();
                    execute(async move {
                        let file = task.await;
                        if let Some(file) = file {
                            let data = file.read().await;
                            if let Ok(grid) = parse_into_grid(data.as_slice()) {
                                let _ = sender.send(grid);
                            };
                        }
                    });
                };

                ui.separator();

                if ui.button("⏮").clicked() {
                    self.paused = true;
                    self.bfs.reset();
                    self.goal_path.clear();
                };

                if ui.button("Step").clicked() {
                    self.paused = true;
                    if self.grid.is_none() || !self.goal_path.is_empty() {
                        return;
                    };
                    let grid = self.grid.as_ref().unwrap();
                    if let Some(end_coord) = &grid.get_end_coord() {
                        match self.b_step_up {
                            true => self.bfs.step_up(grid),
                            false => self.bfs.step(grid),
                        };
                        if self.bfs.current.contains(end_coord) {
                            self.goal_path = self.bfs.trace_back_path(*end_coord).unwrap();
                        }
                    };
                };

                let paused = self.paused;
                ui.toggle_value(&mut self.paused, if paused { "▶" } else { "⏸" });

                ui.horizontal(|ui| {
                    ui.label("Speed: ");
                    ui.add(egui::widgets::Slider::new(&mut self.speed, 1..=20).prefix("x"));
                });

                if ui.button("⏭").clicked() {
                    self.paused = true;
                    if self.grid.is_none() || !self.goal_path.is_empty() {
                        return;
                    };
                    let grid = self.grid.as_ref().unwrap();
                    if let Some(end_coord) = &grid.get_end_coord() {
                        while self.goal_path.is_empty() {
                            self.bfs.step(grid);
                            if self.bfs.current.contains(end_coord) {
                                self.goal_path = self.bfs.trace_back_path(*end_coord).unwrap();
                            };
                            if self.bfs.num_steps >= 1_000_000 {
                                break;
                            }
                        }
                    };
                };

                ui.separator();

                ui.checkbox(&mut self.b_step_up, "Start from any ground tile");

                if !self.paused {
                    if self.grid.is_none() || !self.goal_path.is_empty() {
                        self.paused = true;
                        return;
                    };
                    let time_between_steps = chrono::Duration::milliseconds(100) / self.speed;
                    let time_left = chrono::Utc::now() - self.last_step_ts;
                    if time_left < time_between_steps {
                        ctx.request_repaint_after(time_left.to_std().unwrap());
                        return;
                    }
                    let grid = self.grid.as_ref().unwrap();
                    if let Some(end_coord) = grid.get_end_coord() {
                        if self.bfs.current.contains(&end_coord) {
                            self.goal_path = self.bfs.trace_back_path(end_coord).unwrap();
                            self.paused = true;
                            return;
                        }
                    };
                    match self.b_step_up {
                        true => self.bfs.step_up(grid),
                        false => self.bfs.step(grid),
                    };
                    self.last_step_ts = chrono::Utc::now();
                    ctx.request_repaint_after(time_between_steps.to_std().unwrap());
                };
            })
        });

        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            egui::Ui::horizontal(ui, |ui| {
                ui.label(format!("Step: {}", &self.bfs.num_steps));
                ui.label(format!("Current moves: {}", &self.bfs.current.len()));
                ui.label(format!("Visited coords: {}", &self.bfs.visited.len()));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(grid) = &self.grid {
                // actual grid ui
                let (response, painter) =
                    ui.allocate_painter(ui.available_size_before_wrap(), Sense::hover());

                let rect = response.rect;

                let cell_width = {
                    let maybe_cell_width = rect.width() / grid.width as f32;
                    let maybe_cell_height = rect.height() / grid.height as f32;
                    f32::min(maybe_cell_width, maybe_cell_height)
                };

                let cell_shapes = grid.iter().enumerate().map(|(data_idx, cell)| {
                    let coord = grid.data_idx_to_coord(data_idx).unwrap();
                    let top_left = rect.min
                        + egui::Vec2::from((
                            coord.1 as f32 * cell_width,
                            coord.0 as f32 * cell_width,
                        ));
                    let bottom_right = rect.min
                        + egui::Vec2::from((
                            (coord.1 + 1) as f32 * cell_width,
                            (coord.0 + 1) as f32 * cell_width,
                        ));
                    let cell_rect = egui::Rect::from_two_pos(top_left, bottom_right);

                    let cell_color = match cell {
                        Cell::Start => egui::epaint::Color32::DARK_BLUE,
                        Cell::End => egui::epaint::Color32::LIGHT_RED,
                        Cell::Square(elevation) => {
                            let sat = ((*elevation as u32) * 255 / 25) as u8;
                            egui::epaint::Color32::from_rgb(sat, sat, sat)
                        }
                    };

                    let cell_stroke = egui::epaint::Stroke {
                        width: f32::min(cell_width / 32.0, 1.0),
                        color: egui::epaint::Color32::WHITE,
                    };
                    let cell_rounding = egui::epaint::CornerRadius::ZERO;
                    let cell_rect_shape = egui::epaint::RectShape::new(
                        cell_rect,
                        cell_rounding,
                        cell_color,
                        cell_stroke,
                        egui::StrokeKind::Middle,
                    );
                    egui::Shape::Rect(cell_rect_shape)
                });

                painter.extend(cell_shapes);

                let bfs_current_shapes = self.bfs.current.iter().map(|coord| {
                    let center = rect.min
                        + egui::Vec2::from((
                            (coord.1 as f32 * cell_width) + (cell_width * 0.5),
                            (coord.0 as f32 * cell_width) + (cell_width * 0.5),
                        ));
                    let radius = cell_width * 0.45;
                    let fill_color = egui::epaint::ecolor::Color32::YELLOW;
                    egui::epaint::Shape::circle_filled(center, radius, fill_color)
                });

                painter.extend(bfs_current_shapes);

                let bfs_visited = self
                    .bfs
                    .visited
                    .iter()
                    .filter_map(|(coord, op_prev_coord)| {
                        let curr = rect.min
                            + egui::Vec2::from((
                                (coord.1 as f32 * cell_width) + (cell_width * 0.5),
                                (coord.0 as f32 * cell_width) + (cell_width * 0.5),
                            ));
                        let Some(prev_coord) = op_prev_coord else {
                            return None;
                        };
                        let prev = rect.min
                            + egui::Vec2::from((
                                (prev_coord.1 as f32 * cell_width) + (cell_width * 0.5),
                                (prev_coord.0 as f32 * cell_width) + (cell_width * 0.5),
                            ));
                        let line_stroke = egui::epaint::Stroke::new(
                            cell_width * 0.4,
                            egui::epaint::Color32::YELLOW,
                        );
                        Some(egui::epaint::Shape::line_segment([curr, prev], line_stroke))
                    });

                painter.extend(bfs_visited);

                let bfs_visited_squares = self.bfs.visited.keys().map(|coord| {
                    let top_left = rect.min
                        + egui::Vec2::from((
                            (coord.1 as f32 * cell_width) + (cell_width * 0.3),
                            (coord.0 as f32 * cell_width) + (cell_width * 0.3),
                        ));
                    let bottom_right = rect.min
                        + egui::Vec2::from((
                            (coord.1 as f32 * cell_width) + (cell_width * 0.7),
                            (coord.0 as f32 * cell_width) + (cell_width * 0.7),
                        ));
                    egui::epaint::Shape::rect_filled(
                        egui::Rect::from_two_pos(top_left, bottom_right),
                        egui::epaint::CornerRadius::ZERO,
                        egui::epaint::Color32::YELLOW,
                    )
                });

                painter.extend(bfs_visited_squares);

                let goal_path_points = self
                    .goal_path
                    .iter()
                    .map(|coord| {
                        rect.min
                            + egui::Vec2::from((
                                (coord.1 as f32 * cell_width) + (cell_width * 0.5),
                                (coord.0 as f32 * cell_width) + (cell_width * 0.5),
                            ))
                    })
                    .collect::<Vec<_>>();
                let goal_path_stroke =
                    egui::epaint::Stroke::new(cell_width * 0.4, egui::epaint::Color32::RED);
                let goal_path_shape = egui::epaint::Shape::line(goal_path_points, goal_path_stroke);

                painter.add(goal_path_shape);

                let cell_rects = (0..grid.data.len()).map(|data_idx| {
                    let coord = grid.data_idx_to_coord(data_idx).unwrap();
                    let top_left = rect.min
                        + egui::Vec2::from((
                            coord.1 as f32 * cell_width,
                            coord.0 as f32 * cell_width,
                        ));
                    let bottom_right = rect.min
                        + egui::Vec2::from((
                            (coord.1 + 1) as f32 * cell_width,
                            (coord.0 + 1) as f32 * cell_width,
                        ));
                    (egui::Rect::from_two_pos(top_left, bottom_right), coord)
                });

                if let Some(hover_pos) = response.hover_pos() {
                    response.on_hover_ui_at_pointer(|ui| {
                        cell_rects
                            .filter_map(|x| match x.0.contains(hover_pos) {
                                true => Some(x.1),
                                false => None,
                            })
                            .for_each(|coord| {
                                let cell = grid.get_cell_from_coord(coord).unwrap();
                                let elev = cell.elevation();
                                let label_text = match cell {
                                    Cell::Start => format!("Start,{0}", elev),
                                    Cell::End => format!("End,{0}", elev),
                                    Cell::Square(_) => format!("{0}", elev),
                                };
                                ui.add(egui::Label::new(label_text).extend());
                            });
                    });
                };
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
