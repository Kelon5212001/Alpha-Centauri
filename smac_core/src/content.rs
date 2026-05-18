use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::model::{AI_ID, NATIVE_ID, PLAYER_ID};
use crate::technology_tree::{canonical_technologies, Technology};
use crate::units::definitions::{design_from_definition, try_design_from_content};
use crate::{Ability, Facility, Faction, Improvement, SecretProject, Tech, UnitKind, Yields};

const FACTIONS_JSON: &str = include_str!("../../data/factions.json");
const FACILITIES_JSON: &str = include_str!("../../data/facilities.json");
const PRODUCTION_JSON: &str = include_str!("../../data/production.json");
const START_SCENARIO_JSON: &str = include_str!("../../data/start_scenario.json");
const RUNTIME_RULES_JSON: &str = include_str!("../../data/runtime_rules.json");
const UNIT_DEFINITIONS_JSON: &str = include_str!("../../data/units.json");
const UI_THEME_JSON: &str = include_str!("../../data/ui_theme.json");

static FACTION_DEFINITIONS: OnceLock<Result<Vec<FactionDefinition>, String>> = OnceLock::new();
static FACILITY_DEFINITIONS: OnceLock<Result<Vec<FacilityDefinition>, String>> = OnceLock::new();
static PRODUCTION_DEFINITIONS: OnceLock<Result<Vec<ProductionDefinition>, String>> =
    OnceLock::new();
static RUNTIME_RULES: OnceLock<Result<RuntimeRulesDefinition, String>> = OnceLock::new();
static UNIT_DEFINITIONS: OnceLock<Result<Vec<UnitDefinition>, String>> = OnceLock::new();
static UI_THEME: OnceLock<Result<UiThemeDefinition, String>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeRole {
    Player,
    Ai,
    Native,
}

impl RuntimeRole {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeRole::Player => "player",
            RuntimeRole::Ai => "ai",
            RuntimeRole::Native => "native",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "player" => Some(RuntimeRole::Player),
            "ai" => Some(RuntimeRole::Ai),
            "native" => Some(RuntimeRole::Native),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRoles {
    pub player: usize,
    pub ai: usize,
    pub native: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FactionPersonality {
    pub archetype: String,
    pub preferred_victory: String,
    pub forbidden_actions: Vec<String>,
    pub required_actions: Vec<String>,
    #[serde(default)]
    pub aggression: i32,
    #[serde(default)]
    pub tech_bias: i32,
    #[serde(default)]
    pub diplomatic_tone: i32,
    #[serde(default)]
    pub expansion_bias: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FactionAiPolicy {
    pub expansion_base_target: usize,
    pub attack_bias: u8,
    pub preferred_production: String,
    pub exploration_bias: u8,
    pub terraform_bias: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FactionDefinition {
    pub id: usize,
    pub name: String,
    pub leader: String,
    pub description: String,
    pub bonuses: Vec<String>,
    pub penalties: Vec<String>,
    pub color: String,
    pub color_hex: String,
    pub runtime_role: Option<String>,
    pub starting_energy: Option<i32>,
    pub known_tech_ids: Vec<String>,
    pub current_research_id: Option<String>,
    pub base_names: Vec<String>,
    #[serde(default)]
    pub starting_food_security: i32,
    #[serde(default)]
    pub starting_ai_dependence: i32,
    #[serde(default)]
    pub starting_planet_toxicity: i32,
    pub ai_policy: Option<FactionAiPolicy>,
    pub personality: FactionPersonality,
}

#[derive(Debug, Clone)]
pub struct StartingFactionSetup {
    pub runtime_id: usize,
    pub faction_name: String,
    pub starting_energy: i32,
    pub is_ai: bool,
    pub known_techs: Vec<Tech>,
    pub current_research: Tech,
    pub starting_food_security: i32,
    pub starting_ai_dependence: i32,
    pub starting_planet_toxicity: i32,
    pub unit_designs: Vec<crate::units::definitions::UnitDesign>,
    pub base_attributes: crate::model::FactionAttributes,
    pub social_engineering: crate::model::SocialEngineering,
    pub personality: crate::model::FactionPersonality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartPosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingUnitSetup {
    pub owner: usize,
    pub kind: UnitKind,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone)]
pub struct StartingScenario {
    pub forced_land_positions: Vec<StartPosition>,
    pub starting_units: Vec<StartingUnitSetup>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProductionDefinition {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub build_kind: String,
    pub unit_kind: Option<String>,
    pub facility_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacilityDefinition {
    pub id: String,
    pub name: String,
    pub maintenance: i32,
    pub defense_bonus: i32,
    #[serde(default)]
    pub stability_bonus: i32,
    #[serde(default)]
    pub repair_bonus: i32,
    #[serde(default)]
    pub training_bonus: i32,
    #[serde(default)]
    pub growth_threshold_reduction: i32,
    #[serde(default)]
    pub free_unit_support_bonus: i32,
    #[serde(default)]
    pub mobility_bonus: i32,
    #[serde(default)]
    pub psi_support_bonus: i32,
    #[serde(default)]
    pub convoy_capacity_bonus: i32,
    #[serde(default)]
    pub convoy_security_bonus: i32,
    pub yield_bonus: YieldBonusDefinition,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct YieldBonusDefinition {
    pub nutrients: i32,
    pub minerals: i32,
    pub energy: i32,
}

pub type TechRuntimeDefinition = Technology;

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeRulesDefinition {
    pub base_growth_nutrients_threshold: i32,
    pub supply_pod_energy_reward: i32,
    pub supply_pod_salvage_energy_reward: i32,
    pub native_spawn_turn_interval: i32,
    pub native_spawn_roll_threshold: u32,
    pub ai_colony_base_target: usize,
    pub ai_native_spawn_noise_salt: u32,
    pub player_unit_visibility_radius: isize,
    pub player_base_visibility_radius: isize,
    pub map_ocean_threshold: u32,
    pub map_fungus_threshold: u32,
    pub map_rocky_threshold: u32,
    pub map_flat_moisture_threshold: u32,
    pub map_pod_spawn_threshold: u32,
    pub forced_land_patch_radius: isize,
    pub base_starting_population: i32,
    pub base_starting_nutrients_stock: i32,
    pub base_starting_minerals_stock: i32,
    pub free_unit_support_per_base: i32,
    pub unit_support_cost: i32,
    pub rolling_defense_bonus: i32,
    pub rocky_defense_bonus: i32,
    pub fungus_defense_bonus: i32,
    pub base_defense_bonus: i32,
    pub base_attack_penalty: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnitDefinition {
    pub id: String,
    pub name: String,
    pub chassis: String,
    pub weapon_kind: String,
    pub weapon_power: u8,
    pub armor_kind: String,
    pub armor_power: u8,
    pub cost: u16,
    pub abilities: Vec<String>,
    pub max_moves: i32,
    pub attack: i32,
    pub defense: i32,
    pub base_hp: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiThemeDefinition {
    pub window_title: String,
    pub app_title: String,
    pub command_console_heading: String,
    pub selection_heading: String,
    pub research_heading: String,
    pub factions_heading: String,
    pub event_log_heading: String,
    pub planet_heading: String,
    pub victory_text: String,
    pub defeat_text: String,
    pub warning_text: String,
    pub accent_hex: String,
    pub warning_hex: String,
    pub danger_hex: String,
    pub success_hex: String,
    pub panel_fill_hex: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ScenarioAnchorPoint {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct StartPositionDefinition {
    x: Option<usize>,
    y: Option<usize>,
    anchor: Option<String>,
    x_offset: Option<isize>,
    y_offset: Option<isize>,
}

#[derive(Debug, Clone, Deserialize)]
struct StartingUnitDefinition {
    owner: String,
    kind: String,
    anchor: String,
    x_offset: isize,
    y_offset: isize,
}

#[derive(Debug, Clone, Deserialize)]
struct StartingScenarioDefinition {
    forced_land_positions: Vec<StartPositionDefinition>,
    starting_units: Vec<StartingUnitDefinition>,
}

pub fn load_faction_definitions() -> Result<Vec<FactionDefinition>, String> {
    Ok(faction_definitions()?.clone())
}

pub fn load_production_definitions() -> Result<Vec<ProductionDefinition>, String> {
    Ok(production_definitions()?.clone())
}

pub fn load_facility_definitions() -> Result<Vec<FacilityDefinition>, String> {
    Ok(facility_definitions()?.clone())
}

pub fn load_runtime_tech_definitions() -> Result<Vec<TechRuntimeDefinition>, String> {
    Ok(canonical_technologies()?.clone())
}

pub fn try_runtime_tech_definitions() -> Result<&'static Vec<TechRuntimeDefinition>, String> {
    canonical_technologies()
}

pub fn load_runtime_rules() -> Result<RuntimeRulesDefinition, String> {
    Ok(runtime_rules()?.clone())
}

pub fn try_runtime_rules_definition() -> Result<&'static RuntimeRulesDefinition, String> {
    runtime_rules()
}

pub fn load_unit_definitions() -> Result<Vec<UnitDefinition>, String> {
    Ok(unit_definitions()?.clone())
}

pub fn load_localization_data() -> Result<crate::localization::LocalizationData, String> {
    let json = std::fs::read_to_string("data/localization.json")
        .map_err(|e| format!("Failed to read localization.json: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse localization.json: {}", e))
}

pub fn load_ui_theme_definition() -> Result<UiThemeDefinition, String> {
    Ok(ui_theme()?.clone())
}

pub fn try_ui_theme_definition() -> Result<&'static UiThemeDefinition, String> {
    ui_theme()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartingScenarioError {
    InvalidForcedLandPosition,
    UnknownAnchor(String),
    UnknownOwner(String),
    UnknownUnitKind(String),
}

impl std::fmt::Display for StartingScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartingScenarioError::InvalidForcedLandPosition => {
                write!(
                    f,
                    "starting scenario contains an invalid forced land position"
                )
            }
            StartingScenarioError::UnknownAnchor(anchor) => {
                write!(f, "starting scenario references unknown anchor '{anchor}'")
            }
            StartingScenarioError::UnknownOwner(owner) => {
                write!(f, "starting scenario contains unknown owner id '{owner}'")
            }
            StartingScenarioError::UnknownUnitKind(kind) => {
                write!(f, "starting scenario contains unknown unit kind '{kind}'")
            }
        }
    }
}

impl std::error::Error for StartingScenarioError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentLookupError {
    MissingProductionDefinition(String),
    MissingTechnologyDefinition(String),
    MissingUnitDefinition(String),
    MissingFacilityDefinition(String),
}

impl std::fmt::Display for ContentLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentLookupError::MissingProductionDefinition(id) => {
                write!(f, "missing production definition for '{id}'")
            }
            ContentLookupError::MissingTechnologyDefinition(id) => {
                write!(f, "missing technology definition for '{id}'")
            }
            ContentLookupError::MissingUnitDefinition(id) => {
                write!(f, "missing unit definition for '{id}'")
            }
            ContentLookupError::MissingFacilityDefinition(id) => {
                write!(f, "missing facility definition for '{id}'")
            }
        }
    }
}

impl std::error::Error for ContentLookupError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeFactionSetupError {
    UnknownRuntimeRole {
        faction_name: String,
        role: String,
    },
    MissingCurrentResearch {
        faction_name: String,
    },
    UnknownCurrentResearch {
        faction_name: String,
        tech_id: String,
    },
    UnknownKnownTech {
        faction_name: String,
        tech_id: String,
    },
    MissingStartingEnergy {
        faction_name: String,
    },
}

impl std::fmt::Display for RuntimeFactionSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeFactionSetupError::UnknownRuntimeRole { faction_name, role } => write!(
                f,
                "runtime faction '{faction_name}' uses unknown runtime role '{role}'"
            ),
            RuntimeFactionSetupError::MissingCurrentResearch { faction_name } => write!(
                f,
                "runtime faction '{faction_name}' is missing current_research_id"
            ),
            RuntimeFactionSetupError::UnknownCurrentResearch {
                faction_name,
                tech_id,
            } => write!(
                f,
                "runtime faction '{faction_name}' references unknown current_research_id '{tech_id}'"
            ),
            RuntimeFactionSetupError::UnknownKnownTech {
                faction_name,
                tech_id,
            } => write!(
                f,
                "runtime faction '{faction_name}' references unknown known_tech id '{tech_id}'"
            ),
            RuntimeFactionSetupError::MissingStartingEnergy { faction_name } => write!(
                f,
                "runtime faction '{faction_name}' is missing starting_energy"
            ),
        }
    }
}

