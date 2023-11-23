use egui::{emath, Context, Pos2, Rect, Sense};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::mod_day12::*;

pub struct MyApp {
    grid_channel: (Sender<Grid>, Receiver<Grid>),
    grid: Option<Grid>,
    bfs: Bfs,
    goal_path: Vec<(usize, usize)>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            grid_channel: channel(),
            grid: Some(Grid::default()),
            bfs: Default::default(),
            goal_path: Default::default(),
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        MyApp::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // assign grid once it comes in
        if let Ok(f) = self.grid_channel.1.try_recv() {
            self.grid = Some(f);
            self.bfs.reset();
            self.goal_path.clear();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // a simple button opening the dialog
            egui::Ui::horizontal(ui, |ui| {
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
                if ui.button("⏮").clicked() {
                    self.bfs.reset();
                    self.goal_path.clear();
                };
                if ui.button("▶").clicked() {
                    if self.grid.is_none() || !self.goal_path.is_empty() {
                        return;
                    };
                    let grid = self.grid.as_ref().unwrap();
                    if let Some(end_coord) = &grid.get_end_coord() {
                        self.bfs.step(grid);
                        if self.bfs.current.contains(end_coord) {
                            self.goal_path = self.bfs.trace_back_path(*end_coord).unwrap();
                        }
                    };
                };
                if ui.button("⏭").clicked() {
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
                if ui.button("⏶").clicked() {
                    if self.grid.is_none() || !self.goal_path.is_empty() {
                        return;
                    };
                    let grid = self.grid.as_ref().unwrap();
                    if let Some(end_coord) = &grid.get_end_coord() {
                        self.bfs.step_up(grid);
                        if self.bfs.current.contains(end_coord) {
                            self.goal_path = self.bfs.trace_back_path(*end_coord).unwrap();
                        }
                    };
                };
                if ui.button("⏫").clicked() {
                    if self.grid.is_none() || !self.goal_path.is_empty() {
                        return;
                    };
                    let grid = self.grid.as_ref().unwrap();
                    if let Some(end_coord) = &grid.get_end_coord() {
                        while self.goal_path.is_empty() {
                            self.bfs.step_up(grid);
                            if self.bfs.current.contains(end_coord) {
                                self.goal_path = self.bfs.trace_back_path(*end_coord).unwrap();
                            };
                            if self.bfs.num_steps >= 1_000_000 {
                                break;
                            }
                        }
                    };
                };

                ui.label(format!("Step: {}", &self.bfs.num_steps));
                ui.label(format!("Current moves: {}", &self.bfs.current.len()));
                ui.label(format!("Visited coords: {}", &self.bfs.visited.len()));
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(grid) = &self.grid {
                // placeholder grid ui
                /*
                ui.label(
                    egui::RichText::new(format!("{:?}", grid)).font(egui::FontId::monospace(10.0)),
                );
                */

                // actual grid ui
                let (mut response, painter) =
                    ui.allocate_painter(ui.available_size_before_wrap(), Sense::hover());

                let rect = response.rect;

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
                    response.rect,
                );
                let from_screen = to_screen.inverse();

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
                    let cell_rounding = egui::epaint::Rounding::ZERO;
                    let cell_rect_shape = egui::epaint::RectShape::new(
                        cell_rect,
                        cell_rounding,
                        cell_color,
                        cell_stroke,
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

                let cell_rects = grid.iter().enumerate().map(|(data_idx, cell)| {
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
                        let canvas_pos = from_screen * hover_pos;
                        // ui.label(format!("cp:{:?},sp:{:?}", canvas_pos, hover_pos));

                        cell_rects
                            .filter_map(|x| match x.0.contains(hover_pos) {
                                true => Some(x.1),
                                false => None,
                            })
                            .for_each(|coord| {
                                let cell = grid.get_cell_from_coord(coord).unwrap();
                                let elev = cell.elevation();
                                let label_text = match cell {
                                    Cell::Start => format!("Start, elev:{0}", elev),
                                    Cell::End => format!("End, elev:{0}", elev),
                                    Cell::Square(_) => format!("Elev:{0}", elev),
                                };
                                ui.label(label_text);
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
