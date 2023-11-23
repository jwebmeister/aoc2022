use egui::{emath, Context, Pos2, Rect, Sense};
use std::future::Future;
use std::sync::mpsc::{channel, Receiver, Sender};

use crate::mod_day12::*;

pub struct MyApp {
    grid_channel: (Sender<Grid>, Receiver<Grid>),
    grid: Option<Grid>,
    bfs: Bfs,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            grid_channel: channel(),
            grid: Some(Grid::default()),
            bfs: Default::default(),
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
                };
                if ui.button("▶").clicked() {
                    if self.grid.is_none() {
                        return;
                    };
                    let grid = self.grid.as_ref();
                    self.bfs.step(grid.unwrap());
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
                    let maybe_cell_width = rect.width() as usize / grid.width;
                    let maybe_cell_height = rect.height() as usize / grid.height;
                    core::cmp::min(maybe_cell_width, maybe_cell_height) as f32
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
                        width: f32::max(cell_width / 32.0, 1.0),
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
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