impl std::error::Error for RuntimeFactionSetupError {}

pub fn validate_bundled_content() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    validate_runtime_roles(&mut errors);
    validate_factions(&mut errors);
    validate_technologies(&mut errors);
    validate_units(&mut errors);
    validate_facilities(&mut errors);
    validate_production(&mut errors);
    validate_runtime_rules(&mut errors);
    validate_ui_theme(&mut errors);
    validate_starting_scenario(&mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn default_starting_factions() -> Vec<StartingFactionSetup> {
    try_default_runtime_faction_setups()
        .expect("bundled runtime faction content must resolve for active factions")
}

pub fn build_runtime_factions() -> Vec<Faction> {
    default_starting_factions()
        .into_iter()
        .map(|setup| Faction {
            id: setup.runtime_id,
            name: setup.faction_name,
            energy: setup.starting_energy,
            research: 0,
            techs_discovered: 0,
            is_ai: setup.is_ai,
            known_techs: setup.known_techs,
            current_research: setup.current_research,
            unit_designs: setup.unit_designs,
            food_security: setup.starting_food_security,
            ai_dependence: setup.starting_ai_dependence,
            orbital_index: 0,
            sky_hydroponics: 0,
            solar_transmitters: 0,
            orbital_defenses: 0,
            planet_toxicity: setup.starting_planet_toxicity,
            base_attributes: setup.base_attributes,
            social_engineering: setup.social_engineering,
            personality: setup.personality,
            headquarters_base_id: None,
        })
        .collect()
}

fn parse_faction_attributes(definition: &FactionDefinition) -> crate::model::FactionAttributes {
    let mut attr = crate::model::FactionAttributes::default();

    for line in definition.bonuses.iter().chain(definition.penalties.iter()) {
        let line = line.to_lowercase();
        if line.contains("morale") {
            attr.morale += parse_bonus_value(&line);
        } else if line.contains("research") {
            attr.research += parse_bonus_value(&line);
        } else if line.contains("industry") {
            attr.industry += parse_bonus_value(&line);
        } else if line.contains("efficiency") {
            attr.efficiency += parse_bonus_value(&line);
        } else if line.contains("planet") {
            attr.planet += parse_bonus_value(&line);
        } else if line.contains("support") {
            attr.support += parse_bonus_value(&line);
        } else if line.contains("probe") {
            attr.probe += parse_bonus_value(&line);
        } else if line.contains("economy") {
            attr.economy += parse_bonus_value(&line);
        } else if line.contains("growth") {
            attr.growth += parse_bonus_value(&line);
        } else if line.contains("police") {
            attr.police += parse_bonus_value(&line);
        }

        // Special handling for free facilities or equivalents
        if line.contains("command nexus equivalent") {
            attr.facility_equivalents
                .push(crate::Facility::CommandCenter);
        }
        if line.contains("network node at every base") {
            attr.free_facilities.push(crate::Facility::NetworkNode);
        }
    }

    attr
}

fn parse_bonus_value(line: &str) -> i32 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in parts {
        if let Ok(val) = part.parse::<i32>() {
            return val;
        }
        // Handle "+1" or "-1"
        if (part.starts_with('+') || part.starts_with('-')) && part.len() > 1 {
            if let Ok(val) = part[1..].parse::<i32>() {
                return if part.starts_with('+') { val } else { -val };
            }
        }
    }
    0
}

