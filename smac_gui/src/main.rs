use eframe::egui;
use smac_core::{
    base_focus_filter_label, base_sort_mode_label, current_save_slot_label,
    logistics_route_filter_label, logistics_route_sort_label, normalize_save_id, presentation,
    save::{save_conflict_display_state, PendingSaveConflict, SaveBrowserGlobalActionType},
    save_browser_display_state, save_filter_label, save_slot_label, set_save_sort, BaseFocusFilter,
    BaseSortMode, GameAction, GameState, GameStateSnapshot, GovernorMode, Improvement,
    LogisticsRouteFilter, LogisticsRouteSort, MapOverlay, PlayerOperationsActionType,
    ResearchUnlockPreviewState, SaveBrowserQuery, SaveFilterCategory, SaveSlotCategory,
    SaveSlotListing, SaveSlotMetadata, SaveSortColumn, SidebarTab, Tech,
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const VERIFIED_DEMO_WIDTH: usize = 20;
const VERIFIED_DEMO_HEIGHT: usize = 20;
const VERIFIED_DEMO_SEED: u32 = 7;
const VERIFIED_DEMO_TURN_LIMIT: usize = 100;
const DEFAULT_OBSERVER_TURNS_PER_SECOND: u32 = 4;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 880.0])
            .with_title(presentation::ui_window_title()),
        ..Default::default()
    };

    eframe::run_native(
        presentation::ui_app_title(),
        options,
        Box::new(|_cc| Box::new(SmacApp::default())),
    )
}

struct SmacApp {
    game: GameState,
    selected_unit: Option<usize>,
    selected_tile: Option<(usize, usize)>,
    zoom: f32,
    map_overlay: MapOverlay,
    seed_counter: u32,
    selected_save_id: String,
    save_name_input: String,
    save_notes_input: String,
    save_category_input: SaveSlotCategory,
    save_id_input: String,
    external_save_path_input: String,
    save_filter_text: String,
    last_recovery_notes: Vec<String>,
    save_sort_column: SaveSortColumn,
    save_sort_descending: bool,
    save_filter_category: SaveFilterCategory,
    save_filter_recovered_only: bool,
    save_filter_populated_only: bool,
    base_focus_filter: BaseFocusFilter,
    base_sort_mode: BaseSortMode,
    logistics_route_filter: LogisticsRouteFilter,
    logistics_route_sort: LogisticsRouteSort,
    staged_unlock_preview: Option<ResearchUnlockPreviewState>,
    pending_save_conflict: Option<PendingSaveConflict>,
    active_tab: SidebarTab,
    workshop_design: smac_core::UnitDesign,
    show_diplomacy_dialogue: Option<usize>,
    last_event_index_shown: usize,
    dragged_queue_item: Option<(usize, usize)>, // (base_id, item_index)
    editor_tool: EditorTool,
    observer: ObserverState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EditorTool {
    None,
    PlaceUnit {
        kind: smac_core::UnitKind,
        owner: usize,
    },
    PlaceBase {
        owner: usize,
    },
    ChangeTerrain {
        terrain: smac_core::Terrain,
    },
    ClearTile,
}

#[derive(Debug)]
struct ObserverState {
    running: bool,
    target_turns: usize,
    turns_per_second: u32,
    completed_turns: usize,
    last_tick: Option<Instant>,
}

impl Default for ObserverState {
    fn default() -> Self {
        Self {
            running: false,
            target_turns: VERIFIED_DEMO_TURN_LIMIT,
            turns_per_second: DEFAULT_OBSERVER_TURNS_PER_SECOND,
            completed_turns: 0,
            last_tick: None,
        }
    }
}

impl ObserverState {
    fn interval(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.turns_per_second.max(1) as f32)
    }

    fn reset_progress(&mut self) {
        self.running = false;
        self.completed_turns = 0;
        self.last_tick = None;
    }
}

impl Default for SmacApp {
    fn default() -> Self {
        let game = GameState::new_game(36, 26, 1337);
        let player_owner = game.player_owner();
        let selected_unit = game
            .units
            .iter()
            .find(|u| u.alive && u.owner == player_owner)
            .map(|u| u.id);

        let selected_tile = selected_unit
            .and_then(|id| game.unit(id).map(|u| (u.x, u.y)))
            .or(Some((3, 3)));

        Self {
            game,
            selected_unit,
            selected_tile,
            zoom: 23.0,
            map_overlay: MapOverlay::Terrain,
            seed_counter: 1337,
            selected_save_id: "slot_1".to_string(),
            save_name_input: "Save Slot 1".to_string(),
            save_notes_input: String::new(),
            save_category_input: SaveSlotCategory::Manual,
            save_id_input: "slot_1".to_string(),
            external_save_path_input: String::new(),
            save_filter_text: String::new(),
            last_recovery_notes: Vec::new(),
            save_sort_column: SaveSortColumn::Updated,
            save_sort_descending: true,
            save_filter_category: SaveFilterCategory::All,
            save_filter_recovered_only: false,
            save_filter_populated_only: false,
            base_focus_filter: BaseFocusFilter::All,
            base_sort_mode: BaseSortMode::Frontier,
            logistics_route_filter: LogisticsRouteFilter::All,
            logistics_route_sort: LogisticsRouteSort::Severity,
            staged_unlock_preview: None,
            pending_save_conflict: None,
            active_tab: SidebarTab::Selection,
            workshop_design: smac_core::UnitDesign {
                name: "New Design".to_string(),
                chassis: smac_core::Chassis::Infantry,
                weapon: smac_core::Weapon::HandLaser(1),
                armor: smac_core::Armor::SynthMetal(1),
                cost: 12,
                abilities: Vec::new(),
            },
            show_diplomacy_dialogue: None,
            last_event_index_shown: 0,
            dragged_queue_item: None,
            editor_tool: EditorTool::None,
            observer: ObserverState::default(),
        }
    }
}

impl eframe::App for SmacApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        self.drive_observer(ctx);
        self.draw_top_bar(ctx);
        self.draw_side_panel(ctx);
        self.draw_map(ctx);
        self.draw_event_notifications(ctx);
    }
}

