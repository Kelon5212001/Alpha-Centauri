use eframe::egui;
use smac_core::{GameOver, GameState, Terrain, UnitKind, PLAYER_ID};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 860.0])
            .with_title("Sid Meier's Alpha Centauri - Rust Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "Sid Meier's Alpha Centauri - Rust Edition",
        options,
        Box::new(|_cc| Box::new(SmacApp::default())),
    )
}

struct SmacApp {
    game: GameState,
    selected_unit: Option<usize>,
    selected_tile: Option<(usize, usize)>,
    zoom: f32,
    seed_counter: u32,
}

impl Default for SmacApp {
    fn default() -> Self {
        Self {
            game: GameState::new_game(32, 24, 1337),
            selected_unit: None,
            selected_tile: Some((3, 3)),
            zoom: 24.0,
            seed_counter: 1337,
        }
    }
}

impl eframe::App for SmacApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_top_bar(ctx);
        self.draw_side_panel(ctx);
        self.draw_map(ctx);
    }
}

impl SmacApp {
    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SMAC Rust Edition - Playable Prototype v0.1");
                ui.separator();
                ui.label(format!("Mission Year: {}", 2100 + self.game.turn));

                if ui.button("End Turn").clicked() {
                    self.selected_unit = None;
                    self.game.end_turn();
                }

                if ui.button("New Game").clicked() {
                    self.seed_counter = self.seed_counter.wrapping_add(77);
                    self.game = GameState::new_game(32, 24, self.seed_counter);
                    self.selected_unit = None;
                    self.selected_tile = Some((3, 3));
                }