pub fn faction_definition_by_name(name: &str) -> &'static FactionDefinition {
    faction_definitions()
        .expect("bundled faction content must parse for runtime lookup")
        .iter()
        .find(|definition| definition.name == name)
        .expect("bundled faction content missing a required faction definition")
}

pub fn runtime_faction_definition(name: &str) -> Option<&'static FactionDefinition> {
    faction_definitions()
        .expect("bundled faction content must parse for runtime lookup")
        .iter()
        .find(|definition| definition.name == name)
}

pub fn runtime_faction_definition_by_owner(owner: usize) -> Option<&'static FactionDefinition> {
    let role = runtime_role_for_owner(owner)?.as_str();

    faction_definitions()
        .expect("bundled faction content must parse for runtime lookup")
        .iter()
        .find(|definition| definition.runtime_role.as_deref() == Some(role))
}

pub fn runtime_roles() -> RuntimeRoles {
    RuntimeRoles {
        player: PLAYER_ID,
        ai: AI_ID,
        native: NATIVE_ID,
    }
}

pub fn runtime_role_for_owner(owner: usize) -> Option<RuntimeRole> {
    let roles = runtime_roles();
    match owner {
        owner if owner == roles.player => Some(RuntimeRole::Player),
        owner if owner == roles.ai => Some(RuntimeRole::Ai),
        owner if owner == roles.native => Some(RuntimeRole::Native),
        _ => None,
    }
}

pub fn owner_for_role(role: RuntimeRole) -> usize {
    let roles = runtime_roles();
    match role {
        RuntimeRole::Player => roles.player,
        RuntimeRole::Ai => roles.ai,
        RuntimeRole::Native => roles.native,
    }
}

pub fn next_base_name_for_faction(faction_name: &str, count: usize) -> String {
    let Some(definition) = runtime_faction_definition(faction_name) else {
        return format!("{faction_name} Base {count}");
    };

    if let Some(name) = definition.base_names.get(count.saturating_sub(1)) {
        return name.clone();
    }

    let stem = definition
        .name
        .split_whitespace()
        .next()
        .unwrap_or("Faction");
    format!("{stem} Base {count}")
}

pub fn ai_expansion_base_target(owner: usize) -> usize {
    runtime_faction_definition_by_owner(owner)
        .and_then(|definition| definition.ai_policy.as_ref())
        .map(|policy| policy.expansion_base_target)
        .unwrap_or_else(ai_colony_base_target)
}

pub fn ai_attack_bias(owner: usize) -> u8 {
    runtime_faction_definition_by_owner(owner)
        .and_then(|definition| definition.ai_policy.as_ref())
        .map(|policy| policy.attack_bias)
        .unwrap_or(5)
}

pub fn ai_preferred_production(owner: usize) -> crate::ProductionItem {
    runtime_faction_definition_by_owner(owner)
        .and_then(|definition| definition.ai_policy.as_ref())
        .and_then(|policy| crate::ProductionItem::from_content_id(&policy.preferred_production))
        .unwrap_or(crate::ProductionItem::ScoutPatrol)
}

pub fn ai_exploration_bias(owner: usize) -> u8 {
    runtime_faction_definition_by_owner(owner)
        .and_then(|definition| definition.ai_policy.as_ref())
        .map(|policy| policy.exploration_bias)
        .unwrap_or(0)
}

pub fn ai_terraform_bias(owner: usize) -> u8 {
    runtime_faction_definition_by_owner(owner)
        .and_then(|definition| definition.ai_policy.as_ref())
        .map(|policy| policy.terraform_bias)
        .unwrap_or(0)
}

pub fn default_starting_scenario(width: usize, height: usize) -> StartingScenario {
    try_default_starting_scenario(width, height)
        .expect("bundled starting scenario must resolve for the active runtime")
}

