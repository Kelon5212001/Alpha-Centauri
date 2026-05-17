use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use smac_core::{SelectionPanelDisplayState, UnitSelectionDisplayState, TileSelectionDisplayState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, render_ui);
    }
}

fn render_ui(
    mut contexts: EguiContexts,
    mut game_state: ResMut<crate::GameStateResource>,
    selection: Res<crate::SelectionState>,
) {
    let ctx = contexts.ctx_mut();
    
    // Apply Sci-Fi Theme
    let mut visuals = egui::Visuals::dark();
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_black_alpha(200);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255)); // Cyan glow
    visuals.selection.bg_fill = egui::Color32::from_rgb(255, 140, 0); // Orange accents
    ctx.set_visuals(visuals);

    let owner = game_state.0.player_owner();
    
    if let Some(game_over) = &game_state.0.game_over {
        egui::Window::new("Mission Conclusion")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading(format!("{:?}", game_over));
                ui.separator();
                ui.label("The survival of humanity has been decided.");
                if ui.button("Exit to Terminal").clicked() {
                    std::process::exit(0);
                }
            });
    }

    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(350.0)
        .show(ctx, |ui| {
            ui.heading("SMAC Rust AI");
            ui.separator();
            
            let turn = game_state.0.turn;
            ui.label(format!("Turn: {}", turn));
            
            if ui.button("End Turn").clicked() {
                game_state.0.end_turn();
            }

            ui.separator();
            ui.heading("Commander's Brief");
            
            let summary = game_state.0.player_turn_summary();
            
            if summary.alerts.is_empty() {
                ui.label("No active alerts.");
            } else {
                for alert in &summary.alerts {
                    let color = match alert.priority {
                        smac_core::AlertPriority::Critical => egui::Color32::RED,
                        smac_core::AlertPriority::High => egui::Color32::from_rgb(255, 140, 0),
                        smac_core::AlertPriority::Medium => egui::Color32::YELLOW,
                    };
                    ui.colored_label(color, &alert.message);
                }
            }

            ui.separator();
            ui.heading("Selection");
            let selection_display = game_state.0.selection_panel_display_state(selection.selected_unit, selection.selected_tile, owner);
            match selection_display.unit {
                UnitSelectionDisplayState::None { message_text } => { ui.label(message_text); }
                UnitSelectionDisplayState::Missing { message_text } => { ui.label(message_text); }
                UnitSelectionDisplayState::Selected { label_text, owner_text, hp_text, moves_text, .. } => {
                    ui.label(label_text);
                    ui.label(owner_text);
                    ui.label(hp_text);
                    ui.label(moves_text);
                }
            }
            match selection_display.tile {
                TileSelectionDisplayState::None { message_text } => { ui.label(message_text); }
                TileSelectionDisplayState::Unexplored { coordinates_text, message_text } => {
                    ui.label(coordinates_text);
                    ui.label(message_text);
                }
                TileSelectionDisplayState::Selected { coordinates_text, terrain_text, yield_text, improvement_text, .. } => {
                    ui.label(coordinates_text);
                    ui.label(terrain_text);
                    ui.label(yield_text);
                    if let Some(imp) = improvement_text {
                        ui.label(imp);
                    }
                }
            }

            if let Some(base_id) = selection.selected_base {
                ui.separator();
                if let Some(base_display) = game_state.0.base_panel_display_state(base_id, owner) {
                    ui.heading(base_display.heading_text);
                    ui.label(base_display.population_text);
                    ui.label(base_display.storage_text);
                    ui.label(base_display.effective_output_text);
                    ui.label(base_display.production_text);
                    ui.label(base_display.queue_text);
                }
            }

            ui.separator();
            ui.heading("Event Highlights");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for highlight in &summary.event_highlights {
                    ui.label(highlight);
                }
            });
        });
}
