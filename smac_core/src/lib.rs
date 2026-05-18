mod ai;
pub mod assets;
mod content;
mod game_state;
mod model;
mod narrative;
pub mod presentation;
pub mod save;

pub mod content_api {
    pub use crate::content::{
        ai_attack_bias, ai_colony_base_target, ai_expansion_base_target, ai_exploration_bias,
        ai_native_spawn_noise_salt, ai_preferred_production, ai_terraform_bias,
        base_attack_penalty, base_defense_bonus, base_growth_nutrients_threshold,
        base_starting_minerals_stock, base_starting_nutrients_stock, base_starting_population,
        build_runtime_factions, default_starting_factions, default_starting_scenario,
        facility_convoy_capacity_bonus, facility_convoy_security_bonus, facility_defense_bonus,
        facility_free_unit_support_bonus, facility_growth_threshold_reduction,
        facility_maintenance, facility_mobility_bonus, facility_name, facility_psi_support_bonus,
        facility_repair_bonus, facility_stability_bonus, facility_training_bonus,
        facility_yield_bonus, faction_definition_by_name, forced_land_patch_radius,
        free_unit_support_per_base, fungus_defense_bonus, load_facility_definitions,
        load_faction_definitions, load_production_definitions, load_runtime_rules,
        load_runtime_tech_definitions, load_ui_theme_definition, load_unit_definitions,
        map_flat_moisture_threshold, map_fungus_threshold, map_ocean_threshold,
        map_pod_spawn_threshold, map_rocky_threshold, native_spawn_roll_threshold,
        native_spawn_turn_interval, next_base_name_for_faction, owner_for_role,
        player_base_visibility_radius, player_unit_visibility_radius, production_cost,
        production_facility, production_name, production_unit_kind, required_tech_for_production,
        rocky_defense_bonus, rolling_defense_bonus, runtime_faction_definition,
        runtime_faction_definition_by_owner, runtime_role_for_owner, runtime_roles,
        supply_pod_energy_reward, supply_pod_salvage_energy_reward, tech_cost, tech_description,
        tech_enabled_facility_names, tech_enabled_unit_names, tech_is_available, tech_name,
        tech_prerequisites, try_default_runtime_faction_setups, try_default_starting_scenario,
        try_runtime_rules_definition, try_runtime_tech_definitions, try_ui_theme_definition,
        try_unit_definition_by_id, ui_accent_hex, ui_app_title, ui_command_console_heading,
        ui_danger_hex, ui_defeat_text, ui_event_log_heading, ui_factions_heading,
        ui_panel_fill_hex, ui_planet_heading, ui_research_heading, ui_selection_heading,
        ui_success_hex, ui_victory_text, ui_warning_hex, ui_warning_text, ui_window_title,
        unit_attack, unit_base_hp, unit_defense, unit_definition_by_id, unit_max_moves, unit_name,
        unit_support_cost, validate_bundled_content, ContentLookupError, FacilityDefinition,
        FactionAiPolicy, FactionDefinition, FactionPersonality, ProductionDefinition,
        RuntimeFactionSetupError, RuntimeRole, RuntimeRoles, RuntimeRulesDefinition, StartPosition,
        StartingFactionSetup, StartingScenario, StartingScenarioError, StartingUnitSetup,
        TechRuntimeDefinition, UiThemeDefinition, UnitDefinition,
    };
}
pub mod factions;
pub mod localization;
pub mod map;
pub mod player;
pub mod technology_tree;
pub mod units;

pub use units::components::{Ability, Armor, Chassis, Morale, Weapon};
pub use units::definitions::UnitDesign;