pub fn try_default_starting_scenario(
    width: usize,
    height: usize,
) -> Result<StartingScenario, StartingScenarioError> {
    let raw = load_raw_content("start_scenario.json", START_SCENARIO_JSON);
    let definition: StartingScenarioDefinition =
        serde_json::from_str(&raw).expect("bundled starting scenario must be valid JSON");

    let anchors = scenario_anchors(width, height);

    Ok(StartingScenario {
        forced_land_positions: definition
            .forced_land_positions
            .into_iter()
            .map(|position| resolve_position(position, &anchors, width, height))
            .collect::<Result<Vec<_>, _>>()?,
        starting_units: definition
            .starting_units
            .into_iter()
            .map(|unit| {
                Ok(StartingUnitSetup {
                    owner: owner_id_from_content(&unit.owner)?,
                    kind: UnitKind::from_content_id(&unit.kind)
                        .ok_or_else(|| StartingScenarioError::UnknownUnitKind(unit.kind))?,
                    x: resolve_anchored_coordinate(
                        &anchors,
                        width,
                        &unit.anchor,
                        unit.x_offset,
                        true,
                    )?,
                    y: resolve_anchored_coordinate(
                        &anchors,
                        height,
                        &unit.anchor,
                        unit.y_offset,
                        false,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn scenario_anchors(width: usize, height: usize) -> Vec<(&'static str, ScenarioAnchorPoint)> {
    let player_x = 3.min(width.saturating_sub(5));
    let player_y = 3.min(height.saturating_sub(5));
    let ai_x = width
        .saturating_sub(5)
        .max(player_x.saturating_add(4))
        .min(width.saturating_sub(2));
    let ai_y = height
        .saturating_sub(5)
        .max(player_y.saturating_add(4))
        .min(height.saturating_sub(2));
    vec![
        (
            RuntimeRole::Player.as_str(),
            ScenarioAnchorPoint {
                x: player_x,
                y: player_y,
            },
        ),
        (
            RuntimeRole::Ai.as_str(),
            ScenarioAnchorPoint {
                x: ai_x,
                y: ai_y,
            },
        ),
        (
            "midline",
            ScenarioAnchorPoint {
                x: width / 2,
                y: height / 2,
            },
        ),
    ]
}

fn resolve_position(
    position: StartPositionDefinition,
    anchors: &[(&str, ScenarioAnchorPoint)],
    width: usize,
    height: usize,
) -> Result<StartPosition, StartingScenarioError> {
    match (position.x, position.y, position.anchor.as_deref()) {
        (Some(x), Some(y), None) => Ok(StartPosition { x, y }),
        (_, _, Some(anchor)) => Ok(StartPosition {
            x: resolve_anchored_coordinate(
                anchors,
                width,
                anchor,
                position.x_offset.unwrap_or(0),
                true,
            )?,
            y: resolve_anchored_coordinate(
                anchors,
                height,
                anchor,
                position.y_offset.unwrap_or(0),
                false,
            )?,
        }),
        _ => Err(StartingScenarioError::InvalidForcedLandPosition),
    }
}

fn resolve_anchored_coordinate(
    anchors: &[(&str, ScenarioAnchorPoint)],
    limit: usize,
    anchor: &str,
    offset: isize,
    use_x: bool,
) -> Result<usize, StartingScenarioError> {
    let anchor_point = anchors
        .iter()
        .find(|(name, _)| *name == anchor)
        .map(|(_, point)| point)
        .ok_or_else(|| StartingScenarioError::UnknownAnchor(anchor.to_string()))?;

    let base = if use_x {
        anchor_point.x
    } else {
        anchor_point.y
    } as isize;
    Ok(base
        .saturating_add(offset)
        .clamp(0, limit.saturating_sub(1) as isize) as usize)
}

fn owner_id_from_content(owner: &str) -> Result<usize, StartingScenarioError> {
    let role = RuntimeRole::from_str(owner)
        .ok_or_else(|| StartingScenarioError::UnknownOwner(owner.to_string()))?;
    Ok(owner_for_role(role))
}

fn try_runtime_faction_setup(
    definition: &FactionDefinition,
) -> Result<Option<StartingFactionSetup>, RuntimeFactionSetupError> {
    let Some(runtime_role_str) = definition.runtime_role.as_deref() else {
        return Ok(None);
    };
    let runtime_role = RuntimeRole::from_str(runtime_role_str).ok_or_else(|| {
        RuntimeFactionSetupError::UnknownRuntimeRole {
            faction_name: definition.name.clone(),
            role: runtime_role_str.to_string(),
        }
    })?;
    let runtime_id = owner_for_role(runtime_role);

    let current_research_id = definition.current_research_id.as_deref().ok_or_else(|| {
        RuntimeFactionSetupError::MissingCurrentResearch {
            faction_name: definition.name.clone(),
        }
    })?;
    let current_research = Tech::from_content_id(current_research_id).ok_or_else(|| {
        RuntimeFactionSetupError::UnknownCurrentResearch {
            faction_name: definition.name.clone(),
            tech_id: current_research_id.to_string(),
        }
    })?;

    let known_techs = definition
        .known_tech_ids
        .iter()
        .map(|tech_id| {
            Tech::from_content_id(tech_id).ok_or_else(|| {
                RuntimeFactionSetupError::UnknownKnownTech {
                    faction_name: definition.name.clone(),
                    tech_id: tech_id.clone(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let starting_designs = [
        "scout_patrol",
        "colony_pod",
        "former",
        "speeder",
        "resonance_laser",
        "escort_speeder",
        "raider_speeder",
        "trance_scout",
        "garrison_guard",
        "psi_sentinel",
        "mind_worm",
        "isle_of_the_deep",
        "needlejet",
        "probe_team",
    ]
    .iter()
    .filter_map(|id| try_design_from_content(id).ok())
    .collect::<Vec<_>>();

    let known_techs = if known_techs.is_empty() {
        match definition.name.as_str() {
            "University of Planet" => vec![Tech::from_content_id("information_networks").unwrap()],
            "Morgan Industries" => vec![Tech::from_content_id("industrial_base").unwrap()],
            "Gaia's Stepdaughters" => vec![Tech::from_content_id("centauri_ecology").unwrap()],
            "Spartan Federation" => vec![Tech::from_content_id("doctrine_mobility").unwrap()],
            _ => known_techs,
        }
    } else {
        known_techs
    };

    let attributes = parse_faction_attributes(definition);

    Ok(Some(StartingFactionSetup {
        runtime_id,
        faction_name: definition.name.clone(),
        starting_energy: definition.starting_energy.ok_or_else(|| {
            RuntimeFactionSetupError::MissingStartingEnergy {
                faction_name: definition.name.clone(),
            }
        })?,
        is_ai: runtime_role != RuntimeRole::Player,
        known_techs,
        current_research,
        starting_food_security: definition.starting_food_security,
        starting_ai_dependence: definition.starting_ai_dependence,
        starting_planet_toxicity: definition.starting_planet_toxicity,
        unit_designs: starting_designs,
        base_attributes: attributes,
        social_engineering: crate::model::SocialEngineering::default(),
        personality: crate::model::FactionPersonality {
            aggression: definition.personality.aggression,
            tech_bias: definition.personality.tech_bias,
            diplomatic_tone: definition.personality.diplomatic_tone,
            expansion_bias: definition.personality.expansion_bias,
        },
    }))
}

pub fn unit_design_cost(design: &crate::UnitDesign) -> i32 {
    let mut cost = match design.chassis {
        crate::Chassis::Infantry => 10,
        crate::Chassis::Speeder => 20,
        crate::Chassis::Hovertank => 30,
        crate::Chassis::Aircraft => 40,
        crate::Chassis::Sea => 20,
    };
    cost += match design.weapon {
        crate::Weapon::HandLaser(v) => v as i32 * 2,
        crate::Weapon::ResonanceLaser(v) => v as i32 * 4,
        crate::Weapon::PlasmaBolt(v) => v as i32 * 8,
        crate::Weapon::PlanetBuster(_) => 50,
    };
    cost += match design.armor {
        crate::Armor::SynthMetal(v) => v as i32 * 2,
        crate::Armor::ResonanceArmor(v) => v as i32 * 4,
        crate::Armor::PlasmaSteel(v) => v as i32 * 4,
        crate::Armor::MonolithArmor(v) => v as i32 * 8,
    };
    cost + (design.abilities.len() as i32 * 5)
}

pub fn production_cost(item: crate::ProductionItem) -> i32 {
    if let crate::ProductionItem::CustomUnit(_design_index) = item {
        // We'd need to look up the actual design if we only have an index.
        // For now return a base cost if it's just an index.
        return 20;
    }
    production_definition(item).cost
}

pub fn production_name(item: crate::ProductionItem) -> &'static str {
    if let crate::ProductionItem::CustomUnit(_) = item {
        return "Custom Unit";
    }
    production_definition(item).name.as_str()
}

pub fn production_unit_kind(item: crate::ProductionItem) -> Option<UnitKind> {
    if let crate::ProductionItem::CustomUnit(_) = item {
        return None;
    }
    production_definition(item)
        .unit_kind
        .as_deref()
        .and_then(UnitKind::from_content_id)
}

pub fn production_facility(item: crate::ProductionItem) -> Option<Facility> {
    if let crate::ProductionItem::CustomUnit(_) = item {
        return None;
    }
    production_definition(item)
        .facility_id
        .as_deref()
        .and_then(Facility::from_content_id)
}

pub fn required_tech_for_production(item: crate::ProductionItem) -> Option<Tech> {
    if let crate::ProductionItem::CustomUnit(_) = item {
        return None;
    }
    let production_id = item.content_id();
    canonical_technologies()
        .expect("bundled technology tree content must parse for runtime access")
        .iter()
        .find_map(|technology| {
            let enabled = technology
                .enables
                .units
                .iter()
                .any(|unit| unit == production_id)
                || technology
                    .enables
                    .facilities
                    .iter()
                    .any(|facility| facility == production_id)
                || technology
                    .enables
                    .secret_projects
                    .iter()
                    .any(|project| project == production_id)
                || technology
                    .enables
                    .orbital
                    .iter()
                    .any(|orbital| orbital == production_id);
            if enabled {
                Tech::from_content_id(&technology.id)
            } else {
                None
            }
        })
}

pub fn tech_prerequisites(tech: Tech) -> Vec<Tech> {
    tech_definition(tech)
        .prerequisites
        .iter()
        .filter_map(|id| Tech::from_content_id(id))
        .collect()
}

pub fn tech_is_available(known_techs: &[Tech], tech: Tech) -> bool {
    if known_techs.contains(&tech) {
        return false;
    }
    tech_prerequisites(tech)
        .into_iter()
        .all(|prereq| known_techs.contains(&prereq))
}

pub fn tech_enabled_weapon_ids(tech: Tech) -> Vec<String> {
    tech_definition(tech).enables.weapons.clone()
}

pub fn tech_enabled_armor_ids(tech: Tech) -> Vec<String> {
    tech_definition(tech).enables.armor.clone()
}

pub fn weapon_power_from_id(id: &str) -> u8 {
    match id {
        "hand_laser" => 1,
        "resonance_laser" => 4,
        "plasma_bolt" => 8,
        "planet_buster" => 20,
        _ => 0,
    }
}

pub fn armor_power_from_id(id: &str) -> u8 {
    match id {
        "synth_metal" => 1,
        "resonance_armor" => 2,
        "plasma_steel" => 4,
        "monolith_armor" => 8,
        _ => 0,
    }
}

pub fn tech_enabled_unit_names(tech: Tech) -> Vec<String> {
    tech_definition(tech)
        .enables
        .units
        .iter()
        .map(|id| {
            UnitKind::from_content_id(id)
                .map(unit_name)
                .unwrap_or(id.as_str())
                .to_string()
        })
        .collect()
}

pub fn tech_enabled_facility_names(tech: Tech) -> Vec<String> {
    tech_definition(tech)
        .enables
        .facilities
        .iter()
        .map(|id| {
            Facility::from_content_id(id)
                .map(facility_name)
                .unwrap_or(id.as_str())
                .to_string()
        })
        .collect()
}

pub fn facility_name(facility: Facility) -> &'static str {
    facility_definition(facility).name.as_str()
}

pub fn facility_maintenance(facility: Facility) -> i32 {
    facility_definition(facility).maintenance
}

pub fn facility_defense_bonus(facility: Facility) -> i32 {
    facility_definition(facility).defense_bonus
}

pub fn facility_stability_bonus(facility: Facility) -> i32 {
    facility_definition(facility).stability_bonus
}

pub fn facility_repair_bonus(facility: Facility) -> i32 {
    facility_definition(facility).repair_bonus
}

pub fn facility_training_bonus(facility: Facility) -> i32 {
    facility_definition(facility).training_bonus
}

pub fn facility_growth_threshold_reduction(facility: Facility) -> i32 {
    facility_definition(facility).growth_threshold_reduction
}

pub fn facility_free_unit_support_bonus(facility: Facility) -> i32 {
    facility_definition(facility).free_unit_support_bonus
}

pub fn facility_mobility_bonus(facility: Facility) -> i32 {
    facility_definition(facility).mobility_bonus
}

pub fn facility_psi_support_bonus(facility: Facility) -> i32 {
    facility_definition(facility).psi_support_bonus
}

pub fn facility_convoy_capacity_bonus(facility: Facility) -> i32 {
    facility_definition(facility).convoy_capacity_bonus
}

pub fn facility_convoy_security_bonus(facility: Facility) -> i32 {
    facility_definition(facility).convoy_security_bonus
}

pub fn facility_yield_bonus(facility: Facility) -> Yields {
    let bonus = facility_definition(facility).yield_bonus;
    Yields {
        nutrients: bonus.nutrients,
        minerals: bonus.minerals,
        energy: bonus.energy,
    }
}

fn production_definition(item: crate::ProductionItem) -> &'static ProductionDefinition {
    try_production_definition(item)
        .expect("bundled production content missing a required production item")
}

pub fn tech_name(tech: Tech) -> &'static str {
    tech_definition(tech).name.as_str()
}

pub fn tech_description(tech: Tech) -> &'static str {
    tech_definition(tech).description.as_str()
}

pub fn tech_cost(tech: Tech) -> i32 {
    tech_definition(tech).cost
}

pub fn base_growth_nutrients_threshold() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .base_growth_nutrients_threshold
}

pub fn supply_pod_energy_reward() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .supply_pod_energy_reward
}

pub fn supply_pod_salvage_energy_reward() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .supply_pod_salvage_energy_reward
}

pub fn native_spawn_turn_interval() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .native_spawn_turn_interval
}

pub fn native_spawn_roll_threshold() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .native_spawn_roll_threshold
}

pub fn ai_colony_base_target() -> usize {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .ai_colony_base_target
}

pub fn ai_native_spawn_noise_salt() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .ai_native_spawn_noise_salt
}

pub fn player_unit_visibility_radius() -> isize {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .player_unit_visibility_radius
}

pub fn player_base_visibility_radius() -> isize {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .player_base_visibility_radius
}

pub fn map_ocean_threshold() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .map_ocean_threshold
}

pub fn map_fungus_threshold() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .map_fungus_threshold
}

pub fn map_rocky_threshold() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .map_rocky_threshold
}