impl SmacApp {
    fn draw_event_notifications(&mut self, ctx: &egui::Context) {
        let log = &self.game.log;
        if log.len() > self.last_event_index_shown {
            for i in self.last_event_index_shown..log.len() {
                let entry = &log[i];
                match entry.category {
                    smac_core::EventCategory::SecretProject | smac_core::EventCategory::Crisis => {
                        let mut open = true;
                        egui::Window::new("Important Event")
                            .collapsible(false)
                            .resizable(false)
                            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                            .show(ctx, |ui| {
                                ui.heading(match entry.category {
                                    smac_core::EventCategory::SecretProject => {
                                        "GLOBAL WONDER COMPLETED"
                                    }
                                    _ => "CRISIS ALERT",
                                });
                                ui.add_space(10.0);
                                ui.label(&entry.message);
                                ui.add_space(20.0);
                                if ui.button("Acknowledged").clicked() {
                                    open = false;
                                }
                            });
                        if !open {
                            self.last_event_index_shown = i + 1;
                        }
                        // Only show one popup at a time
                        return;
                    }
                    _ => {
                        self.last_event_index_shown = i + 1;
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.select_next_unit();
            }
            if i.key_pressed(egui::Key::Enter) {
                self.apply_enabled_automations();
                self.selected_unit = None;
                let _ = self.apply_action(GameAction::EndTurn);
                self.select_next_unit();
            }
            if i.key_pressed(egui::Key::Num1) {
                if let Some(tab) = self.game.handle_sidebar_hotkey("1") {
                    self.active_tab = tab;
                }
            }
            if i.key_pressed(egui::Key::Num2) {
                if let Some(tab) = self.game.handle_sidebar_hotkey("2") {
                    self.active_tab = tab;
                }
            }
            if i.key_pressed(egui::Key::W) {
                if let Some(tab) = self.game.handle_sidebar_hotkey("W") {
                    self.active_tab = tab;
                }
            }
            if i.key_pressed(egui::Key::Num3) {
                if let Some(tab) = self.game.handle_sidebar_hotkey("3") {
                    self.active_tab = tab;
                }
            }
            if i.key_pressed(egui::Key::Num4) {
                if let Some(tab) = self.game.handle_sidebar_hotkey("4") {
                    self.active_tab = tab;
                }
            }
            if i.key_pressed(egui::Key::Num5) {
                if let Some(tab) = self.game.handle_sidebar_hotkey("5") {
                    self.active_tab = tab;
                }
            }
        });
    }

    fn player_owner(&self) -> usize {
        self.game.player_owner()
    }

    fn selected_base_id(&self) -> Option<usize> {
        self.selected_tile
            .and_then(|(x, y)| self.game.tile(x, y))
            .and_then(|tile| tile.base)
    }

    fn is_player_owned(&self, owner: usize) -> bool {
        owner == self.player_owner()
    }

    fn stage_unlock_preview_action_for_tech(
        &mut self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
        focus_result: bool,
    ) -> usize {
        let result = self.game.stage_research_unlock_preview_state_action(
            owner,
            tech,
            max_steps,
            self.selected_base_id(),
        );
        self.apply_preview_state_action_result(result, focus_result)
    }

    fn apply_preview_state_action_result(
        &mut self,
        result: smac_core::ResearchUnlockPreviewStateActionResult,
        focus_result: bool,
    ) -> usize {
        let staged_count = result.staged_count;
        self.staged_unlock_preview = result.preview;
        if focus_result {
            if let Some(base_id) = result.focus_base_id {
                self.focus_base(base_id);
            }
        }
        staged_count
    }

    fn sync_staged_unlock_preview(&mut self, owner: usize) {
        let result = self
            .game
            .sync_research_unlock_preview_state_action(owner, self.staged_unlock_preview.clone());
        let _ = self.apply_preview_state_action_result(result, false);
    }

    fn refresh_staged_unlock_preview(&mut self, owner: usize) {
        let result = self.game.refresh_research_unlock_preview_state_action(
            owner,
            self.staged_unlock_preview.clone(),
            self.selected_base_id(),
        );
        let _ = self.apply_preview_state_action_result(result, true);
    }

    fn apply_staged_unlock_preview_to_base(&mut self, owner: usize, base_id: usize) {
        match self.game.apply_research_unlock_preview_state_action(
            owner,
            self.staged_unlock_preview.clone(),
            base_id,
        ) {
            Ok(result) => {
                let _ = self.apply_preview_state_action_result(result, true);
            }
            Err(err) => self.game.push_log(err),
        }
    }

    fn apply_all_staged_unlock_previews(&mut self, owner: usize) {
        match self.game.apply_all_research_unlock_preview_state_action(
            owner,
            self.staged_unlock_preview.clone(),
        ) {
            Ok(result) => {
                let _ = self.apply_preview_state_action_result(result, true);
            }
            Err(err) => self.game.push_log(err),
        }
    }

    fn save_path_for_id(&self, id: &str) -> PathBuf {
        SaveSlotMetadata::save_path_for_id("saves", id)
    }

    fn current_save_path(&self) -> PathBuf {
        self.save_path_for_id(&self.selected_save_id)
    }

    fn metadata_path_for_save_path(path: &Path) -> PathBuf {
        SaveSlotMetadata::metadata_path_for_save_path(path)
    }

    fn default_save_slot_name(&self, id: &str) -> String {
        self.game.default_save_slot_name(id)
    }

    fn available_save_slots(&self) -> Vec<SaveSlotListing> {
        SaveSlotMetadata::discover_slots("saves")
    }

    fn set_save_sort(&mut self, column: SaveSortColumn) {
        let (column, descending) =
            set_save_sort(self.save_sort_column, self.save_sort_descending, column);
        self.save_sort_column = column;
        self.save_sort_descending = descending;
    }

    fn save_browser_query(&self) -> SaveBrowserQuery {
        SaveBrowserQuery {
            filter_text: self.save_filter_text.clone(),
            filter_category: self.save_filter_category,
            recovered_only: self.save_filter_recovered_only,
            populated_only: self.save_filter_populated_only,
            sort_column: self.save_sort_column,
            sort_descending: self.save_sort_descending,
        }
    }

    fn sync_save_inputs_for_selection(&mut self) {
        let path = self.current_save_path();
        self.save_id_input = self.selected_save_id.clone();
        if let Ok(metadata) = SaveSlotMetadata::load_from_path(&path) {
            self.save_name_input = metadata.save_name;
            self.save_notes_input = metadata.notes;
            self.save_category_input = metadata.category.unwrap_or(SaveSlotCategory::Manual);
        } else {
            self.save_name_input = self.default_save_slot_name(&self.selected_save_id);
            self.save_notes_input.clear();
            self.save_category_input = SaveSlotCategory::Manual;
        }
    }

    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        let display_state = self.game.top_bar_display_state(self.player_owner());

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(presentation::ui_app_title());
                ui.separator();
                ui.label(display_state.year_text);
                ui.separator();

                if let Some(energy_text) = display_state.energy_text {
                    ui.label(energy_text);
                    if let Some(research_text) = display_state.research_text {
                        ui.label(research_text);
                    }
                    ui.separator();
                    if let Some(food_text) = display_state.food_text {
                        ui.label(food_text);
                    }
                    if let Some(toxicity_text) = display_state.toxicity_text {
                        ui.label(toxicity_text);
                    }
                    if let Some(ai_text) = display_state.ai_dependence_text {
                        ui.label(ai_text);
                    }

                    if let Some(dust_fall_warning) = display_state.dust_fall_warning {
                        ui.separator();
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 0), dust_fall_warning);
                    }
                }

                ui.separator();

                if ui.button("Next Unit").clicked() {
                    self.select_next_unit();
                }

                if ui.button("End Turn").clicked() {
                    self.apply_enabled_automations();
                    self.selected_unit = None;
                    let _ = self.apply_action(GameAction::EndTurn);
                    self.select_next_unit();
                }

                ui.separator();

                if ui.button("New Game").clicked() {
                    self.seed_counter = self.seed_counter.wrapping_add(77);
                    self.game = GameState::new_game(36, 26, self.seed_counter);
                    self.last_recovery_notes.clear();
                    self.pending_save_conflict = None;
                    self.observer.reset_progress();
                    self.reset_selection_after_game_change();
                }

                ui.separator();

                if ui.button("Restart Demo").clicked() {
                    self.reset_to_verified_demo();
                }

                let watch_label = if self.observer.running {
                    "Pause Watch"
                } else {
                    "Watch Sim"
                };
                if ui.button(watch_label).clicked() {
                    if self.observer.running {
                        self.pause_observer();
                    } else {
                        self.start_observer();
                    }
                }

                if ui.button("Step Sim").clicked() {
                    self.pause_observer();
                    self.advance_observer_turns(1);
                }

                ui.add(egui::Slider::new(&mut self.observer.turns_per_second, 1..=12).text("TPS"));
                ui.add(
                    egui::DragValue::new(&mut self.observer.target_turns)
                        .clamp_range(1..=1_000)
                        .speed(1.0)
                        .prefix("cap "),
                );
                ui.label(self.observer_status_text());

                ui.separator();

                let map_panel = self.game.map_panel_display_state(self.map_overlay);
                egui::ComboBox::from_label(map_panel.overlay_label_text)
                    .selected_text(map_panel.selected_overlay_label_text)
                    .show_ui(ui, |ui| {
                        for option in map_panel.overlay_options {
                            ui.selectable_value(
                                &mut self.map_overlay,
                                option.overlay,
                                option.label_text,
                            );
                        }
                    });

                ui.add(egui::Slider::new(&mut self.zoom, 15.0..=34.0).text("zoom"));
            });
            ui.small(
                self.game
                    .map_panel_display_state(self.map_overlay)
                    .overlay_legend_text,
            );

            if !self.last_recovery_notes.is_empty() {
                ui.separator();
                ui.colored_label(
                    color32_from_hex(presentation::ui_warning_hex(), egui::Color32::YELLOW),
                    "Recovered save loaded",
                );
                for note in &self.last_recovery_notes {
                    ui.small(note);
                }
            }

            if let Some(display) = self.game.game_over_display_state() {
                ui.separator();
                ui.colored_label(
                    color32_from_hex(display.color_hex, egui::Color32::WHITE),
                    display.message_text,
                );
            }
        });
    }

    fn observer_status_text(&self) -> String {
        let state = if self.observer.running {
            "running"
        } else {
            "paused"
        };
        format!(
            "Observer {} {}/{}",
            state, self.observer.completed_turns, self.observer.target_turns
        )
    }

    fn reset_to_verified_demo(&mut self) {
        self.game = GameState::new_game(
            VERIFIED_DEMO_WIDTH,
            VERIFIED_DEMO_HEIGHT,
            VERIFIED_DEMO_SEED,
        );
        self.last_recovery_notes.clear();
        self.pending_save_conflict = None;
        self.observer.target_turns = VERIFIED_DEMO_TURN_LIMIT;
        self.observer.reset_progress();
        self.reset_selection_after_game_change();
        self.game.push_log(format!(
            "Observer demo reset: {} turns on {}x{} with seed {}.",
            VERIFIED_DEMO_TURN_LIMIT, VERIFIED_DEMO_WIDTH, VERIFIED_DEMO_HEIGHT, VERIFIED_DEMO_SEED
        ));
    }

    fn start_observer(&mut self) {
        if self.game.game_over.is_some() {
            self.game
                .push_log("Observer cannot start because the game is already over.".to_string());
            return;
        }
        if self.observer.completed_turns >= self.observer.target_turns {
            self.game.push_log(
                "Observer turn cap already reached. Raise the cap or restart the demo.".to_string(),
            );
            return;
        }

        self.observer.running = true;
        self.observer.last_tick = None;
    }

    fn pause_observer(&mut self) {
        self.observer.running = false;
        self.observer.last_tick = None;
    }

    fn drive_observer(&mut self, ctx: &egui::Context) {
        if !self.observer.running {
            self.observer.last_tick = None;
            return;
        }

        let interval = self.observer.interval();
        let now = Instant::now();
        let should_advance = match self.observer.last_tick {
            Some(last_tick) => now.duration_since(last_tick) >= interval,
            None => true,
        };

        if should_advance {
            self.observer.last_tick = Some(now);
            self.advance_observer_turns(1);
        }

        if self.observer.running {
            let wait = self
                .observer
                .last_tick
                .map(|last_tick| interval.saturating_sub(now.saturating_duration_since(last_tick)))
                .unwrap_or(interval)
                .max(Duration::from_millis(16));
            ctx.request_repaint_after(wait);
        }
    }

    fn advance_observer_turns(&mut self, max_turns: usize) -> usize {
        let mut advanced = 0;
        while advanced < max_turns
            && self.observer.completed_turns < self.observer.target_turns
            && self.game.game_over.is_none()
        {
            self.selected_unit = None;
            self.game.run_autoplay_mission_year();
            self.observer.completed_turns += 1;
            advanced += 1;
        }

        if advanced > 0 {
            self.select_next_unit();
        }

        if self.observer.completed_turns >= self.observer.target_turns {
            self.pause_observer();
            self.game.push_log(format!(
                "Observer stopped after reaching {} turns.",
                self.observer.target_turns
            ));
        } else if self.game.game_over.is_some() {
            self.pause_observer();
            if let Some(display) = self.game.game_over_display_state() {
                self.game
                    .push_log(format!("Observer stopped: {}.", display.message_text));
            }
        }

        advanced
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
        let sidebar = self.game.sidebar_display_state(self.active_tab);
        egui::SidePanel::right("side_panel")
            .default_width(380.0)
            .frame(
                egui::Frame::side_top_panel(&ctx.style()).fill(color32_from_hex(
                    presentation::ui_panel_fill_hex(),
                    egui::Color32::from_rgb(22, 32, 25),
                )),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for state in sidebar.tabs {
                        let label = format!("({}) {}", state.hotkey_text, state.label_text);
                        if ui.selectable_label(state.selected, label).clicked() {
                            self.active_tab = state.tab;
                        }
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(sidebar.heading_text);
                    match self.active_tab {
                        SidebarTab::Selection => {
                            self.draw_selection_info(ui);
                            ui.separator();
                            self.draw_minimap(ui);
                        }
                        SidebarTab::Factions => {
                            self.draw_research_panel(ui);
                            ui.separator();
                            self.draw_tech_tree_panel(ui);
                            ui.separator();
                            self.draw_faction_status(ui);
                            ui.separator();
                            self.draw_social_engineering_panel(ui);
                        }
                        SidebarTab::Projects => {
                            self.draw_secret_projects_panel(ui);
                        }
                        SidebarTab::Workshop => {
                            self.draw_workshop_panel(ui);
                        }
                        SidebarTab::Logistics => {
                            let board = self.game.logistics_board_display_state();
                            ui.heading(board.heading_text);
                            ui.label(board.gameplay_loop_heading_text);
                            for line in &board.gameplay_loop_steps {
                                ui.label(*line);
                            }
                            ui.separator();
                            ui.label(board.overview_text);
                            for route in &board.active_routes {
                                ui.small(route);
                            }
                        }
                        SidebarTab::Saves => {
                            self.draw_save_controls(ui);
                            ui.separator();
                            self.draw_save_browser(ui);
                        }
                        SidebarTab::Logs => {
                            let command_console = self.game.command_console_display_state();
                            ui.heading(command_console.event_log_heading_text);
                            for (message, color_hex) in &command_console.event_log {
                                if let Some(hex) = color_hex {
                                    ui.colored_label(
                                        color32_from_hex(hex, egui::Color32::LIGHT_GRAY),
                                        message,
                                    );
                                } else {
                                    ui.label(message);
                                }
                            }
                        }
                        SidebarTab::Editor => {
                            self.draw_scenario_editor(ui);
                        }
                    }
                });
            });
    }

    fn draw_workshop_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Unit Workshop");
        ui.separator();

        ui.horizontal(|ui| {
            // Configuration Panel
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.label("Design Name:");
                    ui.text_edit_singleline(&mut self.workshop_design.name);
                });

                self.workshop_design.recompute_cost();
                let display_state = self.workshop_design.display_state();

                ui.group(|ui| {
                    ui.label("Chassis:");
                    egui::ComboBox::from_id_source("workshop_chassis")
                        .selected_text(&display_state.chassis_text)
                        .show_ui(ui, |ui| {
                            for (value, label) in display_state.chassis_options {
                                ui.selectable_value(
                                    &mut self.workshop_design.chassis,
                                    value,
                                    label,
                                );
                            }
                        });
                });

                ui.group(|ui| {
                    ui.label("Weapon:");
                    egui::ComboBox::from_id_source("workshop_weapon")
                        .selected_text(&display_state.weapon_text)
                        .show_ui(ui, |ui| {
                            for (value, label) in display_state.weapon_options {
                                ui.selectable_value(&mut self.workshop_design.weapon, value, label);
                            }
                        });
                });

                ui.group(|ui| {
                    ui.label("Armor:");
                    egui::ComboBox::from_id_source("workshop_armor")
                        .selected_text(&display_state.armor_text)
                        .show_ui(ui, |ui| {
                            for (value, label) in display_state.armor_options {
                                ui.selectable_value(&mut self.workshop_design.armor, value, label);
                            }
                        });
                });

                ui.group(|ui| {
                    ui.label("Abilities:");
                    ui.horizontal_wrapped(|ui| {
                        for (ability, label) in display_state.ability_options {
                            let mut has_ability = self.workshop_design.abilities.contains(&ability);
                            if ui.checkbox(&mut has_ability, label).changed() {
                                if has_ability {
                                    self.workshop_design.abilities.push(ability);
                                } else {
                                    self.workshop_design.abilities.retain(|a| *a != ability);
                                }
                            }
                        }
                    });
                });
            });

            // Preview Panel
            ui.separator();
            ui.vertical(|ui| {
                ui.heading("Design Preview");
                ui.add_space(10.0);

                let display_state = self.workshop_design.display_state();
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new(&display_state.attack_text)
                            .strong()
                            .color(egui::Color32::LIGHT_RED),
                    );
                    ui.label(
                        egui::RichText::new(&display_state.defense_text)
                            .strong()
                            .color(egui::Color32::LIGHT_BLUE),
                    );
                    ui.separator();
                    ui.label(egui::RichText::new(&display_state.cost_text).strong());
                });

                ui.add_space(20.0);
                if ui
                    .button(egui::RichText::new("✧ Register Design").size(16.0))
                    .clicked()
                {
                    let _ = self.game.apply_action(smac_core::GameAction::DesignUnit {
                        owner: self.player_owner(),
                        design: self.workshop_design.clone(),
                    });
                }
            });
        });
    }

    fn draw_save_controls(&mut self, ui: &mut egui::Ui) {
        let available_entries = self.available_save_slots();
        let display =
            smac_core::save_management_display_state(&available_entries, &self.selected_save_id);

        ui.heading(display.heading_text);
        ui.horizontal(|ui| {
            if ui.button("Save Manual").clicked() {
                self.save_game_as(SaveSlotCategory::Manual);
            }
            if ui.button("Save Autosave").clicked() {
                self.save_game_as(SaveSlotCategory::Autosave);
            }
            ui.add_enabled_ui(display.can_load, |ui| {
                if ui.button("Load Selection").clicked() {
                    self.load_game();
                }
            });
        });

        ui.group(|ui| {
            let previous_id = self.selected_save_id.clone();
            let current_slot_label =
                current_save_slot_label(&available_entries, &self.selected_save_id);

            ui.label(display.target_slot_label);
            egui::ComboBox::from_id_source("sidebar_save_slot")
                .selected_text(current_slot_label)
                .show_ui(ui, |ui| {
                    for entry in &available_entries {
                        let label = save_slot_label(entry);
                        ui.selectable_value(&mut self.selected_save_id, entry.id.clone(), label);
                    }
                });

            if self.selected_save_id != previous_id {
                self.sync_save_inputs_for_selection();
            }

            ui.label(display.file_id_label);
            ui.text_edit_singleline(&mut self.save_id_input);
            ui.label(display.display_name_label);
            ui.text_edit_singleline(&mut self.save_name_input);

            ui.label(display.category_label);
            egui::ComboBox::from_id_source("sidebar_save_category")
                .selected_text(presentation::save_slot_category_label(
                    self.save_category_input,
                ))
                .show_ui(ui, |ui| {
                    for (category, label) in display.category_options {
                        ui.selectable_value(&mut self.save_category_input, category, label);
                    }
                });
        });
    }

    fn draw_selection_info(&mut self, ui: &mut egui::Ui) {
        let selection_panel = self.game.selection_panel_display_state(
            self.selected_unit,
            self.selected_tile,
            self.player_owner(),
        );
        ui.heading(selection_panel.heading_text);

        match selection_panel.unit {
            smac_core::UnitSelectionDisplayState::None { message_text } => {
                ui.label(message_text);
            }
            smac_core::UnitSelectionDisplayState::Missing { message_text } => {
                ui.label(message_text);
                self.selected_unit = None;
            }
            smac_core::UnitSelectionDisplayState::Selected {
                unit_id,
                owner,
                label_text,
                rank_text,
                rank_color_hex,
                role_text,
                location_text,
                moves_text,
                moves_color_hex,
                hp_text,
                hp_color_hex,
                owner_text,
                advice_text,
                advice_color_hex,
                fallback_text,
                fallback_target,
                found_base_label_text,
                terraform_heading_text,
                terraform_actions,
                fallback_button_text,
                upgrade_options,
                ..
            } => {
                ui.label(label_text);
                ui.label(owner_text);
                ui.colored_label(
                    color32_from_hex(rank_color_hex, egui::Color32::LIGHT_GRAY),
                    rank_text,
                );
                ui.small(role_text);
                ui.label(location_text);
                ui.colored_label(
                    color32_from_hex(moves_color_hex, egui::Color32::WHITE),
                    moves_text,
                );
                ui.colored_label(
                    color32_from_hex(hp_color_hex, egui::Color32::WHITE),
                    hp_text,
                );

                if !upgrade_options.is_empty() {
                    ui.separator();
                    ui.label("Available Upgrades:");
                    ui.horizontal_wrapped(|ui| {
                        for design in upgrade_options {
                            if ui.button(format!("Upgr to {}", design.name)).clicked() {
                                let _ = self.apply_action(GameAction::UpgradeUnit {
                                    unit_id,
                                    new_design: design.clone(),
                                });
                            }
                        }
                    });
                }
                if let Some(advice) = advice_text {
                    ui.colored_label(
                        color32_from_hex(advice_color_hex, egui::Color32::YELLOW),
                        advice,
                    );
                }
                if owner == self.player_owner() {
                    if let Some((fx, fy)) = fallback_target {
                        ui.horizontal(|ui| {
                            if let Some(text) = &fallback_text {
                                ui.small(text);
                            }
                            if ui.button(fallback_button_text).clicked() {
                                match self.apply_action(GameAction::MoveUnit {
                                    unit_id,
                                    target_x: fx,
                                    target_y: fy,
                                }) {
                                    Ok(_) => self.focus_unit(unit_id),
                                    Err(err) => {
                                        self.game.push_log(format!("Fallback failed: {err}"))
                                    }
                                }
                            }
                        });
                    }

                    if let Some(found_base_label_text) = found_base_label_text {
                        if ui.button(found_base_label_text).clicked() {
                            match self.apply_action(GameAction::FoundBase { unit_id }) {
                                Ok(_) => {
                                    self.selected_unit = None;
                                    self.select_next_unit();
                                }
                                Err(err) => self.game.push_log(format!("Cannot found base: {err}")),
                            }
                        }
                    }

                    if !terraform_actions.is_empty() {
                        ui.label(terraform_heading_text);
                        ui.horizontal_wrapped(|ui| {
                            for action in terraform_actions {
                                if ui.button(action.button_text).clicked() {
                                    self.try_build_improvement(unit_id, action.improvement);
                                }
                            }
                        });
                    }
                }
            }
        }

        ui.separator();

        match selection_panel.tile {
            smac_core::TileSelectionDisplayState::None { message_text } => {
                ui.label(message_text);
            }
            smac_core::TileSelectionDisplayState::Unexplored {
                coordinates_text,
                message_text,
            } => {
                ui.label(coordinates_text);
                ui.label(message_text);
            }
            smac_core::TileSelectionDisplayState::Selected {
                coordinates_text,
                terrain_text,
                elevation_text,
                moisture_text,
                yield_text,
                improvement_text,
                warning_text,
                base_id,
            } => {
                ui.label(coordinates_text);
                ui.label(terrain_text);
                ui.label(elevation_text);
                ui.label(moisture_text);
                ui.label(yield_text);
                if let Some(improvement_text) = improvement_text {
                    ui.label(improvement_text);
                }
                if let Some(warning_text) = warning_text {
                    ui.colored_label(
                        color32_from_hex(presentation::ui_warning_hex(), egui::Color32::YELLOW),
                        warning_text,
                    );
                }
                if let Some(base_id) = base_id {
                    self.draw_base_panel(ui, base_id);
                }
            }
        }
    }

    fn draw_base_panel(&mut self, ui: &mut egui::Ui, base_id: usize) {
        let Some(base_panel) = self
            .game
            .base_panel_display_state(base_id, self.player_owner())
        else {
            return;
        };

        ui.separator();
        ui.heading(&base_panel.heading_text);
        ui.label(&base_panel.owner_text);
        ui.label(&base_panel.population_text);
        ui.label(&base_panel.governor_text);
        ui.label(&base_panel.area_role_text);
        ui.label(&base_panel.stability_text);
        ui.label(&base_panel.storage_text);
        ui.label(&base_panel.output_text);
        ui.label(&base_panel.effective_output_text);
        if let Some(waste) = &base_panel.waste_text {
            ui.colored_label(egui::Color32::from_rgb(200, 100, 100), waste);
        }
        if let Some(limit) = &base_panel.expansion_limit_text {
            ui.colored_label(egui::Color32::from_rgb(200, 100, 100), limit);
        }
        ui.label(&base_panel.defense_pressure_text);
        ui.label(&base_panel.psi_pressure_text);
        ui.label(&base_panel.damaged_garrisons_text);
        ui.horizontal_wrapped(|ui| {
            ui.small(base_panel.status_tags_heading_text);
            for tag in &base_panel.status_tags {
                let hex = presentation::base_status_tag_color_hex(tag.kind);
                let color = color32_from_hex(hex, egui::Color32::YELLOW);
                ui.colored_label(color, tag.label_text);
            }
        });
        ui.horizontal(|ui| {
            ui.label(&base_panel.production_text);
            let cost_minerals = smac_core::content_api::production_cost(
                self.game.base(base_id).unwrap().production,
            );
            let remaining =
                (cost_minerals - self.game.base(base_id).unwrap().minerals_stock).max(0);
            let rush_cost = remaining * 2;

            if ui
                .button(format!("Rush ({} E)", rush_cost))
                .on_hover_text("Complete current production item using energy credits.")
                .clicked()
            {
                let _ = self.apply_action(GameAction::RushBuild { base_id });
            }
        });
        ui.small(&base_panel.production_role_text);
        ui.small(&base_panel.production_dependency_text);
        ui.small(&base_panel.production_tooltip_text);
        if let Some(governor_alignment_text) = &base_panel.governor_alignment_text {
            ui.small(governor_alignment_text);
        }
        ui.label(&base_panel.queue_text);
        ui.label(&base_panel.facilities_text);
        ui.small(&base_panel.build_availability_text);
        if let Some(research_focus_heading_text) = base_panel.research_focus_heading_text {
            ui.group(|ui| {
                ui.small(research_focus_heading_text);
                if let Some(research_focus_text) = &base_panel.research_focus_text {
                    ui.small(research_focus_text);
                }
                for line in &base_panel.research_unlock_lines {
                    ui.small(line);
                }
            });
        }
        ui.label(&base_panel.convoy_capacity_text);
        ui.horizontal_wrapped(|ui| {
            for tag in &base_panel.convoy_status_tags {
                let hex = presentation::base_status_tag_color_hex(tag.kind);
                let color = color32_from_hex(hex, egui::Color32::WHITE);
                ui.colored_label(color, tag.label_text);
            }
        });
        ui.label(&base_panel.active_convoy_links_text);
        ui.label(&base_panel.military_supply_links_text);
        if let Some(convoy_routes_empty_text) = base_panel.convoy_routes_empty_text {
            ui.small(convoy_routes_empty_text);
        } else {
            ui.label(base_panel.convoy_routes_heading_text);
            for route in &base_panel.convoy_routes {
                ui.horizontal(|ui| {
                    ui.small(&route.row_text);
                    if base_panel.can_manage && ui.small_button(&route.remove_label_text).clicked()
                    {
                        self.game.remove_convoy_route_action(
                            base_id,
                            route.target_base_id,
                            Some(route.kind),
                        );
                    }
                    if base_panel.can_manage
                        && route.can_repair
                        && ui.small_button(&route.repair_label_text).clicked()
                    {
                        self.game.repair_convoy_route_action(
                            base_id,
                            route.target_base_id,
                            route.kind,
                        );
                    }
                });
            }
        }
        if base_panel.can_manage {
            if let Some(available_convoy_targets_heading_text) =
                base_panel.available_convoy_targets_heading_text
            {
                ui.label(available_convoy_targets_heading_text);
                for opportunity in &base_panel.available_convoy_targets {
                    if ui.button(&opportunity.button_text).clicked() {
                        self.game.add_convoy_route_action(
                            base_id,
                            opportunity.target_base_id,
                            opportunity.kind,
                        );
                    }
                }
            }
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(base_panel.governor_heading_text);
            let mut governor_mode = base_panel.current_governor_mode;
            egui::ComboBox::from_id_source(format!("governor_mode_{}", base_id))
                .selected_text(presentation::governor_mode_label(governor_mode))
                .show_ui(ui, |ui| {
                    for (mode, label) in &base_panel.governor_mode_options {
                        ui.selectable_value(&mut governor_mode, *mode, *label)
                            .on_hover_text(presentation::governor_mode_description(*mode));
                    }
                });
            ui.small(base_panel.current_governor_description);
            if governor_mode != base_panel.current_governor_mode {
                let _ = self
                    .game
                    .set_base_governor_mode_action(base_id, governor_mode);
            }
        });
        if let Some(governor_plan_heading_text) = base_panel.governor_plan_heading_text {
            ui.colored_label(
                color32_from_hex(presentation::ui_warning_hex(), egui::Color32::YELLOW),
                governor_plan_heading_text,
            );
            for step in &base_panel.governor_plan_rows {
                ui.horizontal_wrapped(|ui| {
                    ui.small(&step.reason_text);
                    if step.can_apply && ui.button(&step.apply_label_text).clicked() {
                        let _ = self.game.set_base_production_action(base_id, step.item);
                    }
                });
            }
            if let Some(label) = base_panel.queue_governor_plan_label_text {
                if ui.button(label).clicked() {
                    self.apply_governor_plan_queue(base_id, 3);
                }
            }
            if let Some(label) = base_panel.apply_recovery_plan_label_text {
                if ui.button(label).clicked() {
                    self.apply_recovery_plan(base_id, 3);
                }
            }
            if let Some(label) = base_panel.apply_defense_plan_label_text {
                if ui.button(label).clicked() {
                    self.apply_defense_plan(base_id, 3);
                }
            }
        }

        if base_panel.can_manage {
            if let Some(queue_editor_heading_text) = base_panel.queue_editor_heading_text {
                ui.label(queue_editor_heading_text);
                for row in &base_panel.queue_rows {
                    let mut frame = egui::Frame::group(ui.style());
                    if self.dragged_queue_item == Some((base_id, row.index)) {
                        frame = frame.fill(egui::Color32::from_gray(60));
                    }

                    frame.show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let response = ui
                                .add(egui::Label::new(&row.label_text).sense(egui::Sense::drag()));
                            let rect = response.rect;
                            let drag_started = response.drag_started();
                            let drag_stopped = response.drag_stopped();

                            response.on_hover_text(&row.tooltip_text);

                            if drag_started {
                                self.dragged_queue_item = Some((base_id, row.index));
                            }

                            if ui.rect_contains_pointer(rect) {
                                if let Some((bid, from_idx)) = self.dragged_queue_item {
                                    if bid == base_id && from_idx != row.index {
                                        if ui.input(|i| i.pointer.any_released()) {
                                            // Perform move
                                            if from_idx > row.index {
                                                for _ in 0..(from_idx - row.index) {
                                                    let _ =
                                                        self.game.move_queued_production_up_action(
                                                            base_id, from_idx,
                                                        );
                                                }
                                            } else {
                                                for _ in 0..(row.index - from_idx) {
                                                    let _ = self
                                                        .game
                                                        .move_queued_production_down_action(
                                                            base_id, from_idx,
                                                        );
                                                }
                                            }
                                            self.dragged_queue_item = None;
                                        }
                                    }
                                }
                            }

                            if drag_stopped {
                                self.dragged_queue_item = None;
                            }

                            if let Some(governor_reason_text) = &row.governor_reason_text {
                                ui.small(governor_reason_text);
                            }

                            // Keep old buttons as fallback and for accessibility
                            if ui.small_button("↑").clicked() {
                                let _ = self
                                    .game
                                    .move_queued_production_up_action(base_id, row.index);
                            }
                            if ui.small_button("↓").clicked() {
                                let _ = self
                                    .game
                                    .move_queued_production_down_action(base_id, row.index);
                            }
                            if ui.small_button("✖").clicked() {
                                let _ = self
                                    .game
                                    .remove_queued_production_action(base_id, row.index);
                            }
                        });
                    });
                }
                if let Some(label) = base_panel.clear_queue_label_text {
                    if ui.small_button(label).clicked() {
                        let _ = self.game.clear_production_queue_action(base_id);
                    }
                }
            }
            if let Some(set_production_heading_text) = base_panel.set_production_heading_text {
                ui.label(set_production_heading_text);
                ui.horizontal_wrapped(|ui| {
                    for option in &base_panel.set_production_options {
                        let button =
                            egui::Button::new(&option.button_text).min_size(egui::vec2(0.0, 0.0));
                        let response = ui.add(button);
                        let response = response.on_hover_text(&option.tooltip_text);
                        if response.clicked() {
                            let _ = self.game.set_base_production_action(base_id, option.item);
                        }
                    }
                });
            }
            if let Some(locked_production_heading_text) = &base_panel.locked_production_heading_text
            {
                ui.collapsing(locked_production_heading_text, |ui| {
                    for option in &base_panel.locked_production_options {
                        let response =
                            ui.add_enabled(option.enabled, egui::Button::new(&option.button_text));
                        response.on_hover_text(&option.tooltip_text);
                    }
                });
            }
            if let Some(queue_item_heading_text) = base_panel.queue_item_heading_text {
                ui.label(queue_item_heading_text);
                ui.horizontal_wrapped(|ui| {
                    for option in &base_panel.queue_item_options {
                        let response = ui.add(egui::Button::new(&option.button_text));
                        let response = response.on_hover_text(&option.tooltip_text);
                        if response.clicked() {
                            let _ = self.game.queue_base_production_action(base_id, option.item);
                        }
                    }
                });
            }
        }
    }

    fn draw_research_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(presentation::ui_research_heading());

        let owner = self.player_owner();
        self.sync_staged_unlock_preview(owner);
        let Some(_faction) = self.game.player_faction().cloned() else {
            return;
        };
        let Some(current_research_state) =
            self.game
                .current_research_display_state(owner, 3, self.selected_base_id())
        else {
            return;
        };
        let research_panel = self.game.research_panel_display_state(
            owner,
            self.selected_base_id(),
            self.staged_unlock_preview.as_ref(),
        );

        ui.label(&current_research_state.label_text);
        ui.small(&current_research_state.description_text);
        ui.group(|ui| {
            ui.small(&current_research_state.preview_heading_text);
            for line in &current_research_state.unlock_lines {
                ui.small(line);
            }
            if !current_research_state.affected_entries.is_empty() {
                ui.group(|ui| {
                    ui.small(current_research_state.affected_entries_heading);
                    if let Some(summary) = &current_research_state.affected_summary_text {
                        ui.small(summary);
                    }
                    if let Some(affected_focus_label_text) =
                        &current_research_state.affected_focus_label_text
                    {
                        if ui.small_button(affected_focus_label_text).clicked() {
                            if let Some(base_id) = current_research_state.affected_focus_base_id {
                                self.focus_base(base_id);
                            }
                        }
                    }
                });
            }
            if !current_research_state.preview_section.rows.is_empty() {
                ui.group(|ui| {
                    ui.small(&current_research_state.preview_section.heading_text);
                    for row in current_research_state.preview_section.rows.iter().take(4) {
                        ui.horizontal_wrapped(|ui| {
                            ui.small(&row.row_text);
                            if ui
                                .small_button(
                                    &current_research_state.preview_section.focus_label_text,
                                )
                                .clicked()
                            {
                                self.focus_base(row.base_id);
                            }
                        });
                    }
                    if let Some(hidden_count_text) =
                        &current_research_state.preview_section.hidden_count_text
                    {
                        ui.small(hidden_count_text);
                    }
                    if ui
                        .small_button(&current_research_state.preview_section.keep_open_label_text)
                        .clicked()
                    {
                        let result = self.game.pin_current_research_unlock_preview_state_action(
                            owner,
                            3,
                            self.selected_base_id(),
                        );
                        let _ = self.apply_preview_state_action_result(result, false);
                    }
                    if ui
                        .small_button(
                            &current_research_state
                                .preview_section
                                .stage_all_log_label_text,
                        )
                        .clicked()
                    {
                        let result = self
                            .game
                            .stage_current_research_unlock_preview_state_action(
                                owner,
                                3,
                                self.selected_base_id(),
                            );
                        let _ = self.apply_preview_state_action_result(result, false);
                    }
                });
            }
        });
        if let Some(staged) = self.staged_unlock_preview.clone() {
            let staged_display = self
                .game
                .pinned_research_unlock_preview_display_state(owner, &staged);
            ui.group(|ui| {
                ui.small(&staged_display.heading_text);
                if let Some(availability_text) = &staged_display.availability_text {
                    ui.small(availability_text);
                }
                if let Some(drift_text) = &staged_display.drift_text {
                    ui.small(drift_text);
                }
                for row in staged_display.rows.iter().take(5) {
                    ui.horizontal_wrapped(|ui| {
                        ui.small(&row.row_text);
                        if let Some(stale_label_text) = &row.stale_label_text {
                            ui.small(stale_label_text);
                        }
                        if ui.small_button(&row.focus_label_text).clicked() {
                            self.focus_base(row.base_id);
                        }
                        let response =
                            ui.add_enabled(row.can_apply, egui::Button::new(&row.apply_label_text));
                        let response = response.on_hover_text(&row.apply_tooltip);
                        if response.clicked() {
                            self.apply_staged_unlock_preview_to_base(owner, row.base_id);
                        }
                    });
                }
                if let Some(hidden_count_text) = &staged_display.hidden_count_text {
                    ui.small(hidden_count_text);
                }
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .small_button(&staged_display.stage_log_label_text)
                        .clicked()
                    {
                        self.stage_unlock_preview_action_for_tech(
                            owner,
                            staged_display.tech,
                            staged_display.max_steps,
                            false,
                        );
                    }
                    if ui
                        .small_button(&staged_display.refresh_label_text)
                        .clicked()
                    {
                        self.refresh_staged_unlock_preview(owner);
                    }
                    let response = ui.add_enabled(
                        staged_display.apply_all_enabled,
                        egui::Button::new(&staged_display.apply_all_label_text),
                    );
                    let response = response.on_hover_text(&staged_display.apply_all_tooltip);
                    if response.clicked() {
                        self.apply_all_staged_unlock_previews(owner);
                    }
                    if ui.small_button(&staged_display.clear_label_text).clicked() {
                        self.staged_unlock_preview = None;
                    }
                });
            });
        }
        ui.label(&research_panel.summary_text);

        ui.collapsing(&research_panel.available_heading_text, |ui| {
            for available in &research_panel.available {
                let tech = available.tech;
                let unlock_impact = &available.unlock_impact;
                ui.horizontal_wrapped(|ui| {
                    if ui.button(&available.label_text).clicked() {
                        let _ = self.apply_action(GameAction::ChooseResearch { owner, tech });
                    }
                    ui.small(&available.cost_text);
                    if unlock_impact.recommendation_count > 0 {
                        if let Some(unlock_impact_text) = &available.unlock_impact_text {
                            ui.small(unlock_impact_text);
                        }
                        if let Some(preview_status_text) = &available.preview_status_text {
                            ui.small(preview_status_text);
                        }
                        if let Some(base_id) = available.affected_focus_base_id {
                            if ui
                                .small_button(presentation::research_focus_affected_label())
                                .clicked()
                            {
                                self.focus_base(base_id);
                            }
                        }
                        if let Some(preview_action_label) = &available.preview_action_label {
                            if ui.small_button(preview_action_label).clicked() {
                                self.stage_unlock_preview_action_for_tech(owner, tech, 3, true);
                            }
                        }
                    }
                });
                ui.small(&available.description_text);
                for line in &available.unlock_lines {
                    ui.small(line);
                }
                if unlock_impact.recommendation_count > 0 {
                    if let Some(summary) = &unlock_impact.summary_text {
                        ui.small(summary);
                    }
                }
                ui.separator();
            }
            if research_panel.available.is_empty() {
                ui.small(&research_panel.available_empty_text);
            }
        });

        ui.collapsing(&research_panel.blocked_heading_text, |ui| {
            for blocked in &research_panel.blocked {
                ui.label(&blocked.label_text);
                ui.small(&blocked.description_text);
                for line in &blocked.unlock_lines {
                    ui.small(line);
                }
                ui.separator();
            }
            if research_panel.blocked.is_empty() {
                ui.small(&research_panel.blocked_empty_text);
            }
        });

        ui.collapsing(&research_panel.known_heading_text, |ui| {
            for known in &research_panel.known {
                ui.label(&known.label_text);
            }
        });
    }

    fn draw_tech_tree_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Technology Tree");
        let tree = smac_core::technology_tree::TechnologyTree::new();
        let known: std::collections::HashSet<String> = self
            .game
            .faction(self.player_owner())
            .unwrap()
            .known_techs
            .iter()
            .map(|t| t.content_id().to_string())
            .collect();

        egui::ScrollArea::both().show(ui, |ui| {
            let mut categories: std::collections::BTreeMap<_, Vec<_>> =
                std::collections::BTreeMap::new();
            for tech in tree.all_technologies() {
                categories
                    .entry(tech.category.clone())
                    .or_default()
                    .push(tech);
            }

            for (category, techs) in categories {
                ui.collapsing(format!("{:?}", category), |ui| {
                    for tech in techs {
                        let is_known = known.contains(&tech.id);
                        let color = if is_known {
                            egui::Color32::GREEN
                        } else if tech.prerequisites.iter().all(|p| known.contains(p)) {
                            egui::Color32::WHITE
                        } else {
                            egui::Color32::GRAY
                        };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, &tech.name);
                            if !tech.prerequisites.is_empty() {
                                ui.small(format!("Req: {:?}", tech.prerequisites));
                            }
                        });
                        ui.small(&tech.description);
                    }
                });
            }
        });
    }

    fn draw_faction_status(&mut self, ui: &mut egui::Ui) {
        ui.heading(presentation::ui_factions_heading());

        ui.horizontal_wrapped(|ui| {
            let base_focus_state = self.game.base_focus_state(
                self.player_owner(),
                self.base_focus_filter,
                self.selected_base_id(),
            );
            ui.label("Base focus");
            egui::ComboBox::from_label("base filter")
                .selected_text(base_focus_filter_label(self.base_focus_filter))
                .show_ui(ui, |ui| {
                    for filter in [
                        BaseFocusFilter::All,
                        BaseFocusFilter::Frontier,
                        BaseFocusFilter::Recovery,
                        BaseFocusFilter::Unrest,
                        BaseFocusFilter::QueueGap,
                        BaseFocusFilter::ResearchUnlock,
                        BaseFocusFilter::Logistics,
                        BaseFocusFilter::Saturated,
                        BaseFocusFilter::Tight,
                        BaseFocusFilter::Collapsing,
                        BaseFocusFilter::Balanced,
                        BaseFocusFilter::Defense,
                        BaseFocusFilter::Economy,
                        BaseFocusFilter::LogisticsMode,
                    ] {
                        ui.selectable_value(
                            &mut self.base_focus_filter,
                            filter,
                            base_focus_filter_label(filter),
                        );
                    }
                });
            ui.label("Sort");
            egui::ComboBox::from_label("base sort")
                .selected_text(base_sort_mode_label(self.base_sort_mode))
                .show_ui(ui, |ui| {
                    for sort_mode in [
                        BaseSortMode::Frontier,
                        BaseSortMode::Logistics,
                        BaseSortMode::Recovery,
                        BaseSortMode::Unrest,
                        BaseSortMode::Governor,
                        BaseSortMode::Name,
                    ] {
                        ui.selectable_value(
                            &mut self.base_sort_mode,
                            sort_mode,
                            base_sort_mode_label(sort_mode),
                        );
                    }
                });
            ui.small(base_focus_state.count_label);
            if ui.button("Jump Next Match").clicked() {
                self.cycle_filtered_player_bases(self.base_focus_filter);
            }
            if let Some(label) = &base_focus_state.action_label_text {
                if ui.button(label).clicked() {
                    self.apply_base_focus_action(self.base_focus_filter);
                }
            }
        });
        ui.separator();

        let operations_dashboard_state = self.game.player_operations_dashboard_state(
            self.selected_unit,
            self.selected_base_id(),
            3,
        );
        let operations_focus_state = &operations_dashboard_state.focus;
        if !operations_dashboard_state.advice_lines.is_empty() {
            ui.group(|ui| {
                ui.colored_label(
                    color32_from_hex(presentation::ui_warning_hex(), egui::Color32::YELLOW),
                    &operations_dashboard_state.heading_text,
                );
                for line in &operations_dashboard_state.advice_lines {
                    ui.small(line);
                }
                ui.horizontal_wrapped(|ui| {
                    for action in &operations_dashboard_state.bulk_actions {
                        if ui
                            .add_enabled(action.enabled, egui::Button::new(&action.button_text))
                            .clicked()
                        {
                            match action.action_type {
                                PlayerOperationsActionType::SelectDamagedUnit => {
                                    if let Some(unit_id) =
                                        operations_focus_state.most_damaged_unit_id
                                    {
                                        self.focus_unit(unit_id);
                                    }
                                }
                                PlayerOperationsActionType::CycleDamagedUnits => {
                                    self.cycle_damaged_units();
                                }
                                PlayerOperationsActionType::JumpStressedBase => {
                                    if let Some(base_id) =
                                        operations_focus_state.most_unrested_base_id
                                    {
                                        self.focus_base(base_id);
                                    }
                                }
                                PlayerOperationsActionType::CycleStressedBases => {
                                    self.cycle_stressed_bases();
                                }
                                PlayerOperationsActionType::JumpRecoveryBase => {
                                    if let Some(base_id) =
                                        operations_focus_state.most_recovering_garrison_base_id
                                    {
                                        self.focus_base(base_id);
                                    }
                                }
                                PlayerOperationsActionType::CycleRecoveryBases => {
                                    self.cycle_recovering_bases();
                                }
                                PlayerOperationsActionType::ApplyRecoveryBasePlan => {
                                    if let Some(base_id) =
                                        operations_focus_state.most_recovering_garrison_base_id
                                    {
                                        self.apply_recovery_plan(base_id, 3);
                                        self.focus_base(base_id);
                                    }
                                }
                                PlayerOperationsActionType::FallbackAllDamaged => {
                                    self.apply_fallback_to_damaged_units();
                                }
                                PlayerOperationsActionType::ApplyAllRecoveryPlans => {
                                    self.apply_recovery_plans_all(3);
                                }
                                PlayerOperationsActionType::ApplyFrontierDefensePlans => {
                                    self.apply_defense_plans_all(3);
                                }
                                PlayerOperationsActionType::SuggestGovernors => {
                                    self.apply_recommended_governors();
                                }
                                PlayerOperationsActionType::RepairConvoys => {
                                    self.apply_convoy_repairs();
                                }
                                PlayerOperationsActionType::RebuildConvoys => {
                                    self.apply_convoy_rebuilds();
                                }
                                PlayerOperationsActionType::AssignEscortPatrols => {
                                    self.apply_escort_patrols();
                                }
                                PlayerOperationsActionType::ApplyEconomyPlans => {
                                    self.apply_economy_plans_all(3);
                                }
                                PlayerOperationsActionType::FillEmptyQueues => {
                                    self.apply_empty_queue_governor_plans(3);
                                }
                                PlayerOperationsActionType::JumpResearchUnlock => {
                                    if let Some(base_id) =
                                        operations_focus_state.current_research_unlock_focus_base_id
                                    {
                                        self.focus_base(base_id);
                                    }
                                }
                                PlayerOperationsActionType::SelectRecoveringGarrison => {
                                    self.select_next_recovering_garrison();
                                }
                            }
                        }
                    }
                    for action in &operations_dashboard_state.jump_actions {
                        if ui
                            .add_enabled(action.enabled, egui::Button::new(&action.button_text))
                            .clicked()
                        {
                            self.cycle_filtered_player_bases(action.filter);
                        }
                    }
                });
            });
            ui.separator();
        }

        let faction_overviews = self.game.faction_overview_display_states(
            self.logistics_route_filter,
            self.logistics_route_sort,
        );

        for overview in faction_overviews {
            ui.group(|ui| {
                ui.colored_label(
                    color32_from_hex(&overview.color_hex, egui::Color32::WHITE),
                    &overview.name,
                );
                if let Some(leader) = &overview.leader_text {
                    ui.small(leader);
                }
                if let Some(description) = &overview.description_text {
                    ui.small(description);
                }
                ui.label(&overview.base_count_text);
                ui.label(&overview.unit_count_text);
                ui.label(&overview.energy_text);
                ui.label(&overview.upkeep_text);
                ui.label(&overview.research_progress_text);
                ui.label(&overview.current_tech_text);
                ui.label(&overview.techs_discovered_text);
                ui.group(|ui| {
                    ui.small(overview.indices_heading_text);
                    ui.label(&overview.food_security_text);
                    ui.label(&overview.ai_dependence_text);
                    ui.label(&overview.orbital_index_text);
                    ui.label(&overview.planet_toxicity_text);
                });
                ui.label(&overview.alerts_text);
                ui.label(&overview.base_roles_text);
                ui.label(&overview.logistics_summary_text);
                ui.group(|ui| {
                    ui.small(overview.governor_summary_heading_text);
                    ui.small(&overview.governor_mode_mix_summary);
                });
                ui.small(&overview.production_posture_text);
                ui.small(&overview.production_roles_text);
                ui.small(&overview.queue_posture_text);
                ui.small(&overview.queue_roles_text);
                ui.small(&overview.governor_intent_text);
                ui.small(&overview.governor_queue_intent_text);
                ui.small(&overview.queue_gaps_text);
                ui.small(&overview.tech_blocked_intent_text);

                if !overview.secret_projects_text.is_empty() {
                    ui.group(|ui| {
                        ui.small(overview.secret_projects_heading_text);
                        for project in &overview.secret_projects_text {
                            let hex = presentation::ui_secret_project_hex();
                            let color =
                                color32_from_hex(hex, egui::Color32::from_rgb(200, 200, 255));
                            ui.colored_label(color, project);
                        }
                    });
                }

                if overview.is_player_owned {
                    ui.horizontal_wrapped(|ui| {
                        if let (Some(label), Some(base_id)) = (
                            overview.jump_queue_gap_label_text,
                            overview.queue_gap_base_ids.first(),
                        ) {
                            if ui.small_button(label).clicked() {
                                self.focus_base(*base_id);
                            }
                        }
                        if let (Some(label), Some(base_id)) = (
                            overview.jump_tech_block_label_text,
                            overview.tech_blocked_base_ids.first(),
                        ) {
                            if ui.small_button(label).clicked() {
                                self.focus_base(*base_id);
                            }
                        }
                    });
                }

                if !overview.governor_warnings.is_empty() {
                    ui.group(|ui| {
                        ui.small(overview.governor_warnings_heading_text);
                        for warning in &overview.governor_warnings {
                            ui.small(warning);
                        }
                    });
                }

                let logistics_panel = &overview.logistics_panel;

                if !logistics_panel.alert_lines.is_empty() {
                    ui.group(|ui| {
                        ui.small(&logistics_panel.alerts_heading_text);
                        for warning in &logistics_panel.alert_lines {
                            ui.small(warning);
                        }
                        if overview.is_player_owned {
                            ui.horizontal_wrapped(|ui| {
                                if logistics_panel.jump_saturated_action.enabled
                                    && ui
                                        .small_button(
                                            &logistics_panel.jump_saturated_action.button_text,
                                        )
                                        .clicked()
                                {
                                    self.cycle_filtered_player_bases(BaseFocusFilter::Saturated);
                                }
                                if logistics_panel.jump_costly_logistics_action.enabled
                                    && ui
                                        .small_button(
                                            &logistics_panel
                                                .jump_costly_logistics_action
                                                .button_text,
                                        )
                                        .clicked()
                                {
                                    self.cycle_filtered_player_bases(BaseFocusFilter::Logistics);
                                }
                                if logistics_panel.jump_collapsing_action.enabled
                                    && ui
                                        .small_button(
                                            &logistics_panel.jump_collapsing_action.button_text,
                                        )
                                        .clicked()
                                {
                                    self.cycle_filtered_player_bases(BaseFocusFilter::Collapsing);
                                }
                            });
                        }
                    });
                }

                let diplomacy_panel = &overview.diplomacy_panel;
                if !diplomacy_panel.relations.is_empty() {
                    if ui.button("Diplomacy Options...").clicked() {
                        self.show_diplomacy_dialogue = Some(overview.faction_id);
                    }
                }

                let se_panel = &overview.social_engineering_panel;
                ui.group(|ui| {
                    ui.small(&se_panel.heading_text);
                    for category in &se_panel.categories {
                        ui.horizontal(|ui| {
                            ui.small(format!("{}:", category.name));
                            for option in &category.options {
                                let button = egui::Button::new(&option.choice_text).fill(
                                    if option.selected {
                                        color32_from_hex(
                                            &overview.color_hex,
                                            egui::Color32::DARK_GRAY,
                                        )
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    },
                                );

                                if ui
                                    .add_enabled(option.enabled, button)
                                    .on_hover_text(&option.modifiers_text)
                                    .clicked()
                                {
                                    if overview.is_player_owned {
                                        self.apply_action(GameAction::ChooseSocialEngineering {
                                            owner: overview.faction_id,
                                            politics: option.politics,
                                            economics: option.economics,
                                            values: option.values,
                                            future: option.future,
                                        })
                                        .ok();
                                    }
                                }
                            }
                        });
                    }
                });

                if !logistics_panel.route_rows.is_empty() {
                    ui.group(|ui| {
                        ui.small(&logistics_panel.routes_heading_text);
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Filter");
                            egui::ComboBox::from_id_source(format!(
                                "logistics_route_filter_{}",
                                overview.faction_id
                            ))
                            .selected_text(logistics_route_filter_label(
                                self.logistics_route_filter,
                            ))
                            .show_ui(ui, |ui| {
                                for filter in LogisticsRouteFilter::all() {
                                    ui.selectable_value(
                                        &mut self.logistics_route_filter,
                                        filter,
                                        logistics_route_filter_label(filter),
                                    );
                                }
                            });
                            ui.label("Sort");
                            egui::ComboBox::from_id_source(format!(
                                "logistics_route_sort_{}",
                                overview.faction_id
                            ))
                            .selected_text(logistics_route_sort_label(self.logistics_route_sort))
                            .show_ui(ui, |ui| {
                                for sort in LogisticsRouteSort::all() {
                                    ui.selectable_value(
                                        &mut self.logistics_route_sort,
                                        sort,
                                        logistics_route_sort_label(sort),
                                    );
                                }
                            });
                        });
                        for route in logistics_panel.route_rows.iter().take(8) {
                            self.draw_faction_route_row(ui, overview.faction_id, route);
                        }
                        ui.horizontal_wrapped(|ui| {
                            if overview.is_player_owned
                                && logistics_panel.jump_worst_route_action.enabled
                                && ui
                                    .small_button(
                                        &logistics_panel.jump_worst_route_action.button_text,
                                    )
                                    .clicked()
                            {
                                if let Some(base_id) = self
                                    .game
                                    .worst_convoy_route_focus_action(overview.faction_id)
                                {
                                    self.focus_base(base_id);
                                }
                            }
                            if overview.is_player_owned
                                && logistics_panel.repair_filtered_action.enabled
                                && ui
                                    .small_button(
                                        &logistics_panel.repair_filtered_action.button_text,
                                    )
                                    .clicked()
                            {
                                self.game.apply_filtered_convoy_repairs_for_owner(
                                    overview.faction_id,
                                    self.logistics_route_filter,
                                    self.logistics_route_sort,
                                );
                            }
                            if overview.is_player_owned
                                && logistics_panel.remove_collapsing_action.enabled
                                && ui
                                    .small_button(
                                        &logistics_panel.remove_collapsing_action.button_text,
                                    )
                                    .clicked()
                            {
                                self.game
                                    .remove_collapsing_convoy_routes(overview.faction_id);
                            }
                            ui.small(&logistics_panel.filtered_count_text);
                        });
                        if overview.is_player_owned {
                            if !logistics_panel.route_opportunities.is_empty() {
                                ui.group(|ui| {
                                    ui.small(&logistics_panel.route_opportunities_heading_text);
                                    for opportunity in
                                        logistics_panel.route_opportunities.iter().take(4)
                                    {
                                        if ui.small_button(&opportunity.button_text).clicked() {
                                            self.game.add_convoy_route_action(
                                                opportunity.base_a_id,
                                                opportunity.base_b_id,
                                                opportunity.kind,
                                            );
                                        }
                                    }
                                });
                            }
                        }
                    });
                }
                if !logistics_panel.hub_rows.is_empty() {
                    ui.group(|ui| {
                        ui.small(&logistics_panel.hubs_heading_text);
                        for hub in logistics_panel.hub_rows.iter().take(4) {
                            ui.horizontal_wrapped(|ui| {
                                ui.small(&hub.row_text);
                                if hub.is_saturated {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(220, 110, 60),
                                        hub.saturation_label_text.as_deref().unwrap_or("SATURATED"),
                                    );
                                } else if hub.is_tight {
                                    ui.colored_label(
                                        egui::Color32::YELLOW,
                                        hub.saturation_label_text.as_deref().unwrap_or("TIGHT"),
                                    );
                                }
                            });
                        }
                    });
                }
            });
        }
    }

    fn screen_to_tile(&self, pos: egui::Pos2) -> (usize, usize) {
        // Simple placeholder, logic will depend on grid layout
        let x = (pos.x / 30.0) as usize;
        let y = (pos.y / 30.0) as usize;
        (
            x.clamp(0, self.game.width - 1),
            y.clamp(0, self.game.height - 1),
        )
    }

    fn draw_map(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let map_panel = self.game.map_panel_display_state(self.map_overlay);
            ui.heading(map_panel.heading_text);

            let response = ui.allocate_response(ui.available_size(), egui::Sense::hover());
            if response.hovered() {
                let pos = response.hover_pos().unwrap();
                let (tile_x, tile_y) = self.screen_to_tile(pos);
                if let Some(tile) = self.game.tile(tile_x, tile_y) {
                    let mut tooltip = format!("Tile: {}, {}", tile_x, tile_y);
                    tooltip.push_str(&format!("\nTerrain: {:?}", tile.terrain));
                    if let Some(yields) = self.game.tile(tile_x, tile_y).map(|t| t.terrain.yields())
                    {
                        tooltip.push_str(&format!(
                            "\nBase Yields: {}",
                            presentation::format_yield_breakdown(yields)
                        ));
                    }
                    if let Some(improvement) = tile.improvement {
                        tooltip.push_str(&format!(
                            "\nImprovement: {}",
                            presentation::improvement_name(improvement)
                        ));
                        tooltip.push_str(&format!(
                            "\nYields: {}",
                            presentation::format_yield_breakdown(improvement.yields())
                        ));
                    }

                    let potentials = self.game.tile_potential_improvements(tile_x, tile_y);
                    if !potentials.is_empty() {
                        tooltip.push_str("\n\nPotential Improvements:");
                        for (imp, yields) in potentials {
                            tooltip.push_str(&format!(
                                "\n - {}: {}",
                                presentation::improvement_name(imp),
                                presentation::format_yield_breakdown(yields)
                            ));
                        }
                    }
                    response.on_hover_text(tooltip);
                }
            }

            let mut clicked_tile: Option<(usize, usize)> = None;
            let mut tile_centers = vec![vec![egui::Pos2::ZERO; self.game.width]; self.game.height];

            egui::ScrollArea::both().show(ui, |ui| {
                egui::Grid::new("planet_grid")
                    .spacing([1.0, 1.0])
                    .show(ui, |ui| {
                        for y in 0..self.game.height {
                            for x in 0..self.game.width {
                                let tile_state = self.game.map_tile_display_state(
                                    x,
                                    y,
                                    self.player_owner(),
                                    self.selected_tile,
                                    self.map_overlay,
                                );

                                let mut button = egui::Button::new(&tile_state.label_text)
                                    .fill(color32_from_hex(
                                        &tile_state.color_hex,
                                        egui::Color32::from_rgb(56, 96, 72),
                                    ))
                                    .min_size(egui::vec2(self.zoom, self.zoom));

                                if let Some(hex) = tile_state.selection_stroke_color_hex {
                                    button = button.stroke(egui::Stroke::new(
                                        2.0,
                                        color32_from_hex(hex, egui::Color32::WHITE),
                                    ));
                                }

                                let response = ui.add(button);
                                tile_centers[y][x] = response.rect.center();

                                if let Some(status) = tile_state.status_glyph {
                                    let badge_pos =
                                        response.rect.right_top() + egui::vec2(-8.0, 8.0);
                                    let color = tile_state
                                        .status_glyph_color_hex
                                        .map(|hex| color32_from_hex(hex, egui::Color32::WHITE))
                                        .unwrap_or(egui::Color32::WHITE);
                                    ui.painter().text(
                                        badge_pos,
                                        egui::Align2::CENTER_CENTER,
                                        status,
                                        egui::FontId::monospace(14.0),
                                        color,
                                    );
                                }

                                if response.clicked() {
                                    clicked_tile = Some((x, y));
                                }
                            }
                            ui.end_row();
                        }
                    });
                if map_panel.uses_convoy_lines {
                    self.draw_convoy_lines(ui.painter(), &tile_centers, 1.5);
                }

                if let Some(unit_id) = self.selected_unit {
                    if let Some(hover_pos) = ctx.pointer_hover_pos() {
                        let (tx, ty) = self.screen_to_tile(hover_pos);
                        let path = self.game.unit_path_to(unit_id, tx, ty);
                        if !path.is_empty() {
                            let mut points = Vec::new();
                            let start_unit = self.game.unit(unit_id).unwrap();
                            points.push(tile_centers[start_unit.y][start_unit.x]);

                            for &(px, py) in &path {
                                points.push(tile_centers[py][px]);
                            }

                            ui.painter().add(egui::Shape::line(
                                points,
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
                            ));

                            // Draw turn cost label at the end
                            let cost = path.len(); // Simple estimate for now
                            ui.painter().text(
                                tile_centers[ty][tx],
                                egui::Align2::LEFT_TOP,
                                format!("{} turns", (cost as f32 / 3.0).ceil()),
                                egui::FontId::proportional(12.0),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            });

            if let Some((x, y)) = clicked_tile {
                self.handle_tile_click(x, y);
            }
        });
    }

    fn handle_tile_click(&mut self, x: usize, y: usize) {
        if self.active_tab == SidebarTab::Editor {
            match &self.editor_tool {
                EditorTool::PlaceUnit { kind, owner } => {
                    self.game.spawn_unit(*owner, kind.clone(), x, y);
                    return;
                }
                EditorTool::PlaceBase { owner } => {
                    self.game.spawn_base(*owner, x, y);
                    return;
                }
                EditorTool::ChangeTerrain { terrain } => {
                    if let Some(tile) = self.game.tile_mut(x, y) {
                        tile.terrain = *terrain;
                    }
                    return;
                }
                EditorTool::ClearTile => {
                    if let Some(tile) = self.game.tile_mut(x, y) {
                        tile.unit = None;
                        tile.base = None;
                        tile.improvement = None;
                    }
                    return;
                }
                EditorTool::None => {}
            }
        }

        self.selected_tile = Some((x, y));
        // Context-aware UI: jump to Selection tab when interacting with the map.
        self.active_tab = SidebarTab::Selection;

        let interaction =
            self.game
                .process_map_interaction(x, y, self.selected_unit, self.player_owner());

        match interaction {
            smac_core::MapInteractionResult::SelectUnit(unit_id) => {
                self.selected_unit = Some(unit_id);
            }
            smac_core::MapInteractionResult::MoveUnit {
                unit_id,
                target_x,
                target_y,
            } => {
                match self.apply_action(GameAction::MoveUnit {
                    unit_id,
                    target_x,
                    target_y,
                }) {
                    Ok(_) => {
                        if let Some(unit) = self.game.unit(unit_id) {
                            self.selected_tile = Some((unit.x, unit.y));
                        } else {
                            self.selected_unit = None;
                            self.select_next_unit();
                        }
                    }
                    Err(err) => self.game.push_log(format!("Move failed: {err}")),
                }
            }
            smac_core::MapInteractionResult::Error(msg) => {
                self.game.push_log(msg);
            }
            smac_core::MapInteractionResult::None => {}
        }
    }

    fn try_build_improvement(&mut self, unit_id: usize, improvement: Improvement) {
        match self.apply_action(GameAction::BuildImprovement {
            unit_id,
            improvement,
        }) {
            Ok(_) => {}
            Err(err) => self.game.push_log(format!("Terraforming failed: {err}")),
        }
    }

    fn apply_action(&mut self, action: GameAction) -> Result<(), String> {
        self.game.apply_action(action)
    }

    fn focus_unit(&mut self, unit_id: usize) {
        if let Some(unit) = self.game.unit(unit_id) {
            self.selected_unit = Some(unit_id);
            self.selected_tile = Some((unit.x, unit.y));
        }
    }

    fn focus_base(&mut self, base_id: usize) {
        if let Some(base) = self.game.base(base_id) {
            self.selected_unit = None;
            self.selected_tile = Some((base.x, base.y));
        }
    }

    fn cycle_damaged_units(&mut self) {
        if let Some(next) = self.game.next_damaged_player_unit_id(self.selected_unit) {
            self.focus_unit(next);
        }
    }

    fn cycle_stressed_bases(&mut self) {
        if let Some(next) = self
            .game
            .next_stressed_player_base_id(self.selected_base_id())
        {
            self.focus_base(next);
        }
    }

    fn select_next_recovering_garrison(&mut self) {
        if let Some(next) = self
            .game
            .next_recovering_garrison_unit_id(self.selected_unit)
        {
            self.focus_unit(next);
        }
    }

    fn cycle_recovering_bases(&mut self) {
        if let Some(next) = self.game.next_recovering_base_id(self.selected_base_id()) {
            self.focus_base(next);
        }
    }

    fn apply_base_focus_action(&mut self, filter: BaseFocusFilter) {
        match filter {
            BaseFocusFilter::QueueGap => self.apply_empty_queue_governor_plans(3),
            BaseFocusFilter::ResearchUnlock => {
                let result = self.game.current_research_unlock_base_focus_action(
                    self.player_owner(),
                    3,
                    self.selected_base_id(),
                );
                let _ = self.apply_preview_state_action_result(result, true);
            }
            _ => {}
        }
    }

    fn draw_faction_route_row(
        &mut self,
        ui: &mut egui::Ui,
        owner: usize,
        route: &smac_core::ConvoyRouteDisplayRowState,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(&route.row_text);
            if self.is_player_owned(owner) && ui.small_button(&route.focus_a_label_text).clicked() {
                self.focus_base(route.base_a_id);
            }
            if self.is_player_owned(owner) && ui.small_button(&route.focus_b_label_text).clicked() {
                self.focus_base(route.base_b_id);
            }
            if self.is_player_owned(owner)
                && route.can_repair
                && ui.small_button(&route.repair_label_text).clicked()
            {
                self.game
                    .repair_convoy_route_action(route.base_a_id, route.base_b_id, route.kind);
            }
            if self.is_player_owned(owner) && ui.small_button(&route.remove_label_text).clicked() {
                self.game.remove_convoy_route_action(
                    route.base_a_id,
                    route.base_b_id,
                    Some(route.kind),
                );
            }
        });
    }

    fn cycle_filtered_player_bases(&mut self, filter: BaseFocusFilter) {
        if let Some(next) = self.game.next_base_focus_target_action(
            self.player_owner(),
            filter,
            self.selected_base_id(),
        ) {
            self.focus_base(next);
        }
    }

    fn apply_fallback_to_damaged_units(&mut self) {
        if self.game.apply_player_fallback_moves() > 0 {
            self.select_next_unit();
        }
    }

    fn apply_escort_patrols(&mut self) {
        self.game.apply_escort_patrol_moves(self.player_owner());
    }

    fn apply_convoy_repairs(&mut self) {
        self.game.apply_convoy_repairs_all(self.player_owner());
    }

    fn apply_convoy_rebuilds(&mut self) {
        self.game.apply_convoy_rebuilds_all(self.player_owner());
    }

    fn apply_recommended_governors(&mut self) {
        self.game
            .apply_recommended_governor_modes(self.player_owner());
    }

    fn apply_governor_plan_queue(&mut self, base_id: usize, max_steps: usize) {
        let _ = self
            .game
            .apply_governor_plan_queue_action(base_id, max_steps);
    }

    fn apply_empty_queue_governor_plans(&mut self, max_steps: usize) {
        self.game
            .apply_empty_queue_governor_plans(self.player_owner(), max_steps);
    }

    fn apply_recovery_plan(&mut self, base_id: usize, max_steps: usize) {
        let _ = self.game.apply_recovery_plan_action(base_id, max_steps);
    }

    fn apply_recovery_plans_all(&mut self, max_steps: usize) {
        self.game.apply_recovery_plans_all(max_steps);
    }

    fn apply_defense_plan(&mut self, base_id: usize, max_steps: usize) {
        let _ = self.game.apply_defense_plan_action(base_id, max_steps);
    }

    fn apply_defense_plans_all(&mut self, max_steps: usize) {
        self.game.apply_defense_plans_all(max_steps);
    }

    fn apply_economy_plans_all(&mut self, max_steps: usize) {
        self.game
            .apply_economy_plans_all(self.player_owner(), max_steps);
    }

    fn apply_enabled_automations(&mut self) {
        let _ = self.game.apply_enabled_automations(3);
    }

    fn restore_automation_settings_from_save(&mut self, save_path: &Path) {
        let Ok(metadata) = SaveSlotMetadata::load_from_path(save_path) else {
            return;
        };

        for base in &mut self.game.bases {
            if base.governor_mode != GovernorMode::Off {
                continue;
            }
            if metadata.auto_defense_base_ids.contains(&base.id) {
                base.governor_mode = GovernorMode::Defense;
            } else if metadata.auto_recovery_base_ids.contains(&base.id) {
                base.governor_mode = GovernorMode::Recovery;
            } else if metadata.auto_economy_base_ids.contains(&base.id) {
                base.governor_mode = GovernorMode::Economy;
            }
        }
    }

    fn reset_selection_after_game_change(&mut self) {
        self.selected_unit = None;
        self.selected_tile = Some((3, 3));
        self.select_next_unit();
    }

    fn normalized_target_save_id(&self, category: SaveSlotCategory) -> String {
        match category {
            SaveSlotCategory::Autosave => "autosave_latest".to_string(),
            SaveSlotCategory::Manual => {
                let normalized = normalize_save_id(&self.save_id_input);
                if normalized.is_empty() {
                    "slot_1".to_string()
                } else {
                    normalized
                }
            }
            SaveSlotCategory::Imported => {
                let normalized = normalize_save_id(&self.save_id_input);
                if normalized.is_empty() {
                    "imported_save".to_string()
                } else {
                    normalized
                }
            }
            SaveSlotCategory::Empty => "slot_1".to_string(),
        }
    }

    fn current_save_metadata(&self, category: SaveSlotCategory) -> SaveSlotMetadata {
        let save_name = if self.save_name_input.trim().is_empty() {
            self.default_save_slot_name(&self.selected_save_id)
        } else {
            self.save_name_input.trim().to_string()
        };

        self.game.build_save_metadata(
            save_name,
            self.save_notes_input.trim().to_string(),
            category,
        )
    }

    fn save_game_as(&mut self, category: SaveSlotCategory) {
        self.pending_save_conflict = None;
        self.save_category_input = category;
        self.selected_save_id = self.normalized_target_save_id(category);
        self.save_id_input = self.selected_save_id.clone();
        let save_path = self.current_save_path();

        if let Some(parent) = save_path.parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                self.game
                    .push_log(format!("Save failed creating directory: {err}"));
                return;
            }
        }

        let mut snapshot = GameStateSnapshot::from(&self.game);
        let metadata = self.current_save_metadata(category);
        snapshot.save_name = Some(metadata.save_name.clone());
        match snapshot.save_to_path(&save_path) {
            Ok(_) => {
                if let Err(err) =
                    metadata.save_to_path(Self::metadata_path_for_save_path(&save_path))
                {
                    self.game
                        .push_log(format!("Save metadata write failed: {err}"));
                }
                self.save_name_input = snapshot
                    .save_name
                    .clone()
                    .unwrap_or_else(|| self.default_save_slot_name(&self.selected_save_id));
                self.save_notes_input = metadata.notes;
                self.game
                    .push_log(format!("Saved game to {}.", save_path.display()))
            }
            Err(err) => self.game.push_log(format!("Save failed: {err}")),
        }
    }

    fn load_game(&mut self) {
        self.pending_save_conflict = None;
        self.selected_save_id = normalize_save_id(&self.save_id_input);
        if self.selected_save_id.is_empty() {
            self.selected_save_id = "slot_1".to_string();
        }
        self.save_id_input = self.selected_save_id.clone();
        let save_path = self.current_save_path();
        match GameStateSnapshot::load_from_path(&save_path) {
            Ok(snapshot) => {
                let recovery_notes = snapshot.recovery_notes.clone();
                self.last_recovery_notes = recovery_notes.clone();
                self.game = snapshot.into_game_state();
                self.restore_automation_settings_from_save(&save_path);
                self.observer.reset_progress();
                self.sync_save_inputs_for_selection();
                self.reset_selection_after_game_change();
                self.game
                    .push_log(format!("Loaded game from {}.", save_path.display()));
                for note in recovery_notes {
                    self.game.push_log(format!("Save recovery: {note}"));
                }
            }
            Err(err) => {
                self.last_recovery_notes.clear();
                self.game.push_log(format!("Load failed: {err}"));
            }
        }
    }

    fn draw_social_engineering_panel(&mut self, ui: &mut egui::Ui) {
        let owner = self.player_owner();
        let se_state = self.game.social_engineering_display_state(owner);
        ui.heading(&se_state.heading_text);

        for category in &se_state.categories {
            ui.add_space(4.0);
            ui.small(category.name);
            ui.horizontal_wrapped(|ui| {
                for option in &category.options {
                    let mut button = egui::Button::new(&option.choice_text);
                    if option.selected {
                        button = button.fill(color32_from_hex(
                            presentation::faction_color_hex(
                                &self.game.faction(owner).unwrap().name,
                            )
                            .unwrap_or("#d0d0d0"),
                            egui::Color32::DARK_GRAY,
                        ));
                    }
                    if ui
                        .add_enabled(option.enabled, button)
                        .on_hover_text(&option.modifiers_text)
                        .clicked()
                    {
                        self.apply_action(GameAction::ChooseSocialEngineering {
                            owner,
                            politics: option.politics,
                            economics: option.economics,
                            values: option.values,
                            future: option.future,
                        })
                        .ok();
                    }
                }
            });
        }
    }

    fn draw_secret_projects_panel(&mut self, ui: &mut egui::Ui) {
        let registry = self.game.secret_project_registry_display_state();
        ui.heading(&registry.heading_text);
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for project in &registry.projects {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&project.project_name)
                                .strong()
                                .size(14.0),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(&project.status_text);
                            ui.label("Status:");
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Owner:");
                        ui.colored_label(
                            color32_from_hex(&project.owner_color_hex, egui::Color32::WHITE),
                            &project.owner_name,
                        );
                    });

                    ui.label(
                        egui::RichText::new(&project.effects_text)
                            .italics()
                            .color(egui::Color32::LIGHT_BLUE),
                    );
                });
                ui.add_space(4.0);
            }
        });
    }

    fn draw_scenario_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scenario Editor");
        ui.separator();

        ui.label(format!("Active Tool: {:?}", self.editor_tool));
        if ui.button("None").clicked() {
            self.editor_tool = EditorTool::None;
        }
        if ui.button("Clear Tile").clicked() {
            self.editor_tool = EditorTool::ClearTile;
        }

        ui.separator();
        ui.label("Place Unit:");
        ui.horizontal_wrapped(|ui| {
            for kind in smac_core::UnitKind::all() {
                if ui.small_button(format!("Place {:?}", kind)).clicked() {
                    self.editor_tool = EditorTool::PlaceUnit {
                        kind,
                        owner: self.player_owner(),
                    };
                }
            }
        });

        ui.separator();
        ui.label("Terrain:");
        ui.horizontal_wrapped(|ui| {
            for terrain in [
                smac_core::Terrain::Flat,
                smac_core::Terrain::Rolling,
                smac_core::Terrain::Rocky,
                smac_core::Terrain::Ocean,
                smac_core::Terrain::Fungus,
            ] {
                if ui.small_button(format!("{:?}", terrain)).clicked() {
                    self.editor_tool = EditorTool::ChangeTerrain { terrain };
                }
            }
        });

        if ui.button("Place Base").clicked() {
            self.editor_tool = EditorTool::PlaceBase {
                owner: self.player_owner(),
            };
        }
    }

    fn draw_save_browser(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("File id");
            ui.text_edit_singleline(&mut self.save_name_input);
        });
        ui.horizontal(|ui| {
            ui.label("Notes");
            ui.text_edit_singleline(&mut self.save_notes_input);
        });
        ui.horizontal(|ui| {
            ui.label("External path");
            ui.text_edit_singleline(&mut self.external_save_path_input);
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.save_filter_text);
            egui::ComboBox::from_label("type")
                .selected_text(save_filter_label(self.save_filter_category))
                .show_ui(ui, |ui| {
                    for category in SaveFilterCategory::all() {
                        ui.selectable_value(
                            &mut self.save_filter_category,
                            category,
                            save_filter_label(category),
                        );
                    }
                });
            ui.checkbox(&mut self.save_filter_populated_only, "populated");
            ui.checkbox(&mut self.save_filter_recovered_only, "recovered only");
        });
        let all_entries = self.available_save_slots();
        let save_browser = save_browser_display_state(
            &all_entries,
            &self.selected_save_id,
            &self.save_browser_query(),
        );
        ui.small(&save_browser.counts_text);

        if let Some(conflict) = self.pending_save_conflict.clone() {
            let display = save_conflict_display_state(&conflict);
            let mut confirm_overwrite = false;
            let mut cancel_conflict = false;
            ui.group(|ui| {
                ui.colored_label(egui::Color32::YELLOW, display.message_text);
                ui.horizontal(|ui| {
                    if ui.button(display.confirm_button_text).clicked() {
                        confirm_overwrite = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel_conflict = true;
                    }
                });
            });
            if confirm_overwrite {
                self.confirm_pending_save_conflict();
            } else if cancel_conflict {
                self.pending_save_conflict = None;
            }
        }

        ui.horizontal_wrapped(|ui| {
            for action in &save_browser.global_actions {
                if ui.button(&action.label_text).clicked() {
                    match action.action_type {
                        SaveBrowserGlobalActionType::Rename => self.rename_selected_save(),
                        SaveBrowserGlobalActionType::Duplicate => self.duplicate_selected_save(),
                        SaveBrowserGlobalActionType::SaveAs => {
                            if let Some(category) = action.category {
                                self.save_game_as(category);
                            }
                        }
                        SaveBrowserGlobalActionType::Delete => self.delete_selected_save(),
                        SaveBrowserGlobalActionType::Import => self.import_external_save(),
                        SaveBrowserGlobalActionType::Export => self.export_selected_save(),
                    }
                }
            }
        });

        ui.horizontal_wrapped(|ui| {
            for button in &save_browser.sort_buttons {
                if ui.button(&button.label_text).clicked() {
                    self.set_save_sort(button.column);
                }
            }
        });

        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                egui::Grid::new("save_browser_grid")
                    .striped(true)
                    .min_col_width(64.0)
                    .show(ui, |ui| {
                        for header in &save_browser.column_headers {
                            ui.strong(*header);
                        }
                        ui.end_row();

                        for row in save_browser.rows {
                            if ui
                                .selectable_label(row.selected, if row.selected { ">" } else { "" })
                                .clicked()
                            {
                                self.selected_save_id = row.id.clone();
                                self.sync_save_inputs_for_selection();
                            }
                            if ui
                                .selectable_label(row.selected, &row.id)
                                .on_hover_text(row.file_path_text)
                                .clicked()
                            {
                                self.selected_save_id = row.id.clone();
                                self.sync_save_inputs_for_selection();
                            }
                            ui.label(row.category_text);
                            ui.label(row.display_name);
                            ui.label(row.turn_text);
                            ui.label(row.recovery_text);
                            ui.label(row.updated_text);
                            ui.label(row.notes_preview);
                            ui.end_row();
                        }
                    });
            });
    }

    fn selected_listing(&self) -> Option<SaveSlotListing> {
        self.available_save_slots()
            .into_iter()
            .find(|entry| entry.id == self.selected_save_id)
    }

    fn rename_selected_save(&mut self) {
        let Some(entry) = self.selected_listing() else {
            self.game
                .push_log("Rename failed: no save selected.".to_string());
            return;
        };
        let new_id = normalize_save_id(&self.save_id_input);
        if new_id.is_empty() {
            self.game
                .push_log("Rename failed: save file id is empty.".to_string());
            return;
        }
        if new_id != entry.id && self.save_id_conflicts(&new_id) {
            self.pending_save_conflict = Some(PendingSaveConflict::Rename { new_id });
            return;
        }
        match entry.rename_to_with_options(&new_id, false) {
            Ok(updated) => {
                self.pending_save_conflict = None;
                self.selected_save_id = updated.id.clone();
                self.save_id_input = updated.id;
                self.sync_save_inputs_for_selection();
                self.game.push_log("Save renamed.".to_string());
            }
            Err(err) => self.game.push_log(format!("Rename failed: {err}")),
        }
    }

    fn duplicate_selected_save(&mut self) {
        let Some(entry) = self.selected_listing() else {
            self.game
                .push_log("Duplicate failed: no save selected.".to_string());
            return;
        };
        let new_id = normalize_save_id(&self.save_id_input);
        if new_id.is_empty() {
            self.game
                .push_log("Duplicate failed: save file id is empty.".to_string());
            return;
        }
        if self.save_id_conflicts(&new_id) {
            self.pending_save_conflict = Some(PendingSaveConflict::Duplicate { new_id });
            return;
        }
        match entry.duplicate_to_with_options(&new_id, false) {
            Ok(updated) => {
                self.pending_save_conflict = None;
                self.selected_save_id = updated.id.clone();
                self.save_id_input = updated.id;
                self.sync_save_inputs_for_selection();
                self.game.push_log("Save duplicated.".to_string());
            }
            Err(err) => self.game.push_log(format!("Duplicate failed: {err}")),
        }
    }

    fn delete_selected_save(&mut self) {
        let Some(entry) = self.selected_listing() else {
            self.game
                .push_log("Delete failed: no save selected.".to_string());
            return;
        };
        self.pending_save_conflict = Some(PendingSaveConflict::Delete {
            id: entry.id.clone(),
        });
    }

    fn commit_delete_selected_save(&mut self, id: &str) {
        let Some(entry) = self
            .available_save_slots()
            .into_iter()
            .find(|entry| entry.id == id)
        else {
            self.game
                .push_log("Delete failed: selected save no longer exists.".to_string());
            return;
        };
        match entry.delete() {
            Ok(_) => {
                self.pending_save_conflict = None;
                self.game.push_log("Save deleted.".to_string());
                let next = self.available_save_slots().into_iter().next();
                if let Some(next_entry) = next {
                    self.selected_save_id = next_entry.id.clone();
                    self.sync_save_inputs_for_selection();
                } else {
                    self.selected_save_id = "slot_1".to_string();
                    self.save_id_input = self.selected_save_id.clone();
                    self.save_name_input = self.default_save_slot_name(&self.selected_save_id);
                }
            }
            Err(err) => self.game.push_log(format!("Delete failed: {err}")),
        }
    }

    fn import_external_save(&mut self) {
        let source_path = PathBuf::from(self.external_save_path_input.trim());
        if source_path.as_os_str().is_empty() {
            self.game
                .push_log("Import failed: external path is empty.".to_string());
            return;
        }
        let target_id = normalize_save_id(&self.save_id_input);
        if target_id.is_empty() {
            self.game
                .push_log("Import failed: save file id is empty.".to_string());
            return;
        }
        if self.save_id_conflicts(&target_id) {
            self.pending_save_conflict = Some(PendingSaveConflict::Import {
                source_path,
                target_id,
            });
            return;
        }
        self.run_import(source_path, target_id, false);
    }

    fn export_selected_save(&mut self) {
        let Some(entry) = self.selected_listing() else {
            self.game
                .push_log("Export failed: no save selected.".to_string());
            return;
        };
        let target_path = PathBuf::from(self.external_save_path_input.trim());
        if target_path.as_os_str().is_empty() {
            self.game
                .push_log("Export failed: external path is empty.".to_string());
            return;
        }
        if target_path.exists() {
            self.pending_save_conflict = Some(PendingSaveConflict::Export { target_path });
            return;
        }
        self.run_export(entry, target_path, false);
    }

    fn save_id_conflicts(&self, target_id: &str) -> bool {
        let target_path = self.save_path_for_id(target_id);
        target_path.exists() || Self::metadata_path_for_save_path(&target_path).exists()
    }

    fn confirm_pending_save_conflict(&mut self) {
        let Some(conflict) = self.pending_save_conflict.clone() else {
            return;
        };
        self.pending_save_conflict = None;
        match conflict {
            PendingSaveConflict::Rename { new_id } => {
                if let Some(entry) = self.selected_listing() {
                    match entry.rename_to_with_options(&new_id, true) {
                        Ok(updated) => {
                            self.selected_save_id = updated.id.clone();
                            self.save_id_input = updated.id;
                            self.sync_save_inputs_for_selection();
                            self.game
                                .push_log("Save renamed with overwrite.".to_string());
                        }
                        Err(err) => self.game.push_log(format!("Rename failed: {err}")),
                    }
                }
            }
            PendingSaveConflict::Duplicate { new_id } => {
                if let Some(entry) = self.selected_listing() {
                    match entry.duplicate_to_with_options(&new_id, true) {
                        Ok(updated) => {
                            self.selected_save_id = updated.id.clone();
                            self.save_id_input = updated.id;
                            self.sync_save_inputs_for_selection();
                            self.game
                                .push_log("Save duplicated with overwrite.".to_string());
                        }
                        Err(err) => self.game.push_log(format!("Duplicate failed: {err}")),
                    }
                }
            }
            PendingSaveConflict::Import {
                source_path,
                target_id,
            } => {
                self.run_import(source_path, target_id, true);
            }
            PendingSaveConflict::Export { target_path } => {
                if let Some(entry) = self.selected_listing() {
                    self.run_export(entry, target_path, true);
                }
            }
            PendingSaveConflict::Delete { id } => {
                self.commit_delete_selected_save(&id);
            }
        }
    }

    fn run_import(&mut self, source_path: PathBuf, target_id: String, overwrite: bool) {
        match SaveSlotListing::import_snapshot(&source_path, "saves", &target_id, overwrite) {
            Ok(updated) => {
                self.selected_save_id = updated.id.clone();
                self.save_id_input = updated.id;
                self.sync_save_inputs_for_selection();
                self.game
                    .push_log(format!("Imported save from {}.", source_path.display()));
            }
            Err(err) => self.game.push_log(format!("Import failed: {err}")),
        }
    }

    fn run_export(&mut self, entry: SaveSlotListing, target_path: PathBuf, overwrite: bool) {
        match entry.export_snapshot(&target_path, overwrite) {
            Ok(_) => self
                .game
                .push_log(format!("Exported save to {}.", target_path.display())),
            Err(err) => self.game.push_log(format!("Export failed: {err}")),
        }
    }

    fn select_next_unit(&mut self) {
        let mut player_units: Vec<usize> = self
            .game
            .units
            .iter()
            .filter(|u| u.alive && self.is_player_owned(u.owner) && u.moves_left > 0)
            .map(|u| u.id)
            .collect();

        player_units.sort_unstable();

        if player_units.is_empty() {
            self.selected_unit = None;
            return;
        }

        let next = match self.selected_unit {
            Some(current) => player_units
                .iter()
                .copied()
                .find(|id| *id > current)
                .unwrap_or(player_units[0]),
            None => player_units[0],
        };

        self.selected_unit = Some(next);

        if let Some(unit) = self.game.unit(next) {
            self.selected_tile = Some((unit.x, unit.y));
        }
    }

    fn draw_convoy_lines(
        &self,
        painter: &egui::Painter,
        tile_centers: &[Vec<egui::Pos2>],
        width: f32,
    ) {
        for line in self.game.convoy_overlay_lines(self.player_owner()) {
            if line.start_y >= tile_centers.len()
                || line.end_y >= tile_centers.len()
                || line.start_x >= tile_centers[line.start_y].len()
                || line.end_x >= tile_centers[line.end_y].len()
            {
                continue;
            }
            let stroke_color = color32_from_hex(
                presentation::convoy_overlay_status_color_hex(line.status),
                egui::Color32::from_rgb(72, 156, 88),
            );

            let start = tile_centers[line.start_y][line.start_x];
            let end = tile_centers[line.end_y][line.end_x];
            let elbow = egui::pos2(end.x, start.y);
            painter.line_segment([start, elbow], egui::Stroke::new(width, stroke_color));
            painter.line_segment([elbow, end], egui::Stroke::new(width, stroke_color));
        }
    }

    fn draw_minimap(&mut self, ui: &mut egui::Ui) {
        let minimap = self.game.minimap_display_state();
        let map_panel = self.game.map_panel_display_state(self.map_overlay);
        ui.heading(minimap.heading_text);
        let mut tile_centers = vec![vec![egui::Pos2::ZERO; minimap.width]; minimap.height];
        egui::Grid::new("minimap_grid")
            .spacing([1.0, 1.0])
            .show(ui, |ui| {
                for y in 0..minimap.height {
                    for x in 0..minimap.width {
                        let tile_state = self.game.minimap_tile_display_state(
                            x,
                            y,
                            self.player_owner(),
                            self.selected_tile,
                            self.map_overlay,
                        );
                        let mut button = egui::Button::new(&tile_state.label_text)
                            .fill(color32_from_hex(
                                &tile_state.color_hex,
                                egui::Color32::from_rgb(56, 96, 72),
                            ))
                            .min_size(egui::vec2(minimap.tile_min_size, minimap.tile_min_size));
                        if let Some(hex) = tile_state.selection_stroke_color_hex {
                            button = button.stroke(egui::Stroke::new(
                                1.0,
                                color32_from_hex(hex, egui::Color32::WHITE),
                            ));
                        }
                        let response = ui.add(button);
                        tile_centers[y][x] = response.rect.center();
                        if response.clicked() {
                            self.handle_tile_click(x, y);
                        }
                    }
                    ui.end_row();
                }
            });
        if map_panel.uses_convoy_lines {
            self.draw_convoy_lines(ui.painter(), &tile_centers, 0.75);
        }
    }
}