pub use content::RuntimeRole;
pub use content::RuntimeRoles;
pub use ai::{offense_readiness_for_owner, run_ai_tactics, run_ai_tactics_for_owner, AiOffenseReadiness};
pub use game_state::base_focus_filter_label;
pub use game_state::base_sort_mode_label;
pub use game_state::logistics_route_filter_label;
pub use game_state::logistics_route_sort_label;
pub use game_state::AvailableConvoyTargetOpportunityState;
pub use game_state::AvailableResearchDisplayState;
pub use game_state::BaseConvoyRouteDisplayRowState;
pub use game_state::BaseFocusFilter;
pub use game_state::BaseFocusState;
pub use game_state::BaseGovernorPlanRowState;
pub use game_state::BasePanelDisplayState;
pub use game_state::BaseProductionOptionState;
pub use game_state::BaseQueueRowState;
pub use game_state::BaseSortMode;
pub use game_state::BaseStatusTagKind;
pub use game_state::BaseStatusTagState;
pub use game_state::BlockedResearchDisplayState;
pub use game_state::CommandConsoleDisplayState;
pub use game_state::ConvoyHubDisplayRowState;
pub use game_state::ConvoyOverlayLine;
pub use game_state::ConvoyOverlayStatus;
pub use game_state::ConvoyRouteDisplayRowState;
pub use game_state::ConvoyRouteOpportunityState;
pub use game_state::CurrentResearchDisplayState;
pub use game_state::FactionLogisticsPanelState;
pub use game_state::FactionOverviewDisplayState;
pub use game_state::GovernorPlanStep;
pub use game_state::ImprovementActionState;
pub use game_state::KnownResearchDisplayState;
pub use game_state::LogisticsBoardDisplayState;
pub use game_state::LogisticsPanelActionState;
pub use game_state::LogisticsRouteFilter;
pub use game_state::LogisticsRouteSort;
pub use game_state::MapInteractionResult;
pub use game_state::MapOverlayOptionState;
pub use game_state::MapPanelDisplayState;
pub use game_state::MapTileDisplayState;
pub use game_state::PinnedResearchUnlockPreviewDisplayState;
pub use game_state::PinnedResearchUnlockPreviewRowState;
pub use game_state::PlayerOperationsActionState;
pub use game_state::PlayerOperationsActionType;
pub use game_state::PlayerOperationsDashboardState;
pub use game_state::PlayerOperationsFocusState;
pub use game_state::ProductionPreviewRow;
pub use game_state::ResearchPanelDisplayState;
pub use game_state::ResearchUnlockPreviewApplyResult;
pub use game_state::ResearchUnlockPreviewRefreshResult;
pub use game_state::ResearchUnlockPreviewSelection;
pub use game_state::ResearchUnlockPreviewStageResult;
pub use game_state::ResearchUnlockPreviewState;
pub use game_state::ResearchUnlockPreviewStateActionResult;
pub use game_state::ResearchUnlockPreviewStatus;
pub use game_state::SelectionPanelDisplayState;
pub use game_state::SidebarDisplayState;
pub use game_state::SidebarTab;
pub use game_state::TechUnlockImpactState;
pub use game_state::TileSelectionDisplayState;
pub use game_state::TopBarDisplayState;
pub use game_state::UnitSelectionDisplayState;
pub use model::{
    ActionableAlert, AlertPriority, Base, BaseAreaRole, ConvoyRoute, ConvoyRouteKind,
    ConvoyRouteSummary, DiplomacyStatus, DiplomaticRelation, Economics, EventCategory,
    EventLogEntry, Facility, Faction, FutureSociety, GameAction, GameOver, GameState, GovernorMode,
    Improvement, Politics, ProductionItem, SecretProject, Tech, Terrain, Tile, TurnSummary, Unit,
    UnitActivity, UnitKind, Values, Yields,
};
pub use presentation::MapOverlay;
pub use save::current_save_slot_label;
pub use save::filtered_sorted_save_slots;
pub use save::matches_save_filters;
pub use save::normalize_save_id;
pub use save::save_browser_counts;
pub use save::save_browser_counts_text;
pub use save::save_browser_display_state;
pub use save::save_filter_label;
pub use save::save_management_display_state;
pub use save::save_slot_label;
pub use save::save_sort_button_states;
pub use save::save_sort_label;
pub use save::set_save_sort;
pub use save::sort_save_slots;
pub use save::GameStateSnapshot;
pub use save::SaveBrowserCounts;
pub use save::SaveBrowserDisplayState;
pub use save::SaveBrowserQuery;
pub use save::SaveBrowserRowState;
pub use save::SaveFilterCategory;
pub use save::SaveSlotCategory;
pub use save::SaveSlotListing;
pub use save::SaveSlotMetadata;
pub use save::SaveSortButtonState;
pub use save::SaveSortColumn;
pub use save::GAME_STATE_SNAPSHOT_VERSION;
pub use technology_tree::{TechCategory, TechEnables, Technology, TechnologyTree};