pub fn map_flat_moisture_threshold() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .map_flat_moisture_threshold
}

pub fn map_pod_spawn_threshold() -> u32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .map_pod_spawn_threshold
}

pub fn forced_land_patch_radius() -> isize {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .forced_land_patch_radius
}

pub fn base_starting_population() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .base_starting_population
}

pub fn base_starting_nutrients_stock() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .base_starting_nutrients_stock
}

pub fn base_starting_minerals_stock() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .base_starting_minerals_stock
}

pub fn free_unit_support_per_base() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .free_unit_support_per_base
}

pub fn unit_support_cost() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .unit_support_cost
}

pub fn rolling_defense_bonus() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .rolling_defense_bonus
}

pub fn rocky_defense_bonus() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .rocky_defense_bonus
}

pub fn fungus_defense_bonus() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .fungus_defense_bonus
}

pub fn base_defense_bonus() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .base_defense_bonus
}

pub fn base_attack_penalty() -> i32 {
    runtime_rules()
        .expect("bundled runtime rules must parse for runtime access")
        .base_attack_penalty
}

pub fn unit_max_moves(kind: UnitKind) -> i32 {
    match kind {
        UnitKind::CustomUnit(design) => match design.chassis {
            crate::Chassis::Infantry => 1,
            crate::Chassis::Speeder => 2,
            crate::Chassis::Hovertank => 1,
            crate::Chassis::Aircraft => 8,
            crate::Chassis::Sea => 1,
        },
        _ => unit_definition(kind.content_id()).max_moves,
    }
}

pub fn unit_attack(kind: UnitKind) -> i32 {
    match kind {
        UnitKind::CustomUnit(design) => design.attack_strength() as i32,
        _ => unit_definition(kind.content_id()).attack,
    }
}

pub fn unit_defense(kind: UnitKind) -> i32 {
    match kind {
        UnitKind::CustomUnit(design) => design.defense_strength() as i32,
        _ => unit_definition(kind.content_id()).defense,
    }
}

pub fn unit_base_chassis(kind: UnitKind) -> crate::Chassis {
    match kind {
        UnitKind::ColonyPod => crate::Chassis::Infantry,
        UnitKind::SeaColonyPod => crate::Chassis::Sea,
        UnitKind::ScoutPatrol => crate::Chassis::Infantry,
        UnitKind::Former => crate::Chassis::Infantry,
        UnitKind::Speeder => crate::Chassis::Speeder,
        UnitKind::ResonanceLaser => crate::Chassis::Speeder,
        UnitKind::EscortSpeeder => crate::Chassis::Speeder,
        UnitKind::RaiderSpeeder => crate::Chassis::Speeder,
        UnitKind::TranceScout => crate::Chassis::Infantry,
        UnitKind::GarrisonGuard => crate::Chassis::Infantry,
        UnitKind::PsiSentinel => crate::Chassis::Infantry,
        UnitKind::MindWorm => crate::Chassis::Infantry,
        UnitKind::IsleOfTheDeep => crate::Chassis::Sea,
        UnitKind::Needlejet => crate::Chassis::Aircraft,
        UnitKind::ProbeTeam => crate::Chassis::Infantry,
        UnitKind::SeaTransport => crate::Chassis::Sea,
        UnitKind::CustomUnit(design) => design.chassis,
    }
}

pub fn unit_base_weapon(kind: UnitKind) -> crate::Weapon {
    match kind {
        UnitKind::ResonanceLaser => crate::Weapon::ResonanceLaser(1),
        UnitKind::CustomUnit(design) => design.weapon,
        _ => crate::Weapon::HandLaser(1),
    }
}

pub fn unit_base_armor(kind: UnitKind) -> crate::Armor {
    match kind {
        UnitKind::CustomUnit(design) => design.armor,
        _ => crate::Armor::SynthMetal(1),
    }
}

pub fn unit_base_hp(kind: UnitKind) -> i32 {
    match kind {
        UnitKind::CustomUnit(_) => 10,
        _ => unit_definition(kind.content_id()).base_hp,
    }
}

pub fn unit_name(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::CustomUnit(_) => "Custom Unit",
        _ => unit_definition(kind.content_id()).name.as_str(),
    }
}

pub fn unit_definition_by_id(id: &str) -> &'static UnitDefinition {
    unit_definition(id)
}

pub fn try_unit_definition_by_id(id: &str) -> Result<&'static UnitDefinition, ContentLookupError> {
    try_unit_definition(id)
}