fn color32_from_hex(value: &str, fallback: egui::Color32) -> egui::Color32 {
    let hex = value.trim_start_matches('#');
    if hex.len() != 6 {
        return fallback;
    }

    let Ok(rgb) = u32::from_str_radix(hex, 16) else {
        return fallback;
    };

    egui::Color32::from_rgb(
        ((rgb >> 16) & 0xff) as u8,
        ((rgb >> 8) & 0xff) as u8,
        (rgb & 0xff) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BaseFocusFilter, BaseSortMode, LogisticsRouteFilter, SmacApp, VERIFIED_DEMO_HEIGHT,
        VERIFIED_DEMO_SEED, VERIFIED_DEMO_TURN_LIMIT, VERIFIED_DEMO_WIDTH,
    };
    use smac_core::{
        Base, GameState, GameStateSnapshot, GovernorMode, ProductionItem, Tech, Unit, UnitKind,
    };
    use std::fs;

    #[test]
    fn reset_selection_after_game_change_selects_a_player_unit() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(20, 20, 1234);
        app.selected_unit = Some(9999);
        app.selected_tile = None;

        app.reset_selection_after_game_change();

        assert!(app.selected_unit.is_some());
        assert!(app.selected_tile.is_some());
        let unit_id = app.selected_unit.expect("player unit should be selected");
        let unit = app.game.unit(unit_id).expect("selected unit should exist");
        assert_eq!(unit.owner, app.player_owner());
    }

    #[test]
    fn restart_demo_loads_verified_profile_and_resets_observer_progress() {
        let mut app = SmacApp::default();
        app.observer.completed_turns = 37;
        app.observer.running = true;
        app.observer.target_turns = 250;
        app.game = GameState::new_game(12, 9, 999);

        app.reset_to_verified_demo();

        assert_eq!(app.game.width, VERIFIED_DEMO_WIDTH);
        assert_eq!(app.game.height, VERIFIED_DEMO_HEIGHT);
        assert_eq!(app.game.seed, VERIFIED_DEMO_SEED);
        assert_eq!(app.observer.completed_turns, 0);
        assert!(!app.observer.running);
        assert_eq!(app.observer.target_turns, VERIFIED_DEMO_TURN_LIMIT);
    }

    #[test]
    fn observer_advancement_respects_turn_cap_and_stops_running() {
        let mut app = SmacApp::default();
        app.reset_to_verified_demo();
        app.observer.target_turns = 2;
        app.start_observer();

        let advanced = app.advance_observer_turns(5);

        assert_eq!(advanced, 2);
        assert_eq!(app.observer.completed_turns, 2);
        assert_eq!(app.game.turn, 3);
        assert!(!app.observer.running);
    }

    #[test]
    fn research_buckets_split_known_available_and_blocked_techs() {
        let mut app = SmacApp::default();
        let owner = app.player_owner();
        if let Some(faction) = app.game.faction_mut(owner) {
            faction.known_techs = vec![Tech::CentauriEcology, Tech::SocialPsych];
        }

        let (known, available, blocked) = app.game.research_buckets(owner);

        assert!(known.contains(&Tech::CentauriEcology));
        assert!(available.contains(&Tech::ProgenitorPsych));
        assert!(blocked.iter().any(|(tech, missing)| {
            *tech == Tech::GeneSplicing && missing.contains(&Tech::SecretsOfTheHumanBrain)
        }));
    }

    #[test]
    fn save_slot_label_reflects_saved_metadata() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_label_{}", std::process::id());
        app.selected_save_id = save_id.clone();
        app.save_id_input = save_id.clone();
        app.sync_save_inputs_for_selection();
        let path = app.current_save_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("save directory should be creatable");
        }

        let mut snapshot = GameStateSnapshot::from(&app.game);
        snapshot.save_name = Some("GUI Test Save".to_string());
        snapshot
            .save_to_path(&path)
            .expect("snapshot should save for label test");

        let label =
            smac_core::current_save_slot_label(&app.available_save_slots(), &app.selected_save_id);
        assert!(label.contains("GUI Test Save"));
        assert!(label.contains(&save_id));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("saves/{save_id}.json.meta"));
    }

    #[test]
    fn slot_name_input_syncs_from_saved_metadata() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_name_sync_{}", std::process::id());
        app.selected_save_id = save_id.clone();
        app.save_id_input = save_id;
        let path = app.current_save_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("save directory should be creatable");
        }

        let mut snapshot = GameStateSnapshot::from(&app.game);
        snapshot.save_name = Some("Named Slot".to_string());
        snapshot
            .save_to_path(&path)
            .expect("snapshot should save for name sync test");

        app.sync_save_inputs_for_selection();

        assert_eq!(app.save_name_input, "Named Slot");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}.meta", path.display()));
    }

    #[test]
    fn load_game_captures_recovery_notes_for_banner() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_recovery_{}", std::process::id());
        app.selected_save_id = save_id.clone();
        app.save_id_input = save_id;
        let path = app.current_save_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("save directory should be creatable");
        }

        let mut snapshot = GameStateSnapshot::from(&app.game);
        snapshot.save_name = Some("Recovery Banner Test".to_string());
        snapshot.recovery_notes = vec!["Recovered test note".to_string()];
        snapshot
            .save_to_path(&path)
            .expect("snapshot should save for recovery note test");

        app.load_game();

        assert_eq!(
            app.last_recovery_notes,
            vec!["Recovered test note".to_string()]
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}.meta", path.display()));
    }

    #[test]
    fn focus_helpers_select_expected_unit_and_base() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);

        let player_unit_id = app
            .game
            .units
            .iter()
            .find(|unit| unit.alive && unit.owner == app.player_owner())
            .map(|unit| unit.id)
            .expect("player unit should exist");
        let (unit_x, unit_y) = {
            let unit = app.game.unit(player_unit_id).expect("unit should exist");
            (unit.x, unit.y)
        };

        app.focus_unit(player_unit_id);
        assert_eq!(app.selected_unit, Some(player_unit_id));
        assert_eq!(app.selected_tile, Some((unit_x, unit_y)));

        app.game.bases.push(smac_core::Base {
            id: 999,
            owner: app.player_owner(),
            name: "Focus Base".to_string(),
            x: 6,
            y: 6,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: smac_core::GovernorMode::Off,
        });
        app.game.tiles[6 * app.game.width + 6].base = Some(999);

        app.focus_base(999);
        assert_eq!(app.selected_unit, None);
        assert_eq!(app.selected_tile, Some((6, 6)));
    }

    #[test]
    fn jump_worst_convoy_route_focuses_stressed_endpoint() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(18, 18, 7);
        let player_owner = app.player_owner();
        let ai_owner = app.game.ai_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Alpha".to_string(),
            x: 2,
            y: 2,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::TradeExchange,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        app.game.bases.push(smac_core::Base {
            id: 1,
            owner: player_owner,
            name: "Beta".to_string(),
            x: 6,
            y: 2,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::TradeExchange,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[2 * app.game.width + 2].base = Some(0);
        app.game.tiles[2 * app.game.width + 6].base = Some(1);
        app.game
            .add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
            .expect("route should be created");
        app.game.convoy_routes[0].integrity = 1;
        app.game.units.push(smac_core::Unit {
            design_index: 0,
            id: 400,
            owner: ai_owner,
            kind: UnitKind::Speeder,
            x: 4,
            y: 2,
            moves_left: 0,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });

        if let Some(base_id) = app.game.worst_convoy_route_focus_action(player_owner) {
            app.focus_base(base_id);
        }

        assert_eq!(app.selected_tile, Some((2, 2)));
    }

    #[test]
    fn filtered_convoy_repairs_restore_only_intercepted_routes() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(18, 18, 7);
        let player_owner = app.player_owner();
        let ai_owner = app.game.ai_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        for (id, name, x, y) in [
            (0, "Alpha", 2, 2),
            (1, "Beta", 6, 2),
            (2, "Gamma", 2, 10),
            (3, "Delta", 6, 10),
        ] {
            app.game.bases.push(smac_core::Base {
                id,
                owner: player_owner,
                name: name.to_string(),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::TradeExchange,
                production_queue: Vec::new(),
                facilities: vec![smac_core::Facility::TradeExchange],
                governor_mode: GovernorMode::Off,
            });
            app.game.tiles[y * app.game.width + x].base = Some(id);
        }

        app.game
            .add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
            .expect("first route should be created");
        app.game
            .add_convoy_route_typed(2, 3, smac_core::ConvoyRouteKind::Trade)
            .expect("second route should be created");
        app.game.convoy_routes[0].integrity = 2;
        app.game.convoy_routes[1].integrity = 2;
        app.game.units.push(smac_core::Unit {
            design_index: 0,
            id: 501,
            owner: ai_owner,
            kind: UnitKind::Speeder,
            x: 4,
            y: 2,
            moves_left: 0,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });

        app.logistics_route_filter = LogisticsRouteFilter::Intercepted;
        app.game.apply_filtered_convoy_repairs_for_owner(
            player_owner,
            app.logistics_route_filter,
            app.logistics_route_sort,
        );

        let mut routes = app.game.faction_convoy_route_summaries(player_owner);
        routes.sort_by_key(|route| (route.base_a_id, route.base_b_id));
        assert_eq!(
            routes[0].integrity, 3,
            "intercepted route should be repaired"
        );
        assert_eq!(
            routes[1].integrity, 2,
            "non-intercepted route should remain unchanged"
        );
    }

    #[test]
    fn suggested_route_creations_surface_dashboard_build_candidates() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(18, 18, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        for (id, name, x, y, facilities) in [
            (
                0,
                "Trade Hub",
                2,
                2,
                vec![smac_core::Facility::TradeExchange],
            ),
            (1, "Trade Spoke", 6, 2, vec![]),
            (
                2,
                "Freight Hub",
                2,
                10,
                vec![smac_core::Facility::FreightDepot],
            ),
            (3, "Freight Spoke", 6, 10, vec![]),
        ] {
            app.game.bases.push(smac_core::Base {
                id,
                owner: player_owner,
                name: name.to_string(),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::ScoutPatrol,
                production_queue: Vec::new(),
                facilities,
                governor_mode: GovernorMode::Off,
            });
            app.game.tiles[y * app.game.width + x].base = Some(id);
        }

        let all_candidates = app
            .game
            .convoy_route_opportunities(player_owner, app.logistics_route_filter);
        assert!(all_candidates
            .iter()
            .any(|entry| entry.kind == smac_core::ConvoyRouteKind::Freight));
        assert!(all_candidates
            .iter()
            .any(|entry| entry.kind == smac_core::ConvoyRouteKind::Trade));

        app.logistics_route_filter = LogisticsRouteFilter::Freight;
        let freight_candidates = app
            .game
            .convoy_route_opportunities(player_owner, app.logistics_route_filter);
        assert!(!freight_candidates.is_empty());
        assert!(freight_candidates
            .iter()
            .all(|entry| entry.kind == smac_core::ConvoyRouteKind::Freight));
    }

    #[test]
    fn convoy_hub_summary_sorts_more_saturated_bases_first() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(20, 20, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        for (id, name, x, y, facilities) in [
            (0, "Hub A", 2, 2, vec![smac_core::Facility::TradeExchange]),
            (1, "Hub B", 6, 2, vec![smac_core::Facility::TradeExchange]),
            (2, "Hub C", 10, 2, vec![smac_core::Facility::TransitHub]),
            (3, "Hub D", 14, 2, vec![]),
        ] {
            app.game.bases.push(smac_core::Base {
                id,
                owner: player_owner,
                name: name.to_string(),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::ScoutPatrol,
                production_queue: Vec::new(),
                facilities,
                governor_mode: GovernorMode::Off,
            });
            app.game.tiles[y * app.game.width + x].base = Some(id);
        }

        app.game
            .add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
            .expect("route 0-1 should be added");
        app.game
            .add_convoy_route_typed(0, 2, smac_core::ConvoyRouteKind::Trade)
            .expect("route 0-2 should be added");
        app.game
            .add_convoy_route_typed(1, 2, smac_core::ConvoyRouteKind::Trade)
            .expect("route 1-2 should be added");

        let hubs = app.game.faction_convoy_saturation(player_owner);
        assert_eq!(hubs[0].1, "Hub A");
        assert_eq!(hubs[0].2, 2);
        assert_eq!(hubs[0].3, 2);
        let (used, capacity) = app.game.base_convoy_saturation_ratio(0);
        assert_eq!((used, capacity), (2, 2));
    }

    #[test]
    fn saturated_and_tight_filters_find_expected_bases() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(20, 20, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        for (id, name, x, y, facilities) in [
            (0, "Hub A", 2, 2, vec![smac_core::Facility::TradeExchange]),
            (1, "Hub B", 6, 2, vec![smac_core::Facility::TradeExchange]),
            (2, "Hub C", 10, 2, vec![smac_core::Facility::TransitHub]),
            (3, "Hub D", 14, 2, vec![]),
        ] {
            app.game.bases.push(smac_core::Base {
                id,
                owner: player_owner,
                name: name.to_string(),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::ScoutPatrol,
                production_queue: Vec::new(),
                facilities,
                governor_mode: GovernorMode::Off,
            });
            app.game.tiles[y * app.game.width + x].base = Some(id);
        }
        app.game
            .add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
            .expect("route should be added");
        app.game
            .add_convoy_route_typed(0, 2, smac_core::ConvoyRouteKind::Trade)
            .expect("route should be added");
        app.game
            .add_convoy_route_typed(1, 2, smac_core::ConvoyRouteKind::Trade)
            .expect("route should be added");

        let saturated = app
            .game
            .faction_base_ids_for_focus(app.player_owner(), BaseFocusFilter::Saturated);
        let tight = app
            .game
            .faction_base_ids_for_focus(app.player_owner(), BaseFocusFilter::Tight);
        assert!(saturated.contains(&0));
        assert!(tight.contains(&0));
        assert!(tight.contains(&1));
    }

    #[test]
    fn queue_gap_and_research_unlock_filters_find_expected_bases() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        app.game.bases.push(Base {
            id: 1,
            owner: player_owner,
            name: "Queued Base".to_string(),
            x: 8,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: vec![ProductionItem::Former],
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[5 * app.game.width + 8].base = Some(1);

        if let Some(faction) = app.game.faction_mut(player_owner) {
            faction.current_research = Tech::PlanetaryNetworks;
        }

        let queue_gap = app
            .game
            .faction_base_ids_for_focus(app.player_owner(), BaseFocusFilter::QueueGap);
        let research_unlock = app
            .game
            .faction_base_ids_for_focus(app.player_owner(), BaseFocusFilter::ResearchUnlock);

        assert!(queue_gap.contains(&0));
        assert!(!queue_gap.contains(&1));
        assert_eq!(research_unlock, vec![0]);
    }

    #[test]
    fn queue_gap_focus_action_fills_matching_base_queue() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Queue Gap".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        if let Some(faction) = app.game.faction_mut(player_owner) {
            faction.known_techs.push(Tech::PlanetaryNetworks);
        }

        app.apply_base_focus_action(BaseFocusFilter::QueueGap);

        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            ProductionItem::HologramTheatre
        );
    }

    #[test]
    fn frontier_sort_prioritizes_more_exposed_base() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(20, 20, 42);
        let player_owner = app.player_owner();
        let ai_owner = app.game.ai_owner();
        app.game.bases.clear();
        app.game.units.clear();
        for tile in &mut app.game.tiles {
            tile.base = None;
            tile.unit = None;
        }

        let safe_base_id = 900;
        app.game.bases.push(Base {
            id: safe_base_id,
            owner: player_owner,
            name: "Safe Base".to_string(),
            x: 3,
            y: 3,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[3 * app.game.width + 3].base = Some(safe_base_id);

        let frontier_base_id = 901;
        app.game.bases.push(Base {
            id: frontier_base_id,
            owner: player_owner,
            name: "Frontier Base".to_string(),
            x: 14,
            y: 14,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Defense,
        });
        app.game.tiles[14 * app.game.width + 14].base = Some(frontier_base_id);

        let enemy_unit_id = app.game.units.len();
        app.game.tiles[14 * app.game.width + 16].unit = Some(enemy_unit_id);
        app.game.units.push(Unit {
            design_index: 0,
            id: enemy_unit_id,
            owner: ai_owner,
            kind: UnitKind::ScoutPatrol,
            x: 16,
            y: 14,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });

        app.base_sort_mode = BaseSortMode::Frontier;
        let filtered = app
            .game
            .faction_base_ids_for_focus(app.player_owner(), BaseFocusFilter::Frontier);
        assert_eq!(filtered.first().copied(), Some(frontier_base_id));
    }

    #[test]
    fn logistics_filter_prioritizes_bases_with_route_stress() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(16, 16, 7);
        let player_owner = app.player_owner();
        let ai_owner = app.game.ai_owner();
        app.game.bases.clear();
        app.game.units.clear();
        for tile in &mut app.game.tiles {
            tile.base = None;
            tile.unit = None;
            tile.terrain = smac_core::Terrain::Flat;
            tile.moisture = 70;
        }

        app.game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Logistics Hub".to_string(),
            x: 4,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[4 * app.game.width + 4].base = Some(0);
        app.game.bases.push(Base {
            id: 1,
            owner: player_owner,
            name: "Logistics Spoke".to_string(),
            x: 8,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[4 * app.game.width + 8].base = Some(1);
        app.game
            .add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
            .expect("trade route should be added");

        app.game.tiles[4 * app.game.width + 6].unit = Some(0);
        app.game.units.push(Unit {
            design_index: 0,
            id: 0,
            owner: ai_owner,
            kind: UnitKind::RaiderSpeeder,
            x: 6,
            y: 4,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });

        app.base_sort_mode = BaseSortMode::Logistics;
        let filtered = app
            .game
            .faction_base_ids_for_focus(app.player_owner(), BaseFocusFilter::Logistics);
        assert_eq!(filtered.first().copied(), Some(0));
    }

    #[test]
    fn apply_recommended_governors_sets_defense_for_convoy_pressure() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(16, 16, 7);
        let player_owner = app.player_owner();
        let ai_owner = app.game.ai_owner();
        app.game.bases.clear();
        app.game.units.clear();
        for tile in &mut app.game.tiles {
            tile.base = None;
            tile.unit = None;
            tile.terrain = smac_core::Terrain::Flat;
            tile.moisture = 70;
        }

        app.game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Governor Hub".to_string(),
            x: 4,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[4 * app.game.width + 4].base = Some(0);
        app.game.bases.push(Base {
            id: 1,
            owner: player_owner,
            name: "Governor Spoke".to_string(),
            x: 8,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        app.game.tiles[4 * app.game.width + 8].base = Some(1);
        app.game
            .add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
            .expect("trade route should be added");

        app.game.tiles[4 * app.game.width + 6].unit = Some(0);
        app.game.units.push(Unit {
            design_index: 0,
            id: 0,
            owner: ai_owner,
            kind: UnitKind::RaiderSpeeder,
            x: 6,
            y: 4,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });

        app.apply_recommended_governors();

        assert_eq!(
            app.game.base(0).expect("base should exist").governor_mode,
            GovernorMode::Logistics
        );
    }

    #[test]
    fn rename_conflict_is_staged_for_confirmation() {
        let mut app = SmacApp::default();
        let source_id = format!("gui_rename_source_{}", std::process::id());
        let target_id = format!("gui_rename_target_{}", std::process::id());
        let source_path = app.save_path_for_id(&source_id);
        let target_path = app.save_path_for_id(&target_id);
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent).expect("save directory should be creatable");
        }

        let mut source = GameStateSnapshot::from(&app.game);
        source.save_name = Some("Source Save".to_string());
        source
            .save_to_path(&source_path)
            .expect("source save should write");

        let mut target = GameStateSnapshot::from(&app.game);
        target.save_name = Some("Target Save".to_string());
        target
            .save_to_path(&target_path)
            .expect("target save should write");

        app.selected_save_id = source_id;
        app.save_id_input = target_id;
        app.rename_selected_save();

        assert!(app.pending_save_conflict.is_some());

        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(format!("{}.meta", source_path.display()));
        let _ = fs::remove_file(&target_path);
        let _ = fs::remove_file(format!("{}.meta", target_path.display()));
    }

    #[test]
    fn save_slot_label_formats_timestamp_for_display() {
        let entry = smac_core::SaveSlotListing {
            id: "custom_save".to_string(),
            snapshot_path: "saves/custom_save.json".into(),
            metadata_path: "saves/custom_save.json.meta".into(),
            category: smac_core::SaveSlotCategory::Imported,
            metadata: Some(smac_core::SaveSlotMetadata {
                save_name: "Custom".to_string(),
                saved_turn: 9,
                recovery_note_count: 1,
                last_updated_unix: Some(0),
                notes: "Long running campaign".to_string(),
                category: Some(smac_core::SaveSlotCategory::Imported),
                auto_recovery_base_ids: Vec::new(),
                auto_defense_base_ids: Vec::new(),
                auto_economy_base_ids: Vec::new(),
            }),
        };

        let label = smac_core::save_slot_label(&entry);
        assert!(label.contains("1970-01-01 00:00 UTC"));
        assert!(label.contains("Recovered (1)"));
        assert!(label.contains("[Imported]"));
    }

    #[test]
    fn delete_conflict_is_staged_for_confirmation() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_delete_{}", std::process::id());
        let source_path = app.save_path_for_id(&save_id);
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent).expect("save directory should be creatable");
        }

        let mut source = GameStateSnapshot::from(&app.game);
        source.save_name = Some("Delete Me".to_string());
        source
            .save_to_path(&source_path)
            .expect("source save should write");

        app.selected_save_id = save_id;
        app.delete_selected_save();

        assert!(matches!(
            app.pending_save_conflict,
            Some(super::PendingSaveConflict::Delete { .. })
        ));

        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(format!("{}.meta", source_path.display()));
    }

    #[test]
    fn manual_save_persists_notes_and_category_metadata() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_manual_save_{}", std::process::id());
        app.save_id_input = save_id.clone();
        app.save_name_input = "Manual Save".to_string();
        app.save_notes_input = "Frontier checkpoint".to_string();

        app.save_game_as(smac_core::SaveSlotCategory::Manual);

        let path = app.save_path_for_id(&save_id);
        let metadata =
            smac_core::SaveSlotMetadata::load_from_path(&path).expect("saved metadata should load");
        assert_eq!(metadata.notes, "Frontier checkpoint");
        assert_eq!(metadata.category, Some(smac_core::SaveSlotCategory::Manual));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}.meta", path.display()));
    }

    #[test]
    fn manual_save_persists_automation_metadata() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_automation_save_{}", std::process::id());
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }
        for id in 0..3 {
            app.game.bases.push(smac_core::Base {
                id,
                owner: app.player_owner(),
                name: format!("Base {id}"),
                x: 3 + id,
                y: 4,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: smac_core::ProductionItem::ScoutPatrol,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: smac_core::GovernorMode::Off,
            });
            app.game.tiles[4 * app.game.width + 3 + id].base = Some(id);
        }

        app.game
            .base_mut(0)
            .expect("base 0 should exist")
            .governor_mode = smac_core::GovernorMode::Recovery;
        app.game
            .base_mut(1)
            .expect("base 1 should exist")
            .governor_mode = smac_core::GovernorMode::Defense;
        app.game
            .base_mut(2)
            .expect("base 2 should exist")
            .governor_mode = smac_core::GovernorMode::Economy;
        app.save_id_input = save_id.clone();
        app.save_game_as(smac_core::SaveSlotCategory::Manual);

        let path = app.save_path_for_id(&save_id);
        let metadata =
            smac_core::SaveSlotMetadata::load_from_path(&path).expect("saved metadata should load");
        assert_eq!(metadata.auto_recovery_base_ids, vec![0]);
        assert_eq!(metadata.auto_defense_base_ids, vec![1]);
        assert_eq!(metadata.auto_economy_base_ids, vec![2]);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}.meta", path.display()));
    }

    #[test]
    fn load_game_restores_automation_sets_from_metadata() {
        let mut app = SmacApp::default();
        let save_id = format!("gui_automation_load_{}", std::process::id());
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }
        for id in 0..3 {
            app.game.bases.push(smac_core::Base {
                id,
                owner: app.player_owner(),
                name: format!("Base {id}"),
                x: 3 + id,
                y: 4,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: smac_core::ProductionItem::ScoutPatrol,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: smac_core::GovernorMode::Off,
            });
            app.game.tiles[4 * app.game.width + 3 + id].base = Some(id);
        }

        app.selected_save_id = save_id.clone();
        app.save_id_input = save_id.clone();
        let path = app.current_save_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("save directory should be creatable");
        }

        let mut snapshot = GameStateSnapshot::from(&app.game);
        snapshot.save_name = Some("Automation Restore".to_string());
        snapshot
            .save_to_path(&path)
            .expect("snapshot should save for automation restore test");
        smac_core::SaveSlotMetadata {
            save_name: "Automation Restore".to_string(),
            saved_turn: app.game.turn,
            recovery_note_count: 0,
            last_updated_unix: None,
            notes: String::new(),
            category: Some(smac_core::SaveSlotCategory::Manual),
            auto_recovery_base_ids: vec![0],
            auto_defense_base_ids: vec![1],
            auto_economy_base_ids: vec![2],
        }
        .save_to_path(format!("{}.meta", path.display()))
        .expect("metadata should save");

        app.load_game();

        assert!(
            app.game.base(0).expect("base 0 should exist").governor_mode
                == smac_core::GovernorMode::Recovery
        );
        assert!(
            app.game.base(1).expect("base 1 should exist").governor_mode
                == smac_core::GovernorMode::Defense
        );
        assert!(
            app.game.base(2).expect("base 2 should exist").governor_mode
                == smac_core::GovernorMode::Economy
        );

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}.meta", path.display()));
    }

    #[test]
    fn cycle_recovering_bases_focuses_a_base_with_damaged_garrison() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: app.player_owner(),
            name: "Recovery Base".to_string(),
            x: 5,
            y: 5,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::CommandCenter],
            governor_mode: smac_core::GovernorMode::Off,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);
        app.game.units.push(smac_core::Unit {
            design_index: 0,
            id: 0,
            owner: app.player_owner(),
            kind: smac_core::UnitKind::ScoutPatrol,
            x: 5,
            y: 5,
            moves_left: 1,
            hp: 4,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        app.game.tiles[5 * app.game.width + 5].unit = Some(0);

        app.cycle_recovering_bases();

        assert_eq!(app.selected_tile, Some((5, 5)));
        assert_eq!(app.selected_unit, None);
    }

    #[test]
    fn current_research_unlock_cycle_focuses_affected_base() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        if let Some(faction) = app.game.faction_mut(player_owner) {
            faction.current_research = smac_core::Tech::PlanetaryNetworks;
        }

        let affected = app
            .game
            .player_operations_focus_state(app.selected_unit, app.selected_base_id());
        assert_eq!(affected.current_research_unlock_base_count, 1);

        app.focus_base(
            affected
                .current_research_unlock_focus_base_id
                .expect("affected focus target should exist"),
        );

        assert_eq!(app.selected_tile, Some((5, 5)));
        assert_eq!(app.selected_unit, None);
    }

    #[test]
    fn available_tech_focus_target_targets_affected_base() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        let affected = app
            .game
            .pin_research_unlock_preview_state_action(
                player_owner,
                smac_core::Tech::PlanetaryNetworks,
                3,
                app.selected_base_id(),
            )
            .affected_base_ids;
        assert_eq!(affected, vec![0]);

        let focus_base_id = app.game.research_unlock_preview_focus_base_id(
            player_owner,
            smac_core::Tech::PlanetaryNetworks,
            3,
            app.selected_base_id(),
        );
        assert_eq!(focus_base_id, Some(0));
        app.focus_base(focus_base_id.expect("affected focus target should exist"));

        assert_eq!(app.selected_tile, Some((5, 5)));
        assert_eq!(app.selected_unit, None);
    }

    #[test]
    fn preview_queue_action_stages_persistent_unlock_preview() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        let result = app.game.pin_research_unlock_preview_state_action(
            player_owner,
            smac_core::Tech::PlanetaryNetworks,
            3,
            app.selected_base_id(),
        );
        let _ = app.apply_preview_state_action_result(result, false);

        let staged = app
            .staged_unlock_preview
            .as_ref()
            .expect("staged preview should exist");
        assert_eq!(staged.tech, smac_core::Tech::PlanetaryNetworks);
        assert!(staged.previews.iter().any(|(base_id, items)| *base_id == 0
            && items.first() == Some(&smac_core::ProductionItem::HologramTheatre)));
    }

    #[test]
    fn available_tech_preview_state_reports_current_and_stale_counts() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        let result = app.game.pin_research_unlock_preview_state_action(
            player_owner,
            smac_core::Tech::PlanetaryNetworks,
            3,
            app.selected_base_id(),
        );
        let _ = app.apply_preview_state_action_result(result, false);
        let panel = app.game.research_panel_display_state(
            player_owner,
            app.selected_base_id(),
            app.staged_unlock_preview.as_ref(),
        );
        assert!(panel.available.iter().all(|entry| {
            entry.preview_action_label.is_some() == (entry.unlock_impact.recommendation_count > 0)
        }));

        let display = app.game.pinned_research_unlock_preview_display_state(
            player_owner,
            app.staged_unlock_preview
                .as_ref()
                .expect("staged preview should exist"),
        );
        assert!(display.drift_text.is_none());
        assert_eq!(display.hidden_count, 0);
        assert!(display
            .rows
            .iter()
            .all(|preview_row| preview_row.is_current));

        assert_eq!(
            app.game.research_unlock_preview_counts_for_tech(
                app.staged_unlock_preview.as_ref(),
                smac_core::Tech::PlanetaryNetworks
            ),
            Some((1, 0))
        );

        if let Some(base) = app.game.bases.iter_mut().find(|base| base.id == 0) {
            base.facilities.push(smac_core::Facility::HologramTheatre);
        }

        let panel = app.game.research_panel_display_state(
            player_owner,
            app.selected_base_id(),
            app.staged_unlock_preview.as_ref(),
        );
        assert!(panel.available.iter().all(|entry| {
            entry.preview_action_label.is_some() == (entry.unlock_impact.recommendation_count > 0)
        }));
        let display = app.game.pinned_research_unlock_preview_display_state(
            player_owner,
            app.staged_unlock_preview
                .as_ref()
                .expect("staged preview should exist"),
        );
        assert_eq!(
            display.drift_text.as_deref(),
            Some("1 staged base preview(s) drifted from current governor intent.")
        );
    }

    #[test]
    fn applying_staged_unlock_preview_updates_base_and_clears_preview() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        let result = app.game.pin_research_unlock_preview_state_action(
            player_owner,
            smac_core::Tech::PlanetaryNetworks,
            3,
            app.selected_base_id(),
        );
        let _ = app.apply_preview_state_action_result(result, false);
        let display = app.game.pinned_research_unlock_preview_display_state(
            player_owner,
            app.staged_unlock_preview
                .as_ref()
                .expect("staged preview should exist"),
        );
        assert!(!display.can_apply);

        if let Some(faction) = app.game.faction_mut(player_owner) {
            faction.known_techs.push(smac_core::Tech::PlanetaryNetworks);
        }

        let display = app.game.pinned_research_unlock_preview_display_state(
            player_owner,
            app.staged_unlock_preview
                .as_ref()
                .expect("staged preview should exist"),
        );
        assert!(display.can_apply);
        app.apply_staged_unlock_preview_to_base(player_owner, 0);

        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            smac_core::ProductionItem::HologramTheatre
        );
        assert!(app.staged_unlock_preview.is_none());
    }

    #[test]
    fn stale_staged_unlock_preview_requires_refresh_before_apply() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        let result = app.game.pin_research_unlock_preview_state_action(
            player_owner,
            smac_core::Tech::PlanetaryNetworks,
            3,
            app.selected_base_id(),
        );
        let _ = app.apply_preview_state_action_result(result, false);
        if let Some(base) = app.game.bases.iter_mut().find(|base| base.id == 0) {
            base.facilities.push(smac_core::Facility::HologramTheatre);
        }
        if let Some(faction) = app.game.faction_mut(player_owner) {
            faction.known_techs.push(smac_core::Tech::PlanetaryNetworks);
        }

        let display = app.game.pinned_research_unlock_preview_display_state(
            player_owner,
            app.staged_unlock_preview
                .as_ref()
                .expect("staged preview should exist"),
        );
        assert_eq!(display.drifted_base_ids, vec![0]);

        app.apply_staged_unlock_preview_to_base(player_owner, 0);
        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            smac_core::ProductionItem::ScoutPatrol
        );
        assert!(app.staged_unlock_preview.is_some());

        app.refresh_staged_unlock_preview(player_owner);
        assert!(app.staged_unlock_preview.is_none());
    }

    #[test]
    fn sync_staged_unlock_preview_clears_unknown_preview_when_no_longer_relevant() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        let player_owner = app.player_owner();
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rocky;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: player_owner,
            name: "Locked Morale".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![smac_core::Facility::RecreationCommons],
            governor_mode: smac_core::GovernorMode::Economy,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        let result = app.game.pin_research_unlock_preview_state_action(
            player_owner,
            smac_core::Tech::PlanetaryNetworks,
            3,
            app.selected_base_id(),
        );
        let _ = app.apply_preview_state_action_result(result, false);
        assert!(app.staged_unlock_preview.is_some());

        if let Some(base) = app.game.bases.iter_mut().find(|base| base.id == 0) {
            base.facilities.push(smac_core::Facility::HologramTheatre);
        }

        app.sync_staged_unlock_preview(player_owner);
        assert!(app.staged_unlock_preview.is_none());
    }

    #[test]
    fn apply_recovery_plans_all_updates_a_stressed_base_production() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rolling;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: app.player_owner(),
            name: "Recovery Plan".to_string(),
            x: 5,
            y: 5,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: smac_core::GovernorMode::Off,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        if let Some(faction) = app.game.faction_mut(app.player_owner()) {
            faction.known_techs.push(smac_core::Tech::ProgenitorPsych);
        }

        app.game.units.push(smac_core::Unit {
            design_index: 0,
            id: 0,
            owner: app.game.native_owner(),
            kind: smac_core::UnitKind::MindWorm,
            x: 3,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        app.game.tiles[5 * app.game.width + 3].unit = Some(0);

        app.apply_recovery_plans_all(3);

        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            smac_core::ProductionItem::PsiSentinel
        );
    }

    #[test]
    fn apply_defense_plans_all_updates_a_frontier_base_production() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rolling;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: app.player_owner(),
            name: "Defense Plan".to_string(),
            x: 5,
            y: 5,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: smac_core::GovernorMode::Off,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        if let Some(faction) = app.game.faction_mut(app.player_owner()) {
            faction.known_techs.push(smac_core::Tech::DoctrineMobility);
        }

        app.game.units.push(smac_core::Unit {
            design_index: 0,
            id: 0,
            owner: app.game.ai_owner(),
            kind: smac_core::UnitKind::ScoutPatrol,
            x: 3,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        app.game.tiles[5 * app.game.width + 3].unit = Some(0);

        app.apply_defense_plans_all(3);

        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            smac_core::ProductionItem::PerimeterDefense
        );
    }

    #[test]
    fn enabled_defense_automation_applies_before_end_turn() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Rolling;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: app.player_owner(),
            name: "Auto Defense".to_string(),
            x: 5,
            y: 5,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: smac_core::GovernorMode::Off,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);

        if let Some(faction) = app.game.faction_mut(app.player_owner()) {
            faction.known_techs.push(smac_core::Tech::DoctrineMobility);
        }

        app.game.units.push(smac_core::Unit {
            design_index: 0,
            id: 0,
            owner: app.game.ai_owner(),
            kind: smac_core::UnitKind::ScoutPatrol,
            x: 3,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        app.game.tiles[5 * app.game.width + 3].unit = Some(0);

        app.game
            .base_mut(0)
            .expect("base should exist")
            .governor_mode = smac_core::GovernorMode::Defense;
        app.apply_enabled_automations();

        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            smac_core::ProductionItem::PerimeterDefense
        );
    }

    #[test]
    fn enabled_economy_automation_applies_before_end_turn() {
        let mut app = SmacApp::default();
        app.game = GameState::new_game(12, 12, 7);
        app.game.units.clear();
        app.game.bases.clear();
        for tile in &mut app.game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = smac_core::Terrain::Flat;
        }

        app.game.bases.push(smac_core::Base {
            id: 0,
            owner: app.player_owner(),
            name: "Auto Economy".to_string(),
            x: 5,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: smac_core::GovernorMode::Off,
        });
        app.game.tiles[5 * app.game.width + 5].base = Some(0);
        if let Some(faction) = app.game.faction_mut(app.player_owner()) {
            faction.known_techs.push(smac_core::Tech::SocialPsych);
        }

        app.game
            .base_mut(0)
            .expect("base should exist")
            .governor_mode = smac_core::GovernorMode::Economy;
        app.apply_enabled_automations();

        assert_eq!(
            app.game.base(0).expect("base should exist").production,
            smac_core::ProductionItem::RecreationCommons
        );
    }
}