                ui.add(egui::Slider::new(&mut self.zoom, 16.0..=36.0).text("tile size"));
            });

            if let Some(result) = self.game.game_over {
                ui.separator();
                match result {
                    GameOver::PlayerWon => {
                        ui.colored_label(
                            egui::Color32::LIGHT_GREEN,
                            "VICTORY: rival faction eliminated.",
                        );
                    }
                    GameOver::PlayerLost => {
                        ui.colored_label(
                            egui::Color32::LIGHT_RED,
                            "DEFEAT: your faction has been wiped out.",
                        );
                    }
                }
            }
        });
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel")
            .default_width(340.0)
            .show(ctx, |ui| {
                ui.heading("Command Console");

                ui.separator();
                ui.label("Controls:");
                ui.label("1. Click your unit: C = Colony Pod, S = Scout.");
                ui.label("2. Click adjacent land tile to move.");
                ui.label("3. Use Found Base with a colony pod.");
                ui.label("4. End Turn to let bases produce and AI move.");

                ui.separator();
                self.draw_selection_info(ui);

                ui.separator();
                self.draw_faction_status(ui);

                ui.separator();
                ui.heading("Event Log");
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for message in &self.game.log {
                            ui.label(message);
                        }
                    });
            });
    }

    fn draw_selection_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("Selection");

        if let Some(unit_id) = self.selected_unit {
            if let Some(unit) = self.game.unit(unit_id) {
                let owner = &self.game.factions[unit.owner].name;
                ui.label(format!("Unit: {}", unit.kind.name()));
                ui.label(format!("Owner: {owner}"));
                ui.label(format!("Location: {}, {}", unit.x, unit.y));
                ui.label(format!("Moves left: {}", unit.moves_left));
                ui.label(format!("HP: {}", unit.hp));

                if unit.owner == PLAYER_ID && unit.kind == UnitKind::ColonyPod {
                    if ui.button("Found Base Here").clicked() {
                        match self.game.found_base(unit_id) {
                            Ok(_) => {
                                self.selected_unit = None;
                            }
                            Err(err) => self.game.push_log(format!("Cannot found base: {err}")),
                        }
                    }
                }
            } else {
                ui.label("Selected unit no longer exists.");
                self.selected_unit = None;
            }
        } else {
            ui.label("No unit selected.");
        }

        ui.separator();

        if let Some((x, y)) = self.selected_tile {
            if let Some(tile) = self.game.tile(x, y) {
                ui.label(format!("Tile: {}, {}", tile.x, tile.y));
                ui.label(format!("Terrain: {}", tile.terrain.name()));
                ui.label(format!("Elevation: {}", tile.elevation));
                ui.label(format!("Moisture: {}", tile.moisture));

                let yields = tile.terrain.yields();
                ui.label(format!(
                    "Yield: {} nutrients / {} minerals / {} energy",
                    yields.nutrients, yields.minerals, yields.energy
                ));

                if let Some(base_id) = tile.base {
                    if let Some(base) = self.game.base(base_id) {
                        ui.separator();
                        ui.label(format!("Base: {}", base.name));
                        ui.label(format!("Owner: {}", self.game.factions[base.owner].name));
                        ui.label(format!("Population: {}", base.population));
                        ui.label(format!(
                            "Stored: {} nutrients / {} minerals",
                            base.nutrients_stock, base.minerals_stock
                        ));
                        ui.label(format!("Producing: {}", base.production.name()));

                        let base_yields = self.game.base_yields(base.x, base.y);
                        ui.label(format!(
                            "Base output: {} nutrients / {} minerals / {} energy per production pass",
                            base_yields.nutrients, base_yields.minerals, base_yields.energy
                        ));
                    }
                }
            }
        } else {
            ui.label("No tile selected.");
        }
    }

    fn draw_faction_status(&self, ui: &mut egui::Ui) {
        ui.heading("Factions");

        for faction in &self.game.factions {
            let base_count = self.game.bases_for(faction.id).len();
            let unit_count = self.game.live_units_for(faction.id).len();

            ui.group(|ui| {
                ui.label(format!("{}", faction.name));
                ui.label(format!("Bases: {base_count}"));
                ui.label(format!("Units: {unit_count}"));
                ui.label(format!("Energy: {}", faction.energy));
                ui.label(format!("Research: {}/50", faction.research));
                ui.label(format!("Techs: {}", faction.techs_discovered));
            });
        }
    }

    fn draw_map(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Planet Surface");

            let mut clicked_tile: Option<(usize, usize)> = None;

            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("planet_grid")
                    .spacing([1.0, 1.0])
                    .show(ui, |ui| {
                        for y in 0..self.game.height {
                            for x in 0..self.game.width {
                                let tile = self.game.tile(x, y).unwrap();
                                let label = self.tile_label(x, y);
                                let color = self.tile_color(tile.terrain);

                                let mut button = egui::Button::new(label)
                                    .fill(color)
                                    .min_size(egui::vec2(self.zoom, self.zoom));

                                if self.selected_tile == Some((x, y)) {
                                    button =
                                        button.stroke(egui::Stroke::new(2.0, egui::Color32::WHITE));
                                }

                                if ui.add(button).clicked() {
                                    clicked_tile = Some((x, y));
                                }
                            }
                            ui.end_row();
                        }
                    });
            });

            if let Some((x, y)) = clicked_tile {
                self.handle_tile_click(x, y);
            }
        });
    }

    fn handle_tile_click(&mut self, x: usize, y: usize) {
        self.selected_tile = Some((x, y));

        let tile = self.game.tile(x, y).cloned();
        if tile.is_none() {
            return;
        }

        let tile = tile.unwrap();

        if let Some(unit_id) = tile.unit {
            if let Some(unit) = self.game.unit(unit_id) {
                if unit.owner == PLAYER_ID {
                    self.selected_unit = Some(unit_id);
                    return;
                }
            }
        }

        if let Some(unit_id) = self.selected_unit {
            match self.game.move_unit_to(unit_id, x, y) {
                Ok(_) => {
                    if self.game.unit(unit_id).is_none() {
                        self.selected_unit = None;
                    }
                }
                Err(err) => self.game.push_log(format!("Move failed: {err}")),
            }
        }
    }

    fn tile_label(&self, x: usize, y: usize) -> String {
        let tile = self.game.tile(x, y).unwrap();

        if let Some(unit_id) = tile.unit {
            if let Some(unit) = self.game.unit(unit_id) {
                let symbol = match unit.kind {
                    UnitKind::ColonyPod => "C",
                    UnitKind::ScoutPatrol => "S",
                };

                if unit.owner == PLAYER_ID {
                    return symbol.to_string();
                }

                return symbol.to_lowercase();
            }
        }

        if let Some(base_id) = tile.base {
            if let Some(base) = self.game.base(base_id) {
                if base.owner == PLAYER_ID {
                    return "⌂".to_string();
                }

                return "⌁".to_string();
            }
        }

        tile.terrain.symbol().to_string()
    }

    fn tile_color(&self, terrain: Terrain) -> egui::Color32 {
        match terrain {
            Terrain::Ocean => egui::Color32::from_rgb(28, 72, 120),
            Terrain::Flat => egui::Color32::from_rgb(46, 120, 62),
            Terrain::Rolling => egui::Color32::from_rgb(90, 128, 56),
            Terrain::Rocky => egui::Color32::from_rgb(105, 96, 86),
            Terrain::Fungus => egui::Color32::from_rgb(136, 54, 128),
        }
    }
}