pub fn try_default_runtime_faction_setups(
) -> Result<Vec<StartingFactionSetup>, RuntimeFactionSetupError> {
    let mut setups = faction_definitions()
        .expect("bundled faction content must parse for runtime setup validation")
        .iter()
        .filter_map(|definition| try_runtime_faction_setup(definition).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    setups.sort_by_key(|setup| setup.runtime_id);
    Ok(setups)
}

pub fn loc_str(key: &str) -> String {
    localization_data()
        .map(|d| {
            d.strings
                .get(key)
                .cloned()
                .unwrap_or_else(|| key.to_string())
        })
        .unwrap_or_else(|_| key.to_string())
}

pub fn ui_window_title() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .window_title
        .as_str()
}

pub fn ui_app_title() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .app_title
        .as_str()
}

pub fn ui_command_console_heading() -> String {
    loc_str("ui_command_console_heading")
}

pub fn ui_selection_heading() -> String {
    loc_str("ui_selection_heading")
}

pub fn ui_research_heading() -> String {
    loc_str("ui_research_heading")
}

pub fn ui_factions_heading() -> String {
    loc_str("ui_factions_heading")
}

pub fn ui_event_log_heading() -> String {
    loc_str("ui_event_log_heading")
}

pub fn ui_planet_heading() -> String {
    loc_str("ui_planet_heading")
}

pub fn ui_victory_text() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .victory_text
        .as_str()
}

pub fn ui_defeat_text() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .defeat_text
        .as_str()
}

pub fn ui_warning_text() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .warning_text
        .as_str()
}

pub fn ui_accent_hex() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .accent_hex
        .as_str()
}

pub fn ui_warning_hex() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .warning_hex
        .as_str()
}

pub fn ui_danger_hex() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .danger_hex
        .as_str()
}

pub fn ui_success_hex() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .success_hex
        .as_str()
}

pub fn ui_panel_fill_hex() -> &'static str {
    ui_theme()
        .expect("bundled ui theme must parse for runtime access")
        .panel_fill_hex
        .as_str()
}

fn tech_definition(tech: Tech) -> &'static TechRuntimeDefinition {
    try_tech_definition(tech)
        .expect("bundled runtime tech content missing a required tech definition")
}

static ASSET_PACK: once_cell::sync::OnceCell<crate::assets::AssetPack> =
    once_cell::sync::OnceCell::new();

// Reserved for the packed-asset bootstrap path; runtime still defaults to loose data files today.
#[allow(dead_code)]
pub fn init_asset_pack(data: Vec<u8>) -> Result<(), String> {
    let pack = crate::assets::AssetPack::from_bytes(&data)
        .map_err(|e| format!("Failed to parse asset pack: {}", e))?;
    ASSET_PACK
        .set(pack)
        .map_err(|_| "Asset pack already initialized.".to_string())
}

fn load_raw_content(name: &str, bundled: &'static str) -> String {
    if let Some(pack) = ASSET_PACK.get() {
        if let Some(data) = pack.get_file(name) {
            return String::from_utf8_lossy(data).to_string();
        }
    }
    bundled.to_string()
}

fn parse_bundled_json<T: DeserializeOwned>(label: &str, raw: &str) -> Result<T, String> {
    serde_json::from_str(raw).map_err(|error| format!("{label}: {error}"))
}

