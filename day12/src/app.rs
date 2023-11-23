use egui::Context;
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
            grid: None,
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
                            match parse_into_grid(data.as_slice()) {
                                Ok(grid) => {
                                    let _ = sender.send(grid);
                                }
                                Err(_) => {}
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
            ui.label(format!("{:?}", &self.grid));
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
