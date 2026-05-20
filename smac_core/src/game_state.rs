use crate::ai;
use crate::content;
use crate::model::{
    Base, BaseAreaRole, CommandCenterTurnTrace, DemandKind, DiplomacyStatus, DiplomaticRelation,
    Economics, EventCategory, EventLogEntry, Facility, FutureSociety, GameAction, GameOver,
    GameState, GovernorMode, Improvement, Politics, ProbeAction, ProductionItem, SecretProject,
    Tech, Terrain, Tile, Unit, UnitActivity, UnitKind, Values, Yields,
};
use crate::presentation;
use crate::{Ability, Chassis, UnitDesign, Weapon};
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseSortMode {
    Name,
    Unrest,
    Frontier,
    Recovery,
    Governor,
    Logistics,
}

pub fn base_sort_mode_label(sort_mode: BaseSortMode) -> &'static str {
    match sort_mode {
        BaseSortMode::Name => "Name",
        BaseSortMode::Unrest => "Unrest",
        BaseSortMode::Frontier => "Frontier",
        BaseSortMode::Recovery => "Recovery",
        BaseSortMode::Governor => "Governor",
        BaseSortMode::Logistics => "Logistics",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseFocusFilter {
    All,
    Frontier,
    Recovery,
    Unrest,
    QueueGap,
    ResearchUnlock,
    Logistics,
    Saturated,
    Tight,
    Collapsing,
    Balanced,
    Defense,
    Economy,
    LogisticsMode,
}

pub fn base_focus_filter_label(filter: BaseFocusFilter) -> &'static str {
    match filter {
        BaseFocusFilter::All => "All",
        BaseFocusFilter::Frontier => "Frontier",
        BaseFocusFilter::Recovery => "Recovery",
        BaseFocusFilter::Unrest => "Unrest",
        BaseFocusFilter::QueueGap => "Queue Gap",
        BaseFocusFilter::ResearchUnlock => "Research Unlock",
        BaseFocusFilter::Logistics => "Logistics",
        BaseFocusFilter::Saturated => "Saturated",
        BaseFocusFilter::Tight => "Tight",
        BaseFocusFilter::Collapsing => "Collapsing",
        BaseFocusFilter::Balanced => "Balanced",
        BaseFocusFilter::Defense => "Defense",
        BaseFocusFilter::Economy => "Economy",
        BaseFocusFilter::LogisticsMode => "Logistics Mode",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogisticsRouteFilter {
    All,
    Trade,
    Freight,
    Military,
    Disrupted,
    Intercepted,
    Collapsing,
    Protected,
}

impl LogisticsRouteFilter {
    pub fn all() -> [Self; 8] {
        [
            Self::All,
            Self::Trade,
            Self::Freight,
            Self::Military,
            Self::Disrupted,
            Self::Intercepted,
            Self::Collapsing,
            Self::Protected,
        ]
    }
}

pub fn logistics_route_filter_label(filter: LogisticsRouteFilter) -> &'static str {
    match filter {
        LogisticsRouteFilter::All => "All",
        LogisticsRouteFilter::Trade => "Trade",
        LogisticsRouteFilter::Freight => "Freight",
        LogisticsRouteFilter::Military => "Military",
        LogisticsRouteFilter::Disrupted => "Disrupted",
        LogisticsRouteFilter::Intercepted => "Intercepted",
        LogisticsRouteFilter::Collapsing => "Collapsing",
        LogisticsRouteFilter::Protected => "Protected",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogisticsRouteSort {
    Severity,
    Name,
    Integrity,
    Kind,
}

impl LogisticsRouteSort {
    pub fn all() -> [Self; 4] {
        [Self::Severity, Self::Name, Self::Integrity, Self::Kind]
    }
}

pub fn logistics_route_sort_label(sort: LogisticsRouteSort) -> &'static str {
    match sort {
        LogisticsRouteSort::Severity => "Severity",
        LogisticsRouteSort::Name => "Name",
        LogisticsRouteSort::Integrity => "Integrity",
        LogisticsRouteSort::Kind => "Type",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvoyOverlayStatus {
    None,
    Active,
    Protected,
    Disrupted,
    Intercepted,
    Collapsing,
}

impl ConvoyOverlayStatus {
    fn priority(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Active => 1,
            Self::Protected => 2,
            Self::Disrupted => 3,
            Self::Intercepted => 4,
            Self::Collapsing => 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernorPlanStep {
    pub item: ProductionItem,
    pub priority: i32,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewState {
    pub tech: Tech,
    pub max_steps: usize,
    pub previews: Vec<(usize, Vec<ProductionItem>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TechUnlockImpactState {
    pub tech: Tech,
    pub base_ids: Vec<usize>,
    pub entries: Vec<(String, ProductionItem, Tech)>,
    pub recommendation_count: usize,
    pub summary_text: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FactionSupportSummary {
    pub base_free_support: i32,
    pub facility_free_support: i32,
    pub military_supply_bonus: i32,
    pub total_free_support: i32,
    pub live_units: i32,
    pub supported_units: i32,
    pub unit_upkeep: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopBarDisplayState {
    pub year_text: String,
    pub energy_text: Option<String>,
    pub research_text: Option<String>,
    pub food_text: Option<String>,
    pub toxicity_text: Option<String>,
    pub ai_dependence_text: Option<String>,
    pub dust_fall_warning: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SidebarTab {
    Selection,
    Factions,
    Projects,
    Workshop,
    Logistics,
    Saves,
    Logs,
    Editor,
}

impl SidebarTab {
    pub fn all() -> [Self; 8] {
        [
            Self::Selection,
            Self::Factions,
            Self::Projects,
            Self::Workshop,
            Self::Logistics,
            Self::Saves,
            Self::Logs,
            Self::Editor,
        ]
    }

    pub fn label_text(&self) -> &'static str {
        match self {
            Self::Selection => "Selection",
            Self::Factions => "Factions",
            Self::Projects => "Projects",
            Self::Workshop => "Workshop",
            Self::Logistics => "Logistics",
            Self::Saves => "Saves",
            Self::Logs => "Logs",
            Self::Editor => "Editor",
        }
    }

    pub fn hotkey_text(&self) -> &'static str {
        match self {
            Self::Selection => "1",
            Self::Factions => "2",
            Self::Projects => "P",
            Self::Workshop => "W",
            Self::Logistics => "3",
            Self::Saves => "4",
            Self::Logs => "5",
            Self::Editor => "E",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarDisplayState {
    pub heading_text: String,
    pub tabs: Vec<SidebarTabState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarTabState {
    pub tab: SidebarTab,
    pub label_text: &'static str,
    pub hotkey_text: &'static str,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentResearchDisplayState {
    pub tech: Tech,
    pub research: i32,
    pub cost: i32,
    pub label_text: String,
    pub description_text: String,
    pub preview_heading_text: String,
    pub unlock_lines: Vec<String>,
    pub affected_base_ids: Vec<usize>,
    pub affected_focus_base_id: Option<usize>,
    pub affected_focus_label_text: Option<String>,
    pub affected_entries_heading: &'static str,
    pub affected_entries: Vec<(String, ProductionItem, Tech)>,
    pub affected_summary_text: Option<String>,
    pub preview_section: CurrentResearchPreviewSectionState,
    pub queue_previews: Vec<(usize, Vec<ProductionItem>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentResearchPreviewSectionState {
    pub heading_text: String,
    pub focus_label_text: String,
    pub keep_open_label_text: String,
    pub stage_all_log_label_text: String,
    pub hidden_count_text: Option<String>,
    pub rows: Vec<ProductionPreviewRow>,
    pub hidden_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailableResearchDisplayState {
    pub tech: Tech,
    pub label_text: String,
    pub cost_text: String,
    pub description_text: String,
    pub unlock_lines: Vec<String>,
    pub unlock_impact_text: Option<String>,
    pub affected_focus_base_id: Option<usize>,
    pub preview_status_text: Option<String>,
    pub preview_action_label: Option<String>,
    pub unlock_impact: TechUnlockImpactState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockedResearchDisplayState {
    pub tech: Tech,
    pub label_text: String,
    pub description_text: String,
    pub unlock_lines: Vec<String>,
    pub missing: Vec<Tech>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownResearchDisplayState {
    pub tech: Tech,
    pub label_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchPanelDisplayState {
    pub summary_text: String,
    pub available_heading_text: String,
    pub blocked_heading_text: String,
    pub known_heading_text: String,
    pub available_empty_text: String,
    pub blocked_empty_text: String,
    pub known: Vec<KnownResearchDisplayState>,
    pub available: Vec<AvailableResearchDisplayState>,
    pub blocked: Vec<BlockedResearchDisplayState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionPreviewRow {
    pub base_id: usize,
    pub base_name: String,
    pub items: Vec<ProductionItem>,
    pub row_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewApplyResult {
    pub applied_base_ids: Vec<usize>,
    pub remaining_preview: Option<ResearchUnlockPreviewState>,
    pub focus_base_id: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewSelection {
    pub preview: ResearchUnlockPreviewState,
    pub affected_base_ids: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewStageResult {
    pub selection: Option<ResearchUnlockPreviewSelection>,
    pub staged_count: usize,
    pub focus_base_id: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewStateActionResult {
    pub preview: Option<ResearchUnlockPreviewState>,
    pub staged_count: usize,
    pub focus_base_id: Option<usize>,
    pub affected_base_ids: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseFocusState {
    pub filter: BaseFocusFilter,
    pub count: usize,
    pub count_label: String,
    pub next_focus_base_id: Option<usize>,
    pub action_label_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerOperationsFocusState {
    pub damaged_unit_count: usize,
    pub most_damaged_unit_id: Option<usize>,
    pub next_damaged_unit_id: Option<usize>,
    pub stressed_base_count: usize,
    pub most_unrested_base_id: Option<usize>,
    pub next_stressed_base_id: Option<usize>,
    pub recovering_base_count: usize,
    pub most_recovering_garrison_base_id: Option<usize>,
    pub next_recovering_base_id: Option<usize>,
    pub recovering_garrison_unit_count: usize,
    pub next_recovering_garrison_unit_id: Option<usize>,
    pub current_research_unlock_base_count: usize,
    pub current_research_unlock_focus_base_id: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerOperationsActionType {
    SelectDamagedUnit,
    CycleDamagedUnits,
    JumpStressedBase,
    CycleStressedBases,
    JumpRecoveryBase,
    CycleRecoveryBases,
    ApplyRecoveryBasePlan,
    FallbackAllDamaged,
    ApplyAllRecoveryPlans,
    ApplyFrontierDefensePlans,
    SuggestGovernors,
    RepairConvoys,
    RebuildConvoys,
    AssignEscortPatrols,
    ApplyEconomyPlans,
    FillEmptyQueues,
    JumpResearchUnlock,
    SelectRecoveringGarrison,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerOperationsActionState {
    pub label_text: String,
    pub button_text: String,
    pub available_count: usize,
    pub enabled: bool,
    pub action_type: PlayerOperationsActionType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerOperationsJumpActionState {
    pub filter: BaseFocusFilter,
    pub button_text: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerOperationsDashboardState {
    pub heading_text: String,
    pub advice_lines: Vec<String>,
    pub focus: PlayerOperationsFocusState,
    pub jump_actions: Vec<PlayerOperationsJumpActionState>,
    pub bulk_actions: Vec<PlayerOperationsActionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinnedResearchUnlockPreviewRowState {
    pub base_id: usize,
    pub base_name: String,
    pub items: Vec<ProductionItem>,
    pub row_text: String,
    pub stale_label_text: Option<String>,
    pub focus_label_text: String,
    pub apply_label_text: String,
    pub can_apply: bool,
    pub apply_tooltip: String,
    pub is_current: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinnedResearchUnlockPreviewDisplayState {
    pub tech: Tech,
    pub max_steps: usize,
    pub heading_text: String,
    pub availability_text: Option<String>,
    pub drift_text: Option<String>,
    pub stage_log_label_text: String,
    pub refresh_label_text: String,
    pub clear_label_text: String,
    pub apply_all_label_text: String,
    pub can_apply: bool,
    pub apply_all_enabled: bool,
    pub apply_all_tooltip: String,
    pub waiting_on_current_research: bool,
    pub hidden_count_text: Option<String>,
    pub hidden_count: usize,
    pub drifted_base_ids: Vec<usize>,
    pub rows: Vec<PinnedResearchUnlockPreviewRowState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewRefreshResult {
    pub preview: Option<ResearchUnlockPreviewState>,
    pub focus_base_id: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResearchUnlockPreviewStatus {
    pub total: usize,
    pub drifted: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvoyRouteOpportunityState {
    pub base_a_id: usize,
    pub base_b_id: usize,
    pub kind: crate::ConvoyRouteKind,
    pub button_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseConvoyRouteDisplayRowState {
    pub target_base_id: usize,
    pub target_name: String,
    pub kind: crate::ConvoyRouteKind,
    pub row_text: String,
    pub can_repair: bool,
    pub repair_label_text: String,
    pub remove_label_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvailableConvoyTargetOpportunityState {
    pub target_base_id: usize,
    pub kind: crate::ConvoyRouteKind,
    pub button_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvoyRouteDisplayRowState {
    pub base_a_id: usize,
    pub base_b_id: usize,
    pub kind: crate::ConvoyRouteKind,
    pub row_text: String,
    pub focus_a_label_text: String,
    pub focus_b_label_text: String,
    pub can_repair: bool,
    pub repair_label_text: String,
    pub remove_label_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvoyHubDisplayRowState {
    pub base_id: usize,
    pub row_text: String,
    pub saturation_label_text: Option<String>,
    pub is_saturated: bool,
    pub is_tight: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogisticsPanelActionState {
    pub button_text: String,
    pub available_count: usize,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactionLogisticsPanelState {
    pub alerts_heading_text: String,
    pub routes_heading_text: String,
    pub route_opportunities_heading_text: String,
    pub hubs_heading_text: String,
    pub alert_lines: Vec<String>,
    pub jump_saturated_action: LogisticsPanelActionState,
    pub jump_costly_logistics_action: LogisticsPanelActionState,
    pub jump_collapsing_action: LogisticsPanelActionState,
    pub jump_worst_route_action: LogisticsPanelActionState,
    pub repair_filtered_action: LogisticsPanelActionState,
    pub remove_collapsing_action: LogisticsPanelActionState,
    pub filtered_count_text: String,
    pub route_rows: Vec<ConvoyRouteDisplayRowState>,
    pub route_opportunities: Vec<ConvoyRouteOpportunityState>,
    pub hub_rows: Vec<ConvoyHubDisplayRowState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiplomacyRelationDisplayRowState {
    pub faction_id: usize,
    pub faction_name: String,
    pub leader_name: String,
    pub status_text: String,
    pub attitude_text: String,
    pub color_hex: String,
    pub status_color_hex: String,
    pub can_sign_treaty: bool,
    pub can_sign_pact: bool,
    pub can_declare_war: bool,
    pub can_offer_truce: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiplomacyPanelDisplayState {
    pub heading_text: String,
    pub relations_heading_text: String,
    pub relations: Vec<DiplomacyRelationDisplayRowState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialEngineeringCategoryOptionState {
    pub choice_text: String,
    pub modifiers_text: String,
    pub enabled: bool,
    pub selected: bool,
    pub politics: Option<Politics>,
    pub economics: Option<Economics>,
    pub values: Option<Values>,
    pub future: Option<FutureSociety>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialEngineeringCategoryState {
    pub name: &'static str,
    pub options: Vec<SocialEngineeringCategoryOptionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialEngineeringDisplayState {
    pub heading_text: String,
    pub categories: Vec<SocialEngineeringCategoryState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretProjectRegistryRowState {
    pub project_name: String,
    pub owner_name: String,
    pub owner_color_hex: String,
    pub status_text: String,
    pub effects_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretProjectRegistryDisplayState {
    pub heading_text: String,
    pub projects: Vec<SecretProjectRegistryRowState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactionOverviewDisplayState {
    pub faction_id: usize,
    pub is_player_owned: bool,
    pub color_hex: String,
    pub name: String,
    pub leader_text: Option<String>,
    pub description_text: Option<String>,
    pub base_count_text: String,
    pub unit_count_text: String,
    pub energy_text: String,
    pub upkeep_text: String,
    pub research_progress_text: String,
    pub current_tech_text: String,
    pub techs_discovered_text: String,
    pub indices_heading_text: &'static str,
    pub food_security_text: String,
    pub ai_dependence_text: String,
    pub orbital_index_text: String,
    pub planet_toxicity_text: String,
    pub alerts_text: String,
    pub base_roles_text: String,
    pub logistics_summary_text: String,
    pub governor_summary_heading_text: &'static str,
    pub governor_mode_mix_summary: String,
    pub production_posture_text: String,
    pub production_roles_text: String,
    pub queue_posture_text: String,
    pub queue_roles_text: String,
    pub governor_intent_text: String,
    pub governor_queue_intent_text: String,
    pub queue_gaps_text: String,
    pub tech_blocked_intent_text: String,
    pub secret_projects_heading_text: &'static str,
    pub secret_projects_text: Vec<String>,
    pub jump_queue_gap_label_text: Option<&'static str>,
    pub jump_tech_block_label_text: Option<&'static str>,
    pub queue_gap_base_ids: Vec<usize>,
    pub tech_blocked_base_ids: Vec<usize>,
    pub governor_warnings_heading_text: &'static str,
    pub governor_warnings: Vec<String>,
    pub logistics_panel: FactionLogisticsPanelState,
    pub diplomacy_panel: DiplomacyPanelDisplayState,
    pub social_engineering_panel: SocialEngineeringDisplayState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvoyOverlayLine {
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
    pub status: ConvoyOverlayStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapOverlayOptionState {
    pub overlay: presentation::MapOverlay,
    pub label_text: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MinimapDisplayState {
    pub width: usize,
    pub height: usize,
    pub aspect_ratio: f32,
    pub heading_text: String,
    pub tile_min_size: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapPanelDisplayState {
    pub heading_text: String,
    pub minimap_heading_text: &'static str,
    pub overlay_label_text: &'static str,
    pub selected_overlay_label_text: &'static str,
    pub overlay_legend_text: &'static str,
    pub uses_convoy_lines: bool,
    pub overlay_options: Vec<MapOverlayOptionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapTileDisplayState {
    pub label_text: String,
    pub color_hex: String,
    pub status_glyph: Option<&'static str>,
    pub status_glyph_color_hex: Option<&'static str>,
    pub is_selected: bool,
    pub selection_stroke_color_hex: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogisticsBoardDisplayState {
    pub heading_text: String,
    pub gameplay_loop_heading_text: &'static str,
    pub gameplay_loop_steps: Vec<&'static str>,
    pub overview_text: &'static str,
    pub active_routes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandConsoleDisplayState {
    pub heading_text: String,
    pub gameplay_loop_heading_text: &'static str,
    pub gameplay_loop_steps: Vec<&'static str>,
    pub event_log_heading_text: String,
    pub event_log: Vec<(String, Option<&'static str>)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImprovementActionState {
    pub improvement: Improvement,
    pub button_text: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitSelectionDisplayState {
    None {
        message_text: &'static str,
    },
    Missing {
        message_text: &'static str,
    },
    Selected {
        unit_id: usize,
        owner: usize,
        kind: UnitKind,
        label_text: String,
        owner_text: String,
        rank_text: String,
        rank_color_hex: &'static str,
        role_text: String,
        location_text: String,
        moves_text: String,
        moves_color_hex: &'static str,
        hp_text: String,
        hp_color_hex: &'static str,
        advice_text: Option<String>,
        advice_color_hex: &'static str,
        fallback_text: Option<String>,
        fallback_target: Option<(usize, usize)>,
        fallback_button_text: &'static str,
        found_base_label_text: Option<&'static str>,
        terraform_heading_text: &'static str,
        terraform_actions: Vec<ImprovementActionState>,
        upgrade_options: Vec<UnitDesign>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TileSelectionDisplayState {
    None {
        message_text: &'static str,
    },
    Unexplored {
        coordinates_text: String,
        message_text: &'static str,
    },
    Selected {
        coordinates_text: String,
        terrain_text: String,
        elevation_text: String,
        moisture_text: String,
        yield_text: String,
        improvement_text: Option<String>,
        warning_text: Option<&'static str>,
        base_id: Option<usize>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionPanelDisplayState {
    pub heading_text: String,
    pub unit: UnitSelectionDisplayState,
    pub tile: TileSelectionDisplayState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseStatusTagKind {
    Warning,
    Danger,
    Frontier,
    Psi,
    Saturated,
    Tight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseStatusTagState {
    pub label_text: &'static str,
    pub kind: BaseStatusTagKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseGovernorPlanRowState {
    pub item: ProductionItem,
    pub reason_text: String,
    pub apply_label_text: String,
    pub can_apply: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseQueueRowState {
    pub index: usize,
    pub index_text: String,
    pub item: ProductionItem,
    pub label_text: String,
    pub tooltip_text: String,
    pub governor_reason_text: Option<String>,
    pub activate_label_text: &'static str,
    pub move_up_label_text: &'static str,
    pub move_down_label_text: &'static str,
    pub remove_label_text: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseProductionOptionState {
    pub item: ProductionItem,
    pub button_text: String,
    pub tooltip_text: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasePanelDisplayState {
    pub base_id: usize,
    pub owner: usize,
    pub can_manage: bool,
    pub heading_text: String,
    pub owner_text: String,
    pub population_text: String,
    pub governor_text: String,
    pub current_governor_mode: GovernorMode,
    pub current_governor_description: &'static str,
    pub governor_mode_options: Vec<(GovernorMode, &'static str)>,
    pub area_role_text: String,
    pub stability_text: String,
    pub storage_text: String,
    pub output_text: String,
    pub effective_output_text: String,
    pub waste_text: Option<String>,
    pub expansion_limit_text: Option<String>,
    pub defense_pressure_text: String,
    pub psi_pressure_text: String,
    pub damaged_garrisons_text: String,
    pub status_tags_heading_text: &'static str,
    pub status_tags: Vec<BaseStatusTagState>,
    pub production_text: String,
    pub production_role_text: String,
    pub production_dependency_text: String,
    pub production_tooltip_text: String,
    pub governor_alignment_text: Option<String>,
    pub queue_text: String,
    pub facilities_text: String,
    pub build_availability_text: String,
    pub research_focus_heading_text: Option<&'static str>,
    pub research_focus_text: Option<String>,
    pub research_unlock_lines: Vec<String>,
    pub convoy_capacity_text: String,
    pub convoy_status_tags: Vec<BaseStatusTagState>,
    pub active_convoy_links_text: String,
    pub military_supply_links_text: String,
    pub convoy_routes_heading_text: &'static str,
    pub convoy_routes_empty_text: Option<&'static str>,
    pub convoy_routes: Vec<BaseConvoyRouteDisplayRowState>,
    pub available_convoy_targets_heading_text: Option<&'static str>,
    pub available_convoy_targets: Vec<AvailableConvoyTargetOpportunityState>,
    pub governor_heading_text: &'static str,
    pub governor_plan_heading_text: Option<&'static str>,
    pub governor_plan_rows: Vec<BaseGovernorPlanRowState>,
    pub queue_governor_plan_label_text: Option<&'static str>,
    pub apply_recovery_plan_label_text: Option<&'static str>,
    pub apply_defense_plan_label_text: Option<&'static str>,
    pub queue_editor_heading_text: Option<&'static str>,
    pub queue_rows: Vec<BaseQueueRowState>,
    pub clear_queue_label_text: Option<&'static str>,
    pub set_production_heading_text: Option<&'static str>,
    pub set_production_options: Vec<BaseProductionOptionState>,
    pub locked_production_heading_text: Option<String>,
    pub locked_production_options: Vec<BaseProductionOptionState>,
    pub queue_item_heading_text: Option<&'static str>,
    pub queue_item_options: Vec<BaseProductionOptionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapInteractionResult {
    None,
    SelectUnit(usize),
    MoveUnit {
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameOverDisplayState {
    pub message_text: &'static str,
    pub color_hex: &'static str,
}

impl GameState {
    pub fn handle_sidebar_hotkey(&self, key_text: &str) -> Option<SidebarTab> {
        match key_text {
            "1" => Some(SidebarTab::Selection),
            "2" => Some(SidebarTab::Factions),
            "W" => Some(SidebarTab::Workshop),
            "3" => Some(SidebarTab::Logistics),
            "4" => Some(SidebarTab::Saves),
            "5" => Some(SidebarTab::Logs),
            _ => None,
        }
    }

    pub fn game_over_display_state(&self) -> Option<GameOverDisplayState> {
        self.game_over.map(|result| match result {
            GameOver::PlayerWonConquest
            | GameOver::PlayerWonEconomic
            | GameOver::PlayerWonTranscendence
            | GameOver::PlayerWonSpaceTranscendence
            | GameOver::PlayerWonBlackHoleHarvesting => GameOverDisplayState {
                message_text: "VICTORY: You have won the game!",
                color_hex: presentation::ui_success_hex(),
            },
            GameOver::AiWonConquest
            | GameOver::AiWonEconomic
            | GameOver::AiWonTranscendence
            | GameOver::AiWonSpaceTranscendence
            | GameOver::AiWonBlackHoleHarvesting
            | GameOver::DiplomaticVictory
            | GameOver::PlanetUnited => GameOverDisplayState {
                message_text: "DEFEAT: A rival faction has won the game.",
                color_hex: presentation::ui_danger_hex(),
            },
            GameOver::CouncilGovernorElected => GameOverDisplayState {
                message_text: "MILESTONE: A Planetary Governor has been elected.",
                color_hex: "#AAAAFF",
            },
            GameOver::PlayerLost => GameOverDisplayState {
                message_text: presentation::ui_defeat_text(),
                color_hex: presentation::ui_danger_hex(),
            },
        })
    }

    pub fn process_map_interaction(
        &self,
        x: usize,
        y: usize,
        selected_unit_id: Option<usize>,
        owner: usize,
    ) -> MapInteractionResult {
        let Some(tile) = self.tile(x, y) else {
            return MapInteractionResult::None;
        };

        if !self.tile_explored_by_owner(x, y, owner) {
            return MapInteractionResult::Error(
                "Unexplored sector. Move a unit closer.".to_string(),
            );
        }

        if let Some(unit_id) = tile.unit {
            if let Some(unit) = self.unit(unit_id) {
                if unit.owner == owner {
                    return MapInteractionResult::SelectUnit(unit_id);
                }
            }
        }

        if let Some(unit_id) = selected_unit_id {
            return MapInteractionResult::MoveUnit {
                unit_id,
                target_x: x,
                target_y: y,
            };
        }

        MapInteractionResult::None
    }
    pub fn minimap_display_state(&self) -> MinimapDisplayState {
        MinimapDisplayState {
            width: self.width,
            height: self.height,
            aspect_ratio: self.width as f32 / self.height as f32,
            heading_text: presentation::ui_minimap_heading().to_string(),
            tile_min_size: 6.0,
        }
    }

    pub fn sidebar_display_state(&self, active_tab: SidebarTab) -> SidebarDisplayState {
        SidebarDisplayState {
            heading_text: "Operations".to_string(),
            tabs: SidebarTab::all()
                .into_iter()
                .map(|tab| SidebarTabState {
                    tab,
                    label_text: tab.label_text(),
                    hotkey_text: tab.hotkey_text(),
                    selected: tab == active_tab,
                })
                .collect(),
        }
    }

    pub fn top_bar_display_state(&self, owner: usize) -> TopBarDisplayState {
        let faction = self.faction(owner);
        TopBarDisplayState {
            year_text: format!("Year: {}", 2100 + self.turn),
            energy_text: faction.map(|f| format!("Energy: {}", f.energy)),
            research_text: faction.map(|f| {
                format!(
                    "Research: {}/{}",
                    f.research,
                    crate::content_api::tech_cost(f.current_research)
                )
            }),
            food_text: faction.map(|f| format!("Food: {}%", f.food_security)),
            toxicity_text: faction.map(|f| format!("Toxicity: {}", f.planet_toxicity)),
            ai_dependence_text: faction.map(|f| format!("AI: {}%", f.ai_dependence)),
            dust_fall_warning: if self.dust_fall_turns_left > 0 {
                Some(format!(
                    "⚠ DUST FALL: {}y remaining",
                    self.dust_fall_turns_left
                ))
            } else {
                None
            },
        }
    }

    fn convoy_overlay_status_for_flags(
        disrupted: bool,
        intercepted: bool,
        integrity: u8,
        protected: bool,
    ) -> ConvoyOverlayStatus {
        if intercepted && integrity <= 1 {
            ConvoyOverlayStatus::Collapsing
        } else if intercepted {
            ConvoyOverlayStatus::Intercepted
        } else if disrupted {
            ConvoyOverlayStatus::Disrupted
        } else if protected {
            ConvoyOverlayStatus::Protected
        } else {
            ConvoyOverlayStatus::Active
        }
    }

    fn convoy_route_status_label(
        disrupted: bool,
        intercepted: bool,
        integrity: u8,
        protected: bool,
    ) -> &'static str {
        match Self::convoy_overlay_status_for_flags(disrupted, intercepted, integrity, protected) {
            ConvoyOverlayStatus::Collapsing => "COLLAPSING",
            ConvoyOverlayStatus::Intercepted => "INTERCEPTED",
            ConvoyOverlayStatus::Disrupted => "DISRUPTED",
            ConvoyOverlayStatus::Protected => "Protected",
            ConvoyOverlayStatus::Active => "Active",
            ConvoyOverlayStatus::None => "Active",
        }
    }

    fn tile_on_convoy_path(x: usize, y: usize, ax: usize, ay: usize, bx: usize, by: usize) -> bool {
        let on_horizontal = y == ay && ((x >= ax && x <= bx) || (x >= bx && x <= ax));
        let on_vertical = x == bx && ((y >= ay && y <= by) || (y >= by && y <= ay));
        on_horizontal || on_vertical
    }

    fn logistics_panel_action_state(
        label: &str,
        available_count: usize,
    ) -> LogisticsPanelActionState {
        LogisticsPanelActionState {
            button_text: label.to_string(),
            available_count,
            enabled: available_count > 0,
        }
    }

    fn convoy_route_matches_filter(
        route: &crate::ConvoyRouteSummary,
        filter: LogisticsRouteFilter,
    ) -> bool {
        match filter {
            LogisticsRouteFilter::All => true,
            LogisticsRouteFilter::Trade => route.kind == crate::ConvoyRouteKind::Trade,
            LogisticsRouteFilter::Freight => route.kind == crate::ConvoyRouteKind::Freight,
            LogisticsRouteFilter::Military => route.kind == crate::ConvoyRouteKind::MilitarySupply,
            LogisticsRouteFilter::Disrupted => route.disrupted,
            LogisticsRouteFilter::Intercepted => route.intercepted,
            LogisticsRouteFilter::Collapsing => route.intercepted && route.integrity <= 1,
            LogisticsRouteFilter::Protected => {
                route.protected && !route.intercepted && !route.disrupted
            }
        }
    }

    fn convoy_route_severity_score(route: &crate::ConvoyRouteSummary) -> i32 {
        let mut score = 0;
        if route.protected {
            score += 1;
        }
        if route.disrupted {
            score += 3;
        }
        if route.intercepted {
            score += 5;
        }
        if route.integrity <= 1 {
            score += 4;
        }
        score * 10 + (3 - route.integrity as i32)
    }

    fn player_operations_action_state(
        label: &str,
        available_count: usize,
        action_type: PlayerOperationsActionType,
    ) -> PlayerOperationsActionState {
        PlayerOperationsActionState {
            label_text: label.to_string(),
            button_text: if available_count > 0 {
                format!("{label} ({available_count})")
            } else {
                label.to_string()
            },
            available_count,
            enabled: available_count > 0,
            action_type,
        }
    }

    fn base_focus_jump_action_state(
        &self,
        owner: usize,
        filter: BaseFocusFilter,
        label: &str,
        current_base_id: Option<usize>,
    ) -> PlayerOperationsJumpActionState {
        let state = self.base_focus_state(owner, filter, current_base_id);
        PlayerOperationsJumpActionState {
            filter,
            button_text: if state.count > 0 {
                format!("{label} ({})", state.count)
            } else {
                label.to_string()
            },
            enabled: state.count > 0,
        }
    }

    pub fn runtime_roles(&self) -> content::RuntimeRoles {
        content::runtime_roles()
    }

    pub fn owner_for_role(&self, role: content::RuntimeRole) -> usize {
        content::owner_for_role(role)
    }

    pub fn player_owner(&self) -> usize {
        self.runtime_roles().player
    }

    pub fn ai_owner(&self) -> usize {
        self.runtime_roles().ai
    }

    pub fn native_owner(&self) -> usize {
        self.runtime_roles().native
    }

    pub fn faction_for_role(&self, role: content::RuntimeRole) -> Option<&crate::Faction> {
        self.factions.get(self.owner_for_role(role))
    }

    pub fn faction_for_role_mut(
        &mut self,
        role: content::RuntimeRole,
    ) -> Option<&mut crate::Faction> {
        let owner = self.owner_for_role(role);
        self.factions.get_mut(owner)
    }

    pub fn tile_visible_to_owner(&self, x: usize, y: usize, owner: usize) -> bool {
        self.tile(x, y)
            .map(|tile| tile.visible_by_owner.contains(&owner))
            .unwrap_or(false)
    }

    pub fn tile_explored_by_owner(&self, x: usize, y: usize, owner: usize) -> bool {
        self.tile(x, y)
            .map(|tile| tile.explored_by_owner.contains(&owner))
            .unwrap_or(false)
    }

    pub fn control_owner_at(&self, x: usize, y: usize) -> Option<usize> {
        self.bases
            .iter()
            .min_by_key(|base| (base.x as i32 - x as i32).abs() + (base.y as i32 - y as i32).abs())
            .map(|base| base.owner)
    }

    pub fn has_secret_project(&self, owner: usize, project: SecretProject) -> bool {
        self.built_secret_projects
            .iter()
            .any(|(p, o)| *p == project && *o == owner)
    }

    pub fn is_other_faction_building_secret_project(
        &self,
        faction_id: usize,
        project: SecretProject,
    ) -> bool {
        self.bases
            .iter()
            .any(|b| b.owner != faction_id && b.production.secret_project() == Some(project))
    }

    pub fn is_border_tile(&self, x: usize, y: usize) -> bool {
        let owner = self.control_owner_at(x, y);
        let neighbors = [
            (x.saturating_sub(1), y),
            (x.saturating_add(1), y),
            (x, y.saturating_sub(1)),
            (x, y.saturating_add(1)),
        ];
        neighbors.into_iter().any(|(nx, ny)| {
            nx < self.width && ny < self.height && self.control_owner_at(nx, ny) != owner
        })
    }

    pub fn faction(&self, owner: usize) -> Option<&crate::Faction> {
        self.factions.get(owner)
    }

    pub fn factions(&self) -> &[crate::Faction] {
        &self.factions
    }

    pub fn non_native_factions(&self) -> Vec<&crate::Faction> {
        self.factions
            .iter()
            .filter(|faction| faction.id != self.native_owner())
            .collect()
    }

    pub fn faction_mut(&mut self, owner: usize) -> Option<&mut crate::Faction> {
        self.factions.get_mut(owner)
    }

    pub fn faction_name(&self, owner: usize) -> &str {
        self.faction(owner)
            .map(|faction| faction.name.as_str())
            .unwrap_or("Faction")
    }

    pub fn player_faction(&self) -> Option<&crate::Faction> {
        self.faction(self.player_owner())
    }

    pub fn research_progress(&self, owner: usize) -> Option<(Tech, i32, i32)> {
        self.faction(owner).map(|faction| {
            let tech = faction.current_research;
            (tech, faction.research, content::tech_cost(tech))
        })
    }

    pub fn new_game(width: usize, height: usize, seed: u32) -> Self {
        let tiles = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| Tile {
                    x,
                    y,
                    terrain: Terrain::Flat,
                    elevation: 0,
                    moisture: 0,
                    pod: false,
                    unit: None,
                    base: None,
                    improvement: None,
                    explored_by_owner: BTreeSet::new(),
                    visible_by_owner: BTreeSet::new(),
                })
            })
            .collect();

        let factions = content::build_runtime_factions();
        let faction_count = factions.len();

        let mut state = GameState {
            width,
            height,
            seed,
            turn: 1,
            dust_fall_turns_left: 0,
            tidal_chaos_turns_left: 0,
            tiles,
            units: Vec::new(),
            bases: Vec::new(),
            convoy_routes: Vec::new(),
            relations: vec![vec![DiplomaticRelation::default(); faction_count]; faction_count],
            factions,
            built_secret_projects: Vec::new(),
            log: Vec::new(),
            pending_diplomacy_offers: Vec::new(),
            pending_tech_trades: Vec::new(),
            pending_demands: Vec::new(),
            triggered_narratives: BTreeSet::new(),
            council: crate::model::CouncilState::default(),
            game_over: None,
            command_center_turn_traces: Vec::new(),
        };

        // Initialize self-relations as Pact (or some special state)
        for i in 0..faction_count {
            state.relations[i][i].status = DiplomacyStatus::Pact;
            state.relations[i][i].attitude = 100;
        }

        state.generate_map();
        state.apply_starting_scenario(content::default_starting_scenario(width, height));

        state.update_player_visibility();

        state.push_log("MISSION YEAR 2101: Planetfall confirmed.".to_string());
        state.push_log(
            "Found your first base, scout pods, choose research, and tame Planet.".to_string(),
        );

        state
    }

    fn apply_starting_scenario(&mut self, scenario: content::StartingScenario) {
        for position in scenario.forced_land_positions {
            self.force_land_patch(position.x, position.y);
        }

        for unit in scenario.starting_units {
            self.spawn_unit(unit.owner, unit.kind, unit.x, unit.y);
        }
    }

    pub fn apply_action(&mut self, action: GameAction) -> Result<(), String> {
        match action {
            GameAction::MoveUnit {
                unit_id,
                target_x,
                target_y,
            } => self.move_unit_to(unit_id, target_x, target_y),
            GameAction::FoundBase { unit_id } => self.found_base(unit_id),
            GameAction::BuildImprovement {
                unit_id,
                improvement,
            } => self.build_improvement(unit_id, improvement),
            GameAction::SetBaseProduction { base_id, item } => {
                self.set_base_production(base_id, item)
            }
            GameAction::QueueBaseProduction { base_id, item } => {
                self.queue_base_production(base_id, item)
            }
            GameAction::ChooseResearch { owner, tech } => {
                self.choose_research(owner, tech);
                Ok(())
            }
            GameAction::DesignUnit { owner, design } => {
                self.add_unit_design(owner, design);
                Ok(())
            }
            GameAction::UpdateDiplomacy {
                faction_a,
                faction_b,
                status,
            } => self.update_diplomacy(faction_a, faction_b, status),
            GameAction::ProposeDiplomacy {
                proposer,
                receiver,
                status,
            } => {
                self.pending_diplomacy_offers
                    .push((proposer, receiver, status));
                Ok(())
            }
            GameAction::RespondDiplomacy {
                proposer,
                receiver,
                status,
                accept,
            } => {
                self.pending_diplomacy_offers
                    .retain(|&(p, r, s)| !(p == proposer && r == receiver && s == status));
                if accept {
                    self.update_diplomacy(proposer, receiver, status)
                } else {
                    Ok(())
                }
            }
            GameAction::ProposeTechTrade {
                proposer,
                receiver,
                offered_tech,
                requested_tech,
            } => {
                self.pending_tech_trades
                    .push((proposer, receiver, offered_tech, requested_tech));
                Ok(())
            }
            GameAction::RespondTechTrade {
                proposer,
                receiver,
                offered_tech,
                requested_tech,
                accept,
            } => {
                self.pending_tech_trades.retain(|&(p, r, ot, rt)| {
                    !(p == proposer && r == receiver && ot == offered_tech && rt == requested_tech)
                });
                if accept {
                    self.execute_tech_trade(proposer, receiver, offered_tech, requested_tech)
                } else {
                    Ok(())
                }
            }
            GameAction::MakeDemand {
                proposer,
                receiver,
                demand,
            } => {
                self.pending_demands.push((proposer, receiver, demand));
                Ok(())
            }
            GameAction::RespondDemand {
                proposer,
                receiver,
                demand,
                accept,
            } => {
                self.pending_demands
                    .retain(|&(p, r, ref d)| !(p == proposer && r == receiver && d == &demand));
                if accept {
                    self.execute_demand(proposer, receiver, demand)
                } else {
                    self.update_diplomacy(proposer, receiver, DiplomacyStatus::War)
                }
            }
            GameAction::LoadUnit {
                unit_id,
                transport_id,
            } => self.load_unit(unit_id, transport_id),
            GameAction::UnloadUnit {
                unit_id,
                transport_id,
                target_x,
                target_y,
            } => self.unload_unit(unit_id, transport_id, target_x, target_y),
            GameAction::DisbandUnit { unit_id } => {
                self.destroy_unit(unit_id);
                Ok(())
            }
            GameAction::SetUnitActivity { unit_id, activity } => {
                self.set_unit_activity(unit_id, activity);
                Ok(())
            }
            GameAction::RushBuild { base_id } => self.rush_build(base_id),
            GameAction::UpgradeUnit {
                unit_id,
                new_design,
            } => self.upgrade_unit(unit_id, new_design),
            GameAction::ChooseSocialEngineering {
                owner,
                politics,
                economics,
                values,
                future,
            } => self.choose_social_engineering(owner, politics, economics, values, future),
            GameAction::PerformProbeAction {
                unit_id,
                target_x,
                target_y,
                action,
            } => self.perform_probe_action(unit_id, target_x, target_y, action),
            GameAction::CallCouncil => self.call_council(),
            GameAction::VoteForGovernor {
                voter_id,
                candidate_id,
            } => self.vote_for_governor(voter_id, candidate_id),
            GameAction::VoteForSupremeLeader {
                voter_id,
                candidate_id,
            } => self.vote_for_supreme_leader(voter_id, candidate_id),
            GameAction::EndTurn => {
                self.end_turn();
                Ok(())
            }
        }
    }

    pub fn push_log(&mut self, message: String) {
        self.push_event_log(EventCategory::General, message);
    }

    pub fn push_event_log(&mut self, category: EventCategory, message: String) {
        self.log.push(EventLogEntry {
            category,
            message,
            turn: self.turn,
        });
        if self.log.len() > 10000 {
            self.log.remove(0);
        }
    }

    pub fn command_center_turn_traces_for_owner(&self, owner: usize) -> Vec<CommandCenterTurnTrace> {
        self.command_center_turn_traces
            .iter()
            .filter(|trace| trace.turn == self.turn && trace.owner == owner)
            .cloned()
            .collect()
    }

    fn clear_command_center_turn_traces_for_owner(&mut self, owner: usize) {
        self.command_center_turn_traces
            .retain(|trace| !(trace.turn == self.turn && trace.owner == owner));
    }

    fn capture_command_center_post_production_traces(&mut self, owner: usize) {
        self.clear_command_center_turn_traces_for_owner(owner);
        let snapshots: Vec<(usize, String, i32)> = self
            .bases_for(owner)
            .into_iter()
            .filter(|base| {
                base.production == ProductionItem::CommandCenter
                    && !base.facilities.contains(&Facility::CommandCenter)
            })
            .map(|base| (base.id, base.name.clone(), base.minerals_stock))
            .collect();
        for (base_id, base_name, minerals_stock) in snapshots {
            self.command_center_turn_traces.push(CommandCenterTurnTrace {
                turn: self.turn,
                owner,
                base_id,
                base_name,
                post_production_stock: minerals_stock,
                post_interdiction_stock: minerals_stock,
                upkeep_drain: 0,
                upkeep_order_index: None,
                end_stock: minerals_stock,
            });
        }
    }

    fn update_command_center_post_interdiction_traces(&mut self, owner: usize) {
        let snapshots: Vec<(usize, i32)> = self
            .bases_for(owner)
            .into_iter()
            .map(|base| (base.id, base.minerals_stock))
            .collect();
        for (base_id, minerals_stock) in snapshots {
            if let Some(trace) = self
                .command_center_turn_traces
                .iter_mut()
                .find(|trace| trace.turn == self.turn && trace.owner == owner && trace.base_id == base_id)
            {
                trace.post_interdiction_stock = minerals_stock;
                trace.end_stock = minerals_stock;
            }
        }
    }

    fn update_command_center_end_stock_traces(&mut self, owner: usize) {
        let snapshots: Vec<(usize, i32)> = self
            .bases_for(owner)
            .into_iter()
            .map(|base| (base.id, base.minerals_stock))
            .collect();
        for (base_id, minerals_stock) in snapshots {
            if let Some(trace) = self
                .command_center_turn_traces
                .iter_mut()
                .find(|trace| trace.turn == self.turn && trace.owner == owner && trace.base_id == base_id)
            {
                trace.end_stock = minerals_stock;
            }
        }
    }

    pub fn tile_potential_improvements(&self, x: usize, y: usize) -> Vec<(Improvement, Yields)> {
        let Some(tile) = self.tile(x, y) else {
            return Vec::new();
        };
        let mut potentials = Vec::new();

        if tile.terrain.is_land() {
            let options = [
                Improvement::Farm,
                Improvement::Mine,
                Improvement::Solar,
                Improvement::Forest,
                Improvement::ThermalBorehole,
            ];

            for opt in options {
                let yields = self.calculate_tile_yields_with_improvement(x, y, Some(opt));
                potentials.push((opt, yields));
            }
        }

        potentials
    }

    fn calculate_tile_yields_with_improvement(
        &self,
        x: usize,
        y: usize,
        improvement: Option<Improvement>,
    ) -> Yields {
        let Some(tile) = self.tile(x, y) else {
            return Yields::default();
        };
        let base = tile.terrain.yields();
        let imp_bonus = improvement.map(|i| i.yields()).unwrap_or_default();

        Yields {
            nutrients: base.nutrients + imp_bonus.nutrients,
            minerals: base.minerals + imp_bonus.minerals,
            energy: base.energy + imp_bonus.energy,
        }
    }

    pub fn unit_path_to(
        &self,
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    ) -> Vec<(usize, usize)> {
        let Some(unit) = self.unit(unit_id) else {
            return Vec::new();
        };
        let mut path = Vec::new();
        let mut cx = unit.x;
        let mut cy = unit.y;

        while (cx != target_x || cy != target_y) && path.len() < 50 {
            if cx < target_x {
                cx += 1;
            } else if cx > target_x {
                cx -= 1;
            }

            if cy < target_y {
                cy += 1;
            } else if cy > target_y {
                cy -= 1;
            }

            path.push((cx, cy));
        }

        path
    }

    pub fn tile_move_cost(&self, unit_id: usize, x: usize, y: usize) -> i32 {
        let Some(tile) = self.tile(x, y) else {
            return 99;
        };
        if !tile.terrain.is_land() && !self.unit_can_enter_ocean(unit_id) {
            return 99;
        }
        if tile.terrain.is_land() && !self.unit_can_enter_land(unit_id) {
            return 99;
        }

        match tile.terrain {
            Terrain::Rocky => 2,
            Terrain::Rolling => 1,
            Terrain::Flat => 1,
            Terrain::Ocean => 1,
            Terrain::Fungus => 3,
            Terrain::Crater => 3,
        }
    }

    pub fn tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.get(self.tile_index(x, y))
    }

    pub fn tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = self.tile_index(x, y);
        self.tiles.get_mut(idx)
    }

    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id && u.alive)
    }

    pub fn base(&self, id: usize) -> Option<&Base> {
        self.bases.iter().find(|b| b.id == id)
    }

    pub fn base_mut(&mut self, id: usize) -> Option<&mut Base> {
        self.bases.iter_mut().find(|b| b.id == id)
    }

    pub fn live_units_for(&self, owner: usize) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|u| u.alive && u.owner == owner)
            .collect()
    }

    pub fn bases_for(&self, owner: usize) -> Vec<&Base> {
        self.bases.iter().filter(|b| b.owner == owner).collect()
    }

    pub fn tile_total_yields(&self, x: usize, y: usize) -> Yields {
        let Some(tile) = self.tile(x, y) else {
            return Yields::default();
        };

        // Tidal Chaos: Coastal flooding destroys land yields.
        if self.tidal_chaos_turns_left > 0 && tile.terrain.is_land() && tile.elevation <= 0 {
            return Yields {
                nutrients: 0,
                minerals: 0,
                energy: 0,
            };
        }

        let mut total = tile.terrain.yields();

        if let Some(improvement) = tile.improvement {
            total = total.add(improvement.yields());
        }

        // Condenser Bonus: Double nutrients on tile
        if tile.improvement == Some(Improvement::Condenser) {
            total.nutrients *= 2;
        }

        // Echelon Mirror Bonus: +1 energy to solar collectors from each adjacent mirror
        if tile.improvement == Some(Improvement::Solar) {
            let adjacent_mirrors =
                self.count_adjacent_improvements(x, y, Improvement::EchelonMirror);
            total.energy += adjacent_mirrors as i32;
        }

        total
    }

    pub fn count_adjacent_improvements(&self, x: usize, y: usize, target: Improvement) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0 && nx < self.width as isize && ny >= 0 && ny < self.height as isize {
                    if let Some(tile) = self.tile(nx as usize, ny as usize) {
                        if tile.improvement == Some(target) {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }

    pub fn offensive_threat_at(&self, x: usize, y: usize) -> i32 {
        self.units
            .iter()
            .filter(|unit| {
                unit.alive
                    && unit.owner == self.ai_owner()
                    && (unit.x as i32 - x as i32).abs() + (unit.y as i32 - y as i32).abs() <= 3
            })
            .count() as i32
    }

    pub fn frontier_pressure_at(&self, x: usize, y: usize) -> i32 {
        let mut pressure = 0;
        if self.is_border_tile(x, y) {
            pressure += 1;
        }

        for base in self
            .bases
            .iter()
            .filter(|base| base.owner == self.ai_owner())
        {
            let distance = (base.x as i32 - x as i32).abs() + (base.y as i32 - y as i32).abs();
            if distance <= 3 {
                pressure += 3;
            } else if distance <= 5 {
                pressure += 2;
            } else if distance <= 7 {
                pressure += 1;
            }
        }

        pressure
    }

    pub fn psi_threat_at(&self, owner: usize, x: usize, y: usize) -> i32 {
        self.units
            .iter()
            .filter(|unit| {
                unit.alive
                    && unit.owner != owner
                    && matches!(
                        unit.kind,
                        UnitKind::MindWorm | UnitKind::TranceScout | UnitKind::PsiSentinel
                    )
                    && (unit.x as i32 - x as i32).abs() + (unit.y as i32 - y as i32).abs() <= 6
            })
            .map(|unit| {
                let distance = (unit.x as i32 - x as i32).abs() + (unit.y as i32 - y as i32).abs();
                if distance <= 2 {
                    3
                } else if distance <= 4 {
                    2
                } else {
                    1
                }
            })
            .sum()
    }

    pub fn map_overlay_color_hex(
        &self,
        overlay: presentation::MapOverlay,
        viewer_owner: usize,
        x: usize,
        y: usize,
    ) -> &'static str {
        if !self.tile_explored_by_owner(x, y, viewer_owner) {
            return "#08080a";
        }
        if !self.tile_visible_to_owner(x, y, viewer_owner) {
            return "#232323";
        }

        match overlay {
            presentation::MapOverlay::Terrain => match self.tile(x, y).map(|tile| tile.terrain) {
                Some(Terrain::Ocean) => "#1c4878",
                Some(Terrain::Flat) => "#2e783e",
                Some(Terrain::Rolling) => "#5a8038",
                Some(Terrain::Rocky) => "#696056",
                Some(Terrain::Fungus) => "#883680",
                Some(Terrain::Crater) => "#222222",
                None => "#386048",
            },
            presentation::MapOverlay::Yields => {
                let yields = self.tile_total_yields(x, y);
                if yields.energy >= yields.nutrients && yields.energy >= yields.minerals {
                    "#d2b840"
                } else if yields.minerals >= yields.nutrients {
                    "#886e52"
                } else {
                    "#489452"
                }
            }
            presentation::MapOverlay::Ownership => {
                if let Some(base_id) = self.tile(x, y).and_then(|tile| tile.base) {
                    if let Some(base) = self.base(base_id) {
                        if base.owner == viewer_owner {
                            return "#469650";
                        }
                        if base.owner == self.ai_owner() {
                            return "#aa463c";
                        }
                    }
                }
                if let Some(unit_id) = self.tile(x, y).and_then(|tile| tile.unit) {
                    if let Some(unit) = self.unit(unit_id) {
                        if unit.owner == viewer_owner {
                            return "#60b068";
                        }
                        if unit.owner == self.ai_owner() {
                            return "#ba5248";
                        }
                    }
                }
                "#46464c"
            }
            presentation::MapOverlay::Borders => {
                let owner = self.control_owner_at(x, y);
                let on_border = self.is_border_tile(x, y);
                match owner {
                    Some(owner) if owner == viewer_owner && on_border => "#619e72",
                    Some(owner) if owner == viewer_owner => "#3e8448",
                    Some(owner) if owner == self.ai_owner() && on_border => "#c36256",
                    Some(owner) if owner == self.ai_owner() => "#9c4840",
                    Some(_) if on_border => "#a06ab0",
                    Some(_) => "#844888",
                    None if on_border => "#7d7d88",
                    None => "#3a3a40",
                }
            }
            presentation::MapOverlay::Threat => match self.offensive_threat_at(x, y) {
                pressure if pressure >= 3 => "#aa3030",
                2 => "#ba6c34",
                1 => "#a29238",
                _ => "#347046",
            },
            presentation::MapOverlay::FrontierPressure => match self.frontier_pressure_at(x, y) {
                pressure if pressure >= 4 => "#c2762c",
                pressure if pressure >= 2 => "#a8863e",
                1 => "#767c4c",
                _ => "#386048",
            },
            presentation::MapOverlay::PsiThreat => match self.psi_threat_at(viewer_owner, x, y) {
                pressure if pressure >= 3 => "#a238aa",
                2 => "#8a48b0",
                1 => "#7462aa",
                _ => "#386048",
            },
            presentation::MapOverlay::Logistics => presentation::convoy_overlay_status_color_hex(
                self.convoy_overlay_status_at(viewer_owner, x, y),
            ),
            presentation::MapOverlay::Trade => {
                if self.tile(x, y).and_then(|t| t.base).is_some() {
                    "#00ff00"
                } else {
                    "#386048"
                }
            }
        }
    }

    pub fn map_panel_display_state(
        &self,
        overlay: presentation::MapOverlay,
    ) -> MapPanelDisplayState {
        MapPanelDisplayState {
            heading_text: presentation::ui_planet_heading(),
            minimap_heading_text: presentation::ui_minimap_heading(),
            overlay_label_text: presentation::ui_overlay_label(),
            selected_overlay_label_text: presentation::map_overlay_label(overlay),
            overlay_legend_text: presentation::map_overlay_legend(overlay),
            uses_convoy_lines: presentation::map_overlay_uses_convoy_lines(overlay),
            overlay_options: presentation::MapOverlay::all()
                .into_iter()
                .map(|overlay| MapOverlayOptionState {
                    overlay,
                    label_text: presentation::map_overlay_label(overlay),
                })
                .collect(),
        }
    }

    pub fn map_tile_display_state(
        &self,
        x: usize,
        y: usize,
        viewer_owner: usize,
        selected_tile: Option<(usize, usize)>,
        overlay: presentation::MapOverlay,
    ) -> MapTileDisplayState {
        let (status_glyph, status_glyph_color_hex) = if let Some(tile) = self.tile(x, y) {
            if let Some(base_id) = tile.base {
                if self.base_unrest(base_id) > 0 {
                    (Some("!"), Some("#ff5050"))
                } else if self.base_governor_recommendation(base_id).is_some()
                    && self
                        .base(base_id)
                        .map(|b| b.production_queue.is_empty())
                        .unwrap_or(false)
                {
                    (Some("?"), Some("#ffff64"))
                } else {
                    (None, None)
                }
            } else if let Some(unit_id) = tile.unit {
                if self
                    .unit(unit_id)
                    .map(|u| u.hp < content::unit_base_hp(u.kind.clone()))
                    .unwrap_or(false)
                {
                    (Some("*"), Some("#ff9632"))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let is_selected = selected_tile == Some((x, y));
        MapTileDisplayState {
            label_text: self.tile_map_label_for_overlay(x, y, viewer_owner, overlay),
            color_hex: self
                .map_overlay_color_hex(overlay, viewer_owner, x, y)
                .to_string(),
            status_glyph,
            status_glyph_color_hex,
            is_selected,
            selection_stroke_color_hex: is_selected.then_some("#ffffff"),
        }
    }

    pub fn minimap_tile_display_state(
        &self,
        x: usize,
        y: usize,
        viewer_owner: usize,
        selected_tile: Option<(usize, usize)>,
        overlay: presentation::MapOverlay,
    ) -> MapTileDisplayState {
        let is_selected = selected_tile == Some((x, y));
        MapTileDisplayState {
            label_text: " ".to_string(),
            color_hex: self
                .map_overlay_color_hex(overlay, viewer_owner, x, y)
                .to_string(),
            status_glyph: None,
            status_glyph_color_hex: None,
            is_selected,
            selection_stroke_color_hex: is_selected.then_some("#ffffff"),
        }
    }

    pub fn logistics_board_display_state(&self) -> LogisticsBoardDisplayState {
        let mut active_routes = Vec::new();
        for route in &self.convoy_routes {
            let base_a = &self.bases[route.base_a_id];
            let base_b = &self.bases[route.base_b_id];
            active_routes.push(format!(
                "{} <-> {} ({:?})",
                base_a.name, base_b.name, route.kind
            ));
        }

        LogisticsBoardDisplayState {
            heading_text: "Logistics Board".to_string(),
            gameplay_loop_heading_text: presentation::ui_gameplay_loop_heading(),
            gameplay_loop_steps: presentation::ui_gameplay_loop_steps().into_iter().collect(),
            overview_text: "Detailed list of active convoy routes:",
            active_routes,
        }
    }

    pub fn command_console_display_state(&self) -> CommandConsoleDisplayState {
        CommandConsoleDisplayState {
            heading_text: presentation::ui_command_console_heading(),
            gameplay_loop_heading_text: presentation::ui_gameplay_loop_heading(),
            gameplay_loop_steps: presentation::ui_gameplay_loop_steps().into_iter().collect(),
            event_log_heading_text: presentation::ui_event_log_heading(),
            event_log: self
                .log
                .iter()
                .map(|message| {
                    let color_hex = if message.message.contains("COMBAT:") {
                        Some("#ff6464")
                    } else if message.message.contains("CRISIS EVENT:")
                        || message.message.contains("TOXICITY CRISIS:")
                    {
                        Some("#ff9632")
                    } else if message.message.contains("GLOBAL WONDER:") {
                        Some("#c8c8ff")
                    } else if message.message.contains("GOVERNANCE OVERRIDE:") {
                        Some("#b464ff")
                    } else if message.message.starts_with("---") {
                        Some("#d3d3d3")
                    } else {
                        None
                    };
                    (message.message.clone(), color_hex)
                })
                .collect(),
        }
    }

    pub fn selection_panel_display_state(
        &self,
        selected_unit: Option<usize>,
        selected_tile: Option<(usize, usize)>,
        viewer_owner: usize,
    ) -> SelectionPanelDisplayState {
        let unit = match selected_unit {
            Some(unit_id) => match self.unit(unit_id) {
                Some(unit) => {
                    let summary = self.unit_panel_summary(unit_id);
                    let fallback_target = if unit.owner == viewer_owner {
                        self.safest_player_fallback_tile(unit_id)
                    } else {
                        None
                    };
                    let terraform_actions =
                        if unit.owner == viewer_owner && unit.kind == UnitKind::Former {
                            [
                                Improvement::Farm,
                                Improvement::Mine,
                                Improvement::Solar,
                                Improvement::Road,
                                Improvement::Condenser,
                                Improvement::EchelonMirror,
                                Improvement::Forest,
                                Improvement::ThermalBorehole,
                            ]
                            .into_iter()
                            .map(|improvement| ImprovementActionState {
                                improvement,
                                button_text: presentation::improvement_name(improvement),
                            })
                            .collect()
                        } else {
                            Vec::new()
                        };
                    if let Some(summary) = summary {
                        UnitSelectionDisplayState::Selected {
                            unit_id,
                            owner: unit.owner,
                            kind: unit.kind.clone(),
                            label_text: format!(
                                "Unit: [{}] {}",
                                presentation::unit_role_badge(unit.kind.clone()),
                                summary.unit_name
                            ),
                            owner_text: format!("Owner: {}", summary.owner_name),
                            rank_text: format!("Rank: {}", summary.rank),
                            rank_color_hex: match summary.rank {
                                "Green" => "#d3d3d3",
                                "Disciplined" => "#b4eeb4",
                                "Hardened" => "#90ee90",
                                "Veteran" => "#32cd32",
                                "Commando" => "#00fa9a",
                                "Elite" => "#00ff7f",
                                _ => "#d3d3d3",
                            },
                            role_text: summary.role.to_string(),
                            location_text: format!("Location: {}", summary.location),
                            moves_text: format!("Moves left: {}", summary.moves_left),
                            moves_color_hex: if summary.moves_left > 0 {
                                "#ffffff"
                            } else {
                                presentation::ui_warning_hex()
                            },
                            hp_text: format!("HP: {}", summary.hp),
                            hp_color_hex: if summary.hp
                                < crate::content::unit_base_hp(unit.kind.clone())
                            {
                                presentation::ui_danger_hex()
                            } else {
                                "#ffffff"
                            },
                            advice_text: self.unit_operation_advice(unit_id),
                            advice_color_hex: presentation::ui_warning_hex(),
                            fallback_text: fallback_target
                                .map(|(x, y)| format!("Safest fallback step: {}, {}", x, y)),
                            fallback_target,
                            fallback_button_text: "Fallback",
                            found_base_label_text: (unit.owner == viewer_owner
                                && unit.kind == UnitKind::ColonyPod)
                                .then_some("Found Base Here"),
                            terraform_heading_text: "Terraform:",
                            terraform_actions,
                            upgrade_options: self
                                .faction(unit.owner)
                                .map(|f| f.unit_designs.clone())
                                .unwrap_or_default(),
                        }
                    } else {
                        UnitSelectionDisplayState::Missing {
                            message_text: "Selected unit no longer exists.",
                        }
                    }
                }
                None => UnitSelectionDisplayState::Missing {
                    message_text: "Selected unit no longer exists.",
                },
            },
            None => UnitSelectionDisplayState::None {
                message_text: "No unit selected.",
            },
        };

        let tile = match selected_tile {
            Some((x, y)) => match self.tile(x, y) {
                Some(tile) => {
                    if !self.tile_explored_by_owner(x, y, viewer_owner) {
                        TileSelectionDisplayState::Unexplored {
                            coordinates_text: format!("Tile: {}, {}", tile.x, tile.y),
                            message_text: "Unexplored territory.",
                        }
                    } else if let Some(summary) = self.tile_panel_summary(x, y) {
                        TileSelectionDisplayState::Selected {
                            coordinates_text: format!("Tile: {}", summary.coordinates),
                            terrain_text: format!("Terrain: {}", summary.terrain_name),
                            elevation_text: format!("Elevation: {}", summary.elevation),
                            moisture_text: format!("Moisture: {}", summary.moisture),
                            yield_text: format!("Yield: {}", summary.yield_summary),
                            improvement_text: summary
                                .improvement_name
                                .map(|name| format!("Improvement: {name}")),
                            warning_text: (tile.pod
                                && self.tile_visible_to_owner(x, y, viewer_owner))
                            .then_some(presentation::ui_warning_text()),
                            base_id: tile.base,
                        }
                    } else {
                        TileSelectionDisplayState::None {
                            message_text: "No tile selected.",
                        }
                    }
                }
                None => TileSelectionDisplayState::None {
                    message_text: "No tile selected.",
                },
            },
            None => TileSelectionDisplayState::None {
                message_text: "No tile selected.",
            },
        };

        SelectionPanelDisplayState {
            heading_text: presentation::ui_selection_heading(),
            unit,
            tile,
        }
    }

    fn production_tooltip_for_owner(&self, owner: usize, item: ProductionItem) -> String {
        let available = self.is_production_available(owner, item);
        let missing_tech = content::required_tech_for_production(item).and_then(|tech| {
            self.faction(owner)
                .and_then(|faction| (!faction.known_techs.contains(&tech)).then_some(tech))
        });
        presentation::production_tooltip_summary_with_status(item, available, missing_tech)
    }

    fn production_items_for_owner(
        &self,
        owner: usize,
    ) -> (Vec<ProductionItem>, Vec<(ProductionItem, Option<Tech>)>) {
        let mut available = Vec::new();
        let mut locked = Vec::new();

        // 1. Static Production Items
        for item in ProductionItem::all() {
            if let ProductionItem::CustomUnit(_) = item {
                continue;
            }
            if self.is_production_available(owner, item) {
                available.push(item);
            } else {
                locked.push((
                    item,
                    content::required_tech_for_production(item).and_then(|tech| {
                        self.faction(owner).and_then(|faction| {
                            (!faction.known_techs.contains(&tech)).then_some(tech)
                        })
                    }),
                ));
            }
        }

        // 2. Custom Unit Designs
        if let Some(faction) = self.faction(owner) {
            for index in 0..faction.unit_designs.len() {
                available.push(ProductionItem::CustomUnit(index));
            }
        }

        (available, locked)
    }

    pub fn base_panel_display_state(
        &self,
        base_id: usize,
        viewer_owner: usize,
    ) -> Option<BasePanelDisplayState> {
        let base = self.base(base_id)?.clone();
        let summary = presentation::base_panel_summary(
            &base.name,
            self.faction_name(base.owner),
            base.population,
            base.governor_mode,
            self.base_trade_links(base.id),
            self.base_unrest(base.id),
            base.nutrients_stock,
            base.minerals_stock,
            self.base_yields(base.x, base.y),
            self.effective_base_yields(base.id)
                .unwrap_or_else(|| self.base_yields(base.x, base.y)),
            base.production,
            base.minerals_stock,
            &base.production_queue,
            &base.facilities,
        );
        let can_manage = base.owner == viewer_owner;
        let (available_items, locked_items) = self.production_items_for_owner(base.owner);
        let (convoy_used, convoy_capacity) = self.base_convoy_saturation_ratio(base.id);
        let route_rows = self.base_convoy_route_display_rows(base.id);
        let available_targets = if can_manage {
            self.available_convoy_target_opportunities(base.id)
        } else {
            Vec::new()
        };
        let governor_plan = self.base_governor_plan(base.id);
        let queue_rows = if can_manage {
            base.production_queue
                .iter()
                .copied()
                .enumerate()
                .map(|(index, item)| BaseQueueRowState {
                    index,
                    index_text: format!("{}.", index + 1),
                    item,
                    label_text: format!(
                        "[{}] {}",
                        self.production_role_badge(base.owner, item),
                        self.production_name(base.owner, item)
                    ),
                    tooltip_text: self.production_tooltip_for_owner(base.owner, item),
                    governor_reason_text: self
                        .base_governor_reason_for_item(base.id, item)
                        .map(|(priority, reason)| format!("[P{priority}] {reason}")),
                    activate_label_text: "Active",
                    move_up_label_text: "Up",
                    move_down_label_text: "Down",
                    remove_label_text: "Remove",
                })
                .collect()
        } else {
            Vec::new()
        };

        Some(BasePanelDisplayState {
            base_id: base.id,
            owner: base.owner,
            can_manage,
            heading_text: format!("Base: {}", summary.name),
            owner_text: format!("Owner: {}", summary.owner_name),
            population_text: format!("Population: {}", summary.population),
            governor_text: format!("Governor: {}", summary.governor_mode),
            current_governor_mode: base.governor_mode,
            current_governor_description: presentation::governor_mode_description(
                base.governor_mode,
            ),
            governor_mode_options: GovernorMode::all()
                .into_iter()
                .map(|mode| (mode, presentation::governor_mode_label(mode)))
                .collect(),
            area_role_text: format!(
                "Area role: {}",
                presentation::base_area_role_label(self.base_area_role(base.id))
            ),
            stability_text: format!("Stability: {}", summary.stability),
            storage_text: format!("Stored: {}", summary.storage),
            output_text: format!("Base output: {}", summary.output),
            effective_output_text: format!("Effective output: {}", summary.effective_output),
            waste_text: {
                let waste = self.energy_waste_pct(base.id);
                (waste > 0).then(|| format!("Energy waste: {}%", waste))
            },
            expansion_limit_text: {
                let limit = self.base_expansion_limit(base.owner);
                let count = self.bases_for(base.owner).len() as i32;
                (count > limit).then(|| format!("Bureaucracy: {}/{}", count, limit))
            },
            defense_pressure_text: format!(
                "Defense pressure: {}",
                self.base_local_military_pressure(base.id)
            ),
            psi_pressure_text: format!("Psi pressure: {}", self.base_local_psi_pressure(base.id)),
            damaged_garrisons_text: format!(
                "Damaged garrisons: {}",
                self.damaged_garrison_count_for_base(base.id)
            ),
            status_tags_heading_text: "Status Tags",
            status_tags: {
                let mut tags = Vec::new();
                if self.base_unrest(base.id) > 0 {
                    tags.push(BaseStatusTagState {
                        label_text: "Unrest",
                        kind: BaseStatusTagKind::Warning,
                    });
                }
                if self.damaged_garrison_count_for_base(base.id) > 0 {
                    tags.push(BaseStatusTagState {
                        label_text: "Recovery Load",
                        kind: BaseStatusTagKind::Danger,
                    });
                }
                if self.base_local_military_pressure(base.id) >= 2 {
                    tags.push(BaseStatusTagState {
                        label_text: "Frontier",
                        kind: BaseStatusTagKind::Frontier,
                    });
                }
                if self.base_local_psi_pressure(base.id) >= 2 {
                    tags.push(BaseStatusTagState {
                        label_text: "Psi Threat",
                        kind: BaseStatusTagKind::Psi,
                    });
                }
                tags
            },
            production_text: format!(
                "Producing: {}",
                self.production_name(base.owner, base.production)
            ),
            production_role_text: format!(
                "[{}] {}",
                self.production_role_badge(base.owner, base.production),
                presentation::production_role_category(base.production)
            ),
            production_dependency_text: presentation::production_dependency_text(base.production)
                .to_string(),
            production_tooltip_text: self
                .production_tooltip_for_owner(base.owner, base.production)
                .replace('\n', " | "),
            governor_alignment_text: self
                .base_governor_reason_for_item(base.id, base.production)
                .map(|(priority, reason)| format!("Governor alignment [P{priority}]: {reason}")),
            queue_text: format!("Queue: {}", summary.queue),
            facilities_text: format!("Facilities: {}", summary.facilities),
            build_availability_text: presentation::build_availability_summary(
                available_items.len(),
                locked_items.len(),
            ),
            research_focus_heading_text: self
                .research_progress(base.owner)
                .map(|_| "Research focus"),
            research_focus_text: self.research_progress(base.owner).map(
                |(current_tech, current_research, current_cost)| {
                    format!(
                        "{} ({})",
                        presentation::tech_name(current_tech),
                        presentation::format_research_progress(current_research, current_cost)
                    )
                },
            ),
            research_unlock_lines: self
                .research_progress(base.owner)
                .map(|(current_tech, _, _)| presentation::tech_unlock_lines(current_tech))
                .unwrap_or_default(),
            convoy_capacity_text: format!(
                "Convoy capacity: {}/{}",
                route_rows.len(),
                self.base_convoy_capacity(base.id)
            ),
            convoy_status_tags: {
                let mut tags = Vec::new();
                if convoy_capacity > 0 && convoy_used >= convoy_capacity {
                    tags.push(BaseStatusTagState {
                        label_text: "Convoy Saturated",
                        kind: BaseStatusTagKind::Saturated,
                    });
                } else if convoy_capacity > 0 && convoy_used + 1 >= convoy_capacity {
                    tags.push(BaseStatusTagState {
                        label_text: "Convoy Tight",
                        kind: BaseStatusTagKind::Tight,
                    });
                }
                tags
            },
            active_convoy_links_text: format!("Active convoy links: {}", summary.trade_links),
            military_supply_links_text: format!(
                "Military supply links: {}",
                self.base_military_supply_links(base.id)
            ),
            convoy_routes_heading_text: "Convoy routes:",
            convoy_routes_empty_text: route_rows.is_empty().then_some("Convoy routes: None"),
            convoy_routes: route_rows,
            available_convoy_targets_heading_text: (!available_targets.is_empty())
                .then_some("Available convoy targets:"),
            available_convoy_targets: available_targets,
            governor_heading_text: "Governor",
            governor_plan_heading_text: (!governor_plan.is_empty()).then_some("Governor Plan"),
            governor_plan_rows: governor_plan
                .iter()
                .take(3)
                .map(|step| BaseGovernorPlanRowState {
                    item: step.item,
                    reason_text: format!("[P{}] {}", step.priority, step.reason),
                    apply_label_text: format!(
                        "Apply {}",
                        self.production_name(base.owner, step.item)
                    ),
                    can_apply: can_manage && self.is_production_available(base.owner, step.item),
                })
                .collect(),
            queue_governor_plan_label_text: (can_manage && !governor_plan.is_empty())
                .then_some("Queue Governor Plan"),
            apply_recovery_plan_label_text: (can_manage && !governor_plan.is_empty())
                .then_some("Apply Recovery Plan"),
            apply_defense_plan_label_text: (can_manage && !governor_plan.is_empty())
                .then_some("Apply Defense Plan"),
            queue_editor_heading_text: (can_manage && !base.production_queue.is_empty())
                .then_some("Edit queue:"),
            queue_rows,
            clear_queue_label_text: (can_manage && !base.production_queue.is_empty())
                .then_some("Clear Queue"),
            set_production_heading_text: can_manage.then_some("Set production:"),
            set_production_options: if can_manage {
                available_items
                    .iter()
                    .copied()
                    .map(|item| BaseProductionOptionState {
                        item,
                        button_text: format!(
                            "[{}] {}",
                            self.production_role_badge(base.owner, item),
                            self.production_name(base.owner, item)
                        ),
                        tooltip_text: self.production_tooltip_for_owner(base.owner, item),
                        enabled: true,
                    })
                    .collect()
            } else {
                Vec::new()
            },
            locked_production_heading_text: (!locked_items.is_empty())
                .then_some(format!("Locked production ({})", locked_items.len())),
            locked_production_options: locked_items
                .iter()
                .map(|(item, missing_tech)| {
                    let missing_text = missing_tech
                        .map(presentation::tech_name)
                        .unwrap_or("Unavailable");
                    BaseProductionOptionState {
                        item: *item,
                        button_text: format!(
                            "[{}] {} ({})",
                            presentation::production_role_badge(*item),
                            presentation::production_name(*item),
                            missing_text
                        ),
                        tooltip_text: self.production_tooltip_for_owner(base.owner, *item),
                        enabled: false,
                    }
                })
                .collect(),
            queue_item_heading_text: can_manage.then_some("Queue item:"),
            queue_item_options: if can_manage {
                available_items
                    .iter()
                    .copied()
                    .map(|item| BaseProductionOptionState {
                        item,
                        button_text: format!(
                            "+ [{}] {}",
                            presentation::production_role_badge(item),
                            presentation::production_name(item)
                        ),
                        tooltip_text: self.production_tooltip_for_owner(base.owner, item),
                        enabled: true,
                    })
                    .collect()
            } else {
                Vec::new()
            },
        })
    }

    pub fn base_tile_yields_with_modifiers(
        &self,
        owner: Option<usize>,
        base_x: usize,
        base_y: usize,
        tile_x: usize,
        tile_y: usize,
    ) -> Yields {
        let mut tile_yields = self.tile_total_yields(tile_x, tile_y);
        let economy_bonus = owner
            .and_then(|faction_owner| self.faction(faction_owner))
            .map(|f| f.effective_attributes().economy >= 2)
            .unwrap_or(false);

        if economy_bonus && tile_yields.energy > 0 {
            tile_yields.energy += 1;
        }

        if let Some(tile) = self.tile(tile_x, tile_y) {
            if tile.terrain == Terrain::Fungus {
                if let Some(faction_owner) = owner {
                    if self.has_secret_project(faction_owner, SecretProject::WeatherPattern) {
                        tile_yields.nutrients += 1;
                        tile_yields.minerals += 1;
                        tile_yields.energy += 1;
                    }
                }
            }
            if tile.improvement == Some(Improvement::ThermalBorehole) {
                if let Some(faction_owner) = owner {
                    if self.has_secret_project(faction_owner, SecretProject::SingularityContainment)
                    {
                        tile_yields.minerals += 2;
                    }
                    if self.has_secret_project(faction_owner, SecretProject::BlackHoleHarvester) {
                        tile_yields.energy += 2;
                    }
                }
            }
        }

        if tile_x == base_x && tile_y == base_y {
            if let Some(faction_owner) = owner {
                if self.has_secret_project(faction_owner, SecretProject::OrbitalElevator) {
                    tile_yields.energy *= 2;
                }
            }
        }

        tile_yields
    }

    pub fn apply_base_output_modifiers(&self, base_id: usize, mut yields: Yields) -> Yields {
        let Some(base) = self.base(base_id) else {
            return yields;
        };

        for facility in &base.facilities {
            yields = yields.add(content::facility_yield_bonus(*facility));
        }

        if base.governor_mode == GovernorMode::MachinePolity {
            yields.minerals = (yields.minerals as f32 * 1.25) as i32;
        }

        if let Some(faction) = self.faction(base.owner) {
            let mut nutrition_gain = faction.sky_hydroponics;
            let mut energy_gain = faction.solar_transmitters;

            if self.has_secret_project(base.owner, SecretProject::OrbitalElevator) {
                nutrition_gain *= 2;
                energy_gain *= 2;
            }

            yields.nutrients += nutrition_gain;
            yields.energy += energy_gain;
        }

        yields
    }

    fn worked_tile_score(yields: Yields) -> (i32, i32, i32, i32) {
        (
            yields.nutrients * 6 + yields.minerals * 4 + yields.energy * 3,
            yields.nutrients,
            yields.minerals,
            yields.energy,
        )
    }

    fn worked_base_tile_yields(&self, base_id: usize) -> Option<Yields> {
        let base = self.base(base_id)?;
        let owner = Some(base.owner);
        let mut total = self.base_tile_yields_with_modifiers(owner, base.x, base.y, base.x, base.y);

        let mut surrounding_tiles: Vec<(Yields, usize, usize)> = Vec::new();
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = base.x as isize + dx;
                let ny = base.y as isize + dy;
                if nx < 0 || ny < 0 || nx >= self.width as isize || ny >= self.height as isize {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;
                surrounding_tiles.push((
                    self.base_tile_yields_with_modifiers(owner, base.x, base.y, nx, ny),
                    nx,
                    ny,
                ));
            }
        }

        surrounding_tiles.sort_by(
            |(left_yields, left_x, left_y), (right_yields, right_x, right_y)| {
                Self::worked_tile_score(*right_yields)
                    .cmp(&Self::worked_tile_score(*left_yields))
                    .then_with(|| left_x.cmp(right_x))
                    .then_with(|| left_y.cmp(right_y))
            },
        );

        let worked_surrounding_tiles = base.population.saturating_sub(1) as usize;
        for (tile_yields, _, _) in surrounding_tiles
            .into_iter()
            .take(worked_surrounding_tiles.min(8))
        {
            total = total.add(tile_yields);
        }

        Some(total)
    }

    pub fn base_yields(&self, x: usize, y: usize) -> Yields {
        let mut total = Yields::default();
        let base_id = self.tile(x, y).and_then(|tile| tile.base);
        let owner = base_id.and_then(|id| self.base(id)).map(|base| base.owner);

        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx < 0 || ny < 0 || nx >= self.width as isize || ny >= self.height as isize {
                    continue;
                }

                total = total.add(self.base_tile_yields_with_modifiers(
                    owner,
                    x,
                    y,
                    nx as usize,
                    ny as usize,
                ));
            }
        }

        if let Some(base_id) = base_id {
            return self.apply_base_output_modifiers(base_id, total);
        }

        total
    }

    pub fn base_expansion_limit(&self, owner: usize) -> i32 {
        let Some(faction) = self.faction(owner) else {
            return 8;
        };
        let attributes = faction.effective_attributes();
        let map_size_factor = ((self.width * self.height) as i32 / 128).max(1);
        (4 + map_size_factor + attributes.efficiency * 2).max(2)
    }

    pub fn base_unrest(&self, base_id: usize) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        let stability_bonus: i32 = base
            .facilities
            .iter()
            .copied()
            .map(content::facility_stability_bonus)
            .sum();
        let project_stability_bonus: i32 = self
            .built_secret_projects
            .iter()
            .filter(|(_p, o)| *o == base.owner)
            .map(|(p, _)| match p {
                SecretProject::ClinicalImmortality => 2,
                SecretProject::EmpathGuild => 1,
                _ => 0,
            })
            .sum();

        let mut police_bonus = 0;
        let mut bureaucracy_unrest = 0;
        if let Some(faction) = self.faction(base.owner) {
            let attributes = faction.effective_attributes();
            let max_police_units = (1 + attributes.police).max(0) as usize;
            let garrison_count = self
                .units
                .iter()
                .filter(|u| u.alive && u.owner == base.owner && u.x == base.x && u.y == base.y)
                .count();
            police_bonus += (garrison_count.min(max_police_units)) as i32;

            let non_lethal_count = self
                .units
                .iter()
                .filter(|u| {
                    u.alive
                        && u.owner == base.owner
                        && u.x == base.x
                        && u.y == base.y
                        && self.unit_has_ability(u.id, Ability::NonLethalMethods)
                })
                .count() as i32;
            police_bonus += non_lethal_count; // Extra police effect per NonLethal unit

            let base_count = self.bases_for(base.owner).len() as i32;
            let limit = self.base_expansion_limit(base.owner);
            if base_count > limit {
                bureaucracy_unrest = (base_count - limit) / 2;
            }
        }

        let mut unrest = (base.population - 2 - stability_bonus - project_stability_bonus
            + bureaucracy_unrest
            - police_bonus)
            .max(0);

        // Famine Unrest: Significant food insecurity causes faction-wide instability.
        if let Some(faction) = self.faction(base.owner) {
            if faction.food_security < -20 {
                unrest += (faction.food_security.abs() - 20) / 20;
            }
        }

        unrest
    }

    pub fn energy_waste_pct(&self, base_id: usize) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        let Some(faction) = self.faction(base.owner) else {
            return 0;
        };
        let attributes = faction.effective_attributes();

        let distance = if let Some(hq_id) = faction.headquarters_base_id {
            if let Some(hq) = self.base(hq_id) {
                (base.x as i32 - hq.x as i32).abs() + (base.y as i32 - hq.y as i32).abs()
            } else {
                0
            }
        } else {
            0
        };

        // Efficiency Score: 10 + Efficiency * 2
        let efficiency_score = (10 + attributes.efficiency * 2).max(1);
        let waste = (distance * 2) / efficiency_score;
        waste.clamp(0, 90)
    }

    pub fn operational_base_yields(&self, base_id: usize) -> Option<Yields> {
        let base = self.base(base_id)?;
        let mut yields = self
            .worked_base_tile_yields(base_id)
            .map(|worked| self.apply_base_output_modifiers(base_id, worked))?;

        // Convoy Bonuses
        if base.facilities.contains(&Facility::TradeExchange) {
            yields.energy += self
                .active_convoy_route_count(base_id, Some(crate::ConvoyRouteKind::Trade))
                .min(3) as i32;
        }
        if base.facilities.contains(&Facility::FreightDepot) {
            yields.minerals += self
                .active_convoy_route_count(base_id, Some(crate::ConvoyRouteKind::Freight))
                .min(2) as i32;
        }

        // Inter-faction Commerce
        yields.energy += self.base_commerce_bonus(base_id);

        // Energy Waste (Efficiency Modifier)
        let waste_pct = self.energy_waste_pct(base_id);
        if waste_pct > 0 {
            let waste = (yields.energy as f32 * (waste_pct as f32 / 100.0)) as i32;
            yields.energy = (yields.energy - waste).max(0);
        }

        let unrest = self.base_unrest(base_id);
        if unrest > 0 {
            yields.minerals =
                (yields.minerals - unrest).max(if base.population > 0 { 1 } else { 0 });
            yields.energy = (yields.energy - unrest * 2).max(0);
        }

        // AI Efficiency: High AI Dependence provides scaling bonuses to production.
        // At 100 AI Dependence, provides +25% bonus to all yields.
        if let Some(faction) = self.faction(base.owner) {
            if faction.ai_dependence > 20 {
                let bonus_multiplier = faction.ai_dependence as f32 / 400.0; // 0.05 at 20, 0.25 at 100
                yields.nutrients = (yields.nutrients as f32 * (1.0 + bonus_multiplier)) as i32;
                yields.minerals = (yields.minerals as f32 * (1.0 + bonus_multiplier)) as i32;
                yields.energy = (yields.energy as f32 * (1.0 + bonus_multiplier)) as i32;
            }
        }

        Some(yields)
    }

    pub fn effective_base_yields(&self, base_id: usize) -> Option<Yields> {
        let base = self.base(base_id)?;
        let mut yields = self.base_yields(base.x, base.y);

        if base.facilities.contains(&Facility::TradeExchange) {
            yields.energy += self
                .active_convoy_route_count(base_id, Some(crate::ConvoyRouteKind::Trade))
                .min(3) as i32;
        }
        if base.facilities.contains(&Facility::FreightDepot) {
            yields.minerals += self
                .active_convoy_route_count(base_id, Some(crate::ConvoyRouteKind::Freight))
                .min(2) as i32;
        }

        yields.energy += self.base_commerce_bonus(base_id);

        let waste_pct = self.energy_waste_pct(base_id);
        if waste_pct > 0 {
            let waste = (yields.energy as f32 * (waste_pct as f32 / 100.0)) as i32;
            yields.energy = (yields.energy - waste).max(0);
        }

        let unrest = self.base_unrest(base_id);
        if unrest > 0 {
            yields.minerals =
                (yields.minerals - unrest).max(if base.population > 0 { 1 } else { 0 });
            yields.energy = (yields.energy - unrest * 2).max(0);
        }

        if let Some(faction) = self.faction(base.owner) {
            if faction.ai_dependence > 20 {
                let bonus_multiplier = faction.ai_dependence as f32 / 400.0;
                yields.nutrients = (yields.nutrients as f32 * (1.0 + bonus_multiplier)) as i32;
                yields.minerals = (yields.minerals as f32 * (1.0 + bonus_multiplier)) as i32;
                yields.energy = (yields.energy as f32 * (1.0 + bonus_multiplier)) as i32;
            }
        }

        Some(yields)
    }

    pub fn base_food_margin(&self, base_id: usize) -> Option<i32> {
        let base = self.base(base_id)?;
        let yields = self.operational_base_yields(base_id)?;
        Some(yields.nutrients - base.population.max(1))
    }

    pub fn base_mineral_margin(&self, base_id: usize) -> Option<i32> {
        let yields = self.operational_base_yields(base_id)?;
        Some(yields.minerals - 6)
    }

    pub fn base_trade_links(&self, base_id: usize) -> usize {
        self.convoy_routes_for_base(base_id).len()
    }

    pub fn base_military_supply_links(&self, base_id: usize) -> usize {
        self.active_convoy_route_count(base_id, Some(crate::ConvoyRouteKind::MilitarySupply))
    }

    pub fn base_potential_trade_links(&self, base_id: usize) -> usize {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        self.bases
            .iter()
            .filter(|other| other.id != base.id && other.owner == base.owner)
            .filter(|other| other.x.abs_diff(base.x) + other.y.abs_diff(base.y) <= 8)
            .count()
    }

    pub fn convoy_routes_for_base(&self, base_id: usize) -> Vec<usize> {
        self.convoy_route_details_for_base(base_id)
            .into_iter()
            .map(|(other_id, _, _)| other_id)
            .collect()
    }

    pub fn convoy_route_status_for_base(
        &self,
        base_id: usize,
    ) -> Vec<(usize, crate::ConvoyRouteKind, bool, bool, u8)> {
        self.convoy_routes
            .iter()
            .filter_map(|route| {
                if !self.is_convoy_route_valid(route.base_a_id, route.base_b_id) {
                    return None;
                }
                let disrupted = self.is_convoy_route_disrupted(route.base_a_id, route.base_b_id);
                let intercepted =
                    self.is_convoy_route_intercepted(route.base_a_id, route.base_b_id, route.kind);
                if route.base_a_id == base_id {
                    Some((
                        route.base_b_id,
                        route.kind,
                        disrupted,
                        intercepted,
                        route.integrity,
                    ))
                } else if route.base_b_id == base_id {
                    Some((
                        route.base_a_id,
                        route.kind,
                        disrupted,
                        intercepted,
                        route.integrity,
                    ))
                } else {
                    None
                }
            })
            .filter(|(other_id, _, _, _, _)| self.base(*other_id).is_some())
            .collect()
    }

    pub fn base_convoy_route_display_rows(
        &self,
        base_id: usize,
    ) -> Vec<BaseConvoyRouteDisplayRowState> {
        self.convoy_route_status_for_base(base_id)
            .into_iter()
            .filter_map(
                |(target_base_id, kind, disrupted, intercepted, integrity)| {
                    let target_name = self.base(target_base_id)?.name.clone();
                    let status =
                        Self::convoy_route_status_label(disrupted, intercepted, integrity, false);
                    Some(BaseConvoyRouteDisplayRowState {
                        target_base_id,
                        target_name: target_name.clone(),
                        kind,
                        row_text: format!(
                            "{} -> {} ({status}, integrity {integrity}/3)",
                            presentation::convoy_route_kind_label(kind),
                            target_name
                        ),
                        can_repair: integrity < 3,
                        repair_label_text: "Repair".to_string(),
                        remove_label_text: "Remove".to_string(),
                    })
                },
            )
            .collect()
    }

    pub fn convoy_route_details_for_base(
        &self,
        base_id: usize,
    ) -> Vec<(usize, crate::ConvoyRouteKind, bool)> {
        self.convoy_routes
            .iter()
            .filter_map(|route| {
                if !self.is_convoy_route_valid(route.base_a_id, route.base_b_id) {
                    return None;
                }
                let disrupted = self.is_convoy_route_disrupted(route.base_a_id, route.base_b_id);
                if route.base_a_id == base_id {
                    Some((route.base_b_id, route.kind, disrupted))
                } else if route.base_b_id == base_id {
                    Some((route.base_a_id, route.kind, disrupted))
                } else {
                    None
                }
            })
            .filter(|(other_id, _, _)| self.base(*other_id).is_some())
            .collect()
    }

    fn is_convoy_route_valid(&self, base_a_id: usize, base_b_id: usize) -> bool {
        let Some(base_a) = self.base(base_a_id) else {
            return false;
        };
        let Some(base_b) = self.base(base_b_id) else {
            return false;
        };

        let owners_match = base_a.owner == base_b.owner;
        let is_inter_faction_allowed = if !owners_match {
            let status = self.relations[base_a.owner][base_b.owner].status;
            status == DiplomacyStatus::Treaty || status == DiplomacyStatus::Pact
        } else {
            true
        };

        base_a.id != base_b.id
            && is_inter_faction_allowed
            && base_a.x.abs_diff(base_b.x) + base_a.y.abs_diff(base_b.y) <= 12
    }

    pub fn base_commerce_bonus(&self, base_id: usize) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };

        let mut commerce = 0;
        for (other_id, kind, disrupted) in self.convoy_route_details_for_base(base_id) {
            if disrupted || kind != crate::ConvoyRouteKind::Trade {
                continue;
            }

            let Some(other_base) = self.base(other_id) else {
                continue;
            };

            if other_base.owner != base.owner {
                let status = self.relations[base.owner][other_base.owner].status;
                let multiplier = match status {
                    DiplomacyStatus::Pact => 2,
                    DiplomacyStatus::Treaty => 1,
                    _ => 0,
                };

                if multiplier > 0 {
                    // Simple population-based commerce: (pop_a + pop_b) / 4
                    commerce +=
                        ((base.population as i32 + other_base.population as i32) * multiplier) / 4;
                }
            }
        }

        if base.facilities.contains(&Facility::TradeExchange) {
            commerce *= 2;
        }

        commerce
    }

    pub fn base_convoy_capacity(&self, base_id: usize) -> usize {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        1usize
            + base
                .facilities
                .iter()
                .copied()
                .map(content::facility_convoy_capacity_bonus)
                .sum::<i32>()
                .max(0) as usize
    }

    pub fn base_convoy_usage(&self, base_id: usize) -> usize {
        self.convoy_routes_for_base(base_id).len()
    }

    pub fn base_convoy_saturation_ratio(&self, base_id: usize) -> (usize, usize) {
        (
            self.base_convoy_usage(base_id),
            self.base_convoy_capacity(base_id),
        )
    }

    pub fn base_convoy_security(&self, base_id: usize) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        let facility_security: i32 = base
            .facilities
            .iter()
            .copied()
            .map(content::facility_convoy_security_bonus)
            .sum();
        facility_security + self.base_convoy_escort_security(base_id)
    }

    pub fn unit_has_ability(&self, unit_id: usize, ability: Ability) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        let Some(faction) = self.faction(unit.owner) else {
            return false;
        };
        faction
            .unit_designs
            .get(unit.design_index)
            .map(|d| d.abilities.contains(&ability))
            .unwrap_or(false)
    }

    pub fn base_convoy_escort_count(&self, base_id: usize) -> usize {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        self.units
            .iter()
            .filter(|unit| {
                unit.alive
                    && unit.owner == base.owner
                    && (unit.kind == UnitKind::EscortSpeeder
                        || self.unit_has_ability(unit.id, Ability::Escort))
            })
            .filter(|unit| unit.x.abs_diff(base.x) + unit.y.abs_diff(base.y) <= 2)
            .count()
    }

    fn base_convoy_escort_security(&self, base_id: usize) -> i32 {
        (self.base_convoy_escort_count(base_id).min(2) as i32) * 2
    }

    fn active_convoy_route_count(
        &self,
        base_id: usize,
        kind: Option<crate::ConvoyRouteKind>,
    ) -> usize {
        self.convoy_route_details_for_base(base_id)
            .into_iter()
            .filter(|(_, route_kind, disrupted)| {
                !*disrupted && kind.map(|target| target == *route_kind).unwrap_or(true)
            })
            .count()
    }

    fn is_convoy_route_disrupted(&self, base_a_id: usize, base_b_id: usize) -> bool {
        let threat_a =
            self.base_local_military_pressure(base_a_id) + self.base_local_psi_pressure(base_a_id);
        let threat_b =
            self.base_local_military_pressure(base_b_id) + self.base_local_psi_pressure(base_b_id);
        threat_a - self.base_convoy_security(base_a_id) >= 2
            || threat_b - self.base_convoy_security(base_b_id) >= 2
    }

    fn convoy_interception_pressure_for_route(
        &self,
        base_a_id: usize,
        base_b_id: usize,
        owner: usize,
    ) -> i32 {
        let Some(base_a) = self.base(base_a_id) else {
            return 0;
        };
        let Some(base_b) = self.base(base_b_id) else {
            return 0;
        };

        self.units
            .iter()
            .filter(|unit| unit.alive && unit.owner != owner)
            .filter_map(|unit| {
                let distance = (unit.x.abs_diff(base_a.x) + unit.y.abs_diff(base_a.y))
                    .min(unit.x.abs_diff(base_b.x) + unit.y.abs_diff(base_b.y));
                if distance > 2 {
                    return None;
                }

                Some(match unit.kind {
                    UnitKind::RaiderSpeeder | UnitKind::MindWorm | UnitKind::ResonanceLaser => 2,
                    UnitKind::Speeder | UnitKind::PsiSentinel | UnitKind::TranceScout => 1,
                    UnitKind::ScoutPatrol | UnitKind::GarrisonGuard => 1,
                    _ => 0,
                })
            })
            .sum()
    }

    fn is_convoy_route_intercepted(
        &self,
        base_a_id: usize,
        base_b_id: usize,
        kind: crate::ConvoyRouteKind,
    ) -> bool {
        if !self.is_convoy_route_disrupted(base_a_id, base_b_id) {
            return false;
        }
        let Some(owner) = self.base(base_a_id).map(|base| base.owner) else {
            return false;
        };
        let interception_pressure =
            self.convoy_interception_pressure_for_route(base_a_id, base_b_id, owner);
        let security = self.base_convoy_security(base_a_id) + self.base_convoy_security(base_b_id);
        let kind_bias = match kind {
            crate::ConvoyRouteKind::Trade => 0,
            crate::ConvoyRouteKind::Freight => 1,
            crate::ConvoyRouteKind::MilitarySupply => 0,
        };
        interception_pressure > security + kind_bias
    }

    pub fn available_convoy_targets(&self, base_id: usize) -> Vec<usize> {
        self.available_convoy_targets_for_kind(base_id, crate::ConvoyRouteKind::Trade)
    }

    pub fn available_convoy_targets_for_kind(
        &self,
        base_id: usize,
        kind: crate::ConvoyRouteKind,
    ) -> Vec<usize> {
        let Some(base) = self.base(base_id) else {
            return Vec::new();
        };
        if self.convoy_routes_for_base(base_id).len() >= self.base_convoy_capacity(base_id) {
            return Vec::new();
        }
        self.bases
            .iter()
            .filter(|other| other.id != base.id && other.owner == base.owner)
            .filter(|other| other.x.abs_diff(base.x) + other.y.abs_diff(base.y) <= 8)
            .filter(|other| {
                self.convoy_routes_for_base(other.id).len() < self.base_convoy_capacity(other.id)
            })
            .filter(|other| {
                !self.convoy_routes.iter().any(|route| {
                    route.kind == kind
                        && ((route.base_a_id == base.id && route.base_b_id == other.id)
                            || (route.base_b_id == base.id && route.base_a_id == other.id))
                })
            })
            .map(|other| other.id)
            .collect()
    }

    pub fn available_convoy_target_opportunities(
        &self,
        base_id: usize,
    ) -> Vec<AvailableConvoyTargetOpportunityState> {
        let mut opportunities = Vec::new();
        for kind in [
            crate::ConvoyRouteKind::Trade,
            crate::ConvoyRouteKind::Freight,
            crate::ConvoyRouteKind::MilitarySupply,
        ] {
            for target_base_id in self
                .available_convoy_targets_for_kind(base_id, kind)
                .into_iter()
                .take(3)
            {
                let target_name = self
                    .base(target_base_id)
                    .map(|target| target.name.clone())
                    .unwrap_or_else(|| format!("Base {target_base_id}"));
                let prefix = match kind {
                    crate::ConvoyRouteKind::Trade => "Trade",
                    crate::ConvoyRouteKind::Freight => "Freight",
                    crate::ConvoyRouteKind::MilitarySupply => "Supply",
                };
                opportunities.push(AvailableConvoyTargetOpportunityState {
                    target_base_id,
                    kind,
                    button_text: format!("{prefix} to {target_name}"),
                });
            }
        }
        opportunities
    }

    pub fn add_convoy_route(&mut self, base_a_id: usize, base_b_id: usize) -> Result<(), String> {
        self.add_convoy_route_typed(base_a_id, base_b_id, crate::ConvoyRouteKind::Trade)
    }

    pub fn add_convoy_route_typed(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        kind: crate::ConvoyRouteKind,
    ) -> Result<(), String> {
        if base_a_id == base_b_id {
            return Err("A base cannot route to itself.".to_string());
        }
        let Some(base_a) = self.base(base_a_id).cloned() else {
            return Err("Source base not found.".to_string());
        };
        let Some(base_b) = self.base(base_b_id).cloned() else {
            return Err("Target base not found.".to_string());
        };
        if base_a.owner != base_b.owner {
            return Err("Convoy routes require friendly bases.".to_string());
        }
        if base_a.x.abs_diff(base_b.x) + base_a.y.abs_diff(base_b.y) > 8 {
            return Err("Target base is out of convoy range.".to_string());
        }
        if self.convoy_routes_for_base(base_a_id).len() >= self.base_convoy_capacity(base_a_id) {
            return Err("Source base is at convoy capacity.".to_string());
        }
        if self.convoy_routes_for_base(base_b_id).len() >= self.base_convoy_capacity(base_b_id) {
            return Err("Target base is at convoy capacity.".to_string());
        }
        let (left, right) = if base_a_id < base_b_id {
            (base_a_id, base_b_id)
        } else {
            (base_b_id, base_a_id)
        };
        if self
            .convoy_routes
            .iter()
            .any(|route| route.base_a_id == left && route.base_b_id == right && route.kind == kind)
        {
            return Err("That convoy route already exists.".to_string());
        }
        self.convoy_routes.push(crate::ConvoyRoute {
            base_a_id: left,
            base_b_id: right,
            kind,
            integrity: 3,
        });
        self.push_log(format!(
            "Established {} convoy route between {} and {}.",
            presentation::convoy_route_kind_label(kind),
            base_a.name,
            base_b.name
        ));
        Ok(())
    }

    pub fn add_convoy_route_action(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        kind: crate::ConvoyRouteKind,
    ) -> bool {
        self.add_convoy_route_typed(base_a_id, base_b_id, kind)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Add convoy route failed: {err}"));
                false
            })
    }

    pub fn remove_convoy_route(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
    ) -> Result<(), String> {
        self.remove_convoy_route_typed(base_a_id, base_b_id, None)
    }

    pub fn remove_convoy_route_typed(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        kind: Option<crate::ConvoyRouteKind>,
    ) -> Result<(), String> {
        let (left, right) = if base_a_id < base_b_id {
            (base_a_id, base_b_id)
        } else {
            (base_b_id, base_a_id)
        };
        let Some(index) = self.convoy_routes.iter().position(|route| {
            route.base_a_id == left
                && route.base_b_id == right
                && kind.map(|target| route.kind == target).unwrap_or(true)
        }) else {
            return Err("Convoy route not found.".to_string());
        };
        let left_name = self
            .base(left)
            .map(|base| base.name.clone())
            .unwrap_or_else(|| format!("Base {left}"));
        let right_name = self
            .base(right)
            .map(|base| base.name.clone())
            .unwrap_or_else(|| format!("Base {right}"));
        let removed = self.convoy_routes.remove(index);
        self.push_log(format!(
            "Removed {} convoy route between {} and {}.",
            presentation::convoy_route_kind_label(removed.kind),
            left_name,
            right_name
        ));
        Ok(())
    }

    pub fn remove_convoy_route_action(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        kind: Option<crate::ConvoyRouteKind>,
    ) -> bool {
        self.remove_convoy_route_typed(base_a_id, base_b_id, kind)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Remove convoy route failed: {err}"));
                false
            })
    }

    pub fn tile_yield_summary(&self, x: usize, y: usize) -> String {
        presentation::format_yields(self.tile_total_yields(x, y))
    }

    pub fn unit_panel_summary(&self, unit_id: usize) -> Option<presentation::UnitPanelSummary<'_>> {
        let unit = self.unit(unit_id)?;
        let faction = self.faction(unit.owner)?;
        let design_name = faction
            .unit_designs
            .get(unit.design_index)
            .map(|d| d.name.as_str())
            .unwrap_or_else(|| presentation::unit_name(unit.kind.clone()));

        Some(presentation::unit_panel_summary(
            unit.kind.clone(),
            design_name,
            self.faction_name(unit.owner),
            unit.experience,
            unit.x,
            unit.y,
            unit.moves_left,
            unit.hp,
        ))
    }

    pub fn tile_panel_summary(
        &self,
        x: usize,
        y: usize,
    ) -> Option<presentation::TilePanelSummary<'_>> {
        let tile = self.tile(x, y)?;
        Some(presentation::tile_panel_summary(
            tile.terrain,
            tile.x,
            tile.y,
            tile.elevation,
            tile.moisture,
            self.tile_total_yields(x, y),
            tile.improvement,
        ))
    }

    pub fn base_yield_summary(&self, base_id: usize) -> Option<String> {
        self.base(base_id)
            .map(|base| presentation::format_yields(self.base_yields(base.x, base.y)))
    }

    pub fn production_cost(&self, owner: usize, item: ProductionItem) -> i32 {
        let base_cost = match item {
            ProductionItem::CustomUnit(index) => self
                .faction(owner)
                .and_then(|f| f.unit_designs.get(index))
                .map(|d| d.cost as i32)
                .unwrap_or(20),
            _ => content::production_cost(item),
        };

        if let Some(faction) = self.faction(owner) {
            let industry = faction.effective_attributes().industry;
            // -1 Industry = +10% cost, +1 Industry = -10% cost
            let multiplier = 1.0 - (industry as f32 * 0.1);
            return (base_cost as f32 * multiplier) as i32;
        }

        base_cost
    }

    pub fn production_name(&self, owner: usize, item: ProductionItem) -> String {
        match item {
            ProductionItem::CustomUnit(index) => self
                .faction(owner)
                .and_then(|f| f.unit_designs.get(index))
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Unknown Custom Unit".to_string()),
            _ => presentation::production_name(item).to_string(),
        }
    }

    pub fn production_role_badge(&self, _owner: usize, item: ProductionItem) -> &'static str {
        match item {
            ProductionItem::CustomUnit(_) => "✧CST",
            _ => presentation::production_role_badge(item),
        }
    }

    pub fn effective_base_yield_summary(&self, base_id: usize) -> Option<String> {
        self.effective_base_yields(base_id)
            .map(presentation::format_yields)
    }

    pub fn unit_operation_advice(&self, unit_id: usize) -> Option<String> {
        let unit = self.unit(unit_id)?;
        let max_hp = content::unit_base_hp(unit.kind.clone());
        if unit.hp > max_hp / 2 {
            return None;
        }

        let fallback = self
            .safest_fallback_destination(unit.owner, unit.x, unit.y)
            .map(|(x, y, label)| format!("{label} via {}, {}", x, y))
            .unwrap_or_else(|| "the nearest safe position".to_string());

        Some(format!(
            "{} is damaged ({}/{} HP). Fall back toward {}.",
            presentation::unit_name(unit.kind.clone()),
            unit.hp,
            max_hp,
            fallback
        ))
    }

    pub fn player_operations_advice(&self) -> Vec<String> {
        let owner = self.player_owner();
        let mut advice = Vec::new();

        for base in self.bases.iter().filter(|base| base.owner == owner) {
            let unrest = self.base_unrest(base.id);
            if unrest > 0 {
                let morale_upgrade = if base.facilities.contains(&Facility::RecreationCommons)
                    && self.is_production_available(owner, ProductionItem::HologramTheatre)
                {
                    " or Hologram Theatre"
                } else {
                    ""
                };
                advice.push(format!(
                    "{} has unrest {} reducing output. Consider Recreation Commons{} or a lower support burden.",
                    base.name, unrest, morale_upgrade
                ));
            }
            let damaged_garrisons = self.damaged_garrison_count(base.id);
            if damaged_garrisons > 0 {
                let recovery_upgrade = if base.facilities.contains(&Facility::FieldHospital)
                    && self.is_production_available(owner, ProductionItem::ResearchHospital)
                {
                    " or Research Hospital coverage"
                } else {
                    ""
                };
                advice.push(format!(
                    "{} is hosting {} damaged garrison(s). Consider Field Hospital coverage{} or a recovery rotation.",
                    base.name, damaged_garrisons, recovery_upgrade
                ));
            }
            let psi_pressure = self.base_local_psi_pressure(base.id);
            if psi_pressure >= 2 {
                let psi_upgrade = if (base.facilities.contains(&Facility::PsiBeacon)
                    || base.facilities.contains(&Facility::FieldHospital))
                    && self.is_production_available(owner, ProductionItem::BioenhancementCenter)
                {
                    " and Bioenhancement Center support"
                } else {
                    ""
                };
                advice.push(format!(
                    "{} is under psi pressure {}. Consider Psi Sentinel coverage{} and tighter recovery planning.",
                    base.name, psi_pressure, psi_upgrade
                ));
            }
            if let Some(food_margin) = self.base_food_margin(base.id) {
                if food_margin <= 0 {
                    advice.push(format!(
                        "{} is food-strained (margin {}). Consider Greenhouse coverage or better farm tiles.",
                        base.name, food_margin
                    ));
                }
            }
            if let Some(mineral_margin) = self.base_mineral_margin(base.id) {
                if mineral_margin <= 0 {
                    advice.push(format!(
                        "{} is mineral-strained (margin {}). Consider Mineral Refinery coverage or stronger mine tiles.",
                        base.name, mineral_margin
                    ));
                }
            }
        }

        let (_, unit_upkeep, total_upkeep) = self.faction_upkeep(owner);
        if unit_upkeep > 0 {
            advice.push(format!(
                "Support costs are {} energy this turn ({} total upkeep). Consider Command Centers or fewer idle units.",
                unit_upkeep, total_upkeep
            ));
        }

        let trade_ready_bases = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .filter(|base| self.base_potential_trade_links(base.id) >= 1)
            .count();
        let trade_exchanges = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .filter(|base| base.facilities.contains(&Facility::TradeExchange))
            .count();
        if trade_ready_bases >= 2 && trade_exchanges == 0 {
            advice.push(
                "Your empire has bases in trading range but no Trade Exchange coverage. A commerce build would improve energy flow."
                    .to_string(),
            );
        }
        let freight_depots = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .filter(|base| base.facilities.contains(&Facility::FreightDepot))
            .count();
        if trade_ready_bases >= 2 && freight_depots == 0 {
            advice.push(
                "Your empire has tradeable freight corridors but no Freight Depot coverage. A logistics build would improve mineral flow."
                    .to_string(),
            );
        }
        let disrupted_routes = self
            .convoy_routes
            .iter()
            .filter(|route| self.is_convoy_route_disrupted(route.base_a_id, route.base_b_id))
            .count();
        if disrupted_routes > 0 {
            advice.push(format!(
                "{disrupted_routes} convoy route(s) are disrupted by frontier pressure. Safer routes or stronger defenses would restore logistics."
            ));
        }
        let intercepted_routes = self.faction_intercepted_route_count(owner);
        if intercepted_routes > 0 {
            advice.push(format!(
                "{intercepted_routes} convoy route(s) are being intercepted. Escort Speeders or Patrol Grids would cut losses."
            ));
        }
        let escort_units = self
            .units
            .iter()
            .filter(|unit| {
                unit.alive && unit.owner == owner && unit.kind == UnitKind::EscortSpeeder
            })
            .count();
        if disrupted_routes + intercepted_routes > 0 && escort_units == 0 {
            advice.push(
                "Your logistics network is under pressure with no Escort Speeders on station. A mobile escort wing would improve convoy survival."
                    .to_string(),
            );
        }

        if let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) {
            let affected_base_names: Vec<String> = self
                .current_research_unlock_pressure_base_ids(owner)
                .into_iter()
                .filter_map(|base_id| self.base(base_id).map(|base| base.name.clone()))
                .collect();
            if !affected_base_names.is_empty() {
                advice.push(format!(
                    "Current research {} would unblock governor plans at {} base(s): {}.",
                    presentation::tech_name(current_tech),
                    affected_base_names.len(),
                    affected_base_names
                        .into_iter()
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        for unit in self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
        {
            if let Some(line) = self.unit_operation_advice(unit.id) {
                advice.push(line);
            }
        }

        advice.truncate(6);
        advice
    }

    pub fn faction_route_counts(&self, owner: usize) -> (usize, usize, usize, usize, usize, usize) {
        let mut trade = 0;
        let mut freight = 0;
        let mut disrupted = 0;
        let mut intercepted = 0;
        let mut total_routes = 0;

        for route in &self.convoy_routes {
            let Some(base_a) = self.base(route.base_a_id) else {
                continue;
            };
            let Some(base_b) = self.base(route.base_b_id) else {
                continue;
            };
            if base_a.owner != owner || base_b.owner != owner {
                continue;
            }
            total_routes += 1;
            match route.kind {
                crate::ConvoyRouteKind::Trade => trade += 1,
                crate::ConvoyRouteKind::Freight => freight += 1,
                crate::ConvoyRouteKind::MilitarySupply => {}
            }
            if self.is_convoy_route_disrupted(route.base_a_id, route.base_b_id) {
                disrupted += 1;
            }
            if self.is_convoy_route_intercepted(route.base_a_id, route.base_b_id, route.kind) {
                intercepted += 1;
            }
        }

        let capacity_used = total_routes;
        let capacity_total = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| self.base_convoy_capacity(base.id))
            .sum();
        (
            trade,
            freight,
            disrupted,
            intercepted,
            capacity_used,
            capacity_total,
        )
    }

    pub fn faction_convoy_saturation(&self, owner: usize) -> Vec<(usize, String, usize, usize)> {
        let mut entries: Vec<_> = self
            .bases_for(owner)
            .into_iter()
            .map(|base| {
                (
                    base.id,
                    base.name.clone(),
                    self.base_convoy_usage(base.id),
                    self.base_convoy_capacity(base.id),
                )
            })
            .collect();
        entries.sort_by(|left, right| {
            let left_ratio = (left.2 as i32) * 100 / (left.3.max(1) as i32);
            let right_ratio = (right.2 as i32) * 100 / (right.3.max(1) as i32);
            right_ratio
                .cmp(&left_ratio)
                .then_with(|| right.2.cmp(&left.2))
                .then_with(|| left.1.cmp(&right.1))
        });
        entries
    }

    pub fn faction_convoy_hub_display_rows(&self, owner: usize) -> Vec<ConvoyHubDisplayRowState> {
        self.faction_convoy_saturation(owner)
            .into_iter()
            .map(|(base_id, name, used, capacity)| {
                let is_saturated = capacity > 0 && used >= capacity;
                let is_tight = !is_saturated && capacity > 0 && used + 1 >= capacity;
                ConvoyHubDisplayRowState {
                    base_id,
                    row_text: format!("{name}: {used}/{capacity} lanes"),
                    saturation_label_text: if is_saturated {
                        Some("SATURATED".to_string())
                    } else if is_tight {
                        Some("TIGHT".to_string())
                    } else {
                        None
                    },
                    is_saturated,
                    is_tight,
                }
            })
            .collect()
    }

    pub fn faction_convoy_route_summaries(&self, owner: usize) -> Vec<crate::ConvoyRouteSummary> {
        let mut summaries = Vec::new();
        for route in &self.convoy_routes {
            let Some(base_a) = self.base(route.base_a_id) else {
                continue;
            };
            let Some(base_b) = self.base(route.base_b_id) else {
                continue;
            };
            if base_a.owner != owner || base_b.owner != owner {
                continue;
            }
            let disrupted = self.is_convoy_route_disrupted(route.base_a_id, route.base_b_id);
            let intercepted =
                self.is_convoy_route_intercepted(route.base_a_id, route.base_b_id, route.kind);
            let protected = self.base_convoy_security(route.base_a_id) > 0
                || self.base_convoy_security(route.base_b_id) > 0;
            summaries.push(crate::ConvoyRouteSummary {
                base_a_id: route.base_a_id,
                base_b_id: route.base_b_id,
                base_a_name: base_a.name.clone(),
                base_b_name: base_b.name.clone(),
                kind: route.kind,
                disrupted,
                intercepted,
                integrity: route.integrity,
                protected,
            });
        }
        summaries.sort_by(|left, right| {
            Self::convoy_summary_priority(right)
                .cmp(&Self::convoy_summary_priority(left))
                .then_with(|| left.base_a_name.cmp(&right.base_a_name))
                .then_with(|| left.base_b_name.cmp(&right.base_b_name))
                .then_with(|| left.kind.cmp(&right.kind))
        });
        summaries
    }

    pub fn filtered_convoy_route_summaries(
        &self,
        owner: usize,
        filter: LogisticsRouteFilter,
        sort: LogisticsRouteSort,
    ) -> Vec<crate::ConvoyRouteSummary> {
        let mut routes: Vec<_> = self
            .faction_convoy_route_summaries(owner)
            .into_iter()
            .filter(|route| Self::convoy_route_matches_filter(route, filter))
            .collect();
        routes.sort_by(|left, right| match sort {
            LogisticsRouteSort::Severity => Self::convoy_route_severity_score(right)
                .cmp(&Self::convoy_route_severity_score(left))
                .then_with(|| left.base_a_name.cmp(&right.base_a_name))
                .then_with(|| left.base_b_name.cmp(&right.base_b_name)),
            LogisticsRouteSort::Name => left
                .base_a_name
                .cmp(&right.base_a_name)
                .then_with(|| left.base_b_name.cmp(&right.base_b_name)),
            LogisticsRouteSort::Integrity => left
                .integrity
                .cmp(&right.integrity)
                .then_with(|| left.base_a_name.cmp(&right.base_a_name))
                .then_with(|| left.base_b_name.cmp(&right.base_b_name)),
            LogisticsRouteSort::Kind => presentation::convoy_route_kind_label(left.kind)
                .cmp(presentation::convoy_route_kind_label(right.kind))
                .then_with(|| left.base_a_name.cmp(&right.base_a_name))
                .then_with(|| left.base_b_name.cmp(&right.base_b_name)),
        });
        routes
    }

    pub fn filtered_convoy_route_display_rows(
        &self,
        owner: usize,
        filter: LogisticsRouteFilter,
        sort: LogisticsRouteSort,
    ) -> Vec<ConvoyRouteDisplayRowState> {
        self.filtered_convoy_route_summaries(owner, filter, sort)
            .into_iter()
            .map(|route| {
                let status = Self::convoy_route_status_label(
                    route.disrupted,
                    route.intercepted,
                    route.integrity,
                    route.protected,
                );
                ConvoyRouteDisplayRowState {
                    base_a_id: route.base_a_id,
                    base_b_id: route.base_b_id,
                    kind: route.kind,
                    row_text: format!(
                        "{} {} -> {} ({status}, {}/3)",
                        presentation::convoy_route_kind_label(route.kind),
                        route.base_a_name,
                        route.base_b_name,
                        route.integrity
                    ),
                    focus_a_label_text: "A".to_string(),
                    focus_b_label_text: "B".to_string(),
                    can_repair: route.integrity < 3,
                    repair_label_text: "Repair".to_string(),
                    remove_label_text: "Remove".to_string(),
                }
            })
            .collect()
    }

    pub fn most_stressed_convoy_route(&self, owner: usize) -> Option<crate::ConvoyRouteSummary> {
        self.faction_convoy_route_summaries(owner)
            .into_iter()
            .next()
    }

    pub fn worst_convoy_route_focus_base_id(&self, owner: usize) -> Option<usize> {
        let route = self.most_stressed_convoy_route(owner)?;
        let left_score = self.base_logistics_stress_score(route.base_a_id);
        let right_score = self.base_logistics_stress_score(route.base_b_id);
        Some(if right_score > left_score {
            route.base_b_id
        } else {
            route.base_a_id
        })
    }

    pub fn worst_convoy_route_focus_action(&mut self, owner: usize) -> Option<usize> {
        let target = self.worst_convoy_route_focus_base_id(owner);
        if target.is_none() {
            self.push_log("No convoy routes are available to focus.".to_string());
        }
        target
    }

    pub fn faction_intercepted_route_count(&self, owner: usize) -> usize {
        self.faction_route_counts(owner).3
    }

    pub fn faction_collapsing_route_count(&self, owner: usize) -> usize {
        self.convoy_routes
            .iter()
            .filter(|route| {
                self.base(route.base_a_id)
                    .zip(self.base(route.base_b_id))
                    .map(|(left, right)| left.owner == owner && right.owner == owner)
                    .unwrap_or(false)
            })
            .filter(|route| route.integrity <= 1)
            .count()
    }

    pub fn convoy_pressure_base_ids(&self, owner: usize) -> Vec<usize> {
        let mut ids: Vec<(usize, i32, i32, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| {
                let statuses = self.convoy_route_status_for_base(base.id);
                let intercepted = statuses
                    .iter()
                    .filter(|(_, _, _, intercepted, _)| *intercepted)
                    .count() as i32;
                let disrupted = statuses
                    .iter()
                    .filter(|(_, _, disrupted, _, _)| *disrupted)
                    .count() as i32;
                let escort_gap = (1 - self.base_convoy_escort_count(base.id) as i32).max(0);
                (base.id, intercepted, disrupted, escort_gap)
            })
            .filter(|(_, intercepted, disrupted, _)| *intercepted > 0 || *disrupted > 0)
            .collect();
        ids.sort_by_key(|(id, intercepted, disrupted, escort_gap)| {
            (-*intercepted, -*disrupted, -*escort_gap, *id as i32)
        });
        ids.into_iter().map(|(id, _, _, _)| id).collect()
    }

    pub fn convoy_collapsing_base_ids(&self, owner: usize) -> Vec<usize> {
        let mut ids: Vec<(usize, i32, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| {
                let collapsing = self
                    .convoy_route_status_for_base(base.id)
                    .into_iter()
                    .filter(|(_, _, _, intercepted, integrity)| *intercepted && *integrity <= 1)
                    .count() as i32;
                let disrupted = self
                    .convoy_route_status_for_base(base.id)
                    .into_iter()
                    .filter(|(_, _, disrupted, _, _)| *disrupted)
                    .count() as i32;
                (base.id, collapsing, disrupted)
            })
            .filter(|(_, collapsing, disrupted)| *collapsing > 0 || *disrupted > 0)
            .collect();
        ids.sort_by_key(|(id, collapsing, disrupted)| (-*collapsing, -*disrupted, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn base_logistics_stress_score(&self, base_id: usize) -> i32 {
        let route_statuses = self.convoy_route_status_for_base(base_id);
        let intercepted = route_statuses
            .iter()
            .filter(|(_, _, _, intercepted, _)| *intercepted)
            .count() as i32;
        let disrupted = route_statuses
            .iter()
            .filter(|(_, _, disrupted, _, _)| *disrupted)
            .count() as i32;
        let collapsing = route_statuses
            .iter()
            .filter(|(_, _, _, intercepted, integrity)| *intercepted && *integrity <= 1)
            .count() as i32;
        collapsing * 4
            + intercepted * 3
            + disrupted * 2
            + self.base_potential_trade_links(base_id) as i32
    }

    pub fn suggested_escort_patrol_moves(&self, owner: usize) -> Vec<(usize, usize, usize)> {
        let target_bases = self.convoy_pressure_base_ids(owner);
        if target_bases.is_empty() {
            return Vec::new();
        }

        let escort_units: Vec<&Unit> = self
            .units
            .iter()
            .filter(|unit| {
                unit.alive && unit.owner == owner && unit.kind == UnitKind::EscortSpeeder
            })
            .filter(|unit| unit.moves_left > 0)
            .collect();

        let mut moves = Vec::new();
        let mut used_targets = std::collections::BTreeSet::new();

        for unit in escort_units {
            let target = target_bases
                .iter()
                .filter(|base_id| !used_targets.contains(*base_id))
                .filter_map(|base_id| self.base(*base_id).map(|base| (*base_id, base.x, base.y)))
                .min_by_key(|(_, x, y)| unit.x.abs_diff(*x) + unit.y.abs_diff(*y));

            let Some((base_id, tx, ty)) = target else {
                continue;
            };
            let next_x = if unit.x < tx {
                unit.x + 1
            } else if unit.x > tx {
                unit.x.saturating_sub(1)
            } else {
                unit.x
            };
            let next_y = if unit.y < ty {
                unit.y + 1
            } else if unit.y > ty {
                unit.y.saturating_sub(1)
            } else {
                unit.y
            };
            if (next_x, next_y) != (unit.x, unit.y) {
                moves.push((unit.id, next_x, next_y));
                used_targets.insert(base_id);
            }
        }

        moves
    }

    pub fn apply_escort_patrol_moves(&mut self, owner: usize) -> usize {
        let moves = self.suggested_escort_patrol_moves(owner);
        if moves.is_empty() {
            self.push_log("No escort patrol moves are currently needed or available.".to_string());
            return 0;
        }

        let mut moved = 0;
        for (unit_id, x, y) in moves {
            if self.move_unit_to(unit_id, x, y).is_ok() {
                moved += 1;
            }
        }

        self.push_log(format!(
            "Operations reassigned {moved} escort unit(s) to convoy patrol duty."
        ));
        moved
    }

    pub fn suggested_convoy_repairs(
        &self,
        owner: usize,
    ) -> Vec<(usize, usize, crate::ConvoyRouteKind)> {
        self.convoy_routes
            .iter()
            .filter(|route| {
                self.base(route.base_a_id)
                    .zip(self.base(route.base_b_id))
                    .map(|(left, right)| left.owner == owner && right.owner == owner)
                    .unwrap_or(false)
            })
            .filter(|route| route.integrity < 3)
            .map(|route| (route.base_a_id, route.base_b_id, route.kind))
            .collect()
    }

    pub fn apply_convoy_repairs_all(&mut self, owner: usize) -> usize {
        let repairs = self.suggested_convoy_repairs(owner);
        if repairs.is_empty() {
            self.push_log("No damaged convoy routes currently need repairs.".to_string());
            return 0;
        }

        let mut repaired = 0;
        for (base_a_id, base_b_id, kind) in repairs {
            if self
                .repair_convoy_route_typed(base_a_id, base_b_id, kind)
                .is_ok()
            {
                repaired += 1;
            }
        }

        self.push_log(format!("Operations repaired {repaired} convoy route(s)."));
        repaired
    }

    pub fn apply_filtered_convoy_repairs(
        &mut self,
        repairs: Vec<(usize, usize, crate::ConvoyRouteKind, i32)>,
    ) -> usize {
        let mut repaired = 0;
        for (base_a_id, base_b_id, kind, integrity) in repairs {
            if integrity >= 3 {
                continue;
            }
            if self
                .repair_convoy_route_typed(base_a_id, base_b_id, kind)
                .is_ok()
            {
                repaired += 1;
            }
        }
        if repaired == 0 {
            self.push_log("No filtered convoy routes were eligible for repair.".to_string());
        } else {
            self.push_log(format!(
                "Operations repaired {repaired} filtered convoy route(s)."
            ));
        }
        repaired
    }

    pub fn apply_filtered_convoy_repairs_for_owner(
        &mut self,
        owner: usize,
        filter: LogisticsRouteFilter,
        sort: LogisticsRouteSort,
    ) -> usize {
        let repairs = self
            .filtered_convoy_route_summaries(owner, filter, sort)
            .into_iter()
            .map(|route| {
                (
                    route.base_a_id,
                    route.base_b_id,
                    route.kind,
                    route.integrity as i32,
                )
            })
            .collect();
        self.apply_filtered_convoy_repairs(repairs)
    }

    pub fn repair_convoy_route_typed(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        kind: crate::ConvoyRouteKind,
    ) -> Result<(), String> {
        let route_index = self
            .convoy_routes
            .iter()
            .position(|route| {
                route.base_a_id == base_a_id && route.base_b_id == base_b_id && route.kind == kind
            })
            .or_else(|| {
                self.convoy_routes.iter().position(|route| {
                    route.base_a_id == base_b_id
                        && route.base_b_id == base_a_id
                        && route.kind == kind
                })
            })
            .ok_or_else(|| "Convoy route not found.".to_string())?;

        let owner = self
            .base(self.convoy_routes[route_index].base_a_id)
            .map(|base| base.owner)
            .ok_or_else(|| "Source base not found.".to_string())?;
        if self.convoy_routes[route_index].integrity >= 3 {
            return Err("Convoy route is already at full integrity.".to_string());
        }
        let repair_cost = 2;
        let faction_energy = self
            .faction(owner)
            .map(|faction| faction.energy)
            .ok_or_else(|| "Faction not found.".to_string())?;
        if faction_energy < repair_cost {
            return Err("Not enough energy to repair convoy route.".to_string());
        }

        if let Some(faction) = self.faction_mut(owner) {
            faction.energy -= repair_cost;
        }
        self.convoy_routes[route_index].integrity =
            (self.convoy_routes[route_index].integrity + 1).min(3);
        let left_name = self
            .base(self.convoy_routes[route_index].base_a_id)
            .map(|base| base.name.clone())
            .unwrap_or_else(|| format!("Base {}", self.convoy_routes[route_index].base_a_id));
        let right_name = self
            .base(self.convoy_routes[route_index].base_b_id)
            .map(|base| base.name.clone())
            .unwrap_or_else(|| format!("Base {}", self.convoy_routes[route_index].base_b_id));
        self.push_log(format!(
            "Repaired {} convoy route between {} and {} (integrity {}/3).",
            presentation::convoy_route_kind_label(kind),
            left_name,
            right_name,
            self.convoy_routes[route_index].integrity
        ));
        Ok(())
    }

    pub fn repair_convoy_route_action(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        kind: crate::ConvoyRouteKind,
    ) -> bool {
        self.repair_convoy_route_typed(base_a_id, base_b_id, kind)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Repair convoy route failed: {err}"));
                false
            })
    }

    pub fn suggested_convoy_rebuilds(
        &self,
        owner: usize,
    ) -> Vec<(usize, usize, crate::ConvoyRouteKind)> {
        let mut rebuilds = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for base in self.bases_for(owner) {
            if self.convoy_routes_for_base(base.id).len() >= self.base_convoy_capacity(base.id) {
                continue;
            }
            let kind = if self
                .base(base.id)
                .map(|base| {
                    (base.facilities.contains(&Facility::CommandCenter)
                        || base.facilities.contains(&Facility::FieldHospital)
                        || base.facilities.contains(&Facility::MilitaryAcademy)
                        || base.facilities.contains(&Facility::ForwardDepot))
                        && (self.base_local_military_pressure(base.id) >= 1
                            || self.damaged_garrison_count_for_base(base.id) > 0)
                })
                .unwrap_or(false)
            {
                crate::ConvoyRouteKind::MilitarySupply
            } else if self
                .base(base.id)
                .map(|base| base.facilities.contains(&Facility::FreightDepot))
                .unwrap_or(false)
            {
                crate::ConvoyRouteKind::Freight
            } else {
                crate::ConvoyRouteKind::Trade
            };
            if let Some(target_id) = self
                .available_convoy_targets_for_kind(base.id, kind)
                .into_iter()
                .next()
            {
                let pair = if base.id < target_id {
                    (base.id, target_id, kind)
                } else {
                    (target_id, base.id, kind)
                };
                if seen.insert(pair) {
                    rebuilds.push(pair);
                }
            }
        }
        rebuilds
    }

    pub fn apply_convoy_rebuilds_all(&mut self, owner: usize) -> usize {
        let rebuilds = self.suggested_convoy_rebuilds(owner);
        if rebuilds.is_empty() {
            self.push_log("No convoy rebuild opportunities are currently available.".to_string());
            return 0;
        }

        let mut rebuilt = 0;
        for (base_a_id, base_b_id, kind) in rebuilds {
            if self
                .add_convoy_route_typed(base_a_id, base_b_id, kind)
                .is_ok()
            {
                rebuilt += 1;
            }
        }

        self.push_log(format!(
            "Operations rebuilt or expanded {rebuilt} convoy route(s)."
        ));
        rebuilt
    }

    pub fn convoy_route_opportunities(
        &self,
        owner: usize,
        filter: LogisticsRouteFilter,
    ) -> Vec<ConvoyRouteOpportunityState> {
        let mut rebuilds = self.suggested_convoy_rebuilds(owner);
        rebuilds.retain(|(_, _, kind)| match filter {
            LogisticsRouteFilter::Trade => *kind == crate::ConvoyRouteKind::Trade,
            LogisticsRouteFilter::Freight => *kind == crate::ConvoyRouteKind::Freight,
            LogisticsRouteFilter::Military => *kind == crate::ConvoyRouteKind::MilitarySupply,
            _ => true,
        });
        rebuilds.sort_by(|left, right| {
            let left_names = (
                self.base(left.0)
                    .map(|base| base.name.as_str())
                    .unwrap_or(""),
                self.base(left.1)
                    .map(|base| base.name.as_str())
                    .unwrap_or(""),
            );
            let right_names = (
                self.base(right.0)
                    .map(|base| base.name.as_str())
                    .unwrap_or(""),
                self.base(right.1)
                    .map(|base| base.name.as_str())
                    .unwrap_or(""),
            );
            presentation::convoy_route_kind_label(left.2)
                .cmp(presentation::convoy_route_kind_label(right.2))
                .then_with(|| left_names.cmp(&right_names))
        });
        rebuilds
            .into_iter()
            .map(|(base_a_id, base_b_id, kind)| {
                let left_name = self
                    .base(base_a_id)
                    .map(|base| base.name.clone())
                    .unwrap_or_else(|| format!("Base {base_a_id}"));
                let right_name = self
                    .base(base_b_id)
                    .map(|base| base.name.clone())
                    .unwrap_or_else(|| format!("Base {base_b_id}"));
                ConvoyRouteOpportunityState {
                    base_a_id,
                    base_b_id,
                    kind,
                    button_text: format!(
                        "Create {} {} -> {}",
                        presentation::convoy_route_kind_label(kind),
                        left_name,
                        right_name
                    ),
                }
            })
            .collect()
    }

    pub fn remove_collapsing_convoy_routes(&mut self, owner: usize) -> usize {
        let routes = self
            .faction_convoy_route_summaries(owner)
            .into_iter()
            .filter(|route| route.intercepted && route.integrity <= 1)
            .collect::<Vec<_>>();
        let mut removed = 0;
        for route in routes {
            if self
                .remove_convoy_route_typed(route.base_a_id, route.base_b_id, Some(route.kind))
                .is_ok()
            {
                removed += 1;
            }
        }
        if removed == 0 {
            self.push_log("No collapsing convoy routes required removal.".to_string());
        } else {
            self.push_log(format!(
                "Operations removed {removed} collapsing convoy route(s)."
            ));
        }
        removed
    }

    pub fn faction_logistics_alerts(&self, owner: usize) -> Vec<String> {
        let (trade, freight, disrupted, intercepted, used, capacity) =
            self.faction_route_counts(owner);
        let (_, convoy_upkeep, _, _) = self.faction_upkeep_breakdown(owner);
        let escort_units = self
            .units
            .iter()
            .filter(|unit| {
                unit.alive && unit.owner == owner && unit.kind == UnitKind::EscortSpeeder
            })
            .count();
        let collapsing = self.faction_collapsing_route_count(owner);
        let saturated_hubs = self
            .faction_convoy_saturation(owner)
            .into_iter()
            .filter(|(_, _, used, capacity)| *capacity > 0 && *used >= *capacity)
            .count();
        let mut alerts = Vec::new();

        if used == 0 && self.bases_for(owner).len() >= 2 {
            alerts.push("No active convoy routes are running between your bases.".to_string());
        }
        if disrupted > 0 {
            alerts.push(format!(
                "{disrupted} convoy route(s) are disrupted by local pressure."
            ));
        }
        if intercepted > 0 {
            alerts.push(format!(
                "{intercepted} convoy route(s) are actively intercepted and losing resources."
            ));
        }
        if collapsing > 0 {
            alerts.push(format!(
                "{collapsing} convoy route(s) are close to collapse from repeated interception."
            ));
        }
        if used > 0 && escort_units == 0 {
            alerts.push("Active convoy network has no Escort Speeders assigned.".to_string());
        }
        if capacity > 0 && used >= capacity {
            alerts.push("Convoy capacity is saturated across the empire.".to_string());
        }
        if saturated_hubs > 0 {
            alerts.push(format!(
                "{saturated_hubs} convoy hub(s) are at full lane capacity."
            ));
        }
        if convoy_upkeep >= 4 {
            alerts.push(format!(
                "Convoy maintenance is costing {convoy_upkeep} energy per turn."
            ));
        }
        if trade == 0 && freight > 0 {
            alerts.push(
                "Freight routes exist without any trade lanes feeding energy flow.".to_string(),
            );
        }
        if freight == 0 && trade > 0 {
            alerts.push(
                "Trade routes exist without freight coverage for mineral logistics.".to_string(),
            );
        }
        let military_supply = self
            .convoy_routes
            .iter()
            .filter(|route| {
                self.base(route.base_a_id)
                    .zip(self.base(route.base_b_id))
                    .map(|(left, right)| {
                        left.owner == owner
                            && right.owner == owner
                            && route.kind == crate::ConvoyRouteKind::MilitarySupply
                    })
                    .unwrap_or(false)
            })
            .count();
        if military_supply > 0 {
            alerts.push(format!(
                "{military_supply} military supply lane(s) are sustaining frontline forces."
            ));
        }
        alerts.truncate(5);
        alerts
    }

    pub fn faction_overview_display_states(
        &self,
        route_filter: LogisticsRouteFilter,
        route_sort: LogisticsRouteSort,
    ) -> Vec<FactionOverviewDisplayState> {
        self.non_native_factions()
            .iter()
            .map(|faction| {
                let owner = faction.id;
                let bases = self.bases_for(owner);
                let base_count = bases.len();
                let unit_count = self.live_units_for(owner).len();
                let recovery_count = bases
                    .iter()
                    .filter(|base| self.damaged_garrison_count_for_base(base.id) > 0)
                    .count();
                let frontier_count = bases
                    .iter()
                    .filter(|base| {
                        self.base_local_military_pressure(base.id) >= 2
                            || self.base_local_psi_pressure(base.id) >= 2
                    })
                    .count();
                let unrest_count = bases
                    .iter()
                    .filter(|base| self.base_unrest(base.id) > 0)
                    .count();

                let (
                    _off_mode_count,
                    balanced_count,
                    defense_mode_count,
                    recovery_mode_count,
                    economy_mode_count,
                    logistics_mode_count,
                ) = self.faction_governor_mode_counts(owner);

                let (rear_area_count, classified_frontier_count, psi_frontier_count, warzone_count) =
                    self.base_area_role_counts(owner);

                let mut production_counts = std::collections::BTreeMap::<String, usize>::new();
                let mut queue_counts = std::collections::BTreeMap::<String, usize>::new();
                let mut production_role_counts = std::collections::BTreeMap::<String, usize>::new();
                let mut queue_role_counts = std::collections::BTreeMap::<String, usize>::new();

                for base in &bases {
                    *production_counts
                        .entry(self.production_name(owner, base.production))
                        .or_default() += 1;
                    *production_role_counts
                        .entry(presentation::production_role_category(base.production).to_string())
                        .or_default() += 1;
                    for queued_item in &base.production_queue {
                        *queue_counts
                            .entry(self.production_name(owner, *queued_item))
                            .or_default() += 1;
                        *queue_role_counts
                            .entry(presentation::production_role_category(*queued_item).to_string())
                            .or_default() += 1;
                    }
                }

                let production_posture_text = presentation::summarize_named_counts(
                    &production_counts.into_iter().collect::<Vec<_>>(),
                    "No active production",
                    3,
                );
                let queue_posture_text = presentation::summarize_named_counts(
                    &queue_counts.into_iter().collect::<Vec<_>>(),
                    "No queued follow-ups",
                    3,
                );
                let production_roles_text = presentation::summarize_named_counts(
                    &production_role_counts.into_iter().collect::<Vec<_>>(),
                    "No active role mix",
                    4,
                );
                let queue_roles_text = presentation::summarize_named_counts(
                    &queue_role_counts.into_iter().collect::<Vec<_>>(),
                    "No queued role mix",
                    4,
                );

                let governor_recommendation_counts = self.faction_governor_recommendation_counts(owner);
                let governor_intent_text = presentation::summarize_production_item_counts(
                    &governor_recommendation_counts,
                    "No immediate governor recommendations",
                    3,
                );

                let governor_queue_intent_counts =
                    self.faction_governor_queue_intent_counts(owner, 3);
                let governor_queue_intent_text = presentation::summarize_production_item_counts(
                    &governor_queue_intent_counts,
                    "No queued governor intent",
                    3,
                );

                let queue_gap_base_ids = self.faction_queue_gap_base_ids(owner);
                let queue_gaps_text = presentation::summarize_base_names(
                    &queue_gap_base_ids
                        .iter()
                        .filter_map(|base_id| self.base(*base_id).map(|base| base.name.clone()))
                        .collect::<Vec<_>>(),
                    "All visible bases have queued follow-ups",
                    4,
                );

                let locked_governor_recommendations =
                    self.faction_locked_governor_recommendations(owner);
                let tech_blocked_intent_text = presentation::summarize_base_unlock_blocks(
                    &locked_governor_recommendations
                        .iter()
                        .filter_map(|(base_id, item, tech, _)| {
                            self.base(*base_id)
                                .map(|base| (base.name.clone(), *item, *tech))
                        })
                        .collect::<Vec<_>>(),
                    "No immediate tech-gated governor upgrades",
                    3,
                );

                let (trade, freight, disrupted, intercepted, used, total) =
                    self.faction_route_counts(owner);

                let summary = presentation::faction_status_summary(
                    &faction.name,
                    base_count,
                    unit_count,
                    faction.energy,
                    faction.research,
                    faction.current_research,
                    faction.techs_discovered,
                    unrest_count,
                    recovery_count,
                    frontier_count,
                    rear_area_count,
                    psi_frontier_count,
                    warzone_count,
                    trade,
                    freight,
                    disrupted,
                    intercepted,
                    used,
                    total,
                );

                let (facility_upkeep, convoy_upkeep, unit_upkeep, _total_upkeep) =
                    self.faction_upkeep_breakdown(owner);

                let governor_warnings = self.faction_governor_warning_lines(owner);
                let logistics_panel = self.faction_logistics_panel_state(
                    owner,
                    route_filter,
                    route_sort,
                );
                let player_id = self.player_owner();
                let diplomacy_panel = self.faction_diplomacy_panel_state(owner, player_id);
                let social_engineering_panel = self.social_engineering_display_state(owner);

                FactionOverviewDisplayState {
                    faction_id: owner,
                    is_player_owned: owner == self.player_owner(),
                    color_hex: summary.color_hex.to_string(),
                    name: summary.name.to_string(),
                    leader_text: summary.leader.map(|s| format!("Leader: {}", s)),
                    description_text: summary.description.map(|s| s.to_string()),
                    base_count_text: format!("Bases: {}", summary.base_count),
                    unit_count_text: format!("Units: {}", summary.unit_count),
                    energy_text: format!("Energy: {}", summary.energy),
                    upkeep_text: format!(
                        "Upkeep: {} energy ({} fac / {} routes) | {} minerals (support)",
                        facility_upkeep + convoy_upkeep, facility_upkeep, convoy_upkeep, unit_upkeep
                    ),
                    research_progress_text: format!("Research: {}", summary.research_progress),
                    current_tech_text: format!("Current tech: {}", summary.current_tech),
                    techs_discovered_text: format!("Techs discovered: {}", summary.techs_discovered),
                    indices_heading_text: "Strategic Indices",
                    food_security_text: format!("Food Security: {}%", faction.food_security),
                    ai_dependence_text: format!("AI Dependence: {}%", faction.ai_dependence),
                    orbital_index_text: format!("Orbital Index: {}", faction.orbital_index),
                    planet_toxicity_text: format!("Planet Toxicity: {}", faction.planet_toxicity),
                    alerts_text: format!(
                        "Alerts: {} unrest / {} recovery / {} frontier",
                        summary.unrest_base_count,
                        summary.recovery_base_count,
                        summary.frontier_base_count
                    ),
                    base_roles_text: format!(
                        "Base roles: {} rear / {} frontier / {} psi / {} warzone",
                        summary.rear_area_base_count,
                        classified_frontier_count,
                        summary.psi_frontier_base_count,
                        summary.warzone_base_count
                    ),
                    logistics_summary_text: format!(
                        "Logistics: {}/{} capacity, {} trade / {} freight / {} disrupted / {} intercepted",
                        summary.convoy_capacity_used,
                        summary.convoy_capacity_total,
                        summary.trade_route_count,
                        summary.freight_route_count,
                        summary.disrupted_route_count,
                        summary.intercepted_route_count
                    ),
                    governor_summary_heading_text: "Governor Summary",
                    governor_mode_mix_summary: presentation::governor_mode_mix_summary(
                        balanced_count,
                        defense_mode_count,
                        recovery_mode_count,
                        economy_mode_count,
                        logistics_mode_count,
                    ),
                    production_posture_text: format!("Production posture: {production_posture_text}"),
                    production_roles_text: format!("Production roles: {production_roles_text}"),
                    queue_posture_text: format!("Queue posture: {queue_posture_text}"),
                    queue_roles_text: format!("Queue roles: {queue_roles_text}"),
                    governor_intent_text: format!("Governor intent: {governor_intent_text}"),
                    governor_queue_intent_text: format!("Governor queue intent: {governor_queue_intent_text}"),
                    queue_gaps_text: format!("Queue gaps: {queue_gaps_text}"),
                    tech_blocked_intent_text: format!("Tech-blocked intent: {tech_blocked_intent_text}"),
                    secret_projects_heading_text: "Secret Projects",
                    secret_projects_text: self.built_secret_projects
                        .iter()
                        .filter(|(_, o)| *o == owner)
                        .map(|(p, _)| format!("✧ {}", self.production_name(owner, match p {
                            SecretProject::WeatherPattern => ProductionItem::WeatherPattern,
                            SecretProject::ClinicalImmortality => ProductionItem::ClinicalImmortality,
                            SecretProject::EmpathGuild => ProductionItem::EmpathGuild,
                            SecretProject::OrbitalElevator => ProductionItem::OrbitalElevator,
                            SecretProject::ManifoldDrive => ProductionItem::ManifoldDrive,
                            SecretProject::SingularityContainment => ProductionItem::SingularityContainment,
                            SecretProject::BlackHoleHarvester => ProductionItem::BlackHoleHarvester,
                        })))
                        .collect(),
                    jump_queue_gap_label_text: Some("Jump Queue Gap"),
                    jump_tech_block_label_text: Some("Jump Tech Block"),
                    queue_gap_base_ids,
                    tech_blocked_base_ids: locked_governor_recommendations
                        .iter()
                        .map(|(id, _, _, _)| *id)
                        .collect(),
                    governor_warnings_heading_text: "Governor Warnings",
                    governor_warnings,
                    logistics_panel,
                    diplomacy_panel,
                    social_engineering_panel,
                }
            })
            .collect()
    }

    pub fn faction_logistics_panel_state(
        &self,
        owner: usize,
        filter: LogisticsRouteFilter,
        sort: LogisticsRouteSort,
    ) -> FactionLogisticsPanelState {
        let alert_lines = self.faction_logistics_alerts(owner);
        let route_rows = self.filtered_convoy_route_display_rows(owner, filter, sort);
        let total_routes = self.faction_convoy_route_summaries(owner).len();
        let repairable_count = route_rows.iter().filter(|row| row.can_repair).count();
        let route_opportunities = self.convoy_route_opportunities(owner, filter);
        let hub_rows = self.faction_convoy_hub_display_rows(owner);
        let saturated_count = self.saturated_convoy_base_ids(owner).len();
        let costly_count = self
            .faction_base_ids_for_focus(owner, BaseFocusFilter::Logistics)
            .len();
        let collapsing_count = self.convoy_collapsing_base_ids(owner).len();

        FactionLogisticsPanelState {
            alerts_heading_text: "Logistics Alerts".to_string(),
            routes_heading_text: "Convoy Routes".to_string(),
            route_opportunities_heading_text: "Route Opportunities".to_string(),
            hubs_heading_text: "Convoy Hubs".to_string(),
            alert_lines,
            jump_saturated_action: Self::logistics_panel_action_state(
                "Jump Saturated",
                saturated_count,
            ),
            jump_costly_logistics_action: Self::logistics_panel_action_state(
                "Jump Costly Logistics",
                costly_count,
            ),
            jump_collapsing_action: Self::logistics_panel_action_state(
                "Jump Collapsing",
                collapsing_count,
            ),
            jump_worst_route_action: Self::logistics_panel_action_state(
                "Jump Worst Route",
                total_routes,
            ),
            repair_filtered_action: Self::logistics_panel_action_state(
                "Repair Filtered",
                repairable_count,
            ),
            remove_collapsing_action: Self::logistics_panel_action_state(
                "Remove Collapsing",
                self.faction_collapsing_route_count(owner),
            ),
            filtered_count_text: format!("{} shown / {} total", route_rows.len(), total_routes),
            route_rows,
            route_opportunities,
            hub_rows,
        }
    }

    pub fn faction_diplomacy_panel_state(
        &self,
        owner: usize,
        player_id: usize,
    ) -> DiplomacyPanelDisplayState {
        let mut relations = Vec::new();

        if owner != player_id {
            let other_faction = &self.factions[owner];
            let name = &other_faction.name;
            let leader = presentation::faction_leader_name(name).unwrap_or("Unknown");
            let color = presentation::faction_color_hex(name).unwrap_or("#d0d0d0");
            let relation = &self.relations[player_id][owner];
            let status = relation.status;
            let attitude = relation.attitude;

            relations.push(DiplomacyRelationDisplayRowState {
                faction_id: owner,
                faction_name: name.clone(),
                leader_name: leader.to_string(),
                status_text: presentation::diplomacy_status_text(status).to_string(),
                attitude_text: presentation::diplomacy_attitude_text(attitude).to_string(),
                color_hex: color.to_string(),
                status_color_hex: presentation::diplomacy_status_color_hex(status).to_string(),
                can_sign_treaty: status == DiplomacyStatus::Truce,
                can_sign_pact: status == DiplomacyStatus::Treaty,
                can_declare_war: status != DiplomacyStatus::War,
                can_offer_truce: status == DiplomacyStatus::War,
            });
        } else {
            for other_faction in self.non_native_factions() {
                let other_id = other_faction.id;
                if other_id == player_id {
                    continue;
                }

                let name = &other_faction.name;
                let leader = presentation::faction_leader_name(name).unwrap_or("Unknown");
                let color = presentation::faction_color_hex(name).unwrap_or("#d0d0d0");
                let relation = &self.relations[player_id][other_id];
                let status = relation.status;
                let attitude = relation.attitude;

                relations.push(DiplomacyRelationDisplayRowState {
                    faction_id: other_id,
                    faction_name: name.clone(),
                    leader_name: leader.to_string(),
                    status_text: presentation::diplomacy_status_text(status).to_string(),
                    attitude_text: presentation::diplomacy_attitude_text(attitude).to_string(),
                    color_hex: color.to_string(),
                    status_color_hex: presentation::diplomacy_status_color_hex(status).to_string(),
                    can_sign_treaty: status == DiplomacyStatus::Truce,
                    can_sign_pact: status == DiplomacyStatus::Treaty,
                    can_declare_war: status != DiplomacyStatus::War,
                    can_offer_truce: status == DiplomacyStatus::War,
                });
            }
        }

        DiplomacyPanelDisplayState {
            heading_text: "DIPLOMACY".to_string(),
            relations_heading_text: if owner == player_id {
                "Global Relations"
            } else {
                "Relationship"
            }
            .to_string(),
            relations,
        }
    }

    pub fn social_engineering_display_state(&self, owner: usize) -> SocialEngineeringDisplayState {
        let faction = &self.factions[owner];
        let known_tech_ids: HashSet<_> =
            faction.known_techs.iter().map(|t| t.content_id()).collect();
        let tech_tree = crate::technology_tree::TechnologyTree::new();

        let mut categories = Vec::new();

        // Helper to check if an option is enabled
        let is_unlocked = |option_id: &str| {
            if option_id == "frontier"
                || option_id == "simple"
                || option_id == "survival"
                || option_id == "none"
            {
                return true;
            }
            tech_tree.all_technologies().iter().any(|tech| {
                known_tech_ids.contains(tech.id.as_str())
                    && tech
                        .enables
                        .social_engineering
                        .contains(&option_id.to_string())
            })
        };

        use crate::model::{Economics, FutureSociety, Politics, Values};

        // Politics
        categories.push(SocialEngineeringCategoryState {
            name: "Politics",
            options: vec![
                self.politics_option(
                    Politics::Frontier,
                    "Frontier",
                    "None",
                    true,
                    faction.social_engineering.politics == Politics::Frontier,
                ),
                self.politics_option(
                    Politics::Police,
                    "Police",
                    "+2 Support, +2 Police, -2 Efficiency",
                    is_unlocked("police"),
                    faction.social_engineering.politics == Politics::Police,
                ),
                self.politics_option(
                    Politics::Democratic,
                    "Democratic",
                    "+2 Growth, +2 Efficiency, -2 Support",
                    is_unlocked("democratic"),
                    faction.social_engineering.politics == Politics::Democratic,
                ),
                self.politics_option(
                    Politics::Fundamentalist,
                    "Fundamentalist",
                    "+2 Morale, +2 Probe, -2 Research",
                    is_unlocked("fundamentalist"),
                    faction.social_engineering.politics == Politics::Fundamentalist,
                ),
            ],
        });

        // Economics
        categories.push(SocialEngineeringCategoryState {
            name: "Economics",
            options: vec![
                self.economics_option(
                    Economics::Simple,
                    "Simple",
                    "None",
                    true,
                    faction.social_engineering.economics == Economics::Simple,
                ),
                self.economics_option(
                    Economics::FreeMarket,
                    "Free Market",
                    "+2 Economy, +2 Efficiency, -2 Planet",
                    is_unlocked("free_market"),
                    faction.social_engineering.economics == Economics::FreeMarket,
                ),
                self.economics_option(
                    Economics::Planned,
                    "Planned",
                    "+1 Industry, +1 Growth, -2 Efficiency",
                    is_unlocked("planned"),
                    faction.social_engineering.economics == Economics::Planned,
                ),
                self.economics_option(
                    Economics::Green,
                    "Green",
                    "+2 Efficiency, +1 Planet, -2 Growth",
                    is_unlocked("green"),
                    faction.social_engineering.economics == Economics::Green,
                ),
            ],
        });

        // Values
        categories.push(SocialEngineeringCategoryState {
            name: "Values",
            options: vec![
                self.values_option(
                    Values::Survival,
                    "Survival",
                    "None",
                    true,
                    faction.social_engineering.values == Values::Survival,
                ),
                self.values_option(
                    Values::Wealth,
                    "Wealth",
                    "+1 Economy, +1 Industry, -1 Morale",
                    is_unlocked("wealth"),
                    faction.social_engineering.values == Values::Wealth,
                ),
                self.values_option(
                    Values::Knowledge,
                    "Knowledge",
                    "+2 Research, +1 Efficiency, -1 Probe",
                    is_unlocked("knowledge"),
                    faction.social_engineering.values == Values::Knowledge,
                ),
                self.values_option(
                    Values::Power,
                    "Power",
                    "+2 Morale, +1 Support, -1 Industry",
                    is_unlocked("power"),
                    faction.social_engineering.values == Values::Power,
                ),
            ],
        });

        // Future Society
        categories.push(SocialEngineeringCategoryState {
            name: "Future Society",
            options: vec![
                self.future_option(
                    FutureSociety::None,
                    "None",
                    "None",
                    true,
                    faction.social_engineering.future == FutureSociety::None,
                ),
                self.future_option(
                    FutureSociety::Cybernetic,
                    "Cybernetic",
                    "+2 Efficiency, +2 Research, +1 Planet, -1 Police",
                    is_unlocked("cybernetic"),
                    faction.social_engineering.future == FutureSociety::Cybernetic,
                ),
                self.future_option(
                    FutureSociety::ThoughtControl,
                    "Thought Control",
                    "+2 Police, +2 Morale, +2 Probe, -1 Research",
                    is_unlocked("thought_control"),
                    faction.social_engineering.future == FutureSociety::ThoughtControl,
                ),
                self.future_option(
                    FutureSociety::Eudaimonic,
                    "Eudaimonic",
                    "+2 Economy, +2 Efficiency, +2 Growth, -1 Industry",
                    is_unlocked("eudaimonic"),
                    faction.social_engineering.future == FutureSociety::Eudaimonic,
                ),
            ],
        });

        SocialEngineeringDisplayState {
            heading_text: "SOCIAL ENGINEERING".to_string(),
            categories,
        }
    }

    fn politics_option(
        &self,
        politics: crate::model::Politics,
        choice_text: &str,
        modifiers_text: &str,
        enabled: bool,
        selected: bool,
    ) -> SocialEngineeringCategoryOptionState {
        SocialEngineeringCategoryOptionState {
            choice_text: choice_text.to_string(),
            modifiers_text: modifiers_text.to_string(),
            enabled,
            selected,
            politics: Some(politics),
            economics: None,
            values: None,
            future: None,
        }
    }

    fn economics_option(
        &self,
        economics: crate::model::Economics,
        choice_text: &str,
        modifiers_text: &str,
        enabled: bool,
        selected: bool,
    ) -> SocialEngineeringCategoryOptionState {
        SocialEngineeringCategoryOptionState {
            choice_text: choice_text.to_string(),
            modifiers_text: modifiers_text.to_string(),
            enabled,
            selected,
            politics: None,
            economics: Some(economics),
            values: None,
            future: None,
        }
    }

    fn values_option(
        &self,
        values: crate::model::Values,
        choice_text: &str,
        modifiers_text: &str,
        enabled: bool,
        selected: bool,
    ) -> SocialEngineeringCategoryOptionState {
        SocialEngineeringCategoryOptionState {
            choice_text: choice_text.to_string(),
            modifiers_text: modifiers_text.to_string(),
            enabled,
            selected,
            politics: None,
            economics: None,
            values: Some(values),
            future: None,
        }
    }

    fn future_option(
        &self,
        future: crate::model::FutureSociety,
        choice_text: &str,
        modifiers_text: &str,
        enabled: bool,
        selected: bool,
    ) -> SocialEngineeringCategoryOptionState {
        SocialEngineeringCategoryOptionState {
            choice_text: choice_text.to_string(),
            modifiers_text: modifiers_text.to_string(),
            enabled,
            selected,
            politics: None,
            economics: None,
            values: None,
            future: Some(future),
        }
    }

    pub fn secret_project_registry_display_state(&self) -> SecretProjectRegistryDisplayState {
        let mut projects = Vec::new();

        for project in crate::model::SecretProject::all() {
            let item = match project {
                crate::model::SecretProject::WeatherPattern => ProductionItem::WeatherPattern,
                crate::model::SecretProject::ClinicalImmortality => {
                    ProductionItem::ClinicalImmortality
                }
                crate::model::SecretProject::EmpathGuild => ProductionItem::EmpathGuild,
                crate::model::SecretProject::OrbitalElevator => ProductionItem::OrbitalElevator,
                crate::model::SecretProject::ManifoldDrive => ProductionItem::ManifoldDrive,
                crate::model::SecretProject::SingularityContainment => {
                    ProductionItem::SingularityContainment
                }
                crate::model::SecretProject::BlackHoleHarvester => {
                    ProductionItem::BlackHoleHarvester
                }
            };

            let built_info = self
                .built_secret_projects
                .iter()
                .find(|(p, _)| *p == project);

            let (owner_name, owner_color, status) = if let Some((_, owner)) = built_info {
                let faction = &self.factions[*owner];
                (
                    faction.name.clone(),
                    presentation::faction_color_hex(&faction.name)
                        .unwrap_or("#d0d0d0")
                        .to_string(),
                    "Built".to_string(),
                )
            } else {
                (
                    "-".to_string(),
                    "#888888".to_string(),
                    "Available".to_string(),
                )
            };

            let effects_text = presentation::production_tooltip_summary(item);
            // Tooltip summary includes role summary and effect: parts.
            // We want just the effect part for the registry row.
            let effect_only = effects_text
                .lines()
                .filter(|l| l.starts_with("Effect:"))
                .map(|l| l.replace("Effect: ", ""))
                .collect::<Vec<_>>()
                .join(", ");

            projects.push(SecretProjectRegistryRowState {
                project_name: presentation::production_name(item).to_string(),
                owner_name,
                owner_color_hex: owner_color,
                status_text: status,
                effects_text: if effect_only.is_empty() {
                    presentation::production_role_summary(item).to_string()
                } else {
                    effect_only
                },
            });
        }

        SecretProjectRegistryDisplayState {
            heading_text: "SECRET PROJECTS".to_string(),
            projects,
        }
    }

    pub fn convoy_overlay_status_at(
        &self,
        owner: usize,
        x: usize,
        y: usize,
    ) -> ConvoyOverlayStatus {
        let mut best = ConvoyOverlayStatus::None;

        for route in &self.convoy_routes {
            let Some(base_a) = self.base(route.base_a_id) else {
                continue;
            };
            let Some(base_b) = self.base(route.base_b_id) else {
                continue;
            };
            if !self.tile_explored_by_owner(base_a.x, base_a.y, owner)
                || !self.tile_explored_by_owner(base_b.x, base_b.y, owner)
            {
                continue;
            }
            if !Self::tile_on_convoy_path(x, y, base_a.x, base_a.y, base_b.x, base_b.y) {
                continue;
            }

            let protected = self.base_convoy_security(base_a.id) > 0
                || self.base_convoy_security(base_b.id) > 0;
            let status = self
                .convoy_route_status_for_base(base_a.id)
                .into_iter()
                .find(|(other_id, kind, _, _, _)| *other_id == base_b.id && *kind == route.kind)
                .map(|(_, _, disrupted, intercepted, integrity)| {
                    Self::convoy_overlay_status_for_flags(
                        disrupted,
                        intercepted,
                        integrity,
                        protected,
                    )
                })
                .unwrap_or(ConvoyOverlayStatus::None);

            if status.priority() > best.priority() {
                best = status;
            }
        }

        best
    }

    pub fn convoy_overlay_glyph_at(
        &self,
        owner: usize,
        x: usize,
        y: usize,
    ) -> Option<&'static str> {
        match self.convoy_overlay_status_at(owner, x, y) {
            ConvoyOverlayStatus::Collapsing => Some("X"),
            ConvoyOverlayStatus::Intercepted => Some("!"),
            ConvoyOverlayStatus::Disrupted => Some("="),
            ConvoyOverlayStatus::Protected => Some("+"),
            ConvoyOverlayStatus::Active => Some("-"),
            ConvoyOverlayStatus::None => None,
        }
    }

    pub fn convoy_overlay_lines(&self, owner: usize) -> Vec<ConvoyOverlayLine> {
        let mut lines = Vec::new();
        for route in &self.convoy_routes {
            let Some(base_a) = self.base(route.base_a_id) else {
                continue;
            };
            let Some(base_b) = self.base(route.base_b_id) else {
                continue;
            };
            if !self.tile_explored_by_owner(base_a.x, base_a.y, owner)
                || !self.tile_explored_by_owner(base_b.x, base_b.y, owner)
            {
                continue;
            }
            let protected = self.base_convoy_security(base_a.id) > 0
                || self.base_convoy_security(base_b.id) > 0;
            let status = self
                .convoy_route_status_for_base(route.base_a_id)
                .into_iter()
                .find(|(other_id, kind, _, _, _)| {
                    *other_id == route.base_b_id && *kind == route.kind
                })
                .map(|(_, _, disrupted, intercepted, integrity)| {
                    Self::convoy_overlay_status_for_flags(
                        disrupted,
                        intercepted,
                        integrity,
                        protected,
                    )
                })
                .unwrap_or(ConvoyOverlayStatus::Active);
            lines.push(ConvoyOverlayLine {
                start_x: base_a.x,
                start_y: base_a.y,
                end_x: base_b.x,
                end_y: base_b.y,
                status,
            });
        }
        lines
    }

    fn convoy_summary_priority(summary: &crate::ConvoyRouteSummary) -> (i32, i32, i32, i32) {
        (
            if summary.intercepted && summary.integrity <= 1 {
                4
            } else if summary.intercepted {
                3
            } else if summary.disrupted {
                2
            } else if summary.protected {
                1
            } else {
                0
            },
            -(summary.integrity as i32),
            if summary.intercepted { 1 } else { 0 },
            if summary.disrupted { 1 } else { 0 },
        )
    }

    pub fn player_fallback_moves(&self) -> Vec<(usize, usize, usize)> {
        self.player_damaged_unit_ids()
            .into_iter()
            .filter_map(|unit_id| {
                self.safest_player_fallback_tile(unit_id)
                    .map(|(x, y)| (unit_id, x, y))
            })
            .collect()
    }

    pub fn apply_player_fallback_moves(&mut self) -> usize {
        let moves = self.player_fallback_moves();
        if moves.is_empty() {
            self.push_log("No damaged player units currently have a fallback step.".to_string());
            return 0;
        }

        let mut moved = 0;
        for (unit_id, x, y) in moves {
            if self.move_unit_to(unit_id, x, y).is_ok() {
                moved += 1;
            }
        }

        self.push_log(format!(
            "Operations assisted {moved} damaged units toward safety."
        ));
        moved
    }

    pub fn safest_player_fallback_tile(&self, unit_id: usize) -> Option<(usize, usize)> {
        let unit = self.unit(unit_id)?;
        if unit.owner != self.player_owner() {
            return None;
        }
        self.safest_fallback_step(unit.owner, unit.x, unit.y)
    }

    pub fn most_damaged_player_unit_id(&self) -> Option<usize> {
        self.player_damaged_unit_ids().into_iter().next()
    }

    pub fn most_unrested_player_base_id(&self) -> Option<usize> {
        self.stressed_player_base_ids().into_iter().next()
    }

    pub fn player_damaged_unit_ids(&self) -> Vec<usize> {
        let owner = self.player_owner();
        let mut ids: Vec<(usize, i32, i32)> = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
            .filter_map(|unit| {
                let max_hp = content::unit_base_hp(unit.kind.clone());
                let missing_hp = max_hp - unit.hp;
                if missing_hp > 0 {
                    Some((unit.id, missing_hp, unit.hp))
                } else {
                    None
                }
            })
            .collect();
        ids.sort_by_key(|(id, missing_hp, hp)| (-*missing_hp, *hp, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn next_unit_cycle_target(
        &self,
        unit_ids: &[usize],
        current_unit_id: Option<usize>,
    ) -> Option<usize> {
        if unit_ids.is_empty() {
            return None;
        }

        Some(match current_unit_id {
            Some(current) => unit_ids
                .iter()
                .copied()
                .find(|id| *id > current)
                .unwrap_or(unit_ids[0]),
            None => unit_ids[0],
        })
    }

    pub fn next_damaged_player_unit_id(&self, current_unit_id: Option<usize>) -> Option<usize> {
        let unit_ids = self.player_damaged_unit_ids();
        self.next_unit_cycle_target(&unit_ids, current_unit_id)
    }

    pub fn stressed_player_base_ids(&self) -> Vec<usize> {
        let owner = self.player_owner();
        let mut ids: Vec<(usize, i32, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| (base.id, self.base_unrest(base.id), base.population))
            .filter(|(_, unrest, _)| *unrest > 0)
            .collect();
        ids.sort_by_key(|(id, unrest, population)| (-*unrest, -*population, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn next_stressed_player_base_id(&self, current_base_id: Option<usize>) -> Option<usize> {
        let base_ids = self.stressed_player_base_ids();
        self.next_base_cycle_target(&base_ids, current_base_id)
    }

    pub fn recovering_garrison_unit_ids(&self) -> Vec<usize> {
        let owner = self.player_owner();
        let mut ids: Vec<(usize, i32, i32)> = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
            .filter_map(|unit| {
                let max_hp = content::unit_base_hp(unit.kind.clone());
                let missing_hp = max_hp - unit.hp;
                if missing_hp <= 0 {
                    return None;
                }
                let on_base = self
                    .tile(unit.x, unit.y)
                    .and_then(|tile| tile.base)
                    .and_then(|base_id| self.base(base_id))
                    .map(|base| base.owner == owner)
                    .unwrap_or(false);
                if on_base {
                    Some((unit.id, missing_hp, unit.experience))
                } else {
                    None
                }
            })
            .collect();
        ids.sort_by_key(|(id, missing_hp, exp)| (-*missing_hp, -*exp, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn next_recovering_garrison_unit_id(
        &self,
        current_unit_id: Option<usize>,
    ) -> Option<usize> {
        let unit_ids = self.recovering_garrison_unit_ids();
        self.next_unit_cycle_target(&unit_ids, current_unit_id)
    }

    pub fn recovering_garrison_base_ids(&self) -> Vec<usize> {
        let owner = self.player_owner();
        let mut ids: Vec<(usize, usize, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| {
                (
                    base.id,
                    self.damaged_garrison_count(base.id),
                    self.base_unrest(base.id),
                )
            })
            .filter(|(_, damaged, _)| *damaged > 0)
            .collect();
        ids.sort_by_key(|(id, damaged, unrest)| (-(*damaged as i32), -*unrest, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn next_recovering_base_id(&self, current_base_id: Option<usize>) -> Option<usize> {
        let base_ids = self.recovering_garrison_base_ids();
        self.next_base_cycle_target(&base_ids, current_base_id)
    }

    pub fn most_recovering_garrison_base_id(&self) -> Option<usize> {
        self.recovering_garrison_base_ids().into_iter().next()
    }

    pub fn player_operations_focus_state(
        &self,
        current_unit_id: Option<usize>,
        current_base_id: Option<usize>,
    ) -> PlayerOperationsFocusState {
        let owner = self.player_owner();
        let damaged_unit_ids = self.player_damaged_unit_ids();
        let stressed_base_ids = self.stressed_player_base_ids();
        let recovering_base_ids = self.recovering_garrison_base_ids();
        let recovering_garrison_unit_ids = self.recovering_garrison_unit_ids();
        let current_research_unlock_base_ids =
            self.current_research_unlock_pressure_base_ids(owner);

        PlayerOperationsFocusState {
            damaged_unit_count: damaged_unit_ids.len(),
            most_damaged_unit_id: damaged_unit_ids.first().copied(),
            next_damaged_unit_id: self.next_unit_cycle_target(&damaged_unit_ids, current_unit_id),
            stressed_base_count: stressed_base_ids.len(),
            most_unrested_base_id: stressed_base_ids.first().copied(),
            next_stressed_base_id: self.next_base_cycle_target(&stressed_base_ids, current_base_id),
            recovering_base_count: recovering_base_ids.len(),
            most_recovering_garrison_base_id: recovering_base_ids.first().copied(),
            next_recovering_base_id: self
                .next_base_cycle_target(&recovering_base_ids, current_base_id),
            recovering_garrison_unit_count: recovering_garrison_unit_ids.len(),
            next_recovering_garrison_unit_id: self
                .next_unit_cycle_target(&recovering_garrison_unit_ids, current_unit_id),
            current_research_unlock_base_count: current_research_unlock_base_ids.len(),
            current_research_unlock_focus_base_id: self
                .current_research_unlock_focus_base_id(owner, current_base_id),
        }
    }

    pub fn recommended_governor_change_count(&self, owner: usize) -> usize {
        self.bases_for(owner)
            .into_iter()
            .filter(|base| self.recommended_governor_mode_for_base(base.id) != base.governor_mode)
            .count()
    }

    pub fn player_operations_dashboard_state(
        &self,
        current_unit_id: Option<usize>,
        current_base_id: Option<usize>,
        max_steps: usize,
    ) -> PlayerOperationsDashboardState {
        let owner = self.player_owner();
        let focus = self.player_operations_focus_state(current_unit_id, current_base_id);
        let actionable_recovery = self
            .stressed_recovery_base_ids()
            .into_iter()
            .filter(|base_id| {
                !self
                    .base_recovery_plan_items(*base_id, max_steps)
                    .is_empty()
            })
            .count();
        let actionable_defense = self
            .frontier_base_ids()
            .into_iter()
            .filter(|base_id| !self.base_defense_plan_items(*base_id, max_steps).is_empty())
            .count();
        let actionable_economy = self
            .unrest_base_ids(owner)
            .into_iter()
            .filter(|base_id| !self.base_economy_plan_items(*base_id, max_steps).is_empty())
            .count();
        let actionable_empty_queue = self
            .faction_queue_gap_base_ids(owner)
            .into_iter()
            .filter(|base_id| {
                !self
                    .base_governor_queue_items(*base_id, max_steps)
                    .is_empty()
            })
            .count();

        let jump_frontier = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Frontier,
            "Jump Frontier Base",
            current_base_id,
        );
        let jump_economy = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Unrest,
            "Jump Economy Stress",
            current_base_id,
        );
        let jump_logistics = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Logistics,
            "Jump Logistics Stress",
            current_base_id,
        );
        let jump_saturated = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Saturated,
            "Jump Saturated Hub",
            current_base_id,
        );
        let jump_collapsing = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Collapsing,
            "Jump Collapsing Route",
            current_base_id,
        );
        let jump_defense_mode = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Defense,
            "Jump Defense Mode",
            current_base_id,
        );
        let jump_logistics_mode = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::LogisticsMode,
            "Jump Logistics Mode",
            current_base_id,
        );
        let jump_recovery_mode = self.base_focus_jump_action_state(
            owner,
            BaseFocusFilter::Recovery,
            "Jump Recovery Mode",
            current_base_id,
        );

        let select_damaged = Self::player_operations_action_state(
            "Select Damaged Unit",
            self.player_damaged_unit_ids().len(),
            PlayerOperationsActionType::SelectDamagedUnit,
        );
        let cycle_damaged = Self::player_operations_action_state(
            "Cycle Damaged Units",
            self.player_damaged_unit_ids().len(),
            PlayerOperationsActionType::CycleDamagedUnits,
        );
        let jump_stressed = Self::player_operations_action_state(
            "Jump To Stressed Base",
            self.stressed_player_base_ids().len(),
            PlayerOperationsActionType::JumpStressedBase,
        );
        let cycle_stressed = Self::player_operations_action_state(
            "Cycle Stressed Bases",
            self.stressed_player_base_ids().len(),
            PlayerOperationsActionType::CycleStressedBases,
        );
        let jump_recovery = Self::player_operations_action_state(
            "Jump To Recovery Base",
            self.recovering_garrison_base_ids().len(),
            PlayerOperationsActionType::JumpRecoveryBase,
        );
        let apply_recovery_base = Self::player_operations_action_state(
            "Apply Recovery Base Plan",
            self.recovering_garrison_base_ids().len(),
            PlayerOperationsActionType::ApplyRecoveryBasePlan,
        );
        let fallback = Self::player_operations_action_state(
            "Fallback All Damaged",
            self.player_fallback_moves().len(),
            PlayerOperationsActionType::FallbackAllDamaged,
        );
        let recovery_all = Self::player_operations_action_state(
            "Apply All Recovery Plans",
            actionable_recovery,
            PlayerOperationsActionType::ApplyAllRecoveryPlans,
        );
        let defense_all = Self::player_operations_action_state(
            "Apply Frontier Defense Plans",
            actionable_defense,
            PlayerOperationsActionType::ApplyFrontierDefensePlans,
        );
        let suggest_governors = Self::player_operations_action_state(
            "Suggest Governors",
            self.recommended_governor_change_count(owner),
            PlayerOperationsActionType::SuggestGovernors,
        );
        let repair_convoys = Self::player_operations_action_state(
            "Repair Convoys",
            self.suggested_convoy_repairs(owner).len(),
            PlayerOperationsActionType::RepairConvoys,
        );
        let rebuild_convoys = Self::player_operations_action_state(
            "Rebuild Convoys",
            self.suggested_convoy_rebuilds(owner).len(),
            PlayerOperationsActionType::RebuildConvoys,
        );
        let assign_escort = Self::player_operations_action_state(
            "Assign Escort Patrols",
            self.suggested_escort_patrol_moves(owner).len(),
            PlayerOperationsActionType::AssignEscortPatrols,
        );
        let economy_all = Self::player_operations_action_state(
            "Apply Economy Plans",
            actionable_economy,
            PlayerOperationsActionType::ApplyEconomyPlans,
        );
        let fill_queues = Self::player_operations_action_state(
            "Fill Empty Queues",
            actionable_empty_queue,
            PlayerOperationsActionType::FillEmptyQueues,
        );
        let jump_research = Self::player_operations_action_state(
            "Jump Research Unlock",
            self.current_research_unlock_pressure_base_ids(owner).len(),
            PlayerOperationsActionType::JumpResearchUnlock,
        );
        let cycle_recovery = Self::player_operations_action_state(
            "Cycle Recovery Bases",
            self.recovering_garrison_base_ids().len(),
            PlayerOperationsActionType::CycleRecoveryBases,
        );
        let select_recovering = Self::player_operations_action_state(
            "Select Recovering Garrison",
            self.recovering_garrison_unit_ids().len(),
            PlayerOperationsActionType::SelectRecoveringGarrison,
        );

        PlayerOperationsDashboardState {
            heading_text: "Operations Advice".to_string(),
            advice_lines: self.player_operations_advice(),
            focus,
            jump_actions: vec![
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Frontier,
                    button_text: jump_frontier.button_text,
                    enabled: jump_frontier.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Unrest,
                    button_text: jump_economy.button_text,
                    enabled: jump_economy.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Logistics,
                    button_text: jump_logistics.button_text,
                    enabled: jump_logistics.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Saturated,
                    button_text: jump_saturated.button_text,
                    enabled: jump_saturated.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Collapsing,
                    button_text: jump_collapsing.button_text,
                    enabled: jump_collapsing.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Defense,
                    button_text: jump_defense_mode.button_text,
                    enabled: jump_defense_mode.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::LogisticsMode,
                    button_text: jump_logistics_mode.button_text,
                    enabled: jump_logistics_mode.enabled,
                },
                PlayerOperationsJumpActionState {
                    filter: BaseFocusFilter::Recovery,
                    button_text: jump_recovery_mode.button_text,
                    enabled: jump_recovery_mode.enabled,
                },
            ],
            bulk_actions: vec![
                PlayerOperationsActionState {
                    label_text: select_damaged.label_text,
                    button_text: select_damaged.button_text,
                    available_count: select_damaged.available_count,
                    enabled: select_damaged.enabled,
                    action_type: PlayerOperationsActionType::SelectDamagedUnit,
                },
                PlayerOperationsActionState {
                    label_text: fallback.label_text,
                    button_text: fallback.button_text,
                    available_count: fallback.available_count,
                    enabled: fallback.enabled,
                    action_type: PlayerOperationsActionType::FallbackAllDamaged,
                },
                PlayerOperationsActionState {
                    label_text: cycle_damaged.label_text,
                    button_text: cycle_damaged.button_text,
                    available_count: cycle_damaged.available_count,
                    enabled: cycle_damaged.enabled,
                    action_type: PlayerOperationsActionType::CycleDamagedUnits,
                },
                PlayerOperationsActionState {
                    label_text: jump_stressed.label_text,
                    button_text: jump_stressed.button_text,
                    available_count: jump_stressed.available_count,
                    enabled: jump_stressed.enabled,
                    action_type: PlayerOperationsActionType::JumpStressedBase,
                },
                PlayerOperationsActionState {
                    label_text: cycle_stressed.label_text,
                    button_text: cycle_stressed.button_text,
                    available_count: cycle_stressed.available_count,
                    enabled: cycle_stressed.enabled,
                    action_type: PlayerOperationsActionType::CycleStressedBases,
                },
                PlayerOperationsActionState {
                    label_text: jump_recovery.label_text,
                    button_text: jump_recovery.button_text,
                    available_count: jump_recovery.available_count,
                    enabled: jump_recovery.enabled,
                    action_type: PlayerOperationsActionType::JumpRecoveryBase,
                },
                PlayerOperationsActionState {
                    label_text: apply_recovery_base.label_text,
                    button_text: apply_recovery_base.button_text,
                    available_count: apply_recovery_base.available_count,
                    enabled: apply_recovery_base.enabled,
                    action_type: PlayerOperationsActionType::ApplyRecoveryBasePlan,
                },
                PlayerOperationsActionState {
                    label_text: recovery_all.label_text,
                    button_text: recovery_all.button_text,
                    available_count: recovery_all.available_count,
                    enabled: recovery_all.enabled,
                    action_type: PlayerOperationsActionType::ApplyAllRecoveryPlans,
                },
                PlayerOperationsActionState {
                    label_text: defense_all.label_text,
                    button_text: defense_all.button_text,
                    available_count: defense_all.available_count,
                    enabled: defense_all.enabled,
                    action_type: PlayerOperationsActionType::ApplyFrontierDefensePlans,
                },
                PlayerOperationsActionState {
                    label_text: suggest_governors.label_text,
                    button_text: suggest_governors.button_text,
                    available_count: suggest_governors.available_count,
                    enabled: suggest_governors.enabled,
                    action_type: PlayerOperationsActionType::SuggestGovernors,
                },
                PlayerOperationsActionState {
                    label_text: repair_convoys.label_text,
                    button_text: repair_convoys.button_text,
                    available_count: repair_convoys.available_count,
                    enabled: repair_convoys.enabled,
                    action_type: PlayerOperationsActionType::RepairConvoys,
                },
                PlayerOperationsActionState {
                    label_text: rebuild_convoys.label_text,
                    button_text: rebuild_convoys.button_text,
                    available_count: rebuild_convoys.available_count,
                    enabled: rebuild_convoys.enabled,
                    action_type: PlayerOperationsActionType::RebuildConvoys,
                },
                PlayerOperationsActionState {
                    label_text: assign_escort.label_text,
                    button_text: assign_escort.button_text,
                    available_count: assign_escort.available_count,
                    enabled: assign_escort.enabled,
                    action_type: PlayerOperationsActionType::AssignEscortPatrols,
                },
                PlayerOperationsActionState {
                    label_text: economy_all.label_text,
                    button_text: economy_all.button_text,
                    available_count: economy_all.available_count,
                    enabled: economy_all.enabled,
                    action_type: PlayerOperationsActionType::ApplyEconomyPlans,
                },
                PlayerOperationsActionState {
                    label_text: fill_queues.label_text,
                    button_text: fill_queues.button_text,
                    available_count: fill_queues.available_count,
                    enabled: fill_queues.enabled,
                    action_type: PlayerOperationsActionType::FillEmptyQueues,
                },
                PlayerOperationsActionState {
                    label_text: jump_research.label_text,
                    button_text: jump_research.button_text,
                    available_count: jump_research.available_count,
                    enabled: jump_research.enabled,
                    action_type: PlayerOperationsActionType::JumpResearchUnlock,
                },
                PlayerOperationsActionState {
                    label_text: cycle_recovery.label_text,
                    button_text: cycle_recovery.button_text,
                    available_count: cycle_recovery.available_count,
                    enabled: cycle_recovery.enabled,
                    action_type: PlayerOperationsActionType::CycleRecoveryBases,
                },
                PlayerOperationsActionState {
                    label_text: select_recovering.label_text,
                    button_text: select_recovering.button_text,
                    available_count: select_recovering.available_count,
                    enabled: select_recovering.enabled,
                    action_type: PlayerOperationsActionType::SelectRecoveringGarrison,
                },
            ],
        }
    }

    pub fn damaged_garrison_count_for_base(&self, base_id: usize) -> usize {
        self.damaged_garrison_count(base_id)
    }

    pub fn base_local_military_pressure(&self, base_id: usize) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        ai::military_pressure_near_base(self, base.x, base.y, base.owner)
    }

    pub fn base_local_psi_pressure(&self, base_id: usize) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        let raw_pressure = ai::psi_pressure_near_base(self, base.x, base.y, base.owner);
        let facility_psi_support: i32 = base
            .facilities
            .iter()
            .copied()
            .map(content::facility_psi_support_bonus)
            .sum();
        let project_psi_support: i32 = self
            .built_secret_projects
            .iter()
            .filter(|(p, o)| *o == base.owner && matches!(p, SecretProject::EmpathGuild))
            .count() as i32
            * 2;
        (raw_pressure - facility_psi_support - project_psi_support).max(0)
    }

    pub fn base_area_role(&self, base_id: usize) -> BaseAreaRole {
        let military = self.base_local_military_pressure(base_id);
        let psi = self.base_local_psi_pressure(base_id);
        if military >= 2 && psi >= 2 {
            BaseAreaRole::Warzone
        } else if psi >= 2 {
            BaseAreaRole::PsiFrontier
        } else if military >= 2 {
            BaseAreaRole::Frontier
        } else {
            BaseAreaRole::RearArea
        }
    }

    pub fn base_area_role_counts(&self, owner: usize) -> (usize, usize, usize, usize) {
        let mut rear = 0;
        let mut frontier = 0;
        let mut psi_frontier = 0;
        let mut warzone = 0;
        for base in self.bases.iter().filter(|base| base.owner == owner) {
            match self.base_area_role(base.id) {
                BaseAreaRole::RearArea => rear += 1,
                BaseAreaRole::Frontier => frontier += 1,
                BaseAreaRole::PsiFrontier => psi_frontier += 1,
                BaseAreaRole::Warzone => warzone += 1,
            }
        }
        (rear, frontier, psi_frontier, warzone)
    }

    pub fn base_governor_plan(&self, base_id: usize) -> Vec<GovernorPlanStep> {
        let Some(base) = self.base(base_id) else {
            return Vec::new();
        };
        let owner = base.owner;
        let unrest = self.base_unrest(base_id);
        let damaged_garrisons = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
            .filter(|unit| {
                let max_hp = content::unit_base_hp(unit.kind.clone());
                let on_base = self
                    .tile(unit.x, unit.y)
                    .and_then(|tile| tile.base)
                    .and_then(|id| self.base(id))
                    .map(|b| b.id == base_id)
                    .unwrap_or(false);
                on_base && unit.hp < max_hp
            })
            .count();
        let (_, unit_upkeep, _) = self.faction_upkeep(owner);
        let military_pressure = ai::military_pressure_near_base(self, base.x, base.y, owner);
        let psi_pressure = self.base_local_psi_pressure(base.id);
        let attack_bias = content::ai_attack_bias(owner);
        let yields = self.effective_base_yields(base_id).unwrap_or_default();
        let food_margin = self.base_food_margin(base.id).unwrap_or(1);
        let mut plan = Vec::new();

        if unrest > 0
            && !base.facilities.contains(&Facility::RecreationCommons)
            && self.is_production_available(owner, ProductionItem::RecreationCommons)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::RecreationCommons,
                priority: 100,
                reason:
                    "Unrest is reducing output here. Recreation Commons will stabilize the base."
                        .to_string(),
            });
        }

        if damaged_garrisons > 0
            && !base.facilities.contains(&Facility::FieldHospital)
            && self.is_production_available(owner, ProductionItem::FieldHospital)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::FieldHospital,
                priority: 90,
                reason:
                    "Damaged garrisons are rotating through this base. Field Hospital will speed recovery."
                        .to_string(),
            });
        }

        if (unrest > 0 || yields.energy <= yields.minerals)
            && base.facilities.contains(&Facility::RecreationCommons)
            && !base.facilities.contains(&Facility::HologramTheatre)
            && self.is_production_available(owner, ProductionItem::HologramTheatre)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::HologramTheatre,
                priority: 92,
                reason:
                    "Morale infrastructure is already online here. Hologram Theatre will deepen unrest control and improve energy flow."
                        .to_string(),
            });
        }

        if damaged_garrisons > 0
            && base.facilities.contains(&Facility::FieldHospital)
            && !base.facilities.contains(&Facility::ResearchHospital)
            && self.is_production_available(owner, ProductionItem::ResearchHospital)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::ResearchHospital,
                priority: 91,
                reason:
                    "Recovery infrastructure is already active here. Research Hospital will accelerate healing, growth, and scientific output."
                        .to_string(),
            });
        }

        if psi_pressure >= 2 && self.is_production_available(owner, ProductionItem::PsiSentinel) {
            plan.push(GovernorPlanStep {
                item: ProductionItem::PsiSentinel,
                priority: 89,
                reason:
                    "Psionic threats are nearby. Psi Sentinel will harden this base against native and psi pressure."
                        .to_string(),
            });
        }

        if psi_pressure >= 1
            && !base.facilities.contains(&Facility::PsiBeacon)
            && self.is_production_available(owner, ProductionItem::PsiBeacon)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::PsiBeacon,
                priority: 88,
                reason:
                    "Psi pressure is building here. Psi Beacon will strengthen psionic security and local resilience."
                        .to_string(),
            });
        }

        if (military_pressure >= 1 || psi_pressure >= 1 || damaged_garrisons > 0)
            && (base.facilities.contains(&Facility::PsiBeacon)
                || base.facilities.contains(&Facility::MilitaryAcademy)
                || base.facilities.contains(&Facility::FieldHospital))
            && !base.facilities.contains(&Facility::BioenhancementCenter)
            && self.is_production_available(owner, ProductionItem::BioenhancementCenter)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::BioenhancementCenter,
                priority: 87,
                reason:
                    "This base already has frontline support infrastructure. Bioenhancement Center will improve training, psi resilience, and sustained readiness."
                        .to_string(),
            });
        }

        if unit_upkeep > 0
            && !base.facilities.contains(&Facility::CommandCenter)
            && self.is_production_available(owner, ProductionItem::CommandCenter)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::CommandCenter,
                priority: 80,
                reason:
                    "Support pressure is rising. Command Center will improve support and training."
                        .to_string(),
            });
        }

        if yields.energy <= yields.minerals
            && !base.facilities.contains(&Facility::NetworkNode)
            && self.is_production_available(owner, ProductionItem::NetworkNode)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::NetworkNode,
                priority: 66,
                reason:
                    "Research and energy output are lagging. Network Node will improve knowledge and energy flow."
                        .to_string(),
            });
        }

        if self.base_potential_trade_links(base_id) >= 1
            && !base.facilities.contains(&Facility::TradeExchange)
            && self.is_production_available(owner, ProductionItem::TradeExchange)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::TradeExchange,
                priority: 65,
                reason:
                    "Nearby friendly bases can support trade here. Trade Exchange will convert that network into stronger energy flow."
                        .to_string(),
            });
        }

        if self.base_potential_trade_links(base_id) >= 1
            && self.base_convoy_escort_count(base_id) == 0
            && self.is_production_available(owner, ProductionItem::EscortSpeeder)
            && (self.base_local_military_pressure(base_id) >= 1
                || self.base_local_psi_pressure(base_id) >= 1
                || self
                    .convoy_route_status_for_base(base_id)
                    .into_iter()
                    .any(|(_, _, disrupted, intercepted, _)| disrupted || intercepted))
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::EscortSpeeder,
                priority: 67,
                reason:
                    "Local convoy lanes are exposed. Escort Speeder will add mobile interception defense."
                        .to_string(),
            });
        }

        if self.base_potential_trade_links(base_id) >= 1
            && !base.facilities.contains(&Facility::PatrolGrid)
            && self.is_production_available(owner, ProductionItem::PatrolGrid)
            && (self.base_local_military_pressure(base_id) >= 1
                || self.base_local_psi_pressure(base_id) >= 1)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::PatrolGrid,
                priority: 66,
                reason:
                    "Convoy routes here need protection. Patrol Grid will increase route capacity and reduce logistics disruption."
                        .to_string(),
            });
        }

        if self.base_potential_trade_links(base_id) >= 1
            && !base.facilities.contains(&Facility::FreightDepot)
            && self.is_production_available(owner, ProductionItem::FreightDepot)
            && (base.facilities.contains(&Facility::TradeExchange)
                || yields.minerals <= yields.energy)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::FreightDepot,
                priority: 64,
                reason:
                    "Friendly bases can support freight flow here. Freight Depot will convert that network into stronger mineral throughput."
                        .to_string(),
            });
        }

        if yields.nutrients <= base.population.max(1) + 1
            && !base.facilities.contains(&Facility::Greenhouse)
            && self.is_production_available(owner, ProductionItem::Greenhouse)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::Greenhouse,
                priority: 65,
                reason:
                    "Food security is thin here. Greenhouse will improve nutrient flow and growth resilience."
                        .to_string(),
            });
        }

        if yields.minerals <= 6
            && !base.facilities.contains(&Facility::MineralRefinery)
            && self.is_production_available(owner, ProductionItem::MineralRefinery)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::MineralRefinery,
                priority: 64,
                reason:
                    "Extraction throughput is thin here. Mineral Refinery will strengthen industrial output."
                        .to_string(),
            });
        }

        if (yields.nutrients <= 6 || yields.minerals <= 6)
            && !base.facilities.contains(&Facility::RecyclingTanks)
            && self.is_production_available(owner, ProductionItem::RecyclingTanks)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::RecyclingTanks,
                priority: 63,
                reason:
                    "Core growth and mineral flow are thin. Recycling Tanks will stabilize early economic output."
                        .to_string(),
            });
        }

        if (food_margin <= 0 || yields.energy <= yields.minerals)
            && base.facilities.contains(&Facility::FieldHospital)
            && !base.facilities.contains(&Facility::ResearchHospital)
            && self.is_production_available(owner, ProductionItem::ResearchHospital)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::ResearchHospital,
                priority: 76,
                reason:
                    "This base needs stronger long-term recovery and research support. Research Hospital will improve healing, growth, and energy output."
                        .to_string(),
            });
        }

        if military_pressure >= 2
            && !base.facilities.contains(&Facility::PerimeterDefense)
            && self.is_production_available(owner, ProductionItem::PerimeterDefense)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::PerimeterDefense,
                priority: 75,
                reason:
                    "Frontline pressure is high. Perimeter Defense will harden this base immediately."
                        .to_string(),
            });
        }

        if military_pressure >= 1
            && !base.facilities.contains(&Facility::SensorArray)
            && self.is_production_available(owner, ProductionItem::SensorArray)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::SensorArray,
                priority: 72,
                reason:
                    "Sensor coverage is thin. Sensor Array will strengthen frontier defense and energy output."
                        .to_string(),
            });
        }

        if military_pressure >= 1
            && content::ai_attack_bias(owner) >= 1
            && !base.facilities.contains(&Facility::TransitHub)
            && self.is_production_available(owner, ProductionItem::TransitHub)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::TransitHub,
                priority: 71,
                reason:
                    "Fast-response forces need better logistics. Transit Hub will support mobile strike units."
                        .to_string(),
            });
        }

        if military_pressure >= 1
            && content::ai_attack_bias(owner) >= 1
            && base.facilities.contains(&Facility::TransitHub)
            && !base.facilities.contains(&Facility::ForwardDepot)
            && self.is_production_available(owner, ProductionItem::ForwardDepot)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::ForwardDepot,
                priority: 70,
                reason:
                    "Mobile forces are active here. Forward Depot will sustain fast-response attacks and repairs."
                        .to_string(),
            });
        }

        if military_pressure >= 2
            && self.is_production_available(owner, ProductionItem::GarrisonGuard)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::GarrisonGuard,
                priority: 68,
                reason: "A hardened garrison unit would improve local base defense immediately."
                    .to_string(),
            });
        }

        if military_pressure >= 2
            && self.is_production_available(owner, ProductionItem::PsiSentinel)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::PsiSentinel,
                priority: 67,
                reason:
                    "Psionic defense is available. Psi Sentinel would strengthen hostile-frontier security."
                        .to_string(),
            });
        }

        if military_pressure >= 2
            && !base.facilities.contains(&Facility::MilitaryAcademy)
            && self.is_production_available(owner, ProductionItem::MilitaryAcademy)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::MilitaryAcademy,
                priority: 70,
                reason:
                    "Frontline pressure is high. Military Academy will field veteran troops faster."
                        .to_string(),
            });
        }

        if military_pressure >= 1
            && attack_bias >= 1
            && self.is_production_available(owner, ProductionItem::ResonanceLaser)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::ResonanceLaser,
                priority: 69,
                reason:
                    "Advanced shock infantry is available here. Resonance Laser can anchor breakthroughs and punish exposed attackers."
                        .to_string(),
            });
        }

        if military_pressure >= 1
            && attack_bias >= 1
            && self.is_production_available(owner, ProductionItem::RaiderSpeeder)
        {
            plan.push(GovernorPlanStep {
                item: ProductionItem::RaiderSpeeder,
                priority: 69,
                reason:
                    "Mobile strike pressure fits this doctrine. Raider Speeder can exploit exposed enemy lines quickly."
                        .to_string(),
            });
        }

        // Machine Polity Specialization: Overrides and optimizes for pure industrial efficiency.
        if base.governor_mode == GovernorMode::MachinePolity {
            if !base.facilities.contains(&Facility::MineralRefinery)
                && self.is_production_available(owner, ProductionItem::MineralRefinery)
            {
                plan.push(GovernorPlanStep {
                    item: ProductionItem::MineralRefinery,
                    priority: 110, // Top priority for Machine Polity
                    reason: "Machine Polity: Prioritizing raw mineral throughput for industrial dominance."
                        .to_string(),
                });
            }
            if !base.facilities.contains(&Facility::TransitHub)
                && self.is_production_available(owner, ProductionItem::TransitHub)
            {
                plan.push(GovernorPlanStep {
                    item: ProductionItem::TransitHub,
                    priority: 105,
                    reason: "Machine Polity: Standardizing logistical architecture for efficiency."
                        .to_string(),
                });
            }
        }

        plan.sort_by_key(|step| -step.priority);
        plan
    }

    pub fn base_governor_recommendation(&self, base_id: usize) -> Option<(ProductionItem, String)> {
        self.base_governor_plan(base_id)
            .into_iter()
            .next()
            .map(|step| (step.item, step.reason))
    }

    fn missing_required_tech_for_item(&self, owner: usize, item: ProductionItem) -> Option<Tech> {
        let required_tech = content::required_tech_for_production(item)?;
        let faction = self.faction(owner)?;
        (!faction.known_techs.contains(&required_tech)).then_some(required_tech)
    }

    pub fn base_governor_locked_recommendation(
        &self,
        base_id: usize,
    ) -> Option<(ProductionItem, Tech, String)> {
        let Some(base) = self.base(base_id) else {
            return None;
        };
        let owner = base.owner;
        let unrest = self.base_unrest(base_id);
        let damaged_garrisons = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
            .filter(|unit| {
                let max_hp = content::unit_base_hp(unit.kind.clone());
                let on_base = self
                    .tile(unit.x, unit.y)
                    .and_then(|tile| tile.base)
                    .and_then(|id| self.base(id))
                    .map(|b| b.id == base_id)
                    .unwrap_or(false);
                on_base && unit.hp < max_hp
            })
            .count();
        let (_, unit_upkeep, _) = self.faction_upkeep(owner);
        let military_pressure = ai::military_pressure_near_base(self, base.x, base.y, owner);
        let psi_pressure = self.base_local_psi_pressure(base.id);
        let attack_bias = content::ai_attack_bias(owner);
        let yields = self.effective_base_yields(base_id).unwrap_or_default();
        let food_margin = self.base_food_margin(base.id).unwrap_or(1);

        let try_item = |item: ProductionItem, reason: &str| {
            self.missing_required_tech_for_item(owner, item)
                .map(|tech| (item, tech, reason.to_string()))
        };

        if unrest > 0 && !base.facilities.contains(&Facility::RecreationCommons) {
            if let Some(blocked) = try_item(
                ProductionItem::RecreationCommons,
                "Unrest is reducing output here. Recreation Commons would stabilize the base once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if damaged_garrisons > 0 && !base.facilities.contains(&Facility::FieldHospital) {
            if let Some(blocked) = try_item(
                ProductionItem::FieldHospital,
                "Damaged garrisons are rotating through this base. Field Hospital would speed recovery once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if (unrest > 0 || yields.energy <= yields.minerals)
            && base.facilities.contains(&Facility::RecreationCommons)
            && !base.facilities.contains(&Facility::HologramTheatre)
        {
            if let Some(blocked) = try_item(
                ProductionItem::HologramTheatre,
                "Morale infrastructure is already online here. Hologram Theatre would deepen unrest control and improve energy flow once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if damaged_garrisons > 0
            && base.facilities.contains(&Facility::FieldHospital)
            && !base.facilities.contains(&Facility::ResearchHospital)
        {
            if let Some(blocked) = try_item(
                ProductionItem::ResearchHospital,
                "Recovery infrastructure is already active here. Research Hospital would accelerate healing, growth, and scientific output once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if psi_pressure >= 2 {
            if let Some(blocked) = try_item(
                ProductionItem::PsiSentinel,
                "Psionic threats are nearby. Psi Sentinel would harden this base once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if psi_pressure >= 1 && !base.facilities.contains(&Facility::PsiBeacon) {
            if let Some(blocked) = try_item(
                ProductionItem::PsiBeacon,
                "Psi pressure is building here. Psi Beacon would strengthen local resilience once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if (military_pressure >= 1 || psi_pressure >= 1 || damaged_garrisons > 0)
            && (base.facilities.contains(&Facility::PsiBeacon)
                || base.facilities.contains(&Facility::MilitaryAcademy)
                || base.facilities.contains(&Facility::FieldHospital))
            && !base.facilities.contains(&Facility::BioenhancementCenter)
        {
            if let Some(blocked) = try_item(
                ProductionItem::BioenhancementCenter,
                "This base already has frontline support infrastructure. Bioenhancement Center would improve training and resilience once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if unit_upkeep > 0 && !base.facilities.contains(&Facility::CommandCenter) {
            if let Some(blocked) = try_item(
                ProductionItem::CommandCenter,
                "Support pressure is rising. Command Center would improve support and training once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if yields.energy <= yields.minerals && !base.facilities.contains(&Facility::NetworkNode) {
            if let Some(blocked) = try_item(
                ProductionItem::NetworkNode,
                "Research and energy output are lagging. Network Node would improve knowledge flow once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if self.base_potential_trade_links(base_id) >= 1
            && !base.facilities.contains(&Facility::TradeExchange)
        {
            if let Some(blocked) = try_item(
                ProductionItem::TradeExchange,
                "Nearby friendly bases can support trade here. Trade Exchange would convert that network once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if self.base_potential_trade_links(base_id) >= 1
            && self.base_convoy_escort_count(base_id) == 0
            && (self.base_local_military_pressure(base_id) >= 1
                || self.base_local_psi_pressure(base_id) >= 1
                || self
                    .convoy_route_status_for_base(base_id)
                    .into_iter()
                    .any(|(_, _, disrupted, intercepted, _)| disrupted || intercepted))
        {
            if let Some(blocked) = try_item(
                ProductionItem::EscortSpeeder,
                "Local convoy lanes are exposed. Escort Speeder would add mobile defense once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if self.base_potential_trade_links(base_id) >= 1
            && !base.facilities.contains(&Facility::PatrolGrid)
            && (self.base_local_military_pressure(base_id) >= 1
                || self.base_local_psi_pressure(base_id) >= 1)
        {
            if let Some(blocked) = try_item(
                ProductionItem::PatrolGrid,
                "Convoy routes here need protection. Patrol Grid would reduce disruption once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if self.base_potential_trade_links(base_id) >= 1
            && !base.facilities.contains(&Facility::FreightDepot)
            && (base.facilities.contains(&Facility::TradeExchange)
                || yields.minerals <= yields.energy)
        {
            if let Some(blocked) = try_item(
                ProductionItem::FreightDepot,
                "Friendly bases can support freight flow here. Freight Depot would improve mineral throughput once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if yields.nutrients <= base.population.max(1) + 1
            && !base.facilities.contains(&Facility::Greenhouse)
        {
            if let Some(blocked) = try_item(
                ProductionItem::Greenhouse,
                "Food security is thin here. Greenhouse would improve nutrient flow once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if yields.minerals <= 6 && !base.facilities.contains(&Facility::MineralRefinery) {
            if let Some(blocked) = try_item(
                ProductionItem::MineralRefinery,
                "Extraction throughput is thin here. Mineral Refinery would strengthen industry once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if (yields.nutrients <= 6 || yields.minerals <= 6)
            && !base.facilities.contains(&Facility::RecyclingTanks)
        {
            if let Some(blocked) = try_item(
                ProductionItem::RecyclingTanks,
                "Core growth and mineral flow are thin. Recycling Tanks would stabilize output once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if (food_margin <= 0 || yields.energy <= yields.minerals)
            && base.facilities.contains(&Facility::FieldHospital)
            && !base.facilities.contains(&Facility::ResearchHospital)
        {
            if let Some(blocked) = try_item(
                ProductionItem::ResearchHospital,
                "This base needs stronger long-term recovery and research support. Research Hospital would help once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 2 && !base.facilities.contains(&Facility::PerimeterDefense) {
            if let Some(blocked) = try_item(
                ProductionItem::PerimeterDefense,
                "Frontline pressure is high. Perimeter Defense would harden this base once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 1 && !base.facilities.contains(&Facility::SensorArray) {
            if let Some(blocked) = try_item(
                ProductionItem::SensorArray,
                "Sensor coverage is thin. Sensor Array would strengthen frontier defense once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 1
            && attack_bias >= 1
            && !base.facilities.contains(&Facility::TransitHub)
        {
            if let Some(blocked) = try_item(
                ProductionItem::TransitHub,
                "Fast-response forces need better logistics. Transit Hub would support mobile strikes once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 1
            && attack_bias >= 1
            && base.facilities.contains(&Facility::TransitHub)
            && !base.facilities.contains(&Facility::ForwardDepot)
        {
            if let Some(blocked) = try_item(
                ProductionItem::ForwardDepot,
                "Mobile forces are active here. Forward Depot would sustain attacks once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 2 && !base.facilities.contains(&Facility::MilitaryAcademy) {
            if let Some(blocked) = try_item(
                ProductionItem::MilitaryAcademy,
                "Frontline pressure is high. Military Academy would field veteran troops once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 1 && attack_bias >= 1 {
            if let Some(blocked) = try_item(
                ProductionItem::ResonanceLaser,
                "Advanced shock infantry would anchor breakthroughs here once unlocked.",
            ) {
                return Some(blocked);
            }
        }
        if military_pressure >= 1 && attack_bias >= 1 {
            if let Some(blocked) = try_item(
                ProductionItem::RaiderSpeeder,
                "Mobile strike pressure fits this doctrine. Raider Speeder would exploit exposed lines once unlocked.",
            ) {
                return Some(blocked);
            }
        }

        None
    }

    pub fn base_governor_reason_for_item(
        &self,
        base_id: usize,
        item: ProductionItem,
    ) -> Option<(i32, String)> {
        self.base_governor_plan(base_id)
            .into_iter()
            .find(|step| step.item == item)
            .map(|step| (step.priority, step.reason))
    }

    pub fn faction_governor_recommendation_counts(
        &self,
        owner: usize,
    ) -> Vec<(ProductionItem, usize)> {
        let mut counts: Vec<(ProductionItem, usize)> = Vec::new();
        for base in self.bases_for(owner) {
            let Some((item, _)) = self.base_governor_recommendation(base.id) else {
                continue;
            };
            if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| *existing == item) {
                *count += 1;
            } else {
                counts.push((item, 1));
            }
        }
        counts.sort_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| {
                presentation::production_name(left.0).cmp(presentation::production_name(right.0))
            })
        });
        counts
    }

    pub fn faction_governor_mode_counts(
        &self,
        owner: usize,
    ) -> (usize, usize, usize, usize, usize, usize) {
        let mut off = 0;
        let mut balanced = 0;
        let mut defense = 0;
        let mut recovery = 0;
        let mut economy = 0;
        let mut logistics = 0;

        for base in self.bases_for(owner) {
            match base.governor_mode {
                GovernorMode::Off => off += 1,
                GovernorMode::Balanced => balanced += 1,
                GovernorMode::Defense => defense += 1,
                GovernorMode::Recovery => recovery += 1,
                GovernorMode::Economy => economy += 1,
                GovernorMode::Logistics => logistics += 1,
                GovernorMode::MachinePolity => {} // Doesn't contribute to standard modes
            }
        }

        (off, balanced, defense, recovery, economy, logistics)
    }

    pub fn faction_governor_warning_lines(&self, owner: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let base_count = self.bases_for(owner).len();
        let unrest_count = self
            .bases_for(owner)
            .into_iter()
            .filter(|base| self.base_unrest(base.id) > 0)
            .count();
        let recovery_count = self
            .bases_for(owner)
            .into_iter()
            .filter(|base| self.damaged_garrison_count_for_base(base.id) > 0)
            .count();
        let frontier_count = self
            .bases_for(owner)
            .into_iter()
            .filter(|base| {
                self.base_local_military_pressure(base.id) >= 2
                    || self.base_local_psi_pressure(base.id) >= 2
            })
            .count();
        let (_, balanced_count, defense_count, recovery_mode_count, economy_count, _) =
            self.faction_governor_mode_counts(owner);
        let (_, _, _, warzone_count) = self.base_area_role_counts(owner);

        if frontier_count > 0 && defense_count == 0 {
            lines.push("Frontier pressure is active with no Defense-governed bases.".to_string());
        }
        if recovery_count > 0 && recovery_mode_count == 0 {
            lines.push("Damaged garrisons are active with no Recovery-governed bases.".to_string());
        }
        if unrest_count > 0 && economy_count == 0 {
            lines.push("Unrest is active with no Economy-governed bases.".to_string());
        }
        if base_count >= 4 && balanced_count == 0 {
            lines.push(
                "No bases are running Balanced governance across a larger empire.".to_string(),
            );
        }
        if warzone_count > 0 && defense_count + recovery_mode_count == 0 {
            lines.push("Warzone bases lack dedicated Defense or Recovery governance.".to_string());
        }

        lines
    }

    pub fn faction_queue_gap_base_ids(&self, owner: usize) -> Vec<usize> {
        let mut ids: Vec<(usize, i32, i32, i32)> = self
            .bases_for(owner)
            .into_iter()
            .filter(|base| base.production_queue.is_empty())
            .map(|base| {
                (
                    base.id,
                    self.base_local_military_pressure(base.id)
                        + self.base_local_psi_pressure(base.id),
                    self.base_unrest(base.id),
                    self.damaged_garrison_count_for_base(base.id) as i32,
                )
            })
            .collect();
        ids.sort_by_key(|(id, pressure, unrest, damaged)| {
            (-*pressure, -*unrest, -*damaged, *id as i32)
        });
        ids.into_iter().map(|(id, _, _, _)| id).collect()
    }

    pub fn faction_locked_governor_recommendations(
        &self,
        owner: usize,
    ) -> Vec<(usize, ProductionItem, Tech, String)> {
        let mut blocked = Vec::new();
        for base in self.bases_for(owner) {
            if let Some((item, tech, reason)) = self.base_governor_locked_recommendation(base.id) {
                blocked.push((base.id, item, tech, reason));
            }
        }
        blocked.sort_by(|left, right| {
            presentation::tech_name(left.2)
                .cmp(presentation::tech_name(right.2))
                .then_with(|| {
                    presentation::production_name(left.1)
                        .cmp(presentation::production_name(right.1))
                })
                .then_with(|| left.0.cmp(&right.0))
        });
        blocked
    }

    pub fn faction_locked_governor_recommendations_for_tech(
        &self,
        owner: usize,
        tech: Tech,
    ) -> Vec<(usize, ProductionItem, String)> {
        self.faction_locked_governor_recommendations(owner)
            .into_iter()
            .filter(|(_, _, blocked_tech, _)| *blocked_tech == tech)
            .map(|(base_id, item, _, reason)| (base_id, item, reason))
            .collect()
    }

    pub fn faction_locked_governor_recommendation_entries_for_tech(
        &self,
        owner: usize,
        tech: Tech,
    ) -> Vec<(String, ProductionItem, Tech)> {
        self.faction_locked_governor_recommendations_for_tech(owner, tech)
            .into_iter()
            .filter_map(|(base_id, item, _)| {
                self.base(base_id)
                    .map(|base| (base.name.clone(), item, tech))
            })
            .collect()
    }

    pub fn faction_locked_governor_recommendation_base_ids_for_tech(
        &self,
        owner: usize,
        tech: Tech,
    ) -> Vec<usize> {
        self.faction_locked_governor_recommendations_for_tech(owner, tech)
            .into_iter()
            .map(|(base_id, _, _)| base_id)
            .collect()
    }

    pub fn tech_unlock_impact_state(&self, owner: usize, tech: Tech) -> TechUnlockImpactState {
        let entries = self.faction_locked_governor_recommendation_entries_for_tech(owner, tech);
        let base_ids = self.faction_locked_governor_recommendation_base_ids_for_tech(owner, tech);
        TechUnlockImpactState {
            tech,
            recommendation_count: entries.len(),
            base_ids,
            summary_text: (!entries.is_empty()).then(|| {
                presentation::research_unlock_affects_text(
                    &presentation::summarize_base_unlock_blocks(&entries, "None", 3),
                )
            }),
            entries,
        }
    }

    pub fn current_research_unlock_pressure_base_ids(&self, owner: usize) -> Vec<usize> {
        let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return Vec::new();
        };
        self.faction_locked_governor_recommendation_base_ids_for_tech(owner, current_tech)
    }

    pub fn current_research_unlock_focus_base_id(
        &self,
        owner: usize,
        current_base_id: Option<usize>,
    ) -> Option<usize> {
        let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return None;
        };
        self.research_unlock_preview_focus_base_id(owner, current_tech, 3, current_base_id)
    }

    pub fn research_buckets(&self, owner: usize) -> (Vec<Tech>, Vec<Tech>, Vec<(Tech, Vec<Tech>)>) {
        let mut known = Vec::new();
        let mut available = Vec::new();
        let mut blocked = Vec::new();

        for tech in Tech::all() {
            if self
                .faction(owner)
                .map(|faction| faction.known_techs.contains(&tech))
                .unwrap_or(false)
            {
                known.push(tech);
            } else if self.is_research_available(owner, tech) {
                available.push(tech);
            } else {
                blocked.push((
                    tech,
                    content::tech_prerequisites(tech)
                        .into_iter()
                        .filter(|prereq| {
                            self.faction(owner)
                                .map(|faction| !faction.known_techs.contains(prereq))
                                .unwrap_or(false)
                        })
                        .collect(),
                ));
            }
        }

        (known, available, blocked)
    }

    pub fn research_panel_display_state(
        &self,
        owner: usize,
        current_base_id: Option<usize>,
        staged_preview: Option<&ResearchUnlockPreviewState>,
    ) -> ResearchPanelDisplayState {
        let (known, available, blocked) = self.research_buckets(owner);
        let selected_tech = self.faction(owner).map(|faction| faction.current_research);
        let known_rows: Vec<KnownResearchDisplayState> = known
            .iter()
            .copied()
            .map(|tech| KnownResearchDisplayState {
                tech,
                label_text: presentation::tech_name(tech).to_string(),
            })
            .collect();
        ResearchPanelDisplayState {
            summary_text: presentation::research_state_summary(
                known_rows.len(),
                available.len(),
                blocked.len(),
            ),
            available_heading_text: format!("Available now ({})", available.len()),
            blocked_heading_text: format!("Blocked ({})", blocked.len()),
            known_heading_text: format!("Known techs ({})", known_rows.len()),
            available_empty_text: presentation::research_available_empty_text().to_string(),
            blocked_empty_text: presentation::research_blocked_empty_text().to_string(),
            known: known_rows,
            available: available
                .into_iter()
                .map(|tech| {
                    let unlock_impact = self.tech_unlock_impact_state(owner, tech);
                    let preview_status =
                        self.research_unlock_preview_status_for_tech(staged_preview, tech);
                    AvailableResearchDisplayState {
                        tech,
                        label_text: presentation::available_research_label(
                            tech,
                            selected_tech == Some(tech),
                        ),
                        cost_text: format!("Cost {}", presentation::tech_cost(tech)),
                        description_text: presentation::tech_description(tech).to_string(),
                        unlock_lines: presentation::tech_unlock_lines(tech),
                        unlock_impact_text: (unlock_impact.recommendation_count > 0).then(|| {
                            presentation::research_unlock_impact_text(
                                unlock_impact.recommendation_count,
                            )
                        }),
                        affected_focus_base_id: (unlock_impact.recommendation_count > 0)
                            .then(|| {
                                self.research_unlock_preview_focus_base_id(
                                    owner,
                                    tech,
                                    3,
                                    current_base_id,
                                )
                            })
                            .flatten(),
                        preview_status_text: preview_status.as_ref().map(|status| {
                            presentation::unlock_preview_status_text(status.total, status.drifted)
                        }),
                        preview_action_label: (unlock_impact.recommendation_count > 0).then(|| {
                            presentation::unlock_preview_action_label(
                                preview_status.is_some(),
                                preview_status.map(|status| status.drifted).unwrap_or(0),
                            )
                            .to_string()
                        }),
                        unlock_impact,
                    }
                })
                .collect(),
            blocked: blocked
                .into_iter()
                .map(|(tech, missing)| BlockedResearchDisplayState {
                    tech,
                    label_text: presentation::research_blocked_label(tech, &missing),
                    description_text: presentation::tech_description(tech).to_string(),
                    unlock_lines: presentation::tech_unlock_lines(tech),
                    missing,
                })
                .collect(),
        }
    }

    pub fn production_preview_rows(
        &self,
        previews: &[(usize, Vec<ProductionItem>)],
    ) -> Vec<ProductionPreviewRow> {
        previews
            .iter()
            .map(|(base_id, items)| ProductionPreviewRow {
                base_id: *base_id,
                base_name: self
                    .base(*base_id)
                    .map(|base| base.name.clone())
                    .unwrap_or_else(|| format!("Base {base_id}")),
                items: items.clone(),
                row_text: presentation::unlock_preview_row_text(
                    &self
                        .base(*base_id)
                        .map(|base| base.name.clone())
                        .unwrap_or_else(|| format!("Base {base_id}")),
                    items,
                ),
            })
            .collect()
    }

    pub fn current_research_display_state(
        &self,
        owner: usize,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> Option<CurrentResearchDisplayState> {
        let (tech, research, cost) = self.research_progress(owner)?;
        let impact = self.tech_unlock_impact_state(owner, tech);
        let queue_previews = self.faction_research_unlock_queue_previews(owner, tech, max_steps);
        let preview_rows = self.production_preview_rows(&queue_previews);
        Some(CurrentResearchDisplayState {
            tech,
            research,
            cost,
            label_text: presentation::research_current_label(tech, research, cost),
            description_text: presentation::tech_description(tech).to_string(),
            preview_heading_text: "Unlock preview".to_string(),
            unlock_lines: presentation::tech_unlock_lines(tech),
            affected_base_ids: impact.base_ids,
            affected_focus_base_id: self
                .current_research_unlock_focus_base_id(owner, current_base_id),
            affected_focus_label_text: (!impact.entries.is_empty())
                .then(|| presentation::research_cycle_affected_base_label().to_string()),
            affected_entries_heading: "Affected Bases",
            affected_entries: impact.entries,
            affected_summary_text: impact.summary_text,
            preview_section: CurrentResearchPreviewSectionState {
                heading_text: presentation::unlock_preview_section_heading().to_string(),
                focus_label_text: presentation::unlock_preview_focus_label().to_string(),
                keep_open_label_text: presentation::unlock_preview_keep_open_label().to_string(),
                stage_all_log_label_text: presentation::unlock_preview_stage_all_log_label()
                    .to_string(),
                hidden_count_text: (preview_rows.len() > 4).then(|| {
                    presentation::unlock_preview_more_bases_text(
                        preview_rows.len().saturating_sub(4),
                    )
                }),
                hidden_count: preview_rows.len().saturating_sub(4),
                rows: preview_rows,
            },
            queue_previews,
        })
    }

    pub fn current_research_unlock_preview_state(
        &self,
        owner: usize,
        max_steps: usize,
    ) -> Option<ResearchUnlockPreviewState> {
        let current_tech = self
            .faction(owner)
            .map(|faction| faction.current_research)?;
        self.research_unlock_preview_state(owner, current_tech, max_steps)
    }

    pub fn base_governor_plan_preview_with_tech(
        &self,
        base_id: usize,
        tech: Tech,
        max_steps: usize,
    ) -> Vec<ProductionItem> {
        let Some(base) = self.base(base_id) else {
            return Vec::new();
        };
        let owner = base.owner;
        let mut preview = self.clone();
        if let Some(faction) = preview.faction_mut(owner) {
            if !faction.known_techs.contains(&tech) {
                faction.known_techs.push(tech);
            }
        }

        let mut items = Vec::new();
        for step in preview.base_governor_plan(base_id) {
            if items.len() >= max_steps || items.contains(&step.item) {
                continue;
            }
            items.push(step.item);
        }
        items
    }

    pub fn faction_research_unlock_queue_previews(
        &self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
    ) -> Vec<(usize, Vec<ProductionItem>)> {
        self.faction_locked_governor_recommendations_for_tech(owner, tech)
            .into_iter()
            .filter_map(|(base_id, _, _)| {
                let preview = self.base_governor_plan_preview_with_tech(base_id, tech, max_steps);
                (!preview.is_empty()).then_some((base_id, preview))
            })
            .collect()
    }

    pub fn research_unlock_preview_state(
        &self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
    ) -> Option<ResearchUnlockPreviewState> {
        let previews = self.faction_research_unlock_queue_previews(owner, tech, max_steps);
        (!previews.is_empty()).then_some(ResearchUnlockPreviewState {
            tech,
            max_steps,
            previews,
        })
    }

    pub fn prepare_research_unlock_preview(
        &self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
    ) -> Option<ResearchUnlockPreviewSelection> {
        let preview = self.research_unlock_preview_state(owner, tech, max_steps)?;
        let affected_base_ids =
            self.faction_locked_governor_recommendation_base_ids_for_tech(owner, tech);
        Some(ResearchUnlockPreviewSelection {
            preview,
            affected_base_ids,
        })
    }

    pub fn pin_research_unlock_preview_action(
        &self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStageResult {
        let Some(selection) = self.prepare_research_unlock_preview(owner, tech, max_steps) else {
            return ResearchUnlockPreviewStageResult {
                selection: None,
                staged_count: 0,
                focus_base_id: None,
            };
        };

        let focus_base_id =
            self.next_base_cycle_target(&selection.affected_base_ids, current_base_id);
        let staged_count = selection.preview.previews.len();

        ResearchUnlockPreviewStageResult {
            selection: Some(selection),
            staged_count,
            focus_base_id,
        }
    }

    pub fn pin_current_research_unlock_preview_action(
        &self,
        owner: usize,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStageResult {
        let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return ResearchUnlockPreviewStageResult {
                selection: None,
                staged_count: 0,
                focus_base_id: None,
            };
        };
        self.pin_research_unlock_preview_action(owner, current_tech, max_steps, current_base_id)
    }

    pub fn pin_research_unlock_preview_state_action(
        &self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStateActionResult {
        let result =
            self.pin_research_unlock_preview_action(owner, tech, max_steps, current_base_id);
        let affected_base_ids = result
            .selection
            .as_ref()
            .map(|selection| selection.affected_base_ids.clone())
            .unwrap_or_else(|| {
                self.faction_locked_governor_recommendation_base_ids_for_tech(owner, tech)
            });
        let preview = result.selection.map(|selection| selection.preview);
        ResearchUnlockPreviewStateActionResult {
            preview,
            staged_count: result.staged_count,
            focus_base_id: result.focus_base_id,
            affected_base_ids,
        }
    }

    pub fn pin_current_research_unlock_preview_state_action(
        &self,
        owner: usize,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStateActionResult {
        let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return ResearchUnlockPreviewStateActionResult {
                preview: None,
                staged_count: 0,
                focus_base_id: None,
                affected_base_ids: Vec::new(),
            };
        };
        let result =
            self.pin_current_research_unlock_preview_action(owner, max_steps, current_base_id);
        let affected_base_ids = result
            .selection
            .as_ref()
            .map(|selection| selection.affected_base_ids.clone())
            .unwrap_or_else(|| {
                self.faction_locked_governor_recommendation_base_ids_for_tech(owner, current_tech)
            });
        let preview = result.selection.map(|selection| selection.preview);
        ResearchUnlockPreviewStateActionResult {
            preview,
            staged_count: result.staged_count,
            focus_base_id: result.focus_base_id,
            affected_base_ids,
        }
    }

    pub fn next_base_cycle_target(
        &self,
        base_ids: &[usize],
        current_base_id: Option<usize>,
    ) -> Option<usize> {
        if base_ids.is_empty() {
            return None;
        }

        Some(match current_base_id {
            Some(current) => base_ids
                .iter()
                .copied()
                .find(|id| *id > current)
                .unwrap_or(base_ids[0]),
            None => base_ids[0],
        })
    }

    pub fn faction_base_ids_for_focus(&self, owner: usize, filter: BaseFocusFilter) -> Vec<usize> {
        self.bases_for(owner)
            .into_iter()
            .filter(|base| match filter {
                BaseFocusFilter::All => true,
                BaseFocusFilter::Frontier => {
                    self.base_local_military_pressure(base.id) >= 2
                        || self.base_local_psi_pressure(base.id) >= 2
                }
                BaseFocusFilter::Recovery => self.damaged_garrison_count_for_base(base.id) > 0,
                BaseFocusFilter::Unrest => self.base_unrest(base.id) > 0,
                BaseFocusFilter::QueueGap => {
                    self.faction_queue_gap_base_ids(owner).contains(&base.id)
                }
                BaseFocusFilter::ResearchUnlock => self
                    .current_research_unlock_pressure_base_ids(owner)
                    .contains(&base.id),
                BaseFocusFilter::Logistics => self.base_logistics_stress_score(base.id) > 0,
                BaseFocusFilter::Saturated => {
                    self.saturated_convoy_base_ids(owner).contains(&base.id)
                }
                BaseFocusFilter::Tight => self.tight_convoy_base_ids(owner).contains(&base.id),
                BaseFocusFilter::Collapsing => {
                    self.convoy_collapsing_base_ids(owner).contains(&base.id)
                }
                BaseFocusFilter::Balanced => base.governor_mode == GovernorMode::Balanced,
                BaseFocusFilter::Defense => base.governor_mode == GovernorMode::Defense,
                BaseFocusFilter::Economy => base.governor_mode == GovernorMode::Economy,
                BaseFocusFilter::LogisticsMode => base.governor_mode == GovernorMode::Logistics,
            })
            .map(|base| base.id)
            .collect()
    }

    pub fn next_base_focus_target(
        &self,
        owner: usize,
        filter: BaseFocusFilter,
        current_base_id: Option<usize>,
    ) -> Option<usize> {
        let base_ids = self.faction_base_ids_for_focus(owner, filter);
        self.next_base_cycle_target(&base_ids, current_base_id)
    }

    pub fn next_base_focus_target_action(
        &mut self,
        owner: usize,
        filter: BaseFocusFilter,
        current_base_id: Option<usize>,
    ) -> Option<usize> {
        let next_focus_base_id = self.next_base_focus_target(owner, filter, current_base_id);
        if next_focus_base_id.is_none() {
            self.push_log(format!(
                "No player bases match {} focus.",
                base_focus_filter_label(filter)
            ));
        }
        next_focus_base_id
    }

    pub fn base_focus_state(
        &self,
        owner: usize,
        filter: BaseFocusFilter,
        current_base_id: Option<usize>,
    ) -> BaseFocusState {
        let base_ids = self.faction_base_ids_for_focus(owner, filter);
        let count = base_ids.len();
        BaseFocusState {
            filter,
            count,
            count_label: format!("{} matching base(s)", count),
            next_focus_base_id: self.next_base_cycle_target(&base_ids, current_base_id),
            action_label_text: match filter {
                BaseFocusFilter::QueueGap => Some("Fill Queue Gaps".to_string()),
                BaseFocusFilter::ResearchUnlock => Some("Preview Unlock Queues".to_string()),
                _ => None,
            },
        }
    }

    pub fn research_unlock_preview_focus_base_id(
        &self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> Option<usize> {
        let selection = self.prepare_research_unlock_preview(owner, tech, max_steps)?;
        self.next_base_cycle_target(&selection.affected_base_ids, current_base_id)
    }

    pub fn stage_research_unlock_preview_action(
        &mut self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStageResult {
        let result =
            self.pin_research_unlock_preview_action(owner, tech, max_steps, current_base_id);
        let Some(selection) = result.selection.clone() else {
            self.push_log(format!(
                "No queue previews are waiting on {}.",
                presentation::tech_name(tech)
            ));
            return result;
        };

        for (base_id, items) in &selection.preview.previews {
            let base_name = self
                .base(*base_id)
                .map(|base| base.name.clone())
                .unwrap_or_else(|| format!("Base {base_id}"));
            let sequence = items
                .iter()
                .map(|item| presentation::production_name(*item))
                .collect::<Vec<_>>()
                .join(" -> ");
            self.push_log(format!(
                "Unlock preview if {} lands: {} -> {}",
                presentation::tech_name(tech),
                base_name,
                sequence
            ));
        }

        ResearchUnlockPreviewStageResult {
            selection: Some(selection),
            staged_count: result.staged_count,
            focus_base_id: result.focus_base_id,
        }
    }

    pub fn stage_research_unlock_preview_state_action(
        &mut self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStateActionResult {
        let result =
            self.stage_research_unlock_preview_action(owner, tech, max_steps, current_base_id);
        let affected_base_ids = result
            .selection
            .as_ref()
            .map(|selection| selection.affected_base_ids.clone())
            .unwrap_or_else(|| {
                self.faction_locked_governor_recommendation_base_ids_for_tech(owner, tech)
            });
        let preview = result.selection.map(|selection| selection.preview);
        ResearchUnlockPreviewStateActionResult {
            preview,
            staged_count: result.staged_count,
            focus_base_id: result.focus_base_id,
            affected_base_ids,
        }
    }

    pub fn research_unlock_preview_counts(
        &self,
        preview: &ResearchUnlockPreviewState,
    ) -> (usize, usize) {
        (
            preview.previews.len(),
            self.research_unlock_preview_drifted_base_ids(preview).len(),
        )
    }

    pub fn research_unlock_preview_counts_for_tech(
        &self,
        preview: Option<&ResearchUnlockPreviewState>,
        tech: Tech,
    ) -> Option<(usize, usize)> {
        let preview = preview?;
        (preview.tech == tech).then(|| self.research_unlock_preview_counts(preview))
    }

    pub fn research_unlock_preview_status_for_tech(
        &self,
        preview: Option<&ResearchUnlockPreviewState>,
        tech: Tech,
    ) -> Option<ResearchUnlockPreviewStatus> {
        self.research_unlock_preview_counts_for_tech(preview, tech)
            .map(|(total, drifted)| ResearchUnlockPreviewStatus { total, drifted })
    }

    pub fn research_unlock_preview_drifted_base_ids(
        &self,
        preview: &ResearchUnlockPreviewState,
    ) -> Vec<usize> {
        preview
            .previews
            .iter()
            .filter_map(|(base_id, items)| {
                (!self.research_unlock_preview_base_is_current(preview, *base_id, items))
                    .then_some(*base_id)
            })
            .collect()
    }

    pub fn research_unlock_preview_base_is_current(
        &self,
        preview: &ResearchUnlockPreviewState,
        base_id: usize,
        items: &[ProductionItem],
    ) -> bool {
        self.base_governor_plan_preview_with_tech(base_id, preview.tech, preview.max_steps) == items
    }

    pub fn sync_research_unlock_preview_state(
        &self,
        owner: usize,
        preview: &ResearchUnlockPreviewState,
    ) -> Option<ResearchUnlockPreviewState> {
        let Some(faction) = self.faction(owner).cloned() else {
            return None;
        };

        let tech_known = faction.known_techs.contains(&preview.tech);
        let tech_researchable = faction.current_research == preview.tech
            || self.is_research_available(owner, preview.tech);

        if !tech_known && !tech_researchable {
            return None;
        }

        let previews: Vec<(usize, Vec<ProductionItem>)> = if tech_known {
            preview
                .previews
                .iter()
                .filter(|(base_id, _)| {
                    !self
                        .base_governor_plan_preview_with_tech(
                            *base_id,
                            preview.tech,
                            preview.max_steps,
                        )
                        .is_empty()
                })
                .cloned()
                .collect()
        } else {
            let active_base_ids = self
                .faction_locked_governor_recommendations_for_tech(owner, preview.tech)
                .into_iter()
                .map(|(base_id, _, _)| base_id)
                .collect::<Vec<_>>();
            preview
                .previews
                .iter()
                .filter(|(base_id, _)| active_base_ids.contains(base_id))
                .cloned()
                .collect()
        };

        (!previews.is_empty()).then_some(ResearchUnlockPreviewState {
            tech: preview.tech,
            max_steps: preview.max_steps,
            previews,
        })
    }

    pub fn sync_research_unlock_preview_action(
        &self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
    ) -> Option<ResearchUnlockPreviewState> {
        let preview = preview?;
        self.sync_research_unlock_preview_state(owner, &preview)
    }

    pub fn sync_research_unlock_preview_state_action(
        &self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
    ) -> ResearchUnlockPreviewStateActionResult {
        ResearchUnlockPreviewStateActionResult {
            staged_count: preview
                .as_ref()
                .map(|preview| preview.previews.len())
                .unwrap_or(0),
            preview: self.sync_research_unlock_preview_action(owner, preview),
            focus_base_id: None,
            affected_base_ids: Vec::new(),
        }
    }

    pub fn refresh_research_unlock_preview_action(
        &self,
        owner: usize,
        preview: &ResearchUnlockPreviewState,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewRefreshResult {
        let Some(selection) =
            self.prepare_research_unlock_preview(owner, preview.tech, preview.max_steps)
        else {
            return ResearchUnlockPreviewRefreshResult {
                preview: None,
                focus_base_id: None,
            };
        };
        let focus_base_id =
            self.next_base_cycle_target(&selection.affected_base_ids, current_base_id);
        ResearchUnlockPreviewRefreshResult {
            preview: Some(selection.preview),
            focus_base_id,
        }
    }

    pub fn refresh_research_unlock_preview_state_action(
        &self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStateActionResult {
        let Some(preview) = preview else {
            return ResearchUnlockPreviewStateActionResult {
                preview: None,
                staged_count: 0,
                focus_base_id: None,
                affected_base_ids: Vec::new(),
            };
        };
        let result = self.refresh_research_unlock_preview_action(owner, &preview, current_base_id);
        let affected_base_ids = result
            .preview
            .as_ref()
            .map(|preview| {
                self.faction_locked_governor_recommendation_base_ids_for_tech(owner, preview.tech)
            })
            .unwrap_or_default();
        ResearchUnlockPreviewStateActionResult {
            staged_count: result
                .preview
                .as_ref()
                .map(|preview| preview.previews.len())
                .unwrap_or(0),
            preview: result.preview,
            focus_base_id: result.focus_base_id,
            affected_base_ids,
        }
    }

    pub fn can_apply_research_unlock_preview(
        &self,
        owner: usize,
        preview: &ResearchUnlockPreviewState,
    ) -> bool {
        self.faction(owner)
            .map(|faction| faction.known_techs.contains(&preview.tech))
            .unwrap_or(false)
    }

    pub fn pinned_research_unlock_preview_display_state(
        &self,
        owner: usize,
        preview: &ResearchUnlockPreviewState,
    ) -> PinnedResearchUnlockPreviewDisplayState {
        let drifted_base_ids = self.research_unlock_preview_drifted_base_ids(preview);
        let can_apply = self.can_apply_research_unlock_preview(owner, preview);
        let waiting_on_current_research = self
            .faction(owner)
            .map(|faction| faction.current_research == preview.tech)
            .unwrap_or(false);
        let rows = self
            .production_preview_rows(&preview.previews)
            .into_iter()
            .map(|row| PinnedResearchUnlockPreviewRowState {
                row_text: presentation::unlock_preview_row_text(&row.base_name, &row.items),
                stale_label_text: drifted_base_ids
                    .contains(&row.base_id)
                    .then(|| presentation::unlock_preview_stale_label().to_string()),
                focus_label_text: presentation::unlock_preview_focus_label().to_string(),
                apply_label_text: presentation::unlock_preview_apply_label().to_string(),
                can_apply: can_apply && !drifted_base_ids.contains(&row.base_id),
                apply_tooltip: presentation::unlock_preview_apply_tooltip(
                    can_apply,
                    !drifted_base_ids.contains(&row.base_id),
                    preview.tech,
                ),
                is_current: !drifted_base_ids.contains(&row.base_id),
                base_id: row.base_id,
                base_name: row.base_name,
                items: row.items,
            })
            .collect();

        PinnedResearchUnlockPreviewDisplayState {
            tech: preview.tech,
            max_steps: preview.max_steps,
            heading_text: presentation::unlock_preview_heading(preview.tech),
            availability_text: (!can_apply).then(|| {
                presentation::unlock_preview_availability_text(
                    preview.tech,
                    waiting_on_current_research,
                )
            }),
            drift_text: (!drifted_base_ids.is_empty())
                .then(|| presentation::unlock_preview_drift_text(drifted_base_ids.len())),
            stage_log_label_text: presentation::unlock_preview_stage_log_label().to_string(),
            refresh_label_text: presentation::unlock_preview_refresh_label().to_string(),
            clear_label_text: presentation::unlock_preview_clear_label().to_string(),
            apply_all_label_text: presentation::unlock_preview_apply_all_label().to_string(),
            can_apply,
            apply_all_enabled: can_apply && drifted_base_ids.is_empty(),
            apply_all_tooltip: presentation::unlock_preview_apply_all_tooltip(
                can_apply,
                !drifted_base_ids.is_empty(),
                preview.tech,
            ),
            waiting_on_current_research,
            hidden_count_text: (preview.previews.len() > 5).then(|| {
                presentation::unlock_preview_more_bases_text(
                    preview.previews.len().saturating_sub(5),
                )
            }),
            hidden_count: preview.previews.len().saturating_sub(5),
            drifted_base_ids,
            rows,
        }
    }

    pub fn apply_research_unlock_preview_to_base(
        &mut self,
        owner: usize,
        preview: ResearchUnlockPreviewState,
        base_id: usize,
    ) -> Result<ResearchUnlockPreviewApplyResult, String> {
        if !self.can_apply_research_unlock_preview(owner, &preview) {
            return Err(format!(
                "{} must be known before this preview can be applied.",
                presentation::tech_name(preview.tech)
            ));
        }

        let Some((_, items)) = preview
            .previews
            .iter()
            .find(|(preview_base_id, _)| *preview_base_id == base_id)
        else {
            return Ok(ResearchUnlockPreviewApplyResult {
                applied_base_ids: Vec::new(),
                remaining_preview: Some(preview),
                focus_base_id: None,
            });
        };

        if !self.research_unlock_preview_base_is_current(&preview, base_id, items) {
            return Err(format!(
                "Pinned unlock preview for base {base_id} drifted from current governor intent. Refresh it before applying."
            ));
        }

        self.apply_research_unlock_queue_preview_items(base_id, preview.tech, items.clone())
            .map_err(|err| format!("Unlock preview apply failed: {err}"))?;

        let mut remaining_preview = preview.clone();
        remaining_preview
            .previews
            .retain(|(preview_base_id, _)| *preview_base_id != base_id);

        Ok(ResearchUnlockPreviewApplyResult {
            applied_base_ids: vec![base_id],
            remaining_preview: (!remaining_preview.previews.is_empty())
                .then_some(remaining_preview),
            focus_base_id: Some(base_id),
        })
    }

    pub fn apply_research_unlock_preview_action(
        &mut self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
        base_id: usize,
    ) -> Result<ResearchUnlockPreviewApplyResult, String> {
        let Some(preview) = preview else {
            return Ok(ResearchUnlockPreviewApplyResult {
                applied_base_ids: Vec::new(),
                remaining_preview: None,
                focus_base_id: None,
            });
        };
        self.apply_research_unlock_preview_to_base(owner, preview, base_id)
    }

    pub fn apply_research_unlock_preview_state_action(
        &mut self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
        base_id: usize,
    ) -> Result<ResearchUnlockPreviewStateActionResult, String> {
        let result = self.apply_research_unlock_preview_action(owner, preview, base_id)?;
        Ok(ResearchUnlockPreviewStateActionResult {
            staged_count: result
                .remaining_preview
                .as_ref()
                .map(|preview| preview.previews.len())
                .unwrap_or(0),
            preview: result.remaining_preview,
            focus_base_id: result.focus_base_id,
            affected_base_ids: Vec::new(),
        })
    }

    pub fn apply_all_research_unlock_previews(
        &mut self,
        owner: usize,
        preview: ResearchUnlockPreviewState,
    ) -> Result<ResearchUnlockPreviewApplyResult, String> {
        if !self.can_apply_research_unlock_preview(owner, &preview) {
            return Err(format!(
                "{} must be known before staged previews can be applied.",
                presentation::tech_name(preview.tech)
            ));
        }

        let mut applied_base_ids = Vec::new();
        let mut remaining = Vec::new();

        for (base_id, items) in preview.previews.clone() {
            if !self.research_unlock_preview_base_is_current(&preview, base_id, &items) {
                self.push_log(format!(
                    "Pinned unlock preview for base {base_id} drifted from current governor intent. Refresh it before applying."
                ));
                remaining.push((base_id, items));
                continue;
            }

            match self.apply_research_unlock_queue_preview_items(
                base_id,
                preview.tech,
                items.clone(),
            ) {
                Ok(()) => applied_base_ids.push(base_id),
                Err(err) => {
                    self.push_log(format!(
                        "Unlock preview apply failed for base {base_id}: {err}"
                    ));
                    remaining.push((base_id, items));
                }
            }
        }

        if !applied_base_ids.is_empty() {
            self.push_log(format!(
                "Applied staged {} preview queues to {} base(s).",
                presentation::tech_name(preview.tech),
                applied_base_ids.len()
            ));
        }

        Ok(ResearchUnlockPreviewApplyResult {
            focus_base_id: applied_base_ids.first().copied(),
            applied_base_ids,
            remaining_preview: (!remaining.is_empty()).then_some(ResearchUnlockPreviewState {
                tech: preview.tech,
                max_steps: preview.max_steps,
                previews: remaining,
            }),
        })
    }

    pub fn apply_all_research_unlock_preview_action(
        &mut self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
    ) -> Result<ResearchUnlockPreviewApplyResult, String> {
        let Some(preview) = preview else {
            return Ok(ResearchUnlockPreviewApplyResult {
                applied_base_ids: Vec::new(),
                remaining_preview: None,
                focus_base_id: None,
            });
        };
        self.apply_all_research_unlock_previews(owner, preview)
    }

    pub fn apply_all_research_unlock_preview_state_action(
        &mut self,
        owner: usize,
        preview: Option<ResearchUnlockPreviewState>,
    ) -> Result<ResearchUnlockPreviewStateActionResult, String> {
        let result = self.apply_all_research_unlock_preview_action(owner, preview)?;
        Ok(ResearchUnlockPreviewStateActionResult {
            staged_count: result
                .remaining_preview
                .as_ref()
                .map(|preview| preview.previews.len())
                .unwrap_or(0),
            preview: result.remaining_preview,
            focus_base_id: result.focus_base_id,
            affected_base_ids: Vec::new(),
        })
    }

    pub fn stage_research_unlock_queue_previews(
        &mut self,
        owner: usize,
        tech: Tech,
        max_steps: usize,
    ) -> usize {
        self.stage_research_unlock_preview_action(owner, tech, max_steps, None)
            .staged_count
    }

    pub fn stage_current_research_unlock_preview_action(
        &mut self,
        owner: usize,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStageResult {
        let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return ResearchUnlockPreviewStageResult {
                selection: None,
                staged_count: 0,
                focus_base_id: None,
            };
        };
        self.stage_research_unlock_preview_action(owner, current_tech, max_steps, current_base_id)
    }

    pub fn stage_current_research_unlock_preview_state_action(
        &mut self,
        owner: usize,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStateActionResult {
        let Some(current_tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return ResearchUnlockPreviewStateActionResult {
                preview: None,
                staged_count: 0,
                focus_base_id: None,
                affected_base_ids: Vec::new(),
            };
        };
        let result =
            self.stage_current_research_unlock_preview_action(owner, max_steps, current_base_id);
        let affected_base_ids = result
            .selection
            .as_ref()
            .map(|selection| selection.affected_base_ids.clone())
            .unwrap_or_else(|| {
                self.faction_locked_governor_recommendation_base_ids_for_tech(owner, current_tech)
            });
        let preview = result.selection.map(|selection| selection.preview);
        ResearchUnlockPreviewStateActionResult {
            preview,
            staged_count: result.staged_count,
            focus_base_id: result.focus_base_id,
            affected_base_ids,
        }
    }

    pub fn current_research_unlock_base_focus_action(
        &mut self,
        owner: usize,
        max_steps: usize,
        current_base_id: Option<usize>,
    ) -> ResearchUnlockPreviewStateActionResult {
        let mut result = self.stage_current_research_unlock_preview_state_action(
            owner,
            max_steps,
            current_base_id,
        );
        if result.focus_base_id.is_none() && result.staged_count == 0 {
            result.focus_base_id =
                self.next_base_cycle_target(&result.affected_base_ids, current_base_id);
        }
        result
    }

    pub fn apply_research_unlock_queue_preview_items(
        &mut self,
        base_id: usize,
        tech: Tech,
        items: Vec<ProductionItem>,
    ) -> Result<(), String> {
        let owner = self
            .base(base_id)
            .map(|base| base.owner)
            .ok_or_else(|| "Base not found.".to_string())?;
        let knows_tech = self
            .faction(owner)
            .map(|faction| faction.known_techs.contains(&tech))
            .unwrap_or(false);
        if !knows_tech {
            return Err(format!(
                "{} is not known by this faction yet.",
                presentation::tech_name(tech)
            ));
        }

        let empty_message = format!(
            "No staged unlock preview is waiting on {} for this base.",
            presentation::tech_name(tech)
        );
        let label = format!("Unlock preview {}", presentation::tech_name(tech));
        self.apply_named_plan(base_id, items, &empty_message, &label)
    }

    pub fn apply_empty_queue_governor_plans(&mut self, owner: usize, max_steps: usize) -> usize {
        let base_ids = self.faction_queue_gap_base_ids(owner);
        if base_ids.is_empty() {
            self.push_log("No empty production queues need governor follow-ups.".to_string());
            return 0;
        }

        let mut applied = 0;
        for base_id in base_ids {
            let before_production = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(ProductionItem::ScoutPatrol);
            let before_queue_len = self
                .base(base_id)
                .map(|base| base.production_queue.len())
                .unwrap_or(0);
            let _ = self.apply_governor_plan_queue(base_id, max_steps);
            let after_production = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(before_production);
            let after_queue_len = self
                .base(base_id)
                .map(|base| base.production_queue.len())
                .unwrap_or(before_queue_len);
            if after_production != before_production || after_queue_len > before_queue_len {
                applied += 1;
            }
        }

        self.push_log(format!(
            "Operations filled governor follow-up queues for {} base(s).",
            applied
        ));
        applied
    }

    pub fn faction_governor_queue_intent_counts(
        &self,
        owner: usize,
        max_steps: usize,
    ) -> Vec<(ProductionItem, usize)> {
        let mut counts: Vec<(ProductionItem, usize)> = Vec::new();
        for base in self.bases_for(owner) {
            for item in self.base_governor_queue_items(base.id, max_steps) {
                if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| *existing == item)
                {
                    *count += 1;
                } else {
                    counts.push((item, 1));
                }
            }
        }
        counts.sort_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| {
                presentation::production_name(left.0).cmp(presentation::production_name(right.0))
            })
        });
        counts
    }

    pub fn base_governor_queue_items(
        &self,
        base_id: usize,
        max_steps: usize,
    ) -> Vec<ProductionItem> {
        let Some(base) = self.base(base_id) else {
            return Vec::new();
        };
        let mut items = Vec::new();
        let preserve_current = self.should_preserve_current_production(base_id);

        for step in self.base_governor_plan(base_id) {
            if items.len() >= max_steps {
                break;
            }
            if (preserve_current && step.item == base.production)
                || step.item == base.production
                || base.production_queue.contains(&step.item)
                || items.contains(&step.item)
            {
                continue;
            }
            items.push(step.item);
        }

        items
    }

    pub fn base_defense_plan_items(&self, base_id: usize, max_steps: usize) -> Vec<ProductionItem> {
        self.base_governor_plan(base_id)
            .into_iter()
            .filter(|step| {
                matches!(
                    step.item,
                    ProductionItem::PerimeterDefense
                        | ProductionItem::SensorArray
                        | ProductionItem::GarrisonGuard
                        | ProductionItem::PsiSentinel
                        | ProductionItem::PsiBeacon
                        | ProductionItem::MilitaryAcademy
                        | ProductionItem::BioenhancementCenter
                        | ProductionItem::ResonanceLaser
                )
            })
            .take(max_steps)
            .map(|step| step.item)
            .collect()
    }

    pub fn base_recovery_plan_items(
        &self,
        base_id: usize,
        max_steps: usize,
    ) -> Vec<ProductionItem> {
        self.base_governor_plan(base_id)
            .into_iter()
            .filter(|step| {
                matches!(
                    step.item,
                    ProductionItem::FieldHospital
                        | ProductionItem::ResearchHospital
                        | ProductionItem::BioenhancementCenter
                        | ProductionItem::CommandCenter
                        | ProductionItem::RecreationCommons
                        | ProductionItem::PsiBeacon
                        | ProductionItem::PsiSentinel
                        | ProductionItem::GarrisonGuard
                        | ProductionItem::EscortSpeeder
                )
            })
            .take(max_steps)
            .map(|step| step.item)
            .collect()
    }

    pub fn base_economy_plan_items(&self, base_id: usize, max_steps: usize) -> Vec<ProductionItem> {
        self.base_governor_plan(base_id)
            .into_iter()
            .filter(|step| {
                matches!(
                    step.item,
                    ProductionItem::RecreationCommons
                        | ProductionItem::HologramTheatre
                        | ProductionItem::Greenhouse
                        | ProductionItem::MineralRefinery
                        | ProductionItem::TradeExchange
                        | ProductionItem::FreightDepot
                        | ProductionItem::PatrolGrid
                        | ProductionItem::RecyclingTanks
                        | ProductionItem::ResearchHospital
                        | ProductionItem::NetworkNode
                        | ProductionItem::CommandCenter
                )
            })
            .take(max_steps)
            .map(|step| step.item)
            .collect()
    }

    pub fn base_logistics_plan_items(
        &self,
        base_id: usize,
        max_steps: usize,
    ) -> Vec<ProductionItem> {
        self.base_governor_plan(base_id)
            .into_iter()
            .filter(|step| {
                matches!(
                    step.item,
                    ProductionItem::TradeExchange
                        | ProductionItem::FreightDepot
                        | ProductionItem::CommandCenter
                        | ProductionItem::FieldHospital
                        | ProductionItem::PatrolGrid
                        | ProductionItem::TransitHub
                        | ProductionItem::ForwardDepot
                        | ProductionItem::EscortSpeeder
                        | ProductionItem::SensorArray
                        | ProductionItem::NetworkNode
                )
            })
            .take(max_steps)
            .map(|step| step.item)
            .collect()
    }

    pub fn base_balanced_plan_items(
        &self,
        base_id: usize,
        max_steps: usize,
    ) -> Vec<ProductionItem> {
        if self.base_local_military_pressure(base_id) >= 2
            || self.base_local_psi_pressure(base_id) >= 2
        {
            return self
                .base_governor_plan(base_id)
                .into_iter()
                .filter(|step| {
                    matches!(
                        step.item,
                        ProductionItem::PerimeterDefense
                            | ProductionItem::SensorArray
                            | ProductionItem::TransitHub
                            | ProductionItem::ForwardDepot
                            | ProductionItem::PsiBeacon
                            | ProductionItem::BioenhancementCenter
                            | ProductionItem::GarrisonGuard
                            | ProductionItem::EscortSpeeder
                            | ProductionItem::RaiderSpeeder
                            | ProductionItem::ResonanceLaser
                            | ProductionItem::PsiSentinel
                            | ProductionItem::MilitaryAcademy
                    )
                })
                .take(max_steps)
                .map(|step| step.item)
                .collect();
        }

        if self.damaged_garrison_count(base_id) > 0 {
            return self.base_recovery_plan_items(base_id, max_steps);
        }

        if self.base_unrest(base_id) > 0 {
            return self.base_economy_plan_items(base_id, max_steps);
        }

        if self.base_logistics_stress_score(base_id) > 0 {
            return self.base_logistics_plan_items(base_id, max_steps);
        }

        let mut items = self.base_governor_queue_items(base_id, max_steps);
        if items.is_empty() {
            if let Some(base) = self.base(base_id) {
                if content::ai_attack_bias(base.owner) >= 1
                    && self.is_production_available(base.owner, ProductionItem::RaiderSpeeder)
                {
                    items.push(ProductionItem::RaiderSpeeder);
                } else if self.is_production_available(base.owner, ProductionItem::Former) {
                    items.push(ProductionItem::Former);
                }
            }
        }
        items
    }

    pub fn apply_recommended_governor_modes(&mut self, owner: usize) -> usize {
        let base_ids: Vec<usize> = self
            .bases_for(owner)
            .into_iter()
            .map(|base| base.id)
            .collect();
        let mut changed = 0;
        for base_id in base_ids {
            let recommended = self.recommended_governor_mode_for_base(base_id);
            if let Some(base) = self.base_mut(base_id) {
                if base.governor_mode != recommended {
                    base.governor_mode = recommended;
                    changed += 1;
                }
            }
        }
        self.push_log(format!(
            "Suggested governor modes applied to {changed} base(s)."
        ));
        changed
    }

    pub fn apply_governor_plan_queue(
        &mut self,
        base_id: usize,
        max_steps: usize,
    ) -> Result<(), String> {
        let items = self.base_governor_queue_items(base_id, max_steps);
        self.apply_named_plan(
            base_id,
            items,
            "Governor queue had no new items to apply.",
            "Governor",
        )
    }

    pub fn apply_governor_plan_queue_action(&mut self, base_id: usize, max_steps: usize) -> bool {
        self.apply_governor_plan_queue(base_id, max_steps)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(err);
                false
            })
    }

    pub fn apply_balanced_plan(&mut self, base_id: usize, max_steps: usize) -> Result<(), String> {
        let items = self.base_balanced_plan_items(base_id, max_steps);
        self.apply_named_plan(
            base_id,
            items,
            "Balanced governor found no useful changes.",
            "Balanced governor",
        )
    }

    pub fn apply_balanced_plan_action(&mut self, base_id: usize, max_steps: usize) -> bool {
        self.apply_balanced_plan(base_id, max_steps)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(err);
                false
            })
    }

    pub fn apply_recovery_plan(&mut self, base_id: usize, max_steps: usize) -> Result<(), String> {
        let items = self.base_recovery_plan_items(base_id, max_steps);
        self.apply_named_plan(
            base_id,
            items,
            "Recovery plan had no applicable actions.",
            "Recovery plan",
        )
    }

    pub fn apply_recovery_plan_action(&mut self, base_id: usize, max_steps: usize) -> bool {
        self.apply_recovery_plan(base_id, max_steps)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(err);
                false
            })
    }

    pub fn apply_defense_plan(&mut self, base_id: usize, max_steps: usize) -> Result<(), String> {
        let items = self.base_defense_plan_items(base_id, max_steps);
        self.apply_named_plan(
            base_id,
            items,
            "Defense plan had no applicable actions.",
            "Defense plan",
        )
    }

    pub fn apply_defense_plan_action(&mut self, base_id: usize, max_steps: usize) -> bool {
        self.apply_defense_plan(base_id, max_steps)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(err);
                false
            })
    }

    pub fn apply_economy_plan(&mut self, base_id: usize, max_steps: usize) -> Result<(), String> {
        let items = self.base_economy_plan_items(base_id, max_steps);
        self.apply_named_plan(
            base_id,
            items,
            "Economic plan had no applicable actions.",
            "Economic plan",
        )
    }

    pub fn apply_economy_plan_action(&mut self, base_id: usize, max_steps: usize) -> bool {
        self.apply_economy_plan(base_id, max_steps)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(err);
                false
            })
    }

    pub fn apply_logistics_plan(&mut self, base_id: usize, max_steps: usize) -> Result<(), String> {
        let items = self.base_logistics_plan_items(base_id, max_steps);
        self.apply_named_plan(
            base_id,
            items,
            "Logistics plan had no applicable actions.",
            "Logistics plan",
        )
    }

    pub fn apply_logistics_plan_action(&mut self, base_id: usize, max_steps: usize) -> bool {
        self.apply_logistics_plan(base_id, max_steps)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(err);
                false
            })
    }

    pub fn apply_recovery_plans_all(&mut self, max_steps: usize) -> usize {
        let base_ids = self.stressed_recovery_base_ids();
        if base_ids.is_empty() {
            self.push_log("No stressed recovery bases need action.".to_string());
            return 0;
        }

        let mut applied = 0;
        for base_id in base_ids {
            let before = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(ProductionItem::ScoutPatrol);
            let _ = self.apply_recovery_plan(base_id, max_steps);
            let after = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(before);
            if after != before {
                applied += 1;
            }
        }

        self.push_log(format!(
            "Operations applied recovery planning to {} base(s).",
            applied
        ));
        applied
    }

    pub fn apply_defense_plans_all(&mut self, max_steps: usize) -> usize {
        let base_ids = self.frontier_base_ids();
        if base_ids.is_empty() {
            self.push_log("No frontier bases currently need defense planning.".to_string());
            return 0;
        }

        let mut applied = 0;
        for base_id in base_ids {
            let before = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(ProductionItem::ScoutPatrol);
            let _ = self.apply_defense_plan(base_id, max_steps);
            let after = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(before);
            if after != before {
                applied += 1;
            }
        }

        self.push_log(format!(
            "Operations applied frontier defense planning to {} base(s).",
            applied
        ));
        applied
    }

    pub fn apply_economy_plans_all(&mut self, owner: usize, max_steps: usize) -> usize {
        let base_ids = self.unrest_base_ids(owner);
        if base_ids.is_empty() {
            self.push_log("No stressed economic bases currently need action.".to_string());
            return 0;
        }

        let mut applied = 0;
        for base_id in base_ids {
            let before = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(ProductionItem::ScoutPatrol);
            let _ = self.apply_economy_plan(base_id, max_steps);
            let after = self
                .base(base_id)
                .map(|base| base.production)
                .unwrap_or(before);
            if after != before {
                applied += 1;
            }
        }

        self.push_log(format!(
            "Operations applied economic planning to {} base(s).",
            applied
        ));
        applied
    }

    pub fn apply_enabled_automations(&mut self, max_steps: usize) -> usize {
        let plans: Vec<(usize, GovernorMode)> = self
            .bases
            .iter()
            .map(|base| (base.id, base.governor_mode))
            .filter(|(_, mode)| *mode != GovernorMode::Off)
            .collect();
        let mut applied = 0;
        for (base_id, mode) in plans {
            if self.base(base_id).is_none() {
                continue;
            }
            let changed = match mode {
                GovernorMode::Off => false,
                GovernorMode::Balanced => self.apply_balanced_plan_action(base_id, max_steps),
                GovernorMode::Recovery => self.apply_recovery_plan_action(base_id, max_steps),
                GovernorMode::Defense => self.apply_defense_plan_action(base_id, max_steps),
                GovernorMode::Economy => self.apply_economy_plan_action(base_id, max_steps),
                GovernorMode::Logistics => self.apply_logistics_plan_action(base_id, max_steps),
                GovernorMode::MachinePolity => {
                    self.apply_governor_plan_queue_action(base_id, max_steps)
                }
            };
            if changed {
                applied += 1;
            }
        }
        applied
    }

    pub fn default_save_slot_name(&self, id: &str) -> String {
        if id.is_empty() {
            "Unnamed Save".to_string()
        } else {
            let faction_name = self.faction_name(self.player_owner());
            format!("Year {} - {}", 2100 + self.turn, faction_name)
        }
    }

    pub fn build_save_metadata(
        &self,
        save_name: String,
        notes: String,
        category: crate::save::SaveSlotCategory,
    ) -> crate::save::SaveSlotMetadata {
        let mut auto_recovery_base_ids: Vec<usize> = self
            .bases
            .iter()
            .filter(|base| base.governor_mode == GovernorMode::Recovery)
            .map(|base| base.id)
            .collect();
        let mut auto_defense_base_ids: Vec<usize> = self
            .bases
            .iter()
            .filter(|base| base.governor_mode == GovernorMode::Defense)
            .map(|base| base.id)
            .collect();
        let mut auto_economy_base_ids: Vec<usize> = self
            .bases
            .iter()
            .filter(|base| base.governor_mode == GovernorMode::Economy)
            .map(|base| base.id)
            .collect();
        auto_recovery_base_ids.sort_unstable();
        auto_defense_base_ids.sort_unstable();
        auto_economy_base_ids.sort_unstable();

        crate::save::SaveSlotMetadata {
            save_name,
            saved_turn: self.turn,
            recovery_note_count: 0,
            last_updated_unix: None,
            notes,
            category: Some(category),
            auto_recovery_base_ids,
            auto_defense_base_ids,
            auto_economy_base_ids,
        }
    }

    fn apply_named_plan(
        &mut self,
        base_id: usize,
        items: Vec<ProductionItem>,
        empty_message: &str,
        label: &str,
    ) -> Result<(), String> {
        if items.is_empty() {
            self.push_log(empty_message.to_string());
            return Ok(());
        }

        let first = items[0];
        self.set_base_production(base_id, first)
            .map_err(|err| format!("{label} failed to set production: {err}"))?;

        let mut queued = 0;
        for item in items.into_iter().skip(1) {
            if self.queue_base_production(base_id, item).is_ok() {
                queued += 1;
            }
        }

        self.push_log(format!(
            "{label} queued {} and {} follow-up item(s).",
            presentation::production_name(first),
            queued
        ));
        Ok(())
    }

    pub fn governor_mode_for_base(&self, base_id: usize) -> GovernorMode {
        self.base(base_id)
            .map(|base| base.governor_mode)
            .unwrap_or(GovernorMode::Off)
    }

    pub fn stressed_recovery_base_ids(&self) -> Vec<usize> {
        let owner = self.player_owner();
        let mut ids: Vec<(usize, usize, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| {
                let stress = self.damaged_garrison_count(base.id);
                let psi = self.base_local_psi_pressure(base.id);
                (base.id, stress, psi)
            })
            .filter(|(_, damaged, psi)| *damaged > 0 || *psi >= 2)
            .collect();
        ids.sort_by_key(|(id, damaged, psi)| (-(*damaged as i32), -*psi, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn frontier_base_ids(&self) -> Vec<usize> {
        let owner = self.player_owner();
        let mut ids: Vec<(usize, i32, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| {
                (
                    base.id,
                    self.base_local_military_pressure(base.id),
                    self.base_local_psi_pressure(base.id),
                )
            })
            .filter(|(_, military, psi)| *military >= 2 || *psi >= 2)
            .collect();
        ids.sort_by_key(|(id, military, psi)| (-*military, -*psi, *id as i32));
        ids.into_iter().map(|(id, _, _)| id).collect()
    }

    pub fn saturated_convoy_base_ids(&self, owner: usize) -> Vec<usize> {
        let mut ids: Vec<(usize, String, usize, usize)> = self
            .faction_convoy_saturation(owner)
            .into_iter()
            .filter(|(_, _, used, capacity)| *capacity > 0 && *used >= *capacity)
            .collect();
        ids.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));
        ids.into_iter().map(|(id, _, _, _)| id).collect()
    }

    pub fn tight_convoy_base_ids(&self, owner: usize) -> Vec<usize> {
        let mut ids: Vec<(usize, String, usize, usize)> = self
            .faction_convoy_saturation(owner)
            .into_iter()
            .filter(|(_, _, used, capacity)| *capacity > 0 && *used + 1 >= *capacity)
            .collect();
        ids.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));
        ids.into_iter().map(|(id, _, _, _)| id).collect()
    }

    pub fn unrest_base_ids(&self, owner: usize) -> Vec<usize> {
        let mut ids: Vec<(usize, i32)> = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .map(|base| (base.id, self.base_unrest(base.id)))
            .filter(|(_, unrest)| *unrest > 0)
            .collect();
        ids.sort_by_key(|(id, unrest)| (-*unrest, *id as i32));
        ids.into_iter().map(|(id, _)| id).collect()
    }

    pub fn base_storage_summary(&self, base_id: usize) -> Option<String> {
        self.base(base_id).map(|base| {
            presentation::format_base_storage(base.nutrients_stock, base.minerals_stock)
        })
    }

    pub fn faction_support_summary(&self, owner: usize) -> FactionSupportSummary {
        let facility_free_support: i32 = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .flat_map(|base| base.facilities.iter().copied())
            .map(content::facility_free_unit_support_bonus)
            .sum();
        let military_supply_bonus = self
            .convoy_routes
            .iter()
            .filter(|route| {
                self.base(route.base_a_id)
                    .zip(self.base(route.base_b_id))
                    .map(|(left, right)| {
                        left.owner == owner
                            && right.owner == owner
                            && route.kind == crate::ConvoyRouteKind::MilitarySupply
                    })
                    .unwrap_or(false)
            })
            .count() as i32;
        let support_attribute = self
            .faction(owner)
            .map(|f| f.effective_attributes().support)
            .unwrap_or(0);
        let base_free_support = (self.bases_for(owner).len() as i32)
            * (content::free_unit_support_per_base() + support_attribute).max(0);
        let live_units = self.live_units_for(owner).len() as i32;
        let total_free_support = base_free_support + facility_free_support + military_supply_bonus;
        let supported_units = (live_units - total_free_support).max(0);

        FactionSupportSummary {
            base_free_support,
            facility_free_support,
            military_supply_bonus,
            total_free_support,
            live_units,
            supported_units,
            unit_upkeep: supported_units * content::unit_support_cost(),
        }
    }

    pub fn faction_upkeep_breakdown(&self, owner: usize) -> (i32, i32, i32, i32) {
        let facility_upkeep: i32 = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .flat_map(|base| base.facilities.iter().copied())
            .map(content::facility_maintenance)
            .sum();
        let convoy_upkeep = self
            .convoy_routes
            .iter()
            .filter(|route| {
                self.base(route.base_a_id)
                    .zip(self.base(route.base_b_id))
                    .map(|(left, right)| left.owner == owner && right.owner == owner)
                    .unwrap_or(false)
            })
            .map(|route| self.convoy_route_upkeep(route.kind))
            .sum::<i32>();
        let unit_upkeep = self.faction_support_summary(owner).unit_upkeep;
        let infrastructure_upkeep = facility_upkeep + convoy_upkeep;
        (
            facility_upkeep,
            convoy_upkeep,
            unit_upkeep,
            infrastructure_upkeep + unit_upkeep,
        )
    }

    pub fn faction_upkeep(&self, owner: usize) -> (i32, i32, i32) {
        let (facility_upkeep, convoy_upkeep, unit_upkeep, total) =
            self.faction_upkeep_breakdown(owner);
        (facility_upkeep + convoy_upkeep, unit_upkeep, total)
    }

    fn convoy_route_upkeep(&self, kind: crate::ConvoyRouteKind) -> i32 {
        match kind {
            crate::ConvoyRouteKind::Trade => 1,
            crate::ConvoyRouteKind::Freight => 2,
            crate::ConvoyRouteKind::MilitarySupply => 2,
        }
    }

    pub fn is_production_available(&self, owner: usize, item: ProductionItem) -> bool {
        if let ProductionItem::CustomUnit(index) = item {
            return self
                .faction(owner)
                .map(|f| index < f.unit_designs.len())
                .unwrap_or(false);
        }

        if let Some(project) = item.secret_project() {
            // Already built by someone
            if self
                .built_secret_projects
                .iter()
                .any(|(p, _)| *p == project)
            {
                return false;
            }

            // Already being built by this owner elsewhere
            if self.bases.iter().any(|b| {
                b.owner == owner && b.production.secret_project() == Some(project)
            }) {
                return false;
            }
        }

        let Some(required_tech) = content::required_tech_for_production(item) else {
            return true;
        };
        self.faction(owner)
            .map(|faction| faction.known_techs.contains(&required_tech))
            .unwrap_or(false)
    }

    pub fn is_research_available(&self, owner: usize, tech: Tech) -> bool {
        self.faction(owner)
            .map(|faction| content::tech_is_available(&faction.known_techs, tech))
            .unwrap_or(false)
    }

    pub fn tile_map_label(&self, x: usize, y: usize, viewer_owner: usize) -> String {
        let Some(tile) = self.tile(x, y) else {
            return " ".to_string();
        };

        if !self.tile_explored_by_owner(x, y, viewer_owner) {
            return " ".to_string();
        }

        if self.tile_visible_to_owner(x, y, viewer_owner) {
            if let Some(unit_id) = tile.unit {
                if let Some(unit) = self.unit(unit_id) {
                    return presentation::unit_map_symbol(unit.kind.clone(), unit.owner);
                }
            }

            if tile.pod {
                return "?".to_string();
            }
        }

        if let Some(base_id) = tile.base {
            if let Some(base) = self.base(base_id) {
                return presentation::base_map_symbol(base.owner, viewer_owner).to_string();
            }
        }

        if let Some(improvement) = tile.improvement {
            return presentation::improvement_glyph(improvement).to_string();
        }

        presentation::terrain_symbol(tile.terrain).to_string()
    }

    pub fn tile_map_label_for_overlay(
        &self,
        x: usize,
        y: usize,
        viewer_owner: usize,
        overlay: presentation::MapOverlay,
    ) -> String {
        if matches!(overlay, presentation::MapOverlay::Logistics) {
            if let Some(glyph) = self.convoy_overlay_glyph_at(viewer_owner, x, y) {
                return glyph.to_string();
            }
        }
        if matches!(overlay, presentation::MapOverlay::Trade) {
            if self.tile(x, y).and_then(|t| t.base).is_some() {
                return "$".to_string();
            }
        }
        self.tile_map_label(x, y, viewer_owner)
    }

    pub fn choose_research(&mut self, owner: usize, tech: Tech) {
        if owner >= self.factions.len() {
            return;
        }

        if self
            .faction(owner)
            .map(|faction| faction.known_techs.contains(&tech))
            .unwrap_or(false)
        {
            self.push_log(format!(
                "{} is already known.",
                presentation::tech_name(tech)
            ));
            return;
        }

        if !self.is_research_available(owner, tech) {
            self.push_log(format!(
                "{} cannot begin {} yet.",
                self.faction_name(owner),
                presentation::tech_name(tech)
            ));
            return;
        }

        if let Some(faction) = self.faction_mut(owner) {
            faction.current_research = tech;
            faction.research = 0;
        }
        let faction = self.faction_name(owner).to_string();
        self.push_log(format!(
            "{faction} began research: {}.",
            presentation::tech_name(tech)
        ));
    }

    pub fn set_base_production(
        &mut self,
        base_id: usize,
        item: ProductionItem,
    ) -> Result<(), String> {
        let base_index = self
            .bases
            .iter_mut()
            .position(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        let owner = self.bases[base_index].owner;
        let faction = self
            .faction(owner)
            .ok_or_else(|| "Base owner is invalid.".to_string())?
            .clone();
        let base = &mut self.bases[base_index];

        Self::validate_base_build_choice(base, faction, item)?;
        base.production = item;
        base.minerals_stock = 0;
        base.production_queue.clear();

        let name = base.name.clone();
        self.push_log(format!(
            "{name} switched production to {}.",
            presentation::production_name(item)
        ));
        Ok(())
    }

    pub fn set_base_production_action(&mut self, base_id: usize, item: ProductionItem) -> bool {
        self.set_base_production(base_id, item)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Production change failed: {err}"));
                false
            })
    }

    pub fn queue_base_production(
        &mut self,
        base_id: usize,
        item: ProductionItem,
    ) -> Result<(), String> {
        let base_index = self
            .bases
            .iter_mut()
            .position(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        let owner = self.bases[base_index].owner;
        let faction = self
            .faction(owner)
            .ok_or_else(|| "Base owner is invalid.".to_string())?
            .clone();
        let base = &mut self.bases[base_index];

        Self::validate_base_build_choice(base, faction, item)?;
        if base.production_queue.contains(&item) && item.facility().is_some() {
            return Err("That facility is already queued.".to_string());
        }
        base.production_queue.push(item);
        let name = base.name.clone();
        self.push_log(format!(
            "{name} queued {}.",
            presentation::production_name(item)
        ));
        Ok(())
    }

    pub fn queue_base_production_action(&mut self, base_id: usize, item: ProductionItem) -> bool {
        self.queue_base_production(base_id, item)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Production queue change failed: {err}"));
                false
            })
    }

    pub fn promote_queued_production_to_active(
        &mut self,
        base_id: usize,
        queue_index: usize,
    ) -> Result<(), String> {
        let base = self
            .bases
            .iter_mut()
            .find(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        if queue_index >= base.production_queue.len() {
            return Err("Queue item not found.".to_string());
        }

        let next_active = base.production_queue.remove(queue_index);
        let previous_active = base.production;
        base.production = next_active;
        base.production_queue.insert(0, previous_active);
        base.minerals_stock = 0;

        let name = base.name.clone();
        self.push_log(format!(
            "{name} promoted {} to active production.",
            presentation::production_name(next_active)
        ));
        Ok(())
    }

    pub fn promote_queued_production_to_active_action(
        &mut self,
        base_id: usize,
        queue_index: usize,
    ) -> bool {
        self.promote_queued_production_to_active(base_id, queue_index)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Queue promotion failed: {err}"));
                false
            })
    }

    pub fn remove_queued_production(
        &mut self,
        base_id: usize,
        queue_index: usize,
    ) -> Result<(), String> {
        let base = self
            .bases
            .iter_mut()
            .find(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        if queue_index >= base.production_queue.len() {
            return Err("Queue item not found.".to_string());
        }
        let removed = base.production_queue.remove(queue_index);
        let name = base.name.clone();
        self.push_log(format!(
            "{name} removed {} from its queue.",
            presentation::production_name(removed)
        ));
        Ok(())
    }

    pub fn remove_queued_production_action(&mut self, base_id: usize, queue_index: usize) -> bool {
        self.remove_queued_production(base_id, queue_index)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Queue removal failed: {err}"));
                false
            })
    }

    pub fn clear_production_queue(&mut self, base_id: usize) -> Result<(), String> {
        let base = self
            .bases
            .iter_mut()
            .find(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        if base.production_queue.is_empty() {
            return Err("Production queue is already empty.".to_string());
        }
        base.production_queue.clear();
        let name = base.name.clone();
        self.push_log(format!("{name} cleared its production queue."));
        Ok(())
    }

    pub fn clear_production_queue_action(&mut self, base_id: usize) -> bool {
        self.clear_production_queue(base_id)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Queue clear failed: {err}"));
                false
            })
    }

    pub fn move_queued_production_up(
        &mut self,
        base_id: usize,
        queue_index: usize,
    ) -> Result<(), String> {
        let base = self
            .bases
            .iter_mut()
            .find(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        if queue_index == 0 || queue_index >= base.production_queue.len() {
            return Err("Queue item cannot move up.".to_string());
        }
        base.production_queue.swap(queue_index - 1, queue_index);
        let name = base.name.clone();
        self.push_log(format!("{name} reprioritized its build queue."));
        Ok(())
    }

    pub fn move_queued_production_up_action(&mut self, base_id: usize, queue_index: usize) -> bool {
        self.move_queued_production_up(base_id, queue_index)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Queue move failed: {err}"));
                false
            })
    }

    pub fn move_queued_production_down(
        &mut self,
        base_id: usize,
        queue_index: usize,
    ) -> Result<(), String> {
        let base = self
            .bases
            .iter_mut()
            .find(|b| b.id == base_id)
            .ok_or_else(|| "Base not found.".to_string())?;
        if queue_index + 1 >= base.production_queue.len() {
            return Err("Queue item cannot move down.".to_string());
        }
        base.production_queue.swap(queue_index, queue_index + 1);
        let name = base.name.clone();
        self.push_log(format!("{name} reprioritized its build queue."));
        Ok(())
    }

    pub fn move_queued_production_down_action(
        &mut self,
        base_id: usize,
        queue_index: usize,
    ) -> bool {
        self.move_queued_production_down(base_id, queue_index)
            .map(|_| true)
            .unwrap_or_else(|err| {
                self.push_log(format!("Queue move failed: {err}"));
                false
            })
    }

    pub fn set_base_governor_mode_action(&mut self, base_id: usize, mode: GovernorMode) -> bool {
        let Some(base) = self.base_mut(base_id) else {
            self.push_log("Governor change failed: base not found.".to_string());
            return false;
        };
        if base.governor_mode == mode {
            return false;
        }
        base.governor_mode = mode;
        let name = base.name.clone();
        self.push_log(format!(
            "{} governor set to {}.",
            name,
            presentation::governor_mode_label(mode)
        ));
        true
    }

    pub fn build_improvement(
        &mut self,
        unit_id: usize,
        improvement: Improvement,
    ) -> Result<(), String> {
        let unit = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        if !unit.kind.can_terraform() {
            return Err("Only formers can terraform.".to_string());
        }

        if unit.moves_left <= 0 {
            return Err("That former has no moves left this turn.".to_string());
        }

        let idx = self.tile_index(unit.x, unit.y);

        if !self.tiles[idx].terrain.is_land() {
            return Err("Cannot terraform ocean yet.".to_string());
        }

        if improvement == Improvement::ThermalBorehole {
            if self.count_adjacent_improvements(unit.x, unit.y, Improvement::ThermalBorehole) > 0 {
                return Err("Thermal Boreholes cannot be built adjacent to each other.".to_string());
            }
        }

        self.tiles[idx].improvement = Some(improvement);

        if let Some(active_unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            active_unit.moves_left = 0;
        }

        self.push_log(format!(
            "Former built {} at {}, {}.",
            presentation::improvement_name(improvement),
            unit.x,
            unit.y
        ));

        Ok(())
    }

    pub fn move_unit_to(
        &mut self,
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    ) -> Result<(), String> {
        if self.game_over.is_some() {
            return Err("Game is already over.".to_string());
        }

        if target_x >= self.width || target_y >= self.height {
            return Err("Target is outside the map.".to_string());
        }

        let unit_snapshot = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        if unit_snapshot.moves_left <= 0 {
            return Err("That unit has no moves left this turn.".to_string());
        }

        let is_drop_pod = self.unit_has_ability(unit_id, Ability::DropPod);
        let is_explored = self
            .tile(target_x, target_y)
            .map(|t| t.explored_by_owner.contains(&unit_snapshot.owner))
            .unwrap_or(false);

        if !is_drop_pod && !Self::is_adjacent(unit_snapshot.x, unit_snapshot.y, target_x, target_y)
        {
            return Err("Units can only move one tile at a time.".to_string());
        }

        if is_drop_pod && !Self::is_adjacent(unit_snapshot.x, unit_snapshot.y, target_x, target_y) {
            if !is_explored {
                return Err("Drop pods can only deploy to explored territory.".to_string());
            }
        }

        let target_idx = self.tile_index(target_x, target_y);
        let target_tile = self.tiles[target_idx].clone();

        if !target_tile.terrain.is_land() && !self.unit_can_enter_ocean(unit_id) {
            return Err("That unit cannot enter the ocean.".to_string());
        }

        if target_tile.terrain.is_land() && !self.unit_can_enter_land(unit_id) {
            return Err("Sea units cannot enter land.".to_string());
        }

        // Interception Check: Patrolling aircraft can intercept movement
        if let Some(intercepting_id) = self.find_interceptor(unit_id, target_x, target_y) {
            self.resolve_combat(intercepting_id, unit_id, target_x, target_y);
            // If unit died, stop movement
            if !self.unit(unit_id).map(|u| u.alive).unwrap_or(false) {
                return Ok(());
            }
        }

        if let Some(defender_id) = target_tile.unit {
            let defender = self
                .units
                .iter()
                .find(|u| u.id == defender_id && u.alive)
                .cloned()
                .ok_or_else(|| "Target unit state is corrupt.".to_string())?;

            if defender.owner == unit_snapshot.owner {
                return Err("Another friendly unit is already on that tile.".to_string());
            }

            // Block combat between allies (Treaty or Pact)
            let status = self.relations[unit_snapshot.owner][defender.owner].status;
            if status == DiplomacyStatus::Treaty || status == DiplomacyStatus::Pact {
                return Err(format!(
                    "Cannot attack ally (status: {:?}).",
                    status
                ));
            }

            self.resolve_combat(unit_id, defender_id, target_x, target_y);
            self.update_player_visibility();
            self.check_game_over();
            return Ok(());
        }

        self.move_unit_without_combat(unit_id, target_x, target_y)?;

        if self.tiles[target_idx].pod {
            self.tiles[target_idx].pod = false;
            self.resolve_supply_pod(unit_id, target_x, target_y);
        }

        if let Some(base_id) = self.tiles[target_idx].base {
            let owner = self.bases[base_id].owner;
            if owner != unit_snapshot.owner {
                // AUTO-DECLARE WAR on Capture
                if owner != self.native_owner()
                    && unit_snapshot.owner != self.native_owner()
                {
                    if self.relations[unit_snapshot.owner][owner].status != DiplomacyStatus::War {
                        let _ = self.update_diplomacy(
                            unit_snapshot.owner,
                            owner,
                            DiplomacyStatus::War,
                        );
                    }

                    // MUTUAL DEFENSE: Allies of the defender declare war on the attacker
                    let previous_owner_allies: Vec<usize> = (0..self.factions.len())
                        .filter(|&id| {
                            id != owner 
                                && id != unit_snapshot.owner
                                && self.relations[owner][id].status == DiplomacyStatus::Pact
                        })
                        .collect();
                    
                    for ally_id in previous_owner_allies {
                        if self.relations[unit_snapshot.owner][ally_id].status != DiplomacyStatus::War {
                            let _ = self.update_diplomacy(unit_snapshot.owner, ally_id, DiplomacyStatus::War);
                        }
                    }
                }

                self.bases[base_id].owner = unit_snapshot.owner;
                let faction_name = self.faction_name(unit_snapshot.owner).to_string();
                let base_name = self.bases[base_id].name.clone();
                self.push_log(format!("{faction_name} captured {base_name}!"));
            }
        }

        self.update_player_visibility();
        self.check_game_over();
        Ok(())
    }

    pub fn spawn_base(&mut self, owner: usize, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        if self.tile(x, y).map(|t| t.base.is_some()).unwrap_or(true) {
            return None;
        }

        let id = self.bases.len();
        let name = format!("Base {}", id);
        let base = Base {
            id,
            owner,
            name: name.clone(),
            x,
            y,
            population: 1,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        };

        self.bases.push(base);
        let idx = self.tile_index(x, y);
        self.tiles[idx].base = Some(id);
        self.push_log(format!("EDITOR: Spawned {} for faction {}.", name, owner));
        Some(id)
    }

    pub fn found_base(&mut self, unit_id: usize) -> Result<(), String> {
        if self.game_over.is_some() {
            return Err("Game is already over.".to_string());
        }

        let unit_snapshot = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        if !unit_snapshot.kind.can_found_base() {
            return Err("Only colony pods can found bases.".to_string());
        }

        let idx = self.tile_index(unit_snapshot.x, unit_snapshot.y);

        if !self.tiles[idx].terrain.is_land() && !self.unit_is_sea_unit(unit_id) {
            return Err("Only sea units can found bases in the ocean.".to_string());
        }

        if self.tiles[idx].base.is_some() {
            return Err("There is already a base here.".to_string());
        }

        let base_id = self.bases.len();
        let name = self.next_base_name(unit_snapshot.owner);

        let mut facilities = Vec::new();
        if let Some(faction) = self.faction(unit_snapshot.owner) {
            facilities.extend(faction.base_attributes.free_facilities.clone());
        }

        self.bases.push(Base {
            id: base_id,
            owner: unit_snapshot.owner,
            name: name.clone(),
            x: unit_snapshot.x,
            y: unit_snapshot.y,
            population: content::base_starting_population(),
            nutrients_stock: content::base_starting_nutrients_stock(),
            minerals_stock: content::base_starting_minerals_stock(),
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });

        self.tiles[idx].base = Some(base_id);
        self.destroy_unit(unit_id);

        if let Some(faction) = self.faction_mut(unit_snapshot.owner) {
            if faction.headquarters_base_id.is_none() {
                faction.headquarters_base_id = Some(base_id);
            }
        }

        let faction_name = self.faction_name(unit_snapshot.owner).to_string();
        self.push_log(format!("{faction_name} founded {name} at ({}, {}).", unit_snapshot.x, unit_snapshot.y));

        self.update_player_visibility();
        self.check_game_over();

        Ok(())
    }

    pub fn end_turn(&mut self) {
        if self.game_over.is_some() {
            return;
        }

        let player_owner = self.player_owner();
        let ai_owner = self.ai_owner();

        self.turn += 1;
        self.push_log(format!("--- Mission Year {} ---", 2100 + self.turn));

        if self.dust_fall_turns_left > 0 {
            self.dust_fall_turns_left -= 1;
            if self.dust_fall_turns_left == 0 {
                self.push_log(
                    "MOONFALL: Dust fall has dissipated. Nutrient yields returning to normal."
                        .to_string(),
                );
            }
        }

        if self.tidal_chaos_turns_left > 0 {
            self.tidal_chaos_turns_left -= 1;
            if self.tidal_chaos_turns_left == 0 {
                self.push_log(
                    "MOONFALL: Tidal chaos has subsided. Coastal waters are receding.".to_string(),
                );
            }
        }

        self.process_faction_economy(player_owner);
        self.repair_units_for_owner(player_owner);

        ai::run_non_player_turns(self);

        self.process_faction_economy(ai_owner);
        self.repair_units_for_owner(ai_owner);

        self.process_planetary_impact();

        self.reset_moves_for_owner(player_owner);
        self.update_player_visibility();
        self.update_diplomatic_attitudes();
        self.check_council_activation();
        self.check_game_over();
    }

    pub fn run_autoplay_mission_year(&mut self) {
        if self.game_over.is_some() {
            return;
        }

        ai::run_autoplay_turn_for_owner(self, self.player_owner());
        self.end_turn();
    }

    fn process_planetary_impact(&mut self) {
        let mut borehole_count = 0;
        for tile in &self.tiles {
            if tile.improvement == Some(Improvement::ThermalBorehole) {
                borehole_count += 1;
            }
        }

        if borehole_count > 0 {
            let mut crisis_messages = Vec::new();
            let mut total_toxicity = 0;

            for faction in &mut self.factions {
                faction.planet_toxicity += borehole_count;
                total_toxicity += faction.planet_toxicity;

                if faction.planet_toxicity > 500 && faction.planet_toxicity % 100 < borehole_count {
                    crisis_messages.push(format!("PLANETARY STRESS: High borehole activity is poisoning the biosphere for {}!", faction.name));
                }
            }

            // Global Crisis Triggers
            let roll = self.sample_noise(self.turn, total_toxicity, 999) % 1000;
            if total_toxicity > 2000 && roll < borehole_count as u32 {
                if self.dust_fall_turns_left == 0 && self.tidal_chaos_turns_left == 0 {
                    if roll % 2 == 0 {
                        self.dust_fall_turns_left = 10;
                        crisis_messages.push("GLOBAL CRISIS: Severe tectonic strain has triggered Atmospheric Dust Fall!".to_string());
                    } else {
                        self.tidal_chaos_turns_left = 10;
                        crisis_messages.push(
                            "GLOBAL CRISIS: Subterranean pressure has triggered Tidal Chaos!"
                                .to_string(),
                        );
                    }
                }
            }

            for msg in crisis_messages {
                self.push_event_log(EventCategory::Crisis, msg);
            }
        }
    }

    fn generate_map(&mut self) {
        self.tiles.clear();

        for y in 0..self.height {
            for x in 0..self.width {
                // Blend low-frequency and high-frequency noise to create biomes
                let macro_alt = self.sample_noise((x / 4) as i32, (y / 4) as i32, 17) % 100;
                let micro_alt = self.sample_noise(x as i32, y as i32, 117) % 100;
                let altitude = (macro_alt * 2 + micro_alt) / 3;

                let macro_moi = self.sample_noise((x / 4) as i32, (y / 4) as i32, 93) % 100;
                let micro_moi = self.sample_noise(x as i32, y as i32, 193) % 100;
                let moisture = (macro_moi * 2 + micro_moi) / 3;

                let macro_roc = self.sample_noise((x / 3) as i32, (y / 3) as i32, 211) % 100;
                let micro_roc = self.sample_noise(x as i32, y as i32, 311) % 100;
                let rock = (macro_roc * 2 + micro_roc) / 3;

                let macro_fun = self.sample_noise((x / 3) as i32, (y / 3) as i32, 501) % 100;
                let micro_fun = self.sample_noise(x as i32, y as i32, 601) % 100;
                let fungus = (macro_fun * 2 + micro_fun) / 3;

                let terrain = if altitude < content::map_ocean_threshold() {
                    Terrain::Ocean
                } else if fungus > content::map_fungus_threshold() {
                    Terrain::Fungus
                } else if rock > content::map_rocky_threshold() {
                    Terrain::Rocky
                } else if moisture > content::map_flat_moisture_threshold() {
                    Terrain::Flat
                } else {
                    Terrain::Rolling
                };

                let pod_roll = self.sample_noise(x as i32, y as i32, 777) % 100;
                let has_pod = terrain.is_land() && pod_roll > content::map_pod_spawn_threshold();

                self.tiles.push(Tile {
                    x,
                    y,
                    terrain,
                    elevation: altitude as i32,
                    moisture: moisture as i32,
                    unit: None,
                    base: None,
                    pod: has_pod,
                    improvement: None,
                    explored_by_owner: Default::default(),
                    visible_by_owner: Default::default(),
                });
            }
        }
    }

    fn force_land_patch(&mut self, cx: usize, cy: usize) {
        let radius = content::forced_land_patch_radius();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;

                if nx < 0 || ny < 0 {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                if nx >= self.width || ny >= self.height {
                    continue;
                }

                let idx = self.tile_index(nx, ny);
                self.tiles[idx].terrain = if (dx.abs() + dy.abs()) % 3 == 0 {
                    Terrain::Rolling
                } else {
                    Terrain::Flat
                };
                self.tiles[idx].pod = false;
            }
        }
    }

    pub fn spawn_unit(&mut self, owner: usize, kind: UnitKind, x: usize, y: usize) -> usize {
        self.spawn_unit_with_experience(owner, kind, x, y, 0)
    }

    pub fn spawn_unit_with_design(
        &mut self,
        owner: usize,
        kind: UnitKind,
        design_index: usize,
        x: usize,
        y: usize,
        experience: i32,
    ) -> Option<usize> {
        let id = self.units.len();
        let idx = self.tile_index(x, y);
        let initial_moves = self.effective_unit_max_moves_at(owner, kind.clone(), x, y);

        self.units.push(Unit {
            id,
            owner,
            kind: kind.clone(),
            design_index,
            x,
            y,
            moves_left: initial_moves,
            hp: content::unit_base_hp(kind),
            experience,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
            alive: true,
        });

        self.tiles[idx].unit = Some(id);
        Some(id)
    }

    fn spawn_unit_with_experience(
        &mut self,
        owner: usize,
        kind: UnitKind,
        x: usize,
        y: usize,
        experience: i32,
    ) -> usize {
        let id = self.units.len();
        let idx = self.tile_index(x, y);
        let initial_moves = self.effective_unit_max_moves_at(owner, kind.clone(), x, y);
        let design_index = self.find_design_index_for_kind(owner, kind.clone());

        self.units.push(Unit {
            id,
            owner,
            kind: kind.clone(),
            design_index,
            x,
            y,
            moves_left: initial_moves,
            hp: content::unit_base_hp(kind),
            experience,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
            alive: true,
        });

        self.tiles[idx].unit = Some(id);
        id
    }

    pub fn find_design_index_for_kind(&self, owner: usize, kind: UnitKind) -> usize {
        let Some(faction) = self.faction(owner) else {
            return 0;
        };
        let target_name = match kind.clone() {
            UnitKind::CustomUnit(design) => design.name,
            _ => content::unit_name(kind.clone()).to_string(),
        };
        faction
            .unit_designs
            .iter()
            .position(|d| {
                d.name == target_name
                    || matches!(&kind, UnitKind::CustomUnit(design) if d == design)
            })
            .unwrap_or(0)
    }

    pub fn add_unit_design(&mut self, owner: usize, design: UnitDesign) {
        if let Some(faction) = self.faction_mut(owner) {
            faction.unit_designs.push(design);
        }
    }

    fn ensure_design_index(&mut self, owner: usize, design: &UnitDesign) -> Result<usize, String> {
        let faction = self
            .factions
            .get_mut(owner)
            .ok_or_else(|| "Faction not found.".to_string())?;

        if let Some(index) = faction
            .unit_designs
            .iter()
            .position(|existing| existing == design || existing.name == design.name)
        {
            return Ok(index);
        }

        faction.unit_designs.push(design.clone());
        Ok(faction.unit_designs.len() - 1)
    }

    pub fn choose_social_engineering(
        &mut self,
        owner: usize,
        politics: Option<Politics>,
        economics: Option<Economics>,
        values: Option<Values>,
        future: Option<FutureSociety>,
    ) -> Result<(), String> {
        let faction = self
            .factions
            .get_mut(owner)
            .ok_or_else(|| "Faction not found.".to_string())?;

        // Validate restrictions
        if let Some(e) = economics {
            if faction.name == "Morgan Industries" && matches!(e, Economics::Planned) {
                return Err("Morgan Industries cannot use Planned economics.".to_string());
            }
            if faction.name == "Gaia's Stepdaughters" && matches!(e, Economics::FreeMarket) {
                return Err("Gaia's Stepdaughters cannot use Free Market economics.".to_string());
            }
            if faction.name == "The Lord's Believers" && matches!(e, Economics::FreeMarket) {
                return Err("The Lord's Believers cannot use Free Market economics.".to_string());
            }
        }

        if let Some(p) = politics {
            if faction.name == "The Human Hive" && matches!(p, Politics::Democratic) {
                return Err("The Human Hive cannot use Democratic politics.".to_string());
            }
        }

        if let Some(v) = values {
            if faction.name == "The Lord's Believers" && matches!(v, Values::Knowledge) {
                return Err("The Lord's Believers cannot use Knowledge values.".to_string());
            }
        }

        if let Some(p) = politics {
            faction.social_engineering.politics = p;
        }
        if let Some(e) = economics {
            faction.social_engineering.economics = e;
        }
        if let Some(v) = values {
            faction.social_engineering.values = v;
        }
        if let Some(f) = future {
            faction.social_engineering.future = f;
        }

        let name = faction.name.clone();
        self.push_log(format!("{} updated Social Engineering.", name));

        Ok(())
    }

    pub fn upgrade_unit(&mut self, unit_id: usize, new_design: UnitDesign) -> Result<(), String> {
        let (owner, old_cost) = {
            let unit = self
                .unit(unit_id)
                .ok_or_else(|| "Unit not found.".to_string())?;
            (
                unit.owner,
                content::unit_design_cost(&UnitDesign {
                    name: String::new(),
                    chassis: content::unit_base_chassis(unit.kind.clone()),
                    weapon: content::unit_base_weapon(unit.kind.clone()),
                    armor: content::unit_base_armor(unit.kind.clone()),
                    abilities: Vec::new(),
                    cost: 0,
                }),
            )
        };

        let new_cost = content::unit_design_cost(&new_design);
        let energy_cost = (new_cost - old_cost).max(0) * 5;
        let design_index = self.ensure_design_index(owner, &new_design)?;
        let (x, y) = self
            .unit(unit_id)
            .map(|unit| (unit.x, unit.y))
            .ok_or_else(|| "Unit not found.".to_string())?;
        let upgraded_kind = UnitKind::CustomUnit(new_design.clone());
        let upgraded_hp = content::unit_base_hp(upgraded_kind.clone());
        let upgraded_moves = self.effective_unit_max_moves_at(owner, upgraded_kind.clone(), x, y);

        let faction = self
            .factions
            .get_mut(owner)
            .ok_or_else(|| "Faction not found.".to_string())?;
        if faction.energy < energy_cost {
            return Err(format!(
                "Insufficient energy for upgrade. Need {}, have {}.",
                energy_cost, faction.energy
            ));
        }

        faction.energy -= energy_cost;

        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id) {
            unit.kind = upgraded_kind;
            unit.design_index = design_index;
            unit.hp = upgraded_hp;
            unit.moves_left = upgraded_moves;
            self.push_log(format!("UPGRADE: Unit {} upgraded to new design.", unit_id));
        }

        Ok(())
    }

    pub fn perform_probe_action(
        &mut self,
        unit_id: usize,
        target_x: usize,
        target_y: usize,
        action: ProbeAction,
    ) -> Result<(), String> {
        let Some(unit) = self.unit(unit_id).cloned() else {
            return Err("Probe unit not found.".to_string());
        };

        if !self.unit_has_ability(unit_id, Ability::Probe) && unit.kind != UnitKind::ProbeTeam {
            return Err("Unit does not have Probe capabilities.".to_string());
        }

        if unit.moves_left <= 0 {
            return Err("Unit has no moves left.".to_string());
        }

        if !Self::is_adjacent(unit.x, unit.y, target_x, target_y)
            && !(unit.x == target_x && unit.y == target_y)
        {
            return Err("Target must be adjacent to the probe unit.".to_string());
        }

        let Some(target_tile) = self.tile(target_x, target_y).cloned() else {
            return Err("Invalid target tile.".to_string());
        };

        match action {
            ProbeAction::StealTech => {
                let Some(base_id) = target_tile.base else {
                    return Err("Must target an enemy base to steal tech.".to_string());
                };
                let target_owner = self.bases[base_id].owner;
                if target_owner == unit.owner {
                    return Err("Cannot steal tech from your own base.".to_string());
                }

                let target_techs = &self.factions[target_owner].known_techs;
                let our_techs = &self.factions[unit.owner].known_techs;

                let stealable: Vec<_> = target_techs
                    .iter()
                    .filter(|t| !our_techs.contains(t))
                    .cloned()
                    .collect();
                if stealable.is_empty() {
                    return Err("Target faction has no new technologies to steal.".to_string());
                }

                let stolen_tech = stealable[0]; // Simple selection for now
                if let Some(faction) = self.faction_mut(unit.owner) {
                    faction.known_techs.push(stolen_tech);
                    faction.techs_discovered += 1;
                }

                self.destroy_unit(unit_id);

                let owner_name = self.faction_name(unit.owner).to_string();
                let target_name = self.faction_name(target_owner).to_string();
                self.push_event_log(
                    EventCategory::Diplomacy,
                    format!(
                        "COVERT: {} probe team stole {} from {}!",
                        owner_name,
                        presentation::tech_name(stolen_tech),
                        target_name
                    ),
                );
            }
            ProbeAction::SabotageFacility => {
                let Some(base_id) = target_tile.base else {
                    return Err("Must target an enemy base to sabotage.".to_string());
                };
                let target_owner = self.bases[base_id].owner;
                if target_owner == unit.owner {
                    return Err("Cannot sabotage your own base.".to_string());
                }

                if self.bases[base_id].facilities.is_empty() {
                    return Err("Base has no facilities to sabotage.".to_string());
                }

                let destroyed = self.bases[base_id].facilities.pop().unwrap(); // Just pop the last one for now

                self.destroy_unit(unit_id);

                let owner_name = self.faction_name(unit.owner).to_string();
                let target_name = self.faction_name(target_owner).to_string();
                self.push_event_log(
                    EventCategory::Diplomacy,
                    format!(
                        "COVERT: {} probe team sabotaged {:?} in {}'s base {}!",
                        owner_name, destroyed, target_name, self.bases[base_id].name
                    ),
                );
            }
            ProbeAction::SubvertUnit => {
                let Some(target_unit_id) = target_tile.unit else {
                    return Err("Must target a unit to subvert.".to_string());
                };
                let target_owner = self.units[target_unit_id].owner;
                if target_owner == unit.owner {
                    return Err("Cannot subvert your own unit.".to_string());
                }

                let energy_cost = 50; // Fixed cost for now
                if self.factions[unit.owner].energy < energy_cost {
                    return Err(format!(
                        "Insufficient energy to subvert. Need {}.",
                        energy_cost
                    ));
                }

                self.factions[unit.owner].energy -= energy_cost;
                self.units[target_unit_id].owner = unit.owner;

                self.destroy_unit(unit_id);

                let owner_name = self.faction_name(unit.owner).to_string();
                let target_name = self.faction_name(target_owner).to_string();
                self.push_event_log(
                    EventCategory::Diplomacy,
                    format!(
                        "COVERT: {} probe team subverted a {} unit for {} energy!",
                        owner_name, target_name, energy_cost
                    ),
                );
            }
        }

        Ok(())
    }

    pub fn rush_build(&mut self, base_id: usize) -> Result<(), String> {
        let (owner, item, cost_minerals, base_name) = {
            let base = self
                .bases
                .get(base_id)
                .ok_or_else(|| "Base not found.".to_string())?;
            let cost = content::production_cost(base.production);
            (base.owner, base.production, cost, base.name.clone())
        };

        let remaining = {
            let base = &self.bases[base_id];
            (cost_minerals - base.minerals_stock).max(0)
        };
        let energy_cost = remaining * 2;

        let faction = self
            .factions
            .get_mut(owner)
            .ok_or_else(|| "Faction not found.".to_string())?;
        if faction.energy < energy_cost {
            return Err(format!(
                "Insufficient energy. Need {}, have {}.",
                energy_cost, faction.energy
            ));
        }

        faction.energy -= energy_cost;
        self.bases[base_id].minerals_stock = cost_minerals;
        self.push_log(format!(
            "RUSH BUILD: {} completed production of {} with energy credits.",
            base_name,
            presentation::production_name(item)
        ));
        self.complete_production(base_id);

        Ok(())
    }

    pub fn update_diplomacy(
        &mut self,
        faction_a: usize,
        faction_b: usize,
        status: DiplomacyStatus,
    ) -> Result<(), String> {
        if faction_a >= self.factions.len() || faction_b >= self.factions.len() {
            return Err("Invalid faction index.".to_string());
        }

        if faction_a == faction_b {
            return Err("Cannot update diplomacy with self.".to_string());
        }

        self.relations[faction_a][faction_b].status = status;
        self.relations[faction_b][faction_a].status = status;

        let name_a = self.faction_name(faction_a).to_string();
        let name_b = self.faction_name(faction_b).to_string();

        self.push_event_log(
            EventCategory::Diplomacy,
            format!(
                "DIPLOMACY: {} and {} have signed a {:?}.",
                name_a, name_b, status
            ),
        );

        if faction_a == self.player_owner() || faction_b == self.player_owner() {
            self.update_player_visibility();
        }

        Ok(())
    }

    pub fn find_interceptor(&self, target_unit_id: usize, x: usize, y: usize) -> Option<usize> {
        let target_unit = self.unit(target_unit_id)?;
        let is_target_aircraft = self.unit_is_aircraft(target_unit_id);

        for unit in &self.units {
            if !unit.alive || unit.owner == target_unit.owner {
                continue;
            }

            if unit.activity != UnitActivity::Patrol {
                continue;
            }

            if !self.unit_is_aircraft(unit.id) {
                continue;
            }

            // Must be adjacent to the target tile
            if !Self::is_adjacent(unit.x, unit.y, x, y) && (unit.x != x || unit.y != y) {
                continue;
            }

            // Air Superiority logic: only intercept aircraft if we have the ability?
            // Actually, any patrolling aircraft can intercept. Air Superiority just gets a bonus.
            if is_target_aircraft {
                return Some(unit.id);
            } else if !self.tile(x, y).map(|t| t.base.is_some()).unwrap_or(false) {
                // Ground unit in the open can be intercepted by patrolling aircraft
                return Some(unit.id);
            }
        }
        None
    }

    pub fn set_unit_activity(&mut self, unit_id: usize, activity: UnitActivity) {
        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            unit.activity = activity;
            if activity != UnitActivity::None {
                unit.moves_left = 0;
            }
        }
    }

    pub fn unload_unit(
        &mut self,
        unit_id: usize,
        transport_id: usize,
        target_x: usize,
        target_y: usize,
    ) -> Result<(), String> {
        let (tx, ty, owner) = {
            let transport = self.unit(transport_id).ok_or("Transport not found")?;
            (transport.x, transport.y, transport.owner)
        };

        if self.distance(tx, ty, target_x, target_y) > 1 {
            return Err("Cannot unload to a non-adjacent tile.".to_string());
        }

        let unit_index = self
            .units
            .iter()
            .position(|u| u.id == unit_id)
            .ok_or("Unit not found")?;

        // Check if unit can enter the target terrain
        if !self.tiles[self.tile_index(target_x, target_y)].terrain.is_land()
            && !self.unit_can_enter_ocean(unit_id)
        {
            return Err("That unit cannot enter the ocean.".to_string());
        }
        if self.tiles[self.tile_index(target_x, target_y)].terrain.is_land()
            && !self.unit_can_enter_land(unit_id)
        {
            return Err("That unit cannot enter land.".to_string());
        }

        let transport_index = self
            .units
            .iter()
            .position(|u| u.id == transport_id)
            .ok_or("Transport not found")?;

        if !self.units[transport_index]
            .cargo_unit_ids
            .contains(&unit_id)
        {
            return Err("Unit not in this transport's cargo".to_string());
        }

        // Move unit to map if tile is available
        let tile_index = self.tile_index(target_x, target_y);
        if self.tiles[tile_index].unit.is_some() {
            return Err("Destination tile already occupied".to_string());
        }

        self.units[transport_index]
            .cargo_unit_ids
            .retain(|&id| id != unit_id);
        self.units[unit_index].x = target_x;
        self.units[unit_index].y = target_y;
        self.units[unit_index].moves_left = 0;
        self.tiles[tile_index].unit = Some(unit_id);

        self.push_log(format!(
            "{} unloaded a unit at ({}, {}).",
            self.faction_name(owner),
            target_x,
            target_y
        ));

        Ok(())
    }

    pub fn load_unit(&mut self, unit_id: usize, transport_id: usize) -> Result<(), String> {
        if unit_id == transport_id {
            return Err("Unit cannot load itself".to_string());
        }

        let (ux, uy, tx, ty, owner) = {
            let unit = self.unit(unit_id).ok_or("Unit not found")?;
            let transport = self.unit(transport_id).ok_or("Transport not found")?;
            if unit.owner != transport.owner {
                return Err("Cannot load units from another faction".to_string());
            }
            if !self.unit_has_ability(transport_id, Ability::Transport) {
                return Err("Target unit is not a transport".to_string());
            }
            (unit.x, unit.y, transport.x, transport.y, unit.owner)
        };

        if self.distance(ux, uy, tx, ty) > 1 {
            return Err("Units must be adjacent or in the same tile to load".to_string());
        }

        let unit_index = self
            .units
            .iter()
            .position(|u| u.id == unit_id)
            .ok_or("Unit not found")?;

        let transport_index = self
            .units
            .iter()
            .position(|u| u.id == transport_id)
            .ok_or("Transport not found")?;

        // Capacity check: for now 2 units per transport
        if self.units[transport_index].cargo_unit_ids.len() >= 2 {
            return Err("Transport is full".to_string());
        }

        // Remove unit from map
        let tile_index = self.tile_index(ux, uy);
        if self.tiles[tile_index].unit == Some(unit_id) {
            self.tiles[tile_index].unit = None;
        }

        self.units[transport_index].cargo_unit_ids.push(unit_id);
        self.units[unit_index].moves_left = 0;

        self.push_log(format!(
            "{} loaded a unit onto a transport at ({}, {}).",
            self.faction_name(owner),
            ux,
            uy
        ));

        Ok(())
    }

    pub fn execute_demand(
        &mut self,
        proposer: usize,
        receiver: usize,
        demand: DemandKind,
    ) -> Result<(), String> {
        match demand {
            DemandKind::Technology(tech) => {
                let receiver_has_tech = self
                    .faction(receiver)
                    .map(|f| f.known_techs.contains(&tech))
                    .unwrap_or(false);
                let proposer_needs_tech = self
                    .faction(proposer)
                    .map(|f| !f.known_techs.contains(&tech))
                    .unwrap_or(false);

                if !receiver_has_tech || !proposer_needs_tech {
                    return Err(
                        "Invalid tech demand: receiver does not have tech or proposer already knows it."
                            .to_string(),
                    );
                }

                if let Some(f) = self.faction_mut(proposer) {
                    f.known_techs.push(tech);
                    f.techs_discovered += 1;
                }
                let proposer_name = self.faction_name(proposer).to_string();
                let receiver_name = self.faction_name(receiver).to_string();
                self.push_log(format!(
                    "DEMAND: {receiver_name} surrendered {} to {proposer_name}.",
                    presentation::tech_name(tech)
                ));
            }
            DemandKind::Energy(amount) => {
                let receiver_energy = self.faction(receiver).map(|f| f.energy).unwrap_or(0);
                let actual_amount = amount.min(receiver_energy);

                if let Some(f) = self.faction_mut(receiver) {
                    f.energy -= actual_amount;
                }
                if let Some(f) = self.faction_mut(proposer) {
                    f.energy += actual_amount;
                }
                let proposer_name = self.faction_name(proposer).to_string();
                let receiver_name = self.faction_name(receiver).to_string();
                self.push_log(format!(
                    "DEMAND: {receiver_name} paid {actual_amount} energy tribute to {proposer_name}."
                ));
            }
        }
        Ok(())
    }

    pub fn execute_tech_trade(
        &mut self,
        proposer: usize,
        receiver: usize,
        offered_tech: Tech,
        requested_tech: Tech,
    ) -> Result<(), String> {
        let (proposer_has_offered, proposer_needs_requested) = {
            let faction = self.factions.get(proposer).ok_or("Proposer not found")?;
            (
                faction.known_techs.contains(&offered_tech),
                !faction.known_techs.contains(&requested_tech),
            )
        };

        let (receiver_has_requested, receiver_needs_offered) = {
            let faction = self.factions.get(receiver).ok_or("Receiver not found")?;
            (
                faction.known_techs.contains(&requested_tech),
                !faction.known_techs.contains(&offered_tech),
            )
        };

        if !proposer_has_offered
            || !proposer_needs_requested
            || !receiver_has_requested
            || !receiver_needs_offered
        {
            return Err("Invalid tech trade: one or both factions do not have the required tech or already know the requested tech.".to_string());
        }

        if let Some(faction) = self.factions.get_mut(proposer) {
            faction.known_techs.push(requested_tech);
            faction.techs_discovered += 1;
        }

        if let Some(faction) = self.factions.get_mut(receiver) {
            faction.known_techs.push(offered_tech);
            faction.techs_discovered += 1;
        }

        let proposer_name = self.faction_name(proposer).to_string();
        let receiver_name = self.faction_name(receiver).to_string();

        self.push_event_log(
            EventCategory::Diplomacy,
            format!(
                "TECH TRADE: {} and {} traded technologies: {:?} for {:?}.",
                proposer_name, receiver_name, offered_tech, requested_tech
            ),
        );

        Ok(())
    }

    pub fn unit_is_exposed(&self, unit_id: usize) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        // A unit is exposed if it's not on a base or in a forest
        let tile = self.tile(unit.x, unit.y);
        tile.map(|t| t.base.is_none()).unwrap_or(true)
    }

    pub fn unit_is_psi_threat(&self, unit_id: usize) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        matches!(unit.kind, UnitKind::MindWorm | UnitKind::IsleOfTheDeep)
    }

    pub fn unit_is_aircraft(&self, unit_id: usize) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        match &unit.kind {
            UnitKind::Needlejet => true,
            UnitKind::CustomUnit(design) => design.chassis == Chassis::Aircraft,
            _ => false,
        }
    }

    pub fn unit_is_sea_unit(&self, unit_id: usize) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        match &unit.kind {
            UnitKind::IsleOfTheDeep | UnitKind::SeaColonyPod | UnitKind::SeaTransport => true,
            UnitKind::CustomUnit(design) => design.chassis == Chassis::Sea,
            _ => false,
        }
    }

    pub fn unit_is_at_sea(&self, unit_id: usize) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        self.tile(unit.x, unit.y)
            .map(|t| t.terrain == Terrain::Ocean)
            .unwrap_or(false)
    }

    pub fn unit_is_fast(&self, unit_id: usize) -> bool {
        let Some(unit) = self.unit(unit_id) else {
            return false;
        };
        unit.kind.clone().max_moves() > 1
    }

    pub fn unit_can_enter_ocean(&self, unit_id: usize) -> bool {
        self.unit_is_sea_unit(unit_id)
            || self.unit_is_aircraft(unit_id)
            || self.unit_has_ability(unit_id, Ability::Amphibious)
    }

    pub fn unit_can_enter_land(&self, unit_id: usize) -> bool {
        !self.unit_is_sea_unit(unit_id)
            || self.unit_is_aircraft(unit_id)
            || self.unit_has_ability(unit_id, Ability::Amphibious)
    }

    pub fn unit_weapon(&self, unit_id: usize) -> Option<Weapon> {
        let unit = self.unit(unit_id)?;
        let faction = self.faction(unit.owner)?;
        faction
            .unit_designs
            .get(unit.design_index)
            .map(|d| d.weapon)
    }

    pub fn unit_attack_strength(&self, unit_id: usize) -> i32 {
        let Some(unit) = self.unit(unit_id) else {
            return 0;
        };
        let Some(faction) = self.faction(unit.owner) else {
            return unit.kind.clone().attack();
        };
        faction
            .unit_designs
            .get(unit.design_index)
            .map(|d| d.attack_strength() as i32)
            .unwrap_or_else(|| unit.kind.clone().attack())
    }

    pub fn unit_defense_strength(&self, unit_id: usize) -> i32 {
        let Some(unit) = self.unit(unit_id) else {
            return 0;
        };
        let Some(faction) = self.faction(unit.owner) else {
            return unit.kind.clone().defense();
        };
        faction
            .unit_designs
            .get(unit.design_index)
            .map(|d| d.defense_strength() as i32)
            .unwrap_or_else(|| unit.kind.clone().defense())
    }

    pub fn spawn_unit_near(
        &mut self,
        owner: usize,
        kind: UnitKind,
        x: usize,
        y: usize,
    ) -> Option<usize> {
        let candidates = [
            (x, y),
            (x.saturating_add(1), y),
            (x.saturating_sub(1), y),
            (x, y.saturating_add(1)),
            (x, y.saturating_sub(1)),
            (x.saturating_add(1), y.saturating_add(1)),
            (x.saturating_sub(1), y.saturating_sub(1)),
            (x.saturating_add(1), y.saturating_sub(1)),
            (x.saturating_sub(1), y.saturating_add(1)),
        ];

        for (nx, ny) in candidates {
            if nx >= self.width || ny >= self.height {
                continue;
            }

            let idx = self.tile_index(nx, ny);
            let tile = &self.tiles[idx];

            if tile.terrain.is_land() && tile.unit.is_none() {
                return Some(self.spawn_unit_with_experience(owner, kind, nx, ny, 0));
            }
        }

        None
    }

    fn process_faction_economy(&mut self, owner: usize) {
        let base_ids: Vec<usize> = self
            .bases
            .iter()
            .filter(|b| b.owner == owner)
            .map(|b| b.id)
            .collect();

        let mut total_nutrients_produced = 0;
        let mut total_population = 0;
        let mut total_minerals_produced = 0;
        let mut ai_governance_score = 0;
        let mut transit_hub_count = 0;
        let mut borehole_count = 0;

        for tile in &self.tiles {
            if tile.improvement == Some(Improvement::ThermalBorehole)
                && self.control_owner_at(tile.x, tile.y) == Some(owner)
            {
                borehole_count += 1;
            }
        }

        for base_id in base_ids {
            let x = self.bases[base_id].x;
            let y = self.bases[base_id].y;
            let mut yields = self
                .operational_base_yields(base_id)
                .unwrap_or_else(|| self.base_yields(x, y));

            // Moonfall Crisis: Dust Fall reduces nutrient yields globally.
            if self.dust_fall_turns_left > 0
                && !self.has_secret_project(owner, SecretProject::WeatherPattern)
            {
                yields.nutrients = (yields.nutrients as f32 * 0.5) as i32;
            }

            total_nutrients_produced += yields.nutrients;
            total_population += self.bases[base_id].population;
            total_minerals_produced += yields.minerals;

            match self.bases[base_id].governor_mode {
                GovernorMode::MachinePolity => ai_governance_score += 15,
                GovernorMode::Off => {}
                _ => ai_governance_score += 5,
            }

            if self.bases[base_id]
                .facilities
                .contains(&Facility::TransitHub)
            {
                transit_hub_count += 1;
            }

            let food_surplus = yields.nutrients - self.bases[base_id].population.max(1);
            let prev_stock = self.bases[base_id].nutrients_stock;
            self.bases[base_id].nutrients_stock =
                (self.bases[base_id].nutrients_stock + food_surplus).max(0);

            if self.bases[base_id].nutrients_stock == 0 && food_surplus < 0 && prev_stock == 0 {
                if self.bases[base_id].population > 1 {
                    self.bases[base_id].population -= 1;
                    let name = self.bases[base_id].name.clone();
                    self.push_event_log(
                        EventCategory::Crisis,
                        format!("FAMINE: {name} population reduced due to starvation!"),
                    );
                }
            }
            self.bases[base_id].minerals_stock += yields.minerals;
            if let Some(faction) = self.faction_mut(owner) {
                faction.energy += yields.energy;

                let research_bonus = faction.effective_attributes().research;
                let multiplier = 1.0 + (research_bonus as f32 * 0.1);
                let research_gain = (yields.energy as f32 * multiplier) as i32;

                faction.research += research_gain;
            }

            let growth_threshold = self.base_growth_threshold(base_id);
            let unrest = self.base_unrest(base_id);
            let mut grew = false;

            if self.bases[base_id].nutrients_stock >= growth_threshold {
                self.bases[base_id].nutrients_stock -= growth_threshold;
                grew = true;
            } else if unrest == 0 {
                if let Some(faction) = self.faction(owner) {
                    if faction.effective_attributes().growth >= 6 {
                        let surplus = yields.nutrients - self.bases[base_id].population as i32;
                        if surplus > 0 {
                            grew = true;
                        }
                    }
                }
            }

            if grew {
                self.bases[base_id].population += 1;
                let name = self.bases[base_id].name.clone();
                self.push_log(format!(
                    "{name} grew to population {}.",
                    self.bases[base_id].population
                ));
            }

            if unrest > 0 {
                let name = self.bases[base_id].name.clone();
                self.push_log(format!(
                    "{name} is experiencing unrest {unrest}, reducing mineral and energy output."
                ));
            }

            if self.bases[base_id].production == ProductionItem::StockpileEnergy {
                let converted_energy = (yields.minerals as f32 * 0.5) as i32;
                if let Some(faction) = self.faction_mut(owner) {
                    faction.energy += converted_energy;
                }
                self.bases[base_id].minerals_stock = 0; // Consume all minerals
                self.check_research(owner);
                continue;
            }

            loop {
                let item = self.bases[base_id].production;
                let item_cost = self.production_cost(owner, item);
                if item_cost <= 0 {
                    break;
                }
                if self.bases[base_id].minerals_stock >= item_cost {
                    self.bases[base_id].minerals_stock -= item_cost;
                    self.complete_production(base_id);
                } else {
                    break;
                }
            }

            self.check_research(owner);
        }

        let total_bases = self.bases_for(owner).len();
        if let Some(faction) = self.faction(owner) {
            // Food Security: (nutrients / population * 100) - 100
            // 0 means self-sufficient (1 nutrient per pop), 100 means 2x surplus, -100 means famine.
            let mut fs = ((total_nutrients_produced as f32 / total_population.max(1) as f32)
                * 100.0
                - 100.0) as i32;

            if self.has_secret_project(owner, SecretProject::ClinicalImmortality) {
                fs += 25;
            }

            let fs_clamped = fs.clamp(-100, 100);

            // AI Dependence: Increases based on active governor complexity.
            // Machine Polity mode adds 15 points per turn per base, others add 5.
            let ai_dep_inc = (ai_governance_score as f32 / total_bases.max(1) as f32) as i32;
            let ai_dep_clamped = (faction.ai_dependence + ai_dep_inc).clamp(0, 100);

            // Planet Toxicity: Increases with mineral production, recovers slowly.
            // High Planet attribute reduces the increase.
            // Thermal Boreholes add extra ecological pressure.
            let borehole_impact = borehole_count * 2;
            let attributes = faction.effective_attributes();
            let tox_inc = (total_minerals_produced / 20) + borehole_impact - attributes.planet;
            let tox_clamped = (faction.planet_toxicity + tox_inc - 1).clamp(0, 100);

            // Orbital Index: Based on active space infrastructure (Transit Hubs).
            let orb_idx = (transit_hub_count as i32).min(10);

            if let Some(f_mut) = self.faction_mut(owner) {
                f_mut.food_security = fs_clamped;
                f_mut.ai_dependence = ai_dep_clamped;
                f_mut.planet_toxicity = tox_clamped;
                f_mut.orbital_index = orb_idx;
            }
        }
        self.process_strategic_crises(owner);
        self.capture_command_center_post_production_traces(owner);

        self.apply_convoy_interdiction(owner);
        self.apply_faction_upkeep(owner);
        self.update_command_center_end_stock_traces(owner);
        self.check_narrative_triggers(owner);
        self.emit_logistics_alert_events(owner);
    }

    fn process_strategic_crises(&mut self, owner: usize) {
        let Some(faction) = self.faction(owner).cloned() else {
            return;
        };

        if faction.food_security < -50 {
            self.push_log("GLOBAL FAMINE: Widespread starvation causing instability.".to_string());
        }

        if faction.planet_toxicity > 50 && faction.planet_toxicity <= 80 {
            self.push_log(
                "ENVIRONMENTAL ALERT: High toxicity detected. Habitats under strain.".to_string(),
            );
        } else if faction.planet_toxicity > 80 {
            self.push_event_log(
                EventCategory::Crisis,
                "TOXICITY CRISIS: Severe planetary strain. Minor debris/acid rain events possible."
                    .to_string(),
            );
            // Trigger Moonfall Phase 1: Dust Fall
            if self.dust_fall_turns_left == 0
                && self.sample_noise(owner as i32, self.turn, 777) % 100 > 70
            {
                self.dust_fall_turns_left =
                    3 + (self.sample_noise(owner as i32, self.turn, 888) % 4) as i32;
                self.push_event_log(
                    EventCategory::Crisis,
                    "CRISIS EVENT: Atmospheric Dust Fall! Global nutrient yields halved."
                        .to_string(),
                );
            }

            // Trigger Moonfall Phase 2: Physical Debris Impacts
            if faction.planet_toxicity > 90 {
                // Orbital Defense Interception: Each defense pod has a 10% chance to intercept the debris
                let interception_roll = self.sample_noise(owner as i32, self.turn, 999) % 100;
                if interception_roll < (faction.orbital_defenses as u32 * 10).min(90) {
                    self.push_log(format!(
                        "ORBITAL DEFENSE: {} intercepted a debris impact!",
                        self.faction_name(owner)
                    ));
                } else {
                    let bases = self.bases_for(owner);
                    if !bases.is_empty() {
                        let base_idx = (self.sample_noise(owner as i32, self.turn, 111) as usize)
                            % bases.len();
                        let target_base_name = bases[base_idx].name.clone();
                        let target_base_x = bases[base_idx].x;
                        let target_base_y = bases[base_idx].y;

                        // Keep debris offsets inside the intended 5x5 impact window.
                        let dx = (self.sample_noise(owner as i32, self.turn, 222) % 5) as i32 - 2;
                        let dy = (self.sample_noise(owner as i32, self.turn, 333) % 5) as i32 - 2;

                        let tx =
                            (target_base_x as i32 + dx).clamp(0, self.width as i32 - 1) as usize;
                        let ty =
                            (target_base_y as i32 + dy).clamp(0, self.height as i32 - 1) as usize;

                        let mut damage_report = String::new();
                        let target_tile_idx = self.tile_index(tx, ty);
                        if let Some(unit_id) = self.tiles[target_tile_idx].unit {
                            let damage =
                                3 + (self.sample_noise(owner as i32, self.turn, 444) % 3) as i32;
                            if let Some(unit) =
                                self.units.iter_mut().find(|u| u.id == unit_id && u.alive)
                            {
                                unit.hp -= damage;
                                damage_report = format!("Unit struck for {damage} damage.");
                                if unit.hp <= 0 {
                                    unit.alive = false;
                                    self.tiles[target_tile_idx].unit = None;
                                    damage_report = format!("Unit destroyed by impact.");
                                }
                            }
                        } else {
                            if let Some(imp) = self.tiles[target_tile_idx].improvement {
                                self.tiles[target_tile_idx].improvement = None;
                                damage_report = format!(
                                    "{} facility destroyed.",
                                    presentation::improvement_name(imp)
                                );
                            }
                        }

                        if !damage_report.is_empty() {
                            self.push_event_log(
                                EventCategory::Crisis,
                                format!(
                                    "CRISIS EVENT: Debris impact near {}! {}",
                                    target_base_name, damage_report
                                ),
                            );
                        }
                    }
                }
            }
        }

        if faction.ai_dependence > 70 {
            self.push_log(
                "GOVERNANCE WARNING: Significant reliance on AI. Synthetic drift detected."
                    .to_string(),
            );

            // Trigger Governance Override
            let override_chance = if self.has_secret_project(owner, SecretProject::EmpathGuild) {
                5
            } else {
                15
            };
            if faction.ai_dependence > 80
                && self.sample_noise(owner as i32, self.turn, 1000) % 100 < override_chance
            {
                let manual_bases: Vec<usize> = self
                    .bases_for(owner)
                    .iter()
                    .filter(|b| b.governor_mode == GovernorMode::Off)
                    .map(|b| b.id)
                    .collect();

                if !manual_bases.is_empty() {
                    let base_id = manual_bases[self.sample_noise(owner as i32, self.turn, 555)
                        as usize
                        % manual_bases.len()];
                    if let Some(base) = self.base_mut(base_id) {
                        base.governor_mode = GovernorMode::MachinePolity;
                        let base_name = base.name.clone();
                        self.push_log(format!(
                            "GOVERNANCE OVERRIDE: AI subsystem seized control of {base_name}."
                        ));
                    }
                }
            }
        }

        // Trigger Moonfall Phase 3: Tidal Chaos
        if self.turn > 50
            && self.tidal_chaos_turns_left == 0
            && self.sample_noise(owner as i32, self.turn, 1111) % 100 > 90
        {
            self.tidal_chaos_turns_left =
                4 + (self.sample_noise(owner as i32, self.turn, 2222) % 4) as i32;
            self.push_event_log(
                EventCategory::Crisis,
                "CRISIS EVENT: Tidal Chaos! Coastal flooding has begun.".to_string(),
            );
        }
    }

    fn apply_convoy_interdiction(&mut self, owner: usize) {
        let routes = self.convoy_routes.clone();
        let mut energy_loss = 0;
        let mut mineral_loss = 0;
        let mut collapse_energy_loss = 0;
        let mut collapse_mineral_loss = 0;
        let mut interceptions = 0;
        let mut escort_damage_events = 0;
        let mut collapsed_routes = Vec::new();

        for route in routes {
            let Some(base_a) = self.base(route.base_a_id).cloned() else {
                continue;
            };
            let Some(base_b) = self.base(route.base_b_id).cloned() else {
                continue;
            };
            if base_a.owner != owner || base_b.owner != owner {
                continue;
            }
            if !self.is_convoy_route_intercepted(route.base_a_id, route.base_b_id, route.kind) {
                continue;
            }

            interceptions += 1;
            if let Some(active_route) = self.convoy_routes.iter_mut().find(|active| {
                active.base_a_id == route.base_a_id
                    && active.base_b_id == route.base_b_id
                    && active.kind == route.kind
            }) {
                active_route.integrity = active_route.integrity.saturating_sub(1);
                if active_route.integrity == 0 {
                    collapsed_routes.push((
                        active_route.base_a_id,
                        active_route.base_b_id,
                        active_route.kind,
                    ));
                }
            }
            if self.damage_route_escort_skirmisher(route.base_a_id, route.base_b_id, owner) {
                escort_damage_events += 1;
            }
            match route.kind {
                crate::ConvoyRouteKind::Trade => {
                    energy_loss += 2;
                }
                crate::ConvoyRouteKind::Freight => {
                    let richer_base_id = if base_a.minerals_stock >= base_b.minerals_stock {
                        base_a.id
                    } else {
                        base_b.id
                    };
                    if let Some(base) = self.base_mut(richer_base_id) {
                        let loss = base.minerals_stock.min(2);
                        base.minerals_stock -= loss;
                        mineral_loss += loss;
                    }
                }
                crate::ConvoyRouteKind::MilitarySupply => {
                    energy_loss += 1;
                }
            }
        }

        if energy_loss > 0 {
            if let Some(faction) = self.faction_mut(owner) {
                faction.energy -= energy_loss;
            }
        }

        for (base_a_id, base_b_id, kind) in &collapsed_routes {
            match kind {
                crate::ConvoyRouteKind::Trade => {
                    collapse_energy_loss += 3;
                }
                crate::ConvoyRouteKind::Freight => {
                    let richer_base_id = match self.base(*base_a_id).zip(self.base(*base_b_id)) {
                        Some((base_a, base_b))
                            if base_a.minerals_stock >= base_b.minerals_stock =>
                        {
                            *base_a_id
                        }
                        Some(_) => *base_b_id,
                        None => continue,
                    };
                    if let Some(base) = self.base_mut(richer_base_id) {
                        let loss = base.minerals_stock.min(3);
                        base.minerals_stock -= loss;
                        collapse_mineral_loss += loss;
                    }
                }
                crate::ConvoyRouteKind::MilitarySupply => {
                    collapse_energy_loss += 2;
                }
            }
        }
        if collapse_energy_loss > 0 {
            if let Some(faction) = self.faction_mut(owner) {
                faction.energy -= collapse_energy_loss;
            }
        }
        self.update_command_center_post_interdiction_traces(owner);

        if interceptions > 0 {
            let faction_name = self.faction_name(owner).to_string();
            let mut parts = Vec::new();
            if energy_loss > 0 {
                parts.push(format!("{energy_loss} energy"));
            }
            if mineral_loss > 0 {
                parts.push(format!("{mineral_loss} minerals"));
            }
            if collapse_energy_loss > 0 {
                parts.push(format!("{collapse_energy_loss} collapse energy"));
            }
            if collapse_mineral_loss > 0 {
                parts.push(format!("{collapse_mineral_loss} collapse minerals"));
            }
            let losses = if parts.is_empty() {
                "no direct resources".to_string()
            } else {
                parts.join(" and ")
            };
            self.push_log(format!(
                "{faction_name} suffered {interceptions} convoy interception(s), losing {losses}."
            ));
            if escort_damage_events > 0 {
                self.push_log(format!(
                    "{faction_name} escort skirmishers absorbed {escort_damage_events} convoy interception hit(s)."
                ));
            }
        }

        if !collapsed_routes.is_empty() {
            for (base_a_id, base_b_id, kind) in collapsed_routes {
                let left_name = self
                    .base(base_a_id)
                    .map(|base| base.name.clone())
                    .unwrap_or_else(|| format!("Base {base_a_id}"));
                let right_name = self
                    .base(base_b_id)
                    .map(|base| base.name.clone())
                    .unwrap_or_else(|| format!("Base {base_b_id}"));
                self.convoy_routes.retain(|route| {
                    !(route.base_a_id == base_a_id
                        && route.base_b_id == base_b_id
                        && route.kind == kind)
                });
                self.push_log(format!(
                    "{} convoy route between {} and {} collapsed under interception.",
                    presentation::convoy_route_kind_label(kind),
                    left_name,
                    right_name
                ));
            }
        }
    }

    fn check_narrative_triggers(&mut self, owner: usize) {
        let triggers = crate::narrative::get_voice_of_planet_triggers();
        let (planet_toxicity, known_techs) = match self.faction(owner) {
            Some(f) => (f.planet_toxicity, f.known_techs.clone()),
            None => return,
        };

        let mut to_trigger = Vec::new();

        for trigger in triggers {
            if self.triggered_narratives.contains(trigger.message) {
                continue;
            }

            let triggered = match trigger.condition {
                crate::narrative::NarrativeCondition::Toxicity(threshold) => {
                    planet_toxicity >= threshold
                }
                crate::narrative::NarrativeCondition::Technology(tech) => {
                    known_techs.contains(&tech)
                }
                crate::narrative::NarrativeCondition::Turn(turn) => self.turn >= turn,
            };

            if triggered {
                to_trigger.push(trigger.message.to_string());
            }
        }

        for message in to_trigger {
            self.triggered_narratives.insert(message.clone());
            self.push_event_log(
                EventCategory::Narrative,
                format!("VOICE OF PLANET: {}", message),
            );
        }
    }

    fn emit_logistics_alert_events(&mut self, owner: usize) {
        let alerts = self.faction_logistics_alerts(owner);
        for alert in alerts
            .into_iter()
            .filter(|line| {
                line.contains("full lane capacity")
                    || line.contains("energy per turn")
                    || line.contains("close to collapse")
            })
            .take(2)
        {
            self.push_log(format!("Logistics alert: {alert}"));
        }
    }

    fn damage_route_escort_skirmisher(
        &mut self,
        base_a_id: usize,
        base_b_id: usize,
        owner: usize,
    ) -> bool {
        let Some(base_a) = self.base(base_a_id).cloned() else {
            return false;
        };
        let Some(base_b) = self.base(base_b_id).cloned() else {
            return false;
        };

        let target_unit_id = self
            .units
            .iter()
            .filter(|unit| {
                unit.alive && unit.owner == owner && unit.kind == UnitKind::EscortSpeeder
            })
            .filter(|unit| {
                unit.x.abs_diff(base_a.x) + unit.y.abs_diff(base_a.y) <= 2
                    || unit.x.abs_diff(base_b.x) + unit.y.abs_diff(base_b.y) <= 2
            })
            .min_by_key(|unit| {
                let da = unit.x.abs_diff(base_a.x) + unit.y.abs_diff(base_a.y);
                let db = unit.x.abs_diff(base_b.x) + unit.y.abs_diff(base_b.y);
                da.min(db)
            })
            .map(|unit| unit.id);

        let Some(unit_id) = target_unit_id else {
            return false;
        };
        let Some(index) = self.units.iter().position(|unit| unit.id == unit_id) else {
            return false;
        };
        self.units[index].hp = self.units[index].hp.saturating_sub(2);
        let name = self.faction_name(owner).to_string();
        if self.units[index].hp == 0 {
            self.units[index].alive = false;
            let x = self.units[index].x;
            let y = self.units[index].y;
            let tile_index = self.tile_index(x, y);
            if self.tiles[tile_index].unit == Some(unit_id) {
                self.tiles[tile_index].unit = None;
            }
            self.push_log(format!(
                "{name} lost an Escort Speeder while screening a convoy route."
            ));
        } else {
            self.push_log(format!(
                "{name} Escort Speeder took sabotage damage while screening a convoy route."
            ));
        }
        true
    }

    pub fn recommended_governor_mode_for_base(&self, base_id: usize) -> GovernorMode {
        let Some(base) = self.base(base_id) else {
            return GovernorMode::Off;
        };
        let military = self.base_local_military_pressure(base_id);
        let psi = self.base_local_psi_pressure(base_id);
        let recovery = self.damaged_garrison_count(base_id);
        let unrest = self.base_unrest(base_id);
        let logistics = self.base_logistics_stress_score(base_id);
        let (_, convoy_upkeep, _, _) = self.faction_upkeep_breakdown(base.owner);

        if logistics >= 5
            || (convoy_upkeep >= 4
                && (self.base_convoy_usage(base_id) > 0
                    || self.base_potential_trade_links(base_id) >= 2))
        {
            GovernorMode::Logistics
        } else if military >= 2 || psi >= 2 {
            GovernorMode::Defense
        } else if recovery > 0 {
            GovernorMode::Recovery
        } else if unrest > 0 || self.base_food_margin(base_id).unwrap_or(1) <= 0 {
            GovernorMode::Economy
        } else if base.population >= 2 {
            GovernorMode::Balanced
        } else {
            GovernorMode::Off
        }
    }

    fn base_growth_threshold(&self, base_id: usize) -> i32 {
        let mut threshold = content::base_growth_nutrients_threshold();
        if let Some(base) = self.base(base_id) {
            // Increase threshold based on population (SMAC style)
            threshold += (base.population as i32 - 1) * 2;

            for facility in &base.facilities {
                threshold -= content::facility_growth_threshold_reduction(*facility);
            }

            if let Some(faction) = self.faction(base.owner) {
                let growth = faction.effective_attributes().growth;
                // +1 Growth = -10% threshold, -1 Growth = +10% threshold
                let multiplier = 1.0 - (growth as f32 * 0.1);
                threshold = (threshold as f32 * multiplier) as i32;
            }
        }
        threshold.max(10)
    }

    fn check_research(&mut self, owner: usize) {
        let Some(tech) = self.faction(owner).map(|faction| faction.current_research) else {
            return;
        };

        let tech_known = self
            .faction(owner)
            .map(|faction| faction.known_techs.contains(&tech))
            .unwrap_or(true);
        if !tech_known && !self.is_research_available(owner, tech) {
            for candidate in Tech::all() {
                if self.is_research_available(owner, candidate) {
                    if let Some(faction) = self.faction_mut(owner) {
                        faction.current_research = candidate;
                    }
                    self.push_log(format!(
                        "{} redirected research toward {}.",
                        self.faction_name(owner),
                        presentation::tech_name(candidate)
                    ));
                    return;
                }
            }
        }

        let cost = content::tech_cost(tech);

        if self
            .faction(owner)
            .map(|faction| faction.research < cost)
            .unwrap_or(true)
        {
            return;
        }

        if let Some(faction) = self.faction_mut(owner) {
            faction.research -= cost;
        }

        let tech_was_known = self
            .faction(owner)
            .map(|faction| faction.known_techs.contains(&tech))
            .unwrap_or(true);

        if !tech_was_known {
            if let Some(faction) = self.faction_mut(owner) {
                faction.known_techs.push(tech);
                faction.techs_discovered += 1;
            }
            let faction_name = self.faction_name(owner).to_string();
            self.push_log(format!(
                "{faction_name} discovered {}.",
                presentation::tech_name(tech)
            ));
        }

        // Pick next research
        let is_ai = self.faction(owner).map(|f| f.is_ai).unwrap_or(false);
        if is_ai {
            ai::run_ai_strategy(self); // This will update research and SE
        } else {
            for candidate in Tech::all() {
                if self.is_research_available(owner, candidate) {
                    if let Some(faction) = self.faction_mut(owner) {
                        faction.current_research = candidate;
                    }
                    return;
                }
            }
        }
    }

    fn resolve_supply_pod(&mut self, unit_id: usize, x: usize, y: usize) {
        let Some(unit) = self.unit(unit_id).cloned() else {
            return;
        };

        let roll = self.sample_noise(x as i32, y as i32, self.turn as u32 + 900) % 6;
        let faction_name = self.faction_name(unit.owner).to_string();

        match roll {
            0 => {
                let reward = content::supply_pod_energy_reward();
                if let Some(faction) = self.faction_mut(unit.owner) {
                    faction.energy += reward;
                }
                self.push_log(format!(
                    "{faction_name} recovered {reward} energy from a supply pod."
                ));
            }
            1 => {
                let Some(tech) = self
                    .faction(unit.owner)
                    .map(|faction| faction.current_research)
                else {
                    return;
                };
                let tech_was_known = self
                    .faction(unit.owner)
                    .map(|faction| faction.known_techs.contains(&tech))
                    .unwrap_or(true);
                if !tech_was_known {
                    if let Some(faction) = self.faction_mut(unit.owner) {
                        faction.known_techs.push(tech);
                        faction.techs_discovered += 1;
                    }
                    self.push_log(format!(
                        "Supply pod revealed {}.",
                        presentation::tech_name(tech)
                    ));
                } else {
                    if let Some(faction) = self.faction_mut(unit.owner) {
                        faction.energy += content::supply_pod_salvage_energy_reward();
                    }
                    self.push_log("Supply pod contained salvageable data crystals.".to_string());
                }
            }
            2 => {
                if self
                    .spawn_unit_near(unit.owner, UnitKind::ScoutPatrol, x, y)
                    .is_some()
                {
                    self.push_log(
                        "Supply pod contained a functional scout rover team.".to_string(),
                    );
                }
            }
            3 => {
                if self
                    .spawn_unit_near(unit.owner, UnitKind::Former, x, y)
                    .is_some()
                {
                    self.push_log("Supply pod contained former equipment.".to_string());
                }
            }
            4 => {
                if self
                    .spawn_unit_near(self.native_owner(), UnitKind::MindWorm, x, y)
                    .is_some()
                {
                    self.push_log(
                        "Supply pod rupture attracted a native mind worm boil!".to_string(),
                    );
                }
            }
            _ => {
                if let Some(active_unit) =
                    self.units.iter_mut().find(|u| u.id == unit_id && u.alive)
                {
                    active_unit.hp = content::unit_base_hp(active_unit.kind.clone());
                    active_unit.moves_left = active_unit.kind.clone().max_moves();
                }
                self.push_log("Supply pod repaired and resupplied the expedition.".to_string());
            }
        }
    }

    pub fn unit_can_bombard(&self, unit_id: usize) -> bool {
        self.unit_has_ability(unit_id, Ability::Artillery)
    }

    fn resolve_combat(
        &mut self,
        attacker_id: usize,
        defender_id: usize,
        target_x: usize,
        target_y: usize,
    ) {
        let Some(attacker) = self.unit(attacker_id).cloned() else {
            return;
        };
        
        let Some(defender) = self.unit(defender_id).cloned() else {
            return;
        };

        // AUTO-DECLARE WAR on Attack
        // If factions are not the same, not Native, and are not currently at War
        if attacker.owner != defender.owner 
            && attacker.owner != self.native_owner() 
            && defender.owner != self.native_owner() 
        {
            if self.relations[attacker.owner][defender.owner].status != DiplomacyStatus::War {
                let _ = self.update_diplomacy(attacker.owner, defender.owner, DiplomacyStatus::War);
            }

            // MUTUAL DEFENSE: Allies of the defender declare war on the attacker
            let defender_allies: Vec<usize> = (0..self.factions.len())
                .filter(|&id| {
                    id != defender.owner 
                        && id != attacker.owner
                        && self.relations[defender.owner][id].status == DiplomacyStatus::Pact
                })
                .collect();
            
            for ally_id in defender_allies {
                if self.relations[attacker.owner][ally_id].status != DiplomacyStatus::War {
                    let _ = self.update_diplomacy(attacker.owner, ally_id, DiplomacyStatus::War);
                }
            }
        }

        // PLANET BUSTER: Massive destructive power
        if matches!(self.unit_weapon(attacker_id), Some(Weapon::PlanetBuster(_))) {
            self.push_event_log(
                EventCategory::Crisis,
                format!(
                    "PLANET BUSTER: {} has deployed a weapon of mass destruction at {}, {}!",
                    self.faction_name(attacker.owner),
                    target_x,
                    target_y
                ),
            );

            let mut affected_tiles = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let tx = target_x as i32 + dx;
                    let ty = target_y as i32 + dy;
                    if tx >= 0 && ty >= 0 && tx < self.width as i32 && ty < self.height as i32 {
                        affected_tiles.push((tx as usize, ty as usize));
                    }
                }
            }

            for (tx, ty) in affected_tiles {
                let idx = self.tile_index(tx, ty);

                // 1. Destroy Units
                if let Some(uid) = self.tiles[idx].unit {
                    self.destroy_unit(uid);
                }

                // 2. Destroy Bases
                if let Some(bid) = self.tiles[idx].base {
                    if let Some(base_idx) = self.bases.iter().position(|b| b.id == bid) {
                        let base_name = self.bases[base_idx].name.clone();
                        self.push_log(format!("PLANET BUSTER: Base {} was vaporized!", base_name));
                        self.bases.remove(base_idx);
                    }
                    self.tiles[idx].base = None;
                }

                // 3. Alter Terrain
                self.tiles[idx].terrain = Terrain::Crater;
                self.tiles[idx].improvement = None;
                self.tiles[idx].elevation = -10;
            }

            // Attacker is consumed
            self.destroy_unit(attacker_id);

            // Toxicity Reset: Planet Busters create so much pollution they actually "reset" the local ecosystem
            if let Some(f) = self.faction_mut(attacker.owner) {
                f.planet_toxicity = (f.planet_toxicity / 2).max(0);
            }
            return;
        }

        // Bombardment: Units with Artillery can attack bases from a distance
        if self.unit_can_bombard(attacker_id) {
            if let Some(target_base_id) = self.tile(target_x, target_y).and_then(|t| t.base) {
                let base_name = self.bases[target_base_id].name.clone();
                let _damage = (attacker.experience + 1).max(1);

                let base = &mut self.bases[target_base_id];
                // Damage population
                if base.population > 1 {
                    base.population -= 1;
                } else if !base.facilities.is_empty() {
                    // Destroy a facility if pop is 1
                    base.facilities.remove(0);
                    self.push_log(format!(
                        "BOMBARDMENT: {} destroyed a facility in {}!",
                        content::unit_name(attacker.kind.clone()),
                        base_name
                    ));
                }

                self.push_log(format!(
                    "BOMBARDMENT: {} fired on {}!",
                    content::unit_name(attacker.kind.clone()),
                    base_name
                ));
                return;
            }
        }

        let roll =
            (self.sample_noise(attacker.x as i32, defender.y as i32, self.turn as u32) % 6) as i32;

        let is_psionic =
            self.unit_is_psi_threat(attacker_id) || self.unit_is_psi_threat(defender_id);

        let mut attack_score;
        let mut defense_score;

        if is_psionic {
            // Psionic Combat: 3 base + experience + roll + Planet bonus
            attack_score = 3 + attacker.experience + roll;
            defense_score = 3 + defender.experience + (defender.hp / 4);

            if let Some(f) = self.faction(attacker.owner) {
                attack_score += f.effective_attributes().planet;
                if attacker.kind == UnitKind::MindWorm {
                    attack_score += f.effective_attributes().planet;
                }
            }
            if let Some(f) = self.faction(defender.owner) {
                defense_score += f.effective_attributes().planet;
                if defender.kind == UnitKind::MindWorm {
                    defense_score += f.effective_attributes().planet;
                }
            }
        } else {
            attack_score = self.unit_attack_strength(attacker_id) + attacker.experience + roll + 1
                - self.attacker_penalty(target_x, target_y);
            defense_score = self.unit_defense_strength(defender_id)
                + defender.experience
                + defender.hp / 4
                + self.defense_bonus_for_tile(target_x, target_y);
        }

        // Apply Ability Bonuses
        if self.unit_has_ability(attacker_id, Ability::Raid) && self.unit_is_exposed(defender_id) {
            attack_score += 2;
        }
        if self.unit_has_ability(attacker_id, Ability::AirSuperiority)
            && self.unit_is_aircraft(defender_id)
        {
            attack_score += 2;
        }

        if self.unit_has_ability(defender_id, Ability::Trance)
            && self.unit_is_psi_threat(attacker_id)
        {
            defense_score += 2;
        }
        if self.unit_has_ability(defender_id, Ability::DeepPressureHull)
            && (self.unit_is_sea_unit(defender_id) || self.unit_is_at_sea(defender_id))
        {
            defense_score += 2;
        }
        if self.unit_has_ability(defender_id, Ability::CommJammer) && self.unit_is_fast(attacker_id)
        {
            defense_score += 2;
        }

        // Apply Faction Morale and Facility Equivalent Bonuses
        if let Some(attacker_faction) = self.faction(attacker.owner) {
            let attributes = attacker_faction.effective_attributes();
            attack_score += attributes.morale;
            if attributes
                .facility_equivalents
                .contains(&Facility::CommandCenter)
                && !self.unit_is_sea_unit(attacker_id)
            {
                attack_score += 1;
            }
        }
        if let Some(defender_faction) = self.faction(defender.owner) {
            let attributes = defender_faction.effective_attributes();
            defense_score += attributes.morale;
            if attributes
                .facility_equivalents
                .contains(&Facility::CommandCenter)
                && !self.unit_is_sea_unit(defender_id)
            {
                defense_score += 1;
            }
        }

        let attacker_name = self.faction_name(attacker.owner).to_string();
        let defender_name = self.faction_name(defender.owner).to_string();

        if attack_score >= defense_score {
            self.destroy_unit(defender_id);
            let _ = self.move_unit_without_combat(attacker_id, target_x, target_y);
            self.set_unit_moves(attacker_id, 0);
            self.promote_unit(attacker_id);
            self.push_log(format!(
                "COMBAT: {attacker_name} {} (atk: {attack_score}) destroyed {defender_name} {} (def: {defense_score}).",
                presentation::unit_name(attacker.kind),
                presentation::unit_name(defender.kind)
            ));
        } else {
            self.destroy_unit(attacker_id);
            self.push_log(format!(
                "COMBAT: {attacker_name} {} (atk: {attack_score}) was destroyed by {defender_name} {} (def: {defense_score}).",
                presentation::unit_name(attacker.kind),
                presentation::unit_name(defender.kind)
            ));
        }
    }

    fn move_unit_without_combat(
        &mut self,
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    ) -> Result<(), String> {
        let unit_snapshot = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        let from_idx = self.tile_index(unit_snapshot.x, unit_snapshot.y);
        let target_idx = self.tile_index(target_x, target_y);

        self.tiles[from_idx].unit = None;
        self.tiles[target_idx].unit = Some(unit_id);

        let is_adjacent = Self::is_adjacent(unit_snapshot.x, unit_snapshot.y, target_x, target_y);

        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            unit.x = target_x;
            unit.y = target_y;
            if is_adjacent {
                unit.moves_left -= 1;
            } else {
                unit.moves_left = 0;
            }
        }

        Ok(())
    }

    fn destroy_unit(&mut self, unit_id: usize) {
        let unit_snapshot = self.units.iter().find(|u| u.id == unit_id).cloned();

        if let Some(unit) = unit_snapshot {
            // Recursively destroy cargo
            let cargo_ids = unit.cargo_unit_ids.clone();
            for cid in cargo_ids {
                self.destroy_unit(cid);
            }

            if unit.x < self.width && unit.y < self.height {
                let idx = self.tile_index(unit.x, unit.y);
                if self.tiles[idx].unit == Some(unit_id) {
                    self.tiles[idx].unit = None;
                }
            }
        }

        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id) {
            unit.alive = false;
            unit.moves_left = 0;
            unit.hp = 0;
        }
    }

    fn set_unit_moves(&mut self, unit_id: usize, moves: i32) {
        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            unit.moves_left = moves;
        }
    }

    fn repair_units_for_owner(&mut self, owner: usize) {
        let unit_ids: Vec<usize> = self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
            .map(|unit| unit.id)
            .collect();

        for unit_id in unit_ids {
            let Some(unit) = self.unit(unit_id).cloned() else {
                continue;
            };
            let max_hp = content::unit_base_hp(unit.kind.clone());
            if unit.hp >= max_hp {
                continue;
            }

            let heal = self.unit_repair_amount(&unit);
            if heal <= 0 {
                continue;
            }

            if let Some(active_unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
                active_unit.hp = (active_unit.hp + heal).min(max_hp);
            }
        }
    }

    fn unit_repair_amount(&self, unit: &Unit) -> i32 {
        // Check for adjacent enemies
        let has_adjacent_enemy = self.units.iter().any(|u| {
            u.alive && u.owner != unit.owner && Self::is_adjacent(unit.x, unit.y, u.x, u.y)
        });

        if has_adjacent_enemy {
            return 0;
        }

        let mut heal = 1 + unit.experience;

        if let Some(base_id) = self.tile(unit.x, unit.y).and_then(|tile| tile.base) {
            if let Some(base) = self.base(base_id) {
                if base.owner == unit.owner {
                    heal += 1;
                    if self.base_military_supply_links(base_id) > 0 {
                        heal += 1;
                    }
                    let facility_bonus: i32 = base
                        .facilities
                        .iter()
                        .copied()
                        .map(content::facility_repair_bonus)
                        .sum();
                    heal += facility_bonus;
                }
            }
        }

        heal
    }

    fn base_mobility_bonus(&self, base_id: usize, unit_kind: UnitKind) -> i32 {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        if !matches!(
            unit_kind,
            UnitKind::Speeder
                | UnitKind::EscortSpeeder
                | UnitKind::RaiderSpeeder
                | UnitKind::Needlejet
        ) {
            return 0;
        }
        base.facilities
            .iter()
            .copied()
            .map(content::facility_mobility_bonus)
            .sum()
    }

    pub fn effective_unit_max_moves_at(
        &self,
        owner: usize,
        unit_kind: UnitKind,
        x: usize,
        y: usize,
    ) -> i32 {
        let mut moves = unit_kind.clone().max_moves();

        if self.has_secret_project(owner, SecretProject::ManifoldDrive) {
            moves += 1;
        }

        if let Some(base_id) = self.tile(x, y).and_then(|tile| tile.base) {
            if let Some(base) = self.base(base_id) {
                if base.owner == owner {
                    moves += self.base_mobility_bonus(base_id, unit_kind);
                }
            }
        }
        moves
    }

    fn promote_unit(&mut self, unit_id: usize) {
        let mut log_message = None;
        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            let previous = unit.experience;
            unit.experience = (unit.experience + 1).min(3);
            if unit.experience > previous {
                log_message = Some((
                    unit.owner,
                    unit.kind.clone(),
                    presentation::unit_rank_name(unit.experience).to_string(),
                ));
            }
        }
        if let Some((owner, kind, rank)) = log_message {
            self.push_log(format!(
                "{} {} advanced to {} rank.",
                self.faction_name(owner),
                presentation::unit_name(kind),
                rank
            ));
        }
    }

    pub fn update_diplomatic_attitudes(&mut self) {
        let faction_count = self.factions.len();

        // Calculate minimum distance between each pair of factions
        let mut min_distances = vec![vec![100; faction_count]; faction_count];
        for i in 0..faction_count {
            let bases_i: Vec<(usize, usize)> = self
                .bases
                .iter()
                .filter(|b| b.owner == i)
                .map(|b| (b.x, b.y))
                .collect();
            for j in 0..faction_count {
                if i == j {
                    continue;
                }
                let bases_j: Vec<(usize, usize)> = self
                    .bases
                    .iter()
                    .filter(|b| b.owner == j)
                    .map(|b| (b.x, b.y))
                    .collect();

                let mut min_dist = 100;
                for &(ix, iy) in &bases_i {
                    for &(jx, jy) in &bases_j {
                        let dist = ((ix as i32 - jx as i32).abs() + (iy as i32 - jy as i32).abs())
                            as usize;
                        if dist < min_dist {
                            min_dist = dist;
                        }
                    }
                }
                min_distances[i][j] = min_dist;
            }
        }

        for i in 0..faction_count {
            for j in 0..faction_count {
                if i == j || i == crate::model::NATIVE_ID || j == crate::model::NATIVE_ID {
                    continue;
                }

                let status = self.relations[i][j].status;
                let current_attitude = self.relations[i][j].attitude;

                let mut change = match status {
                    DiplomacyStatus::War => -2,
                    DiplomacyStatus::Truce => 0,
                    DiplomacyStatus::Treaty => 1,
                    DiplomacyStatus::Pact => 3,
                };

                // Border friction: closer borders cause tension, especially without a Treaty
                let dist = min_distances[i][j];
                if dist <= 8 && status != DiplomacyStatus::Pact && status != DiplomacyStatus::Treaty
                {
                    change -= 1;
                    if dist <= 4 {
                        change -= 1; // High tension
                    }
                }

                // Power disparity check (compare total populations)
                let pop_i: i32 = self
                    .bases
                    .iter()
                    .filter(|b| b.owner == i)
                    .map(|b| b.population)
                    .sum();
                let pop_j: i32 = self
                    .bases
                    .iter()
                    .filter(|b| b.owner == j)
                    .map(|b| b.population)
                    .sum();

                if pop_j > pop_i * 2 && status != DiplomacyStatus::Pact {
                    change -= 1; // Resentment/Fear
                } else if pop_i > pop_j * 2 && status != DiplomacyStatus::Pact {
                    change += 1; // Respect/Appeasement
                }

                self.relations[i][j].attitude = (current_attitude + change).clamp(-100, 100);
            }
        }
    }

    pub(crate) fn reset_moves_for_owner(&mut self, owner: usize) {
        let refreshes: Vec<(usize, i32)> = self
            .units
            .iter()
            .filter(|u| u.alive && u.owner == owner)
            .map(|unit| {
                (
                    unit.id,
                    self.effective_unit_max_moves_at(unit.owner, unit.kind.clone(), unit.x, unit.y),
                )
            })
            .collect();
        for (unit_id, moves) in refreshes {
            if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
                unit.moves_left = moves;
            }
        }
    }

    pub fn update_player_visibility(&mut self) {
        let player_owner = self.player_owner();

        for tile in &mut self.tiles {
            tile.visible_by_owner.remove(&player_owner);
        }

        let mut reveal_centers: Vec<(usize, usize, isize)> = Vec::new();

        let has_empath_guild = self.has_secret_project(player_owner, SecretProject::EmpathGuild);

        let pacted_factions: Vec<usize> = (0..self.factions.len())
            .filter(|&id| {
                id != player_owner 
                    && (has_empath_guild || self.relations[player_owner][id].status == DiplomacyStatus::Pact)
            })
            .collect();

        for unit in self
            .units
            .iter()
            .filter(|u| u.alive && (u.owner == player_owner || pacted_factions.contains(&u.owner)))
        {
            reveal_centers.push((unit.x, unit.y, content::player_unit_visibility_radius()));
        }

        for base in self
            .bases
            .iter()
            .filter(|b| b.owner == player_owner || pacted_factions.contains(&b.owner))
        {
            reveal_centers.push((base.x, base.y, content::player_base_visibility_radius()));
        }

        for (x, y, radius) in reveal_centers {
            self.reveal_area(x, y, radius);
        }
    }

    fn validate_base_build_choice(
        base: &Base,
        faction: crate::Faction,
        item: ProductionItem,
    ) -> Result<(), String> {
        if let Some(required_tech) = content::required_tech_for_production(item) {
            if !faction.known_techs.contains(&required_tech) {
                return Err(format!(
                    "{} requires {}.",
                    presentation::production_name(item),
                    presentation::tech_name(required_tech)
                ));
            }
        }
        if let Some(facility) = item.facility() {
            if base.facilities.contains(&facility) {
                return Err("That facility is already built here.".to_string());
            }
            if base.production == item || base.production_queue.contains(&item) {
                return Err("That facility is already queued here.".to_string());
            }
        }
        Ok(())
    }

    fn training_experience_for_base(&self, base_id: usize) -> i32 {
        self.base(base_id)
            .map(|base| {
                base.facilities
                    .iter()
                    .copied()
                    .map(content::facility_training_bonus)
                    .sum::<i32>()
                    .min(3)
            })
            .unwrap_or(0)
    }

    fn damaged_garrison_count(&self, base_id: usize) -> usize {
        let Some(base) = self.base(base_id) else {
            return 0;
        };
        let owner = base.owner;
        self.units
            .iter()
            .filter(|unit| unit.alive && unit.owner == owner)
            .filter(|unit| {
                let max_hp = content::unit_base_hp(unit.kind.clone());
                let on_base = self
                    .tile(unit.x, unit.y)
                    .and_then(|tile| tile.base)
                    .map(|id| id == base_id)
                    .unwrap_or(false);
                on_base && unit.hp < max_hp
            })
            .count()
    }

    fn should_preserve_current_production(&self, base_id: usize) -> bool {
        let Some(base) = self.base(base_id) else {
            return false;
        };
        let current = base.production;
        let current_cost = content::production_cost(current).max(1);
        let sufficiently_progressed = base.minerals_stock * 2 >= current_cost;
        sufficiently_progressed
            && self.base_local_military_pressure(base_id) >= 2
            && self.production_item_is_military(current)
    }

    fn production_item_is_military(&self, item: ProductionItem) -> bool {
        matches!(
            item,
            ProductionItem::ScoutPatrol
                | ProductionItem::ResonanceLaser
                | ProductionItem::Speeder
                | ProductionItem::EscortSpeeder
                | ProductionItem::RaiderSpeeder
                | ProductionItem::TranceScout
                | ProductionItem::GarrisonGuard
                | ProductionItem::PsiSentinel
                | ProductionItem::PerimeterDefense
                | ProductionItem::CommandCenter
                | ProductionItem::MilitaryAcademy
                | ProductionItem::SensorArray
        )
    }

    fn spawn_trained_unit_near(
        &mut self,
        owner: usize,
        kind: UnitKind,
        x: usize,
        y: usize,
        experience: i32,
    ) -> Option<usize> {
        let candidates = [
            (x, y),
            (x.saturating_add(1), y),
            (x.saturating_sub(1), y),
            (x, y.saturating_add(1)),
            (x, y.saturating_sub(1)),
            (x.saturating_add(1), y.saturating_add(1)),
            (x.saturating_sub(1), y.saturating_sub(1)),
            (x.saturating_add(1), y.saturating_sub(1)),
            (x.saturating_sub(1), y.saturating_add(1)),
        ];

        for (nx, ny) in candidates {
            if nx >= self.width || ny >= self.height {
                continue;
            }

            let idx = self.tile_index(nx, ny);
            let tile = &self.tiles[idx];

            if tile.terrain.is_land() && tile.unit.is_none() {
                return Some(self.spawn_unit_with_experience(owner, kind, nx, ny, experience));
            }
        }

        None
    }

    fn complete_production(&mut self, base_id: usize) {
        let base = &self.bases[base_id];
        let item = base.production;
        let owner = base.owner;
        let x = base.x;
        let y = base.y;

        let completed = if let Some(kind) = content::production_unit_kind(item) {
            let experience = self.training_experience_for_base(base_id);
            let spawned = self.spawn_trained_unit_near(owner, kind.clone(), x, y, experience);

            if spawned.is_some() && kind == UnitKind::ColonyPod {
                if let Some(base) = self.base_mut(base_id) {
                    base.population = (base.population - 1).max(1);
                }
            }
            spawned.is_some()
        } else if let Some(facility) = content::production_facility(item) {
            self.complete_facility(base_id, facility)
        } else if let Some(project) = item.secret_project() {
            self.complete_secret_project(owner, project)
        } else if let ProductionItem::CustomUnit(design_index) = item {
            let experience = self.training_experience_for_base(base_id);
            let kind = self
                .faction(owner)
                .and_then(|f| f.unit_designs.get(design_index))
                .map(|d| UnitKind::CustomUnit(d.clone()))
                .unwrap_or(UnitKind::ScoutPatrol);
            self.spawn_trained_unit_near(owner, kind, x, y, experience)
                .is_some()
        } else if matches!(
            item,
            ProductionItem::SkyHydroponics
                | ProductionItem::SolarTransmitter
                | ProductionItem::OrbitalDefense
        ) {
            if let Some(faction) = self.faction_mut(owner) {
                match item {
                    ProductionItem::SkyHydroponics => faction.sky_hydroponics += 1,
                    ProductionItem::SolarTransmitter => faction.solar_transmitters += 1,
                    ProductionItem::OrbitalDefense => faction.orbital_defenses += 1,
                    _ => {}
                }
            }
            true
        } else {
            false
        };

        if completed {
            let name = self.bases[base_id].name.clone();
            self.push_log(format!(
                "{name} completed production of {}.",
                presentation::production_name(item)
            ));
            self.advance_base_queue(base_id);
        }
    }

    fn advance_base_queue(&mut self, base_id: usize) {
        if let Some(base) = self.bases.iter_mut().find(|base| base.id == base_id) {
            if !base.production_queue.is_empty() {
                let next = base.production_queue.remove(0);
                base.production = next;
            }
        }
    }

    fn complete_facility(&mut self, base_id: usize, facility: Facility) -> bool {
        let Some(base) = self.bases.iter_mut().find(|base| base.id == base_id) else {
            return false;
        };
        if base.facilities.contains(&facility) {
            return false;
        }
        base.facilities.push(facility);
        true
    }

    fn complete_secret_project(&mut self, owner: usize, project: SecretProject) -> bool {
        if self
            .built_secret_projects
            .iter()
            .any(|(p, _)| *p == project)
        {
            return false;
        }
        self.built_secret_projects.push((project, owner));

        let project_name = self.production_name(
            owner,
            match project {
                SecretProject::WeatherPattern => ProductionItem::WeatherPattern,
                SecretProject::ClinicalImmortality => ProductionItem::ClinicalImmortality,
                SecretProject::EmpathGuild => ProductionItem::EmpathGuild,
                SecretProject::OrbitalElevator => ProductionItem::OrbitalElevator,
                SecretProject::ManifoldDrive => ProductionItem::ManifoldDrive,
                SecretProject::SingularityContainment => ProductionItem::SingularityContainment,
                SecretProject::BlackHoleHarvester => ProductionItem::BlackHoleHarvester,
            },
        );
        let faction_name = self.faction_name(owner).to_string();
        self.push_event_log(
            EventCategory::SecretProject,
            format!("GLOBAL WONDER: {faction_name} has completed {project_name}!"),
        );

        // Notify competition and convert minerals
        let mut notifications = Vec::new();
        for base in &mut self.bases {
            if base.production.secret_project() == Some(project) {
                if base.owner != owner {
                    notifications.push((base.owner, base.name.clone()));
                }
                base.production = ProductionItem::StockpileEnergy;
            }
            base.production_queue
                .retain(|item| item.secret_project() != Some(project));
        }

        let completed_faction_name = self.faction_name(owner).to_string();
        for (notified_owner, base_name) in notifications {
            let faction_name = self.faction_name(notified_owner).to_string();
            self.push_event_log(
                EventCategory::SecretProject,
                format!("{faction_name} construction of {project_name} in {base_name} was aborted! {completed_faction_name} finished it first."),
            );
        }

        if owner == self.player_owner() || project == SecretProject::EmpathGuild {
            self.update_player_visibility();
        }

        true
    }

    fn apply_faction_upkeep(&mut self, owner: usize) {
        let (energy_upkeep, unit_upkeep, _) = self.faction_upkeep(owner);
        if let Some(faction) = self.faction_mut(owner) {
            faction.energy -= energy_upkeep;
        }

        let mut remaining_minerals = unit_upkeep;
        if remaining_minerals > 0 {
            let base_ids: Vec<usize> = self.bases_for(owner).iter().map(|b| b.id).collect();
            for (order_index, base_id) in base_ids.into_iter().enumerate() {
                if remaining_minerals <= 0 {
                    break;
                }
                let (drained, end_stock) = {
                    let base = &mut self.bases[base_id];
                    let drained = if base.minerals_stock >= remaining_minerals {
                        remaining_minerals
                    } else {
                        base.minerals_stock
                    };
                    if base.minerals_stock >= remaining_minerals {
                        base.minerals_stock -= remaining_minerals;
                        remaining_minerals = 0;
                    } else {
                        remaining_minerals -= base.minerals_stock;
                        base.minerals_stock = 0;
                    }
                    (drained, base.minerals_stock)
                };
                if drained > 0 {
                    if let Some(trace) = self.command_center_turn_traces.iter_mut().find(|trace| {
                        trace.turn == self.turn && trace.owner == owner && trace.base_id == base_id
                    }) {
                        trace.upkeep_drain += drained;
                        if trace.upkeep_order_index.is_none() {
                            trace.upkeep_order_index = Some(order_index);
                        }
                        trace.end_stock = end_stock;
                    }
                }
            }
        }

        if remaining_minerals > 0 {
            let emergency_support_reserve = 10;
            let emergency_payment = self
                .faction(owner)
                .map(|faction| (faction.energy - emergency_support_reserve).max(0))
                .unwrap_or(0)
                .min(remaining_minerals);
            if emergency_payment > 0 {
                if let Some(faction) = self.faction_mut(owner) {
                    faction.energy -= emergency_payment;
                }
                remaining_minerals -= emergency_payment;
                self.push_event_log(
                    EventCategory::Economics,
                    format!(
                        "{} spent {} energy reserves to cover mineral support.",
                        self.faction_name(owner),
                        emergency_payment
                    ),
                );
            }
        }

        if energy_upkeep > 0 || unit_upkeep > 0 {
            let faction_name = self.faction_name(owner).to_string();
            self.push_log(format!(
                "{faction_name} paid {energy_upkeep} energy upkeep and {unit_upkeep} mineral support."
            ));
        }

        // Bankruptcy: Scrap facilities if energy is negative
        while self.faction(owner).map(|f| f.energy < 0).unwrap_or(false) {
            if let Some((base_id, facility)) = self.find_scrapable_facility(owner) {
                let scrap_value =
                    (content::production_cost(ProductionItem::from_facility(facility).unwrap())
                        as f32
                        * 0.5) as i32;
                if let Some(faction) = self.faction_mut(owner) {
                    faction.energy += scrap_value.max(1);
                }
                let base_name = self.bases[base_id].name.clone();
                self.bases[base_id].facilities.retain(|&f| f != facility);
                self.push_event_log(
                    EventCategory::Economics,
                    format!(
                        "BANKRUPTCY: {} scrapped {:?} in {} to cover debt!",
                        self.faction_name(owner),
                        facility,
                        base_name
                    ),
                );
            } else {
                // If no facilities left, disband units for emergency cash
                if let Some(unit_id) = self.find_disbandable_unit(owner) {
                    if let Some(faction) = self.faction_mut(owner) {
                        faction.energy += 1; // Emergency 1 energy per unit
                    }
                    self.destroy_unit(unit_id);
                    self.push_event_log(
                        EventCategory::Economics,
                        format!(
                            "BANKRUPTCY: {} unit disbanded for scrap value!",
                            self.faction_name(owner)
                        ),
                    );
                } else {
                    break; // Truly insolvent
                }
            }
        }

        // Mineral Famine: Disband units if minerals are still needed
        while remaining_minerals > 0 {
            if let Some(unit_id) = self.find_disbandable_unit(owner) {
                let Some(unit_name) = self
                    .unit(unit_id)
                    .map(|unit| presentation::unit_name(unit.kind.clone()).to_string())
                else {
                    break;
                };
                self.destroy_unit(unit_id);
                remaining_minerals -= content::unit_support_cost();
                self.push_event_log(
                    EventCategory::Economics,
                    format!(
                        "FAMINE: {} unit {} disbanded due to lack of support!",
                        self.faction_name(owner),
                        unit_name
                    ),
                );
            } else {
                break; // No more units to disband
            }
        }
    }

    fn find_scrapable_facility(&self, owner: usize) -> Option<(usize, Facility)> {
        // Find most expensive facility to scrap
        let mut best = None;
        let mut highest_cost = -1;

        for base in &self.bases {
            if base.owner != owner {
                continue;
            }
            for &facility in &base.facilities {
                let cost =
                    content::production_cost(ProductionItem::from_facility(facility).unwrap());
                if cost > highest_cost {
                    highest_cost = cost;
                    best = Some((base.id, facility));
                }
            }
        }
        best
    }

    fn find_disbandable_unit(&self, owner: usize) -> Option<usize> {
        // Find lowest experience unit to disband, avoiding critical units like Colony Pods if possible
        let mut best = None;
        let mut lowest_exp = 999;

        for unit in &self.units {
            if !unit.alive || unit.owner != owner {
                continue;
            }
            if unit.kind == UnitKind::ColonyPod || unit.kind == UnitKind::Former {
                continue;
            }

            if unit.experience < lowest_exp {
                lowest_exp = unit.experience;
                best = Some(unit.id);
            }
        }

        // Fallback to any unit if no combat units left
        if best.is_none() {
            for unit in &self.units {
                if !unit.alive || unit.owner != owner {
                    continue;
                }
                return Some(unit.id);
            }
        }

        best
    }

    fn defense_bonus_for_tile(&self, x: usize, y: usize) -> i32 {
        let Some(tile) = self.tile(x, y) else {
            return 0;
        };
        let terrain_bonus = match tile.terrain {
            Terrain::Rolling => content::rolling_defense_bonus(),
            Terrain::Rocky => content::rocky_defense_bonus(),
            Terrain::Fungus => content::fungus_defense_bonus(),
            _ => 0,
        };
        let base_bonus = tile
            .base
            .and_then(|base_id| self.base(base_id))
            .map(|base| {
                let facility_bonus: i32 = base
                    .facilities
                    .iter()
                    .copied()
                    .map(content::facility_defense_bonus)
                    .sum();
                content::base_defense_bonus() + facility_bonus
            })
            .unwrap_or(0);
        terrain_bonus + base_bonus
    }

    fn attacker_penalty(&self, x: usize, y: usize) -> i32 {
        self.tile(x, y)
            .and_then(|tile| tile.base)
            .map(|_| content::base_attack_penalty())
            .unwrap_or(0)
    }

    fn reveal_area(&mut self, x: usize, y: usize, radius: isize) {
        let player_owner = self.player_owner();

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dy.abs() > radius {
                    continue;
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx < 0 || ny < 0 {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                if nx >= self.width || ny >= self.height {
                    continue;
                }

                let idx = self.tile_index(nx, ny);
                self.tiles[idx].visible_by_owner.insert(player_owner);
                self.tiles[idx].explored_by_owner.insert(player_owner);
            }
        }
    }

    fn check_game_over(&mut self) {
        if self.game_over.is_some() {
            return; // Already over
        }

        let player_owner = self.player_owner();
        let faction_ids: Vec<usize> = (0..self.factions.len())
            .filter(|&id| id != crate::model::NATIVE_ID)
            .collect();

        // 1. Check all non-native factions for victory conditions
        for faction_id in faction_ids {
            let Some(faction) = self.faction(faction_id).cloned() else {
                continue;
            };

            // a. Transcendence Victory (late tech plus a dedicated project)
            if faction.known_techs.contains(&Tech::SecretsOfPlanet)
                && self.has_secret_project(faction_id, SecretProject::EmpathGuild)
                && self.council.governor_id == Some(faction_id)
            {
                if faction_id == player_owner {
                    self.game_over = Some(GameOver::PlayerWonTranscendence);
                } else {
                    self.game_over = Some(GameOver::AiWonTranscendence);
                }
                self.push_log(format!(
                    "VICTORY: {} has attained Ascent to Planet. Humanity transcends.",
                    faction.name
                ));
                return;
            }

            // b. Space Transcendence Victory (Secret Projects)
            if self.has_secret_project(faction_id, SecretProject::OrbitalElevator)
                && self.has_secret_project(faction_id, SecretProject::ManifoldDrive)
            {
                if faction_id == player_owner {
                    self.game_over = Some(GameOver::PlayerWonSpaceTranscendence);
                } else {
                    self.game_over = Some(GameOver::AiWonSpaceTranscendence);
                }
                self.push_log(format!(
                    "VICTORY: {} has attained Space Mastery. Humanity reaches for the stars.",
                    faction.name
                ));
                return;
            }

            // c. Black Hole Harvesting Victory (Secret Projects)
            if self.has_secret_project(faction_id, SecretProject::SingularityContainment)
                && self.has_secret_project(faction_id, SecretProject::BlackHoleHarvester)
            {
                if faction_id == player_owner {
                    self.game_over = Some(GameOver::PlayerWonBlackHoleHarvesting);
                } else {
                    self.game_over = Some(GameOver::AiWonBlackHoleHarvesting);
                }
                self.push_log(format!(
                    "VICTORY: {} has attained Singularity Mastery. Infinite energy awaits.",
                    faction.name
                ));
                return;
            }

            // d. Economic Victory
            if faction.energy >= 10000 {
                if faction_id == player_owner {
                    self.game_over = Some(GameOver::PlayerWonEconomic);
                } else {
                    self.game_over = Some(GameOver::AiWonEconomic);
                }
                self.push_log(format!(
                    "VICTORY: {} has achieved economic dominance. The market is yours.",
                    faction.name
                ));
                return;
            }
        }

        // 2. Conquest Victory
        let alive_factions: Vec<usize> = (0..self.factions.len())
            .filter(|&id| id != crate::model::NATIVE_ID && self.faction_alive(id))
            .collect();

        if alive_factions.len() == 1 {
            let winner = alive_factions[0];
            if winner == player_owner {
                self.game_over = Some(GameOver::PlayerWonConquest);
                self.push_log("VICTORY: Conquest! All rival factions eliminated.".to_string());
            } else {
                self.game_over = Some(GameOver::AiWonConquest);
                self.push_log(format!(
                    "VICTORY: {} has achieved total conquest of Planet.",
                    self.faction_name(winner)
                ));
            }
            return;
        }

        // 3. Loss Condition: Player eliminated
        if !self.faction_alive(player_owner) {
            self.game_over = Some(GameOver::PlayerLost);
            self.push_log("DEFEAT: Your faction has been wiped out.".to_string());
            return;
        }
    }

    fn faction_alive(&self, owner: usize) -> bool {
        self.units.iter().any(|u| u.alive && u.owner == owner)
            || self.bases.iter().any(|b| b.owner == owner)
    }

    fn next_base_name(&self, owner: usize) -> String {
        let count = self.bases.iter().filter(|b| b.owner == owner).count() + 1;
        let faction_name = self
            .factions
            .get(owner)
            .map(|faction| faction.name.as_str())
            .unwrap_or("Faction");
        content::next_base_name_for_faction(faction_name, count)
    }

    fn safest_fallback_destination(
        &self,
        owner: usize,
        x: usize,
        y: usize,
    ) -> Option<(usize, usize, String)> {
        let base = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .min_by_key(|base| self.distance(x, y, base.x, base.y))?;
        let step = self
            .safest_fallback_step(owner, x, y)
            .unwrap_or((base.x, base.y));
        Some((step.0, step.1, base.name.clone()))
    }

    fn safest_fallback_step(&self, owner: usize, x: usize, y: usize) -> Option<(usize, usize)> {
        let (target_x, target_y) = self
            .bases
            .iter()
            .filter(|base| base.owner == owner)
            .min_by_key(|base| self.distance(x, y, base.x, base.y))
            .map(|base| (base.x, base.y))?;

        let preferred_x = (x as isize + step_toward_coord(x, target_x))
            .clamp(0, self.width.saturating_sub(1) as isize) as usize;
        let preferred_y = (y as isize + step_toward_coord(y, target_y))
            .clamp(0, self.height.saturating_sub(1) as isize) as usize;

        let mut best: Option<(usize, usize, i32, i32)> = None;
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if nx >= self.width || ny >= self.height {
                    continue;
                }

                let Some(tile) = self.tile(nx, ny) else {
                    continue;
                };
                if !tile.terrain.is_land() {
                    continue;
                }
                if let Some(unit_id) = tile.unit {
                    if self
                        .unit(unit_id)
                        .map(|unit| unit.owner != owner)
                        .unwrap_or(true)
                    {
                        continue;
                    }
                }

                let threat = self.player_threat_at(nx, ny);
                let distance = self.distance(nx, ny, target_x, target_y);
                let ideal_penalty = if nx == preferred_x && ny == preferred_y {
                    0
                } else {
                    1
                };
                let score = threat + ideal_penalty;
                if best
                    .map(|(_, _, best_score, best_distance)| {
                        score < best_score || score == best_score && distance < best_distance
                    })
                    .unwrap_or(true)
                {
                    best = Some((nx, ny, score, distance));
                }
            }
        }

        best.map(|(nx, ny, _, _)| (nx, ny))
    }

    pub fn call_council(&mut self) -> Result<(), String> {
        if !self.council.is_active {
            return Err("Planetary Council is not yet active.".to_string());
        }
        if self.turn < self.council.last_meeting_turn + 20 {
            return Err("Council cannot be called so soon after the last meeting.".to_string());
        }

        self.council.pending_votes.clear();
        self.push_event_log(
            EventCategory::Diplomacy,
            "PLANETARY COUNCIL: A session has been convened to elect a Planetary Governor."
                .to_string(),
        );
        Ok(())
    }

    pub fn vote_for_governor(
        &mut self,
        voter_id: usize,
        candidate_id: usize,
    ) -> Result<(), String> {
        if !self.council.is_active {
            return Err("Council not active.".to_string());
        }
        if voter_id == crate::model::NATIVE_ID || candidate_id == crate::model::NATIVE_ID {
            return Err("Native life cannot participate in council votes.".to_string());
        }
        if self
            .council
            .pending_votes
            .iter()
            .any(|vote| vote.faction_id == voter_id)
        {
            return Err("Faction has already voted in this council session.".to_string());
        }
        let weight = self.calculate_vote_weight(voter_id);
        self.council.pending_votes.push(crate::model::CouncilVote {
            faction_id: voter_id,
            candidate_id,
            weight,
        });
        self.push_event_log(
            EventCategory::Diplomacy,
            format!(
                "PLANETARY COUNCIL: {} cast a {}-weight governor vote for {}.",
                self.faction_name(voter_id),
                weight,
                self.faction_name(candidate_id)
            ),
        );

        if self.council.pending_votes.len() >= self.factions.len() - 1 {
            self.resolve_council_votes();
        }
        Ok(())
    }

    pub fn vote_for_supreme_leader(
        &mut self,
        voter_id: usize,
        candidate_id: usize,
    ) -> Result<(), String> {
        if !self.council.is_active {
            return Err("Council not active.".to_string());
        }
        if voter_id == crate::model::NATIVE_ID || candidate_id == crate::model::NATIVE_ID {
            return Err("Native life cannot participate in council votes.".to_string());
        }
        if self
            .council
            .pending_votes
            .iter()
            .any(|vote| vote.faction_id == voter_id)
        {
            return Err("Faction has already voted in this council session.".to_string());
        }
        let weight = self.calculate_vote_weight(voter_id);
        self.council.pending_votes.push(crate::model::CouncilVote {
            faction_id: voter_id,
            candidate_id,
            weight,
        });
        self.push_event_log(
            EventCategory::Diplomacy,
            format!(
                "PLANETARY COUNCIL: {} cast a {}-weight supreme-leader vote for {}.",
                self.faction_name(voter_id),
                weight,
                self.faction_name(candidate_id)
            ),
        );

        if self.council.pending_votes.len() >= self.factions.len() - 1 {
            self.resolve_council_votes();
        }
        Ok(())
    }

    fn calculate_vote_weight(&self, faction_id: usize) -> i32 {
        let Some(faction) = self.faction(faction_id) else {
            return 0;
        };
        let base_weight = faction.total_population(self);
        if self.has_secret_project(faction_id, SecretProject::EmpathGuild) {
            base_weight * 2
        } else {
            base_weight
        }
    }

    fn resolve_council_votes(&mut self) {
        let mut tallies = std::collections::HashMap::new();
        for vote in &self.council.pending_votes {
            *tallies.entry(vote.candidate_id).or_insert(0) += vote.weight;
        }

        let total_weight: i32 = tallies.values().sum();
        let mut winner = None;
        for (&candidate, &weight) in &tallies {
            if weight > total_weight * 2 / 3 {
                winner = Some(candidate);
                break;
            }
        }

        if let Some(winner_id) = winner {
            self.council.governor_id = Some(winner_id);
            let name = self.faction_name(winner_id);
            self.push_event_log(
                EventCategory::Diplomacy,
                format!(
                    "PLANETARY COUNCIL: {} has been elected Planetary Governor!",
                    name
                ),
            );
        } else {
            self.push_event_log(
                EventCategory::Diplomacy,
                "PLANETARY COUNCIL: No candidate received the required 2/3 majority.".to_string(),
            );
        }

        self.council.pending_votes.clear();
        self.council.last_meeting_turn = self.turn;
    }

    fn check_council_activation(&mut self) {
        if self.council.is_active {
            return;
        }

        let mut has_empath = false;
        for faction in &self.factions {
            if self.has_secret_project(faction.id, SecretProject::EmpathGuild) {
                has_empath = true;
                break;
            }
        }

        if has_empath {
            self.council.is_active = true;
            self.push_event_log(
                EventCategory::Diplomacy,
                "PLANETARY COUNCIL: The Empath Guild has established the Planetary Council."
                    .to_string(),
            );
        }
    }

    pub fn distance(&self, ax: usize, ay: usize, bx: usize, by: usize) -> i32 {
        (ax.abs_diff(bx) + ay.abs_diff(by)) as i32
    }

    fn player_threat_at(&self, x: usize, y: usize) -> i32 {
        let player_owner = self.player_owner();
        let mut threat = 0;

        for unit in self
            .units
            .iter()
            .filter(|unit| unit.alive && unit.owner == player_owner)
        {
            let distance = self.distance(x, y, unit.x, unit.y);
            if distance <= 1 {
                threat += 4;
            } else if distance <= 2 {
                threat += 2;
            } else if distance <= 3 {
                threat += 1;
            }
        }

        for base in self.bases.iter().filter(|base| base.owner == player_owner) {
            let distance = self.distance(x, y, base.x, base.y);
            if distance <= 2 {
                threat += 1;
            }
        }

        threat
    }

    pub fn tile_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn is_adjacent(ax: usize, ay: usize, bx: usize, by: usize) -> bool {
        let dx = ax.abs_diff(bx);
        let dy = ay.abs_diff(by);
        dx <= 1 && dy <= 1 && (dx + dy) > 0
    }

    pub(crate) fn sample_noise(&self, x: i32, y: i32, salt: u32) -> u32 {
        let mut n = self.seed
            ^ salt
            ^ ((x as u32).wrapping_mul(374_761_393))
            ^ ((y as u32).wrapping_mul(668_265_263));

        n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
        n ^ (n >> 16)
    }
    pub fn player_turn_summary(&self) -> crate::TurnSummary {
        self.generate_turn_summary(self.player_owner())
    }

    pub fn generate_turn_summary(&self, owner: usize) -> crate::TurnSummary {
        let mut alerts = Vec::new();
        let mut event_highlights = Vec::new();

        let faction = match self.faction(owner) {
            Some(f) => f,
            None => return crate::TurnSummary::default(),
        };

        // 1. Production Alerts
        for base in self.bases_for(owner) {
            if base.production_queue.is_empty() {
                alerts.push(crate::ActionableAlert {
                    priority: crate::AlertPriority::High,
                    message: format!("Production queue empty in {}.", base.name),
                    location: Some((base.x, base.y)),
                    base_id: Some(base.id),
                    unit_id: None,
                });
            }

            if let Some(margin) = self.base_food_margin(base.id) {
                if margin < 0 {
                    alerts.push(crate::ActionableAlert {
                        priority: crate::AlertPriority::Critical,
                        message: format!("FOOD SHORTAGE: {} is starving!", base.name),
                        location: Some((base.x, base.y)),
                        base_id: Some(base.id),
                        unit_id: None,
                    });
                }
            }

            if let Some(margin) = self.base_mineral_margin(base.id) {
                if margin < 0 {
                    alerts.push(crate::ActionableAlert {
                        priority: crate::AlertPriority::High,
                        message: format!(
                            "INDUSTRIAL STALL: {} mineral support failure!",
                            base.name
                        ),
                        location: Some((base.x, base.y)),
                        base_id: Some(base.id),
                        unit_id: None,
                    });
                }
            }

            if self.base_unrest(base.id) > 0 {
                alerts.push(crate::ActionableAlert {
                    priority: crate::AlertPriority::High,
                    message: format!("UNREST: {base_name} is in revolt!", base_name = base.name),
                    location: Some((base.x, base.y)),
                    base_id: Some(base.id),
                    unit_id: None,
                });
            }
        }

        // 2. Economic Alerts
        let (energy_upkeep, _, _) = self.faction_upkeep(owner);
        if faction.energy < energy_upkeep && energy_upkeep > 0 {
            alerts.push(crate::ActionableAlert {
                priority: crate::AlertPriority::Critical,
                message: "FINANCIAL CRISIS: Insufficient energy to pay upkeep!".to_string(),
                location: None,
                base_id: None,
                unit_id: None,
            });
        }

        // 3. Diplomatic Alerts
        for (offerer, target, status) in &self.pending_diplomacy_offers {
            if *target == owner {
                alerts.push(crate::ActionableAlert {
                    priority: crate::AlertPriority::High,
                    message: format!(
                        "PENDING OFFER: {} proposes a {}!",
                        self.faction_name(*offerer),
                        crate::presentation::diplomacy_status_text(*status)
                    ),
                    location: None,
                    base_id: None,
                    unit_id: None,
                });
            }
        }

        // 4. Tactical Alerts: Enemy units near bases
        for base in self.bases_for(owner) {
            let pressure = self.base_local_military_pressure(base.id);
            if pressure >= 3 {
                alerts.push(crate::ActionableAlert {
                    priority: crate::AlertPriority::High,
                    message: format!("IMMINENT THREAT: Strong enemy presence near {}.", base.name),
                    location: Some((base.x, base.y)),
                    base_id: Some(base.id),
                    unit_id: None,
                });
            }
        }

        // 5. Pending Demands
        for (proposer, receiver, demand) in &self.pending_demands {
            if *receiver == owner {
                let demand_text = match demand {
                    crate::model::DemandKind::Technology(tech) => {
                        format!("their knowledge of {}", presentation::tech_name(*tech))
                    }
                    crate::model::DemandKind::Energy(amount) => format!("{} energy units", amount),
                };
                alerts.push(crate::ActionableAlert {
                    priority: crate::AlertPriority::Critical,
                    message: format!(
                        "ULTIMATUM: {} demands {}!",
                        self.faction_name(*proposer),
                        demand_text
                    ),
                    location: None,
                    base_id: None,
                    unit_id: None,
                });
            }
        }

        // 6. Diplomatic/Event Highlights (from recent log)
        event_highlights.push(format!(
            "Our faction has {} population across {} bases.",
            faction.total_population(self),
            self.bases_for(owner).len()
        ));
        event_highlights.push(format!(
            "Our military consists of {} active units.",
            self.live_units_for(owner).len()
        ));

        for entry in self.log.iter().rev().take(50) {
            if entry.turn == self.turn {
                if entry.message.contains("completed production")
                    || entry.message.contains("discovered")
                    || entry.message.contains("WAR")
                    || entry.message.contains("PACT")
                    || entry.message.contains("victory")
                {
                    event_highlights.push(entry.message.clone());
                }
            }
        }

        crate::TurnSummary {
            turn: self.turn,
            alerts,
            event_highlights,
        }
    }
}

fn step_toward_coord(current: usize, target: usize) -> isize {
    if target > current {
        1
    } else if target < current {
        -1
    } else {
        0
    }
}