fn faction_definitions() -> Result<&'static Vec<FactionDefinition>, String> {
    FACTION_DEFINITIONS
        .get_or_init(|| {
            let raw = load_raw_content("factions.json", FACTIONS_JSON);
            parse_bundled_json(
                "bundled faction content must be valid JSON for the core game",
                &raw,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn production_definitions() -> Result<&'static Vec<ProductionDefinition>, String> {
    PRODUCTION_DEFINITIONS
        .get_or_init(|| {
            parse_bundled_json(
                "bundled production content must be valid JSON for the core game",
                PRODUCTION_JSON,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn try_production_definition(
    item: crate::ProductionItem,
) -> Result<&'static ProductionDefinition, ContentLookupError> {
    production_definitions()
        .expect("bundled production content must parse before definition lookup")
        .iter()
        .find(|definition| definition.id == item.content_id())
        .ok_or_else(|| {
            ContentLookupError::MissingProductionDefinition(item.content_id().to_string())
        })
}

fn try_tech_definition(tech: Tech) -> Result<&'static TechRuntimeDefinition, ContentLookupError> {
    canonical_technologies()
        .expect("bundled technology tree content must parse for runtime access")
        .iter()
        .find(|definition| definition.id == tech.content_id())
        .ok_or_else(|| {
            ContentLookupError::MissingTechnologyDefinition(tech.content_id().to_string())
        })
}

fn validate_runtime_roles(errors: &mut Vec<String>) {
    let roles = runtime_roles();
    let runtime_role_names = faction_definitions()
        .expect("bundled faction content must parse for validation")
        .iter()
        .filter_map(|definition| definition.runtime_role.as_deref())
        .collect::<BTreeSet<_>>();

    for required_role in [
        RuntimeRole::Player.as_str(),
        RuntimeRole::Ai.as_str(),
        RuntimeRole::Native.as_str(),
    ] {
        if !runtime_role_names.contains(required_role) {
            errors.push(format!(
                "missing runtime faction definition for role '{required_role}'"
            ));
        }
    }

    for owner in [roles.player, roles.ai, roles.native] {
        if runtime_role_for_owner(owner).is_none() {
            errors.push(format!("runtime owner id {owner} has no mapped role"));
        }
    }
}

fn validate_factions(errors: &mut Vec<String>) {
    let mut faction_names = BTreeSet::new();
    let technology_ids = canonical_technologies()
        .expect("bundled technology tree content must parse for validation")
        .iter()
        .map(|technology| technology.id.as_str())
        .collect::<BTreeSet<_>>();
    let production_ids = production_definitions()
        .expect("bundled production content must parse for validation")
        .iter()
        .map(|definition| definition.id.as_str())
        .collect::<BTreeSet<_>>();
    let unit_ids = unit_definitions()
        .expect("bundled unit content must parse for validation")
        .iter()
        .map(|definition| definition.id.as_str())
        .collect::<BTreeSet<_>>();

    for faction in faction_definitions().expect("bundled faction content must parse for validation")
    {
        if !faction_names.insert(faction.name.as_str()) {
            errors.push(format!("duplicate faction name '{}'", faction.name));
        }

        if let Some(role) = faction.runtime_role.as_deref() {
            if RuntimeRole::from_str(role).is_none() {
                errors.push(format!(
                    "faction '{}' uses unknown runtime role '{}'",
                    faction.name, role
                ));
            }
            if let Err(error) = try_runtime_faction_setup(faction) {
                errors.push(error.to_string());
            }
        }

        for tech_id in &faction.known_tech_ids {
            if !technology_ids.contains(tech_id.as_str()) {
                errors.push(format!(
                    "faction '{}' references unknown known_tech id '{}'",
                    faction.name, tech_id
                ));
            }
        }

        if let Some(tech_id) = faction.current_research_id.as_deref() {
            if !technology_ids.contains(tech_id) {
                errors.push(format!(
                    "faction '{}' references unknown current_research_id '{}'",
                    faction.name, tech_id
                ));
            }
        }

        if let Some(policy) = faction.ai_policy.as_ref() {
            let production_id = policy.preferred_production.as_str();
            if !production_ids.contains(production_id) && !unit_ids.contains(production_id) {
                errors.push(format!(
                    "faction '{}' AI policy references unknown production or unit '{}'",
                    faction.name, policy.preferred_production
                ));
            }
        }

        if faction.base_names.is_empty() {
            errors.push(format!(
                "faction '{}' must define at least one base name",
                faction.name
            ));
        }
    }
}

fn validate_technologies(errors: &mut Vec<String>) {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let supported_techs = crate::Tech::all()
        .into_iter()
        .map(|tech| (tech.content_id(), tech))
        .collect::<Vec<_>>();
    let technology_ids = canonical_technologies()
        .expect("bundled technology tree content must parse for validation")
        .iter()
        .map(|technology| technology.id.as_str())
        .collect::<BTreeSet<_>>();

    for technology in
        canonical_technologies().expect("bundled technology tree content must parse for validation")
    {
        if !ids.insert(technology.id.as_str()) {
            errors.push(format!("duplicate technology id '{}'", technology.id));
        }
        if !names.insert(technology.name.as_str()) {
            errors.push(format!("duplicate technology name '{}'", technology.name));
        }
        for prereq in &technology.prerequisites {
            if !technology_ids.contains(prereq.as_str()) {
                errors.push(format!(
                    "technology '{}' has unknown prerequisite '{}'",
                    technology.id, prereq
                ));
            }
        }

        // Validate 'enables'
        for unit_id in &technology.enables.units {
            if UnitKind::from_content_id(unit_id).is_none() {
                errors.push(format!(
                    "technology '{}' enables unknown unit '{}'",
                    technology.id, unit_id
                ));
            }
        }
        for facility_id in &technology.enables.facilities {
            if Facility::from_content_id(facility_id).is_none() {
                errors.push(format!(
                    "technology '{}' enables unknown facility '{}'",
                    technology.id, facility_id
                ));
            }
        }
        for project_id in &technology.enables.secret_projects {
            if SecretProject::from_content_id(project_id).is_none() {
                errors.push(format!(
                    "technology '{}' enables unknown secret project '{}'",
                    technology.id, project_id
                ));
            }
        }
        for orbital_id in &technology.enables.orbital {
            if crate::model::ProductionItem::from_content_id(orbital_id).is_none() {
                errors.push(format!(
                    "technology '{}' enables unknown orbital item '{}'",
                    technology.id, orbital_id
                ));
            }
        }
        for weapon_id in &technology.enables.weapons {
            if !is_valid_weapon_id(weapon_id) {
                errors.push(format!(
                    "technology '{}' enables unknown weapon '{}'",
                    technology.id, weapon_id
                ));
            }
        }
        for armor_id in &technology.enables.armor {
            if !is_valid_armor_id(armor_id) {
                errors.push(format!(
                    "technology '{}' enables unknown armor '{}'",
                    technology.id, armor_id
                ));
            }
        }
        for ability_id in &technology.enables.abilities {
            if !Ability::all()
                .iter()
                .any(|a| format!("{:?}", a).to_lowercase() == ability_id.to_lowercase())
            {
                // Note: this check might be brittle if Ability names change
            }
        }
        for terraform_id in &technology.enables.terraforming {
            if !Improvement::all()
                .iter()
                .any(|i| i.content_id().to_lowercase() == terraform_id.to_lowercase())
            {
                errors.push(format!(
                    "technology '{}' enables unknown terraforming '{}'",
                    technology.id, terraform_id
                ));
            }
        }
    }
    for (content_id, tech) in supported_techs {
        let Some(definition) = canonical_technologies()
            .expect("bundled technology tree content must parse for validation")
            .iter()
            .find(|technology| technology.id == content_id)
        else {
            errors.push(format!(
                "missing technology definition for '{}'",
                content_id
            ));
            continue;
        };

        if Tech::from_content_id(content_id) != Some(tech) {
            errors.push(format!(
                "technology '{}' is not mapped correctly in Tech::from_content_id",
                content_id
            ));
        }
        if Tech::from_content_name(&definition.name) != Some(tech) {
            errors.push(format!(
                "technology '{}' is not mapped correctly in Tech::from_content_name",
                definition.name
            ));
        }
    }

    validate_technology_cycles(errors);
}

fn validate_technology_cycles(errors: &mut Vec<String>) {
    let techs = match canonical_technologies() {
        Ok(t) => t,
        Err(_) => return,
    };

    let mut adj = std::collections::HashMap::new();
    for t in techs {
        adj.insert(&t.id, &t.prerequisites);
    }

    for tech_id in adj.keys() {
        let mut visited = std::collections::HashSet::new();
        let mut stack = std::collections::HashSet::new();
        if has_cycle(tech_id, &adj, &mut visited, &mut stack) {
            errors.push(format!(
                "circular dependency detected starting at technology '{}'",
                tech_id
            ));
            break;
        }
    }
}

fn has_cycle(
    u: &String,
    adj: &std::collections::HashMap<&String, &Vec<String>>,
    visited: &mut std::collections::HashSet<String>,
    stack: &mut std::collections::HashSet<String>,
) -> bool {
    visited.insert(u.clone());
    stack.insert(u.clone());

    if let Some(neighbors) = adj.get(u) {
        for v in *neighbors {
            if !visited.contains(v) {
                if has_cycle(v, adj, visited, stack) {
                    return true;
                }
            } else if stack.contains(v) {
                return true;
            }
        }
    }

    stack.remove(u);
    false
}

fn validate_units(errors: &mut Vec<String>) {
    let mut ids = BTreeSet::new();

    for unit in unit_definitions().expect("bundled unit content must parse for validation") {
        if !ids.insert(unit.id.as_str()) {
            errors.push(format!("duplicate unit id '{}'", unit.id));
        }
        if let Err(error) = design_from_definition(&unit.id, unit) {
            errors.push(error.to_string());
        }
        if unit.attack < 0 || unit.defense < 0 || unit.max_moves < 0 || unit.base_hp <= 0 {
            errors.push(format!(
                "unit '{}' has invalid stats (attack: {}, defense: {}, moves: {}, hp: {})",
                unit.id, unit.attack, unit.defense, unit.max_moves, unit.base_hp
            ));
        }
    }

    for unit in UnitKind::all() {
        if unit_definitions()
            .expect("bundled unit content must parse for validation")
            .iter()
            .all(|definition| definition.id != unit.clone().content_id())
        {
            errors.push(format!(
                "missing unit definition for '{}'",
                unit.clone().content_id()
            ));
        }
    }
}

fn validate_facilities(errors: &mut Vec<String>) {
    let mut ids = BTreeSet::new();

    for facility in
        facility_definitions().expect("bundled facility content must parse for validation")
    {
        if !ids.insert(facility.id.as_str()) {
            errors.push(format!("duplicate facility id '{}'", facility.id));
        }
        if Facility::from_content_id(&facility.id).is_none() {
            errors.push(format!(
                "facility '{}' is not mapped in Facility::from_content_id",
                facility.id
            ));
        }
        if facility.maintenance < 0 {
            errors.push(format!(
                "facility '{}' has negative maintenance: {}",
                facility.id, facility.maintenance
            ));
        }
    }

    for facility in crate::Facility::all() {
        if facility_definitions()
            .expect("bundled facility content must parse for validation")
            .iter()
            .all(|definition| definition.id != facility.content_id())
        {
            errors.push(format!(
                "missing facility definition for '{}'",
                facility.content_id()
            ));
        }
    }
}

fn validate_production(errors: &mut Vec<String>) {
    let mut ids = BTreeSet::new();

    for definition in
        production_definitions().expect("bundled production content must parse for validation")
    {
        if !ids.insert(definition.id.as_str()) {
            errors.push(format!("duplicate production id '{}'", definition.id));
        }

        if definition.cost < 0 {
            errors.push(format!(
                "production '{}' has negative cost: {}",
                definition.id, definition.cost
            ));
        }

        let Some(item) = crate::ProductionItem::from_content_id(&definition.id) else {
            errors.push(format!(
                "production '{}' is not mapped in ProductionItem::from_content_id",
                definition.id
            ));
            continue;
        };

        match definition.build_kind.as_str() {
            "unit" => {
                let unit_id = definition.unit_kind.as_deref();
                if unit_id.is_none() {
                    errors.push(format!(
                        "unit production '{}' is missing unit_kind",
                        definition.id
                    ));
                }
                if definition.facility_id.is_some() {
                    errors.push(format!(
                        "unit production '{}' should not declare facility_id",
                        definition.id
                    ));
                }
                if let Some(unit_id) = unit_id {
                    if UnitKind::from_content_id(unit_id).is_none() {
                        errors.push(format!(
                            "production '{}' references unknown unit '{}'",
                            definition.id, unit_id
                        ));
                    }
                    if production_unit_kind(item).map(|kind| kind.content_id()) != Some(unit_id) {
                        errors.push(format!(
                            "production '{}' unit mapping does not match ProductionItem",
                            definition.id
                        ));
                    }
                }
            }
            "special" => {
                // No additional validation needed for now
            }
            "facility" => {
                let facility_id = definition.facility_id.as_deref();
                if facility_id.is_none() {
                    errors.push(format!(
                        "facility production '{}' is missing facility_id",
                        definition.id
                    ));
                }
                if definition.unit_kind.is_some() {
                    errors.push(format!(
                        "facility production '{}' should not declare unit_kind",
                        definition.id
                    ));
                }
                if let Some(facility_id) = facility_id {
                    if Facility::from_content_id(facility_id).is_none() {
                        errors.push(format!(
                            "production '{}' references unknown facility '{}'",
                            definition.id, facility_id
                        ));
                    }
                    if item.facility().map(|facility| facility.content_id()) != Some(facility_id) {
                        errors.push(format!(
                            "production '{}' facility mapping does not match ProductionItem",
                            definition.id
                        ));
                    }
                }
            }
            "project" => {
                if item.secret_project().is_none() {
                    errors.push(format!(
                        "production '{}' is marked as project but is not mapped in ProductionItem::secret_project",
                        definition.id
                    ));
                }
            }
            other => errors.push(format!(
                "production '{}' uses unknown build_kind '{}'",
                definition.id, other
            )),
        }
    }

    for item in crate::ProductionItem::all() {
        if production_definitions()
            .expect("bundled production content must parse for validation")
            .iter()
            .all(|definition| definition.id != item.content_id())
        {
            errors.push(format!(
                "missing production definition for '{}'",
                item.content_id()
            ));
        }
    }
}

fn validate_runtime_rules(errors: &mut Vec<String>) {
    let rules = runtime_rules().expect("bundled runtime rules must parse for validation");

    if rules.base_growth_nutrients_threshold <= 0 {
        errors.push("base_growth_nutrients_threshold must be positive".to_string());
    }
    if rules.supply_pod_energy_reward <= 0 {
        errors.push("supply_pod_energy_reward must be positive".to_string());
    }
    if rules.native_spawn_turn_interval <= 0 {
        errors.push("native_spawn_turn_interval must be positive".to_string());
    }
    if rules.map_ocean_threshold > 100
        || rules.map_fungus_threshold > 100
        || rules.map_rocky_threshold > 100
        || rules.map_flat_moisture_threshold > 100
        || rules.map_pod_spawn_threshold > 100
    {
        errors.push("map thresholds must be between 0 and 100".to_string());
    }
}

fn validate_ui_theme(errors: &mut Vec<String>) {
    let theme = ui_theme().expect("bundled ui theme must parse for validation");
    let required_fields = [
        ("window_title", theme.window_title.as_str()),
        ("app_title", theme.app_title.as_str()),
        (
            "command_console_heading",
            theme.command_console_heading.as_str(),
        ),
        ("selection_heading", theme.selection_heading.as_str()),
        ("research_heading", theme.research_heading.as_str()),
        ("factions_heading", theme.factions_heading.as_str()),
        ("event_log_heading", theme.event_log_heading.as_str()),
        ("planet_heading", theme.planet_heading.as_str()),
        ("victory_text", theme.victory_text.as_str()),
        ("defeat_text", theme.defeat_text.as_str()),
        ("warning_text", theme.warning_text.as_str()),
    ];

    for (field, value) in required_fields {
        if value.trim().is_empty() {
            errors.push(format!("ui theme field '{field}' must not be empty"));
        }
    }

    for (field, value) in [
        ("accent_hex", theme.accent_hex.as_str()),
        ("warning_hex", theme.warning_hex.as_str()),
        ("danger_hex", theme.danger_hex.as_str()),
        ("success_hex", theme.success_hex.as_str()),
        ("panel_fill_hex", theme.panel_fill_hex.as_str()),
    ] {
        if !is_hex_color(value) {
            errors.push(format!(
                "ui theme field '{field}' must be a #RRGGBB color, got '{value}'"
            ));
        }
    }
}

fn validate_starting_scenario(errors: &mut Vec<String>) {
    let raw = load_raw_content("start_scenario.json", START_SCENARIO_JSON);
    let definition: StartingScenarioDefinition =
        serde_json::from_str(&raw).expect("bundled starting scenario must be valid JSON");
    if let Err(error) = try_default_starting_scenario(16, 16) {
        errors.push(error.to_string());
    }

    if definition.starting_units.is_empty() {
        errors.push("starting scenario must define at least one starting unit".to_string());
    }

    for (i, unit) in definition.starting_units.iter().enumerate() {
        if UnitKind::from_content_id(&unit.kind).is_none() {
            errors.push(format!(
                "starting scenario unit {} has unknown unit_kind '{}'",
                i, unit.kind
            ));
        }
    }
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value
            .as_bytes()
            .iter()
            .skip(1)
            .all(|byte| byte.is_ascii_hexdigit())
}

fn facility_definitions() -> Result<&'static Vec<FacilityDefinition>, String> {
    FACILITY_DEFINITIONS
        .get_or_init(|| {
            parse_bundled_json(
                "bundled facility content must be valid JSON for the core game",
                FACILITIES_JSON,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn runtime_rules() -> Result<&'static RuntimeRulesDefinition, String> {
    RUNTIME_RULES
        .get_or_init(|| {
            parse_bundled_json(
                "bundled runtime rules content must be valid JSON for the core game",
                RUNTIME_RULES_JSON,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn unit_definitions() -> Result<&'static Vec<UnitDefinition>, String> {
    UNIT_DEFINITIONS
        .get_or_init(|| {
            parse_bundled_json(
                "bundled unit definition content must be valid JSON for the core game",
                UNIT_DEFINITIONS_JSON,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn localization_data() -> Result<&'static crate::localization::LocalizationData, String> {
    static DATA: once_cell::sync::OnceCell<crate::localization::LocalizationData> =
        once_cell::sync::OnceCell::new();
    DATA.get_or_try_init(load_localization_data)
}

fn ui_theme() -> Result<&'static UiThemeDefinition, String> {
    UI_THEME
        .get_or_init(|| {
            parse_bundled_json(
                "bundled ui theme content must be valid JSON for the core game",
                UI_THEME_JSON,
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn unit_definition(id: &str) -> &'static UnitDefinition {
    try_unit_definition(id)
        .expect("bundled unit definition content missing a required unit definition")
}

fn try_unit_definition(id: &str) -> Result<&'static UnitDefinition, ContentLookupError> {
    unit_definitions()
        .expect("bundled unit definitions must parse before definition lookup")
        .iter()
        .find(|definition| definition.id == id)
        .ok_or_else(|| ContentLookupError::MissingUnitDefinition(id.to_string()))
}

fn facility_definition(facility: Facility) -> &'static FacilityDefinition {
    try_facility_definition(facility)
        .expect("bundled facility content missing a required facility definition")
}

fn try_facility_definition(
    facility: Facility,
) -> Result<&'static FacilityDefinition, ContentLookupError> {
    facility_definitions()
        .expect("bundled facility definitions must parse before definition lookup")
        .iter()
        .find(|definition| definition.id == facility.content_id())
        .ok_or_else(|| {
            ContentLookupError::MissingFacilityDefinition(facility.content_id().to_string())
        })
}

fn is_valid_weapon_id(id: &str) -> bool {
    matches!(
        id,
        "hand_laser" | "resonance_laser" | "plasma_bolt" | "planet_buster"
    )
}

fn is_valid_armor_id(id: &str) -> bool {
    matches!(
        id,
        "synth_metal" | "resonance_armor" | "plasma_steel" | "monolith_armor"
    )
}
