use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{SecretProject, Tech};
use crate::{Base, ConvoyRoute, ConvoyRouteKind, Faction, GameOver, GameState, Tile, Unit};

pub const GAME_STATE_SNAPSHOT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub version: u32,
    pub save_name: Option<String>,
    pub saved_turn: i32,
    #[serde(default)]
    pub recovery_notes: Vec<String>,
    pub width: usize,
    pub height: usize,
    pub seed: u32,
    pub turn: i32,
    #[serde(default)]
    pub dust_fall_turns_left: i32,
    #[serde(default)]
    pub tidal_chaos_turns_left: i32,
    pub tiles: Vec<Tile>,
    pub units: Vec<Unit>,
    pub bases: Vec<Base>,
    #[serde(default)]
    pub convoy_routes: Vec<ConvoyRoute>,
    pub factions: Vec<Faction>,
    #[serde(default)]
    pub relations: Vec<Vec<crate::model::DiplomaticRelation>>,
    #[serde(default)]
    pub built_secret_projects: Vec<(SecretProject, usize)>,
    pub log: Vec<crate::model::EventLogEntry>,
    #[serde(default)]
    pub pending_diplomacy_offers: Vec<(usize, usize, crate::model::DiplomacyStatus)>,
    #[serde(default)]
    pub pending_tech_trades: Vec<(usize, usize, Tech, Tech)>,
    #[serde(default)]
    pub pending_demands: Vec<(usize, usize, crate::model::DemandKind)>,
    #[serde(default)]
    pub triggered_narratives: BTreeSet<String>,
    #[serde(default)]
    pub council: crate::model::CouncilState,
    pub game_over: Option<GameOver>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotMetadata {
    pub save_name: String,
    pub saved_turn: i32,
    #[serde(default)]
    pub recovery_note_count: usize,
    #[serde(default)]
    pub last_updated_unix: Option<u64>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub category: Option<SaveSlotCategory>,
    #[serde(default)]
    pub auto_recovery_base_ids: Vec<usize>,
    #[serde(default)]
    pub auto_defense_base_ids: Vec<usize>,
    #[serde(default)]
    pub auto_economy_base_ids: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSlotListing {
    pub id: String,
    pub snapshot_path: PathBuf,
    pub metadata_path: PathBuf,
    pub metadata: Option<SaveSlotMetadata>,
    pub category: SaveSlotCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SaveSlotCategory {
    Autosave,
    Manual,
    Imported,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveSortColumn {
    FileId,
    Category,
    SaveName,
    Turn,
    Recovery,
    Updated,
}

impl SaveSortColumn {
    pub fn all() -> [Self; 6] {
        [
            Self::FileId,
            Self::Category,
            Self::SaveName,
            Self::Turn,
            Self::Recovery,
            Self::Updated,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveFilterCategory {
    All,
    Autosave,
    Manual,
    Imported,
    Empty,
}

impl SaveFilterCategory {
    pub fn all() -> [Self; 5] {
        [
            Self::All,
            Self::Autosave,
            Self::Manual,
            Self::Imported,
            Self::Empty,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBrowserQuery {
    pub filter_text: String,
    pub filter_category: SaveFilterCategory,
    pub recovered_only: bool,
    pub populated_only: bool,
    pub sort_column: SaveSortColumn,
    pub sort_descending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveBrowserCounts {
    pub total: usize,
    pub filtered: usize,
    pub autosave_count: usize,
    pub manual_count: usize,
    pub imported_count: usize,
    pub empty_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSortButtonState {
    pub column: SaveSortColumn,
    pub label_text: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBrowserRowState {
    pub id: String,
    pub selected: bool,
    pub category_text: String,
    pub display_name: String,
    pub turn_text: String,
    pub recovery_text: String,
    pub updated_text: String,
    pub notes_preview: String,
    pub file_path_text: String,
    pub delete_action_label: String,
    pub rename_action_label: String,
    pub duplicate_action_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBrowserGlobalActionState {
    pub label_text: String,
    pub category: Option<SaveSlotCategory>,
    pub action_type: SaveBrowserGlobalActionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveBrowserGlobalActionType {
    Rename,
    Duplicate,
    SaveAs,
    Delete,
    Import,
    Export,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveBrowserDisplayState {
    pub current_slot_label: String,
    pub counts_text: String,
    pub column_headers: Vec<&'static str>,
    pub global_actions: Vec<SaveBrowserGlobalActionState>,
    pub sort_buttons: Vec<SaveSortButtonState>,
    pub rows: Vec<SaveBrowserRowState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyGameStateSnapshotV0 {
    pub width: usize,
    pub height: usize,
    pub seed: u32,
    pub turn: i32,
    pub tiles: Vec<Tile>,
    pub units: Vec<Unit>,
    pub bases: Vec<Base>,
    #[serde(default)]
    pub convoy_routes: Vec<ConvoyRoute>,
    pub factions: Vec<Faction>,
    #[serde(default)]
    pub relations: Vec<Vec<crate::model::DiplomaticRelation>>,
    #[serde(default)]
    pub built_secret_projects: Vec<(SecretProject, usize)>,
    pub log: Vec<String>,
    #[serde(default)]
    pub pending_diplomacy_offers: Vec<(usize, usize, crate::model::DiplomacyStatus)>,
    pub game_over: Option<GameOver>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyGameStateSnapshotV1 {
    pub version: u32,
    pub width: usize,
    pub height: usize,
    pub seed: u32,
    pub turn: i32,
    pub tiles: Vec<Tile>,
    pub units: Vec<Unit>,
    pub bases: Vec<Base>,
    #[serde(default)]
    pub convoy_routes: Vec<ConvoyRoute>,
    pub factions: Vec<Faction>,
    #[serde(default)]
    pub relations: Vec<Vec<crate::model::DiplomaticRelation>>,
    #[serde(default)]
    pub built_secret_projects: Vec<(SecretProject, usize)>,
    pub log: Vec<String>,
    #[serde(default)]
    pub pending_diplomacy_offers: Vec<(usize, usize, crate::model::DiplomacyStatus)>,
    pub game_over: Option<GameOver>,
}

impl From<&GameState> for GameStateSnapshot {
    fn from(game: &GameState) -> Self {
        Self {
            version: GAME_STATE_SNAPSHOT_VERSION,
            save_name: None,
            saved_turn: game.turn,
            recovery_notes: Vec::new(),
            width: game.width,
            height: game.height,
            seed: game.seed,
            turn: game.turn,
            dust_fall_turns_left: game.dust_fall_turns_left,
            tidal_chaos_turns_left: game.tidal_chaos_turns_left,
            tiles: game.tiles.clone(),
            units: game.units.clone(),
            bases: game.bases.clone(),
            convoy_routes: game.convoy_routes.clone(),
            factions: game.factions.clone(),
            relations: game.relations.clone(),
            built_secret_projects: game.built_secret_projects.clone(),
            log: game.log.clone(),
            pending_diplomacy_offers: game.pending_diplomacy_offers.clone(),
            pending_tech_trades: game.pending_tech_trades.clone(),
            pending_demands: game.pending_demands.clone(),
            triggered_narratives: game.triggered_narratives.clone(),
            council: game.council.clone(),
            game_over: game.game_over,
        }
    }
}

impl GameStateSnapshot {
    pub fn into_game_state(self) -> GameState {
        GameState {
            width: self.width,
            height: self.height,
            seed: self.seed,
            turn: self.turn,
            dust_fall_turns_left: self.dust_fall_turns_left,
            tidal_chaos_turns_left: self.tidal_chaos_turns_left,
            tiles: self.tiles,
            units: self.units,
            bases: self.bases,
            convoy_routes: self.convoy_routes,
            factions: self.factions,
            relations: self.relations,
            built_secret_projects: self.built_secret_projects,
            log: self.log,
            pending_diplomacy_offers: self.pending_diplomacy_offers,
            pending_tech_trades: self.pending_tech_trades,
            pending_demands: self.pending_demands,
            triggered_narratives: self.triggered_narratives,
            council: self.council,
            game_over: self.game_over,
        }
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(value: &str) -> Result<Self, serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(value)?;

        let snapshot = match value.get("version").and_then(|version| version.as_u64()) {
            Some(version) => match version as u32 {
                GAME_STATE_SNAPSHOT_VERSION => {
                    let snapshot: Self = serde_json::from_value(value)?;
                    snapshot
                        .validate_version(version as u32)
                        .map_err(serde_json::Error::io)?;
                    snapshot
                }
                1 => {
                    let legacy: LegacyGameStateSnapshotV1 = serde_json::from_value(value)?;
                    Self {
                        version: GAME_STATE_SNAPSHOT_VERSION,
                        save_name: Some("Migrated Save".to_string()),
                        saved_turn: legacy.turn,
                        recovery_notes: vec!["Migrated snapshot from version 1.".to_string()],
                        width: legacy.width,
                        height: legacy.height,
                        seed: legacy.seed,
                        turn: legacy.turn,
                        dust_fall_turns_left: 0,
                        tidal_chaos_turns_left: 0,
                        tiles: legacy.tiles,
                        units: legacy.units,
                        bases: legacy.bases,
                        convoy_routes: legacy.convoy_routes,
                        factions: legacy.factions.clone(),
                        relations: if legacy.relations.is_empty() {
                            let count = legacy.factions.len();
                            vec![vec![crate::model::DiplomaticRelation::default(); count]; count]
                        } else {
                            legacy.relations
                        },
                        built_secret_projects: Vec::new(),
                        log: legacy
                            .log
                            .into_iter()
                            .map(|message| crate::model::EventLogEntry {
                                category: crate::model::EventCategory::General,
                                message,
                                turn: legacy.turn,
                            })
                            .collect(),
                        pending_diplomacy_offers: Vec::new(),
                        pending_tech_trades: Vec::new(),
                        pending_demands: Vec::new(),
                        triggered_narratives: BTreeSet::new(),
                        council: crate::model::CouncilState::default(),
                        game_over: legacy.game_over,
                    }
                }
                other => {
                    let snapshot: Self = serde_json::from_value(value)?;
                    snapshot
                        .validate_version(other)
                        .map_err(serde_json::Error::io)?;
                    snapshot
                }
            },
            None => {
                let legacy: LegacyGameStateSnapshotV0 = serde_json::from_value(value)?;
                Self {
                    version: GAME_STATE_SNAPSHOT_VERSION,
                    save_name: Some("Imported Legacy Save".to_string()),
                    saved_turn: legacy.turn,
                    recovery_notes: vec![
                        "Imported legacy snapshot with no version metadata.".to_string()
                    ],
                    width: legacy.width,
                    height: legacy.height,
                    seed: legacy.seed,
                    turn: legacy.turn,
                    dust_fall_turns_left: 0,
                    tidal_chaos_turns_left: 0,
                    tiles: legacy.tiles,
                    units: legacy.units,
                    bases: legacy.bases,
                    convoy_routes: legacy.convoy_routes,
                    factions: legacy.factions.clone(),
                    relations: if legacy.relations.is_empty() {
                        let count = legacy.factions.len();
                        vec![vec![crate::model::DiplomaticRelation::default(); count]; count]
                    } else {
                        legacy.relations
                    },
                    built_secret_projects: Vec::new(),
                    log: legacy
                        .log
                        .into_iter()
                        .map(|message| crate::model::EventLogEntry {
                            category: crate::model::EventCategory::General,
                            message,
                            turn: legacy.turn,
                        })
                        .collect(),
                    pending_diplomacy_offers: Vec::new(),
                    pending_tech_trades: Vec::new(),
                    pending_demands: Vec::new(),
                    triggered_narratives: BTreeSet::new(),
                    council: crate::model::CouncilState::default(),
                    game_over: legacy.game_over,
                }
            }
        }
        .repair_minor_inconsistencies();

        snapshot
            .validate_integrity()
            .map_err(serde_json::Error::io)?;
        Ok(snapshot)
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        let json = self
            .to_json_pretty()
            .map_err(|err| std::io::Error::other(format!("serialize snapshot: {err}")))?;
        fs::write(path, json)?;
        self.metadata().save_to_path(Self::metadata_path(path))?;
        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let json = fs::read_to_string(path)?;
        Self::from_json(&json)
            .map_err(|err| std::io::Error::other(format!("deserialize snapshot: {err}")))
    }

    pub fn metadata(&self) -> SaveSlotMetadata {
        SaveSlotMetadata {
            save_name: self
                .save_name
                .clone()
                .unwrap_or_else(|| "Unnamed Save".to_string()),
            saved_turn: self.saved_turn,
            recovery_note_count: self.recovery_notes.len(),
            last_updated_unix: None,
            notes: String::new(),
            category: Some(SaveSlotCategory::Manual),
            auto_recovery_base_ids: Vec::new(),
            auto_defense_base_ids: Vec::new(),
            auto_economy_base_ids: Vec::new(),
        }
    }

    fn metadata_path(path: &Path) -> PathBuf {
        let mut metadata_path = path.to_path_buf();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("{name}.meta"))
            .unwrap_or_else(|| "save.meta".to_string());
        metadata_path.set_file_name(file_name);
        metadata_path
    }

    fn validate_version(&self, version: u32) -> Result<(), std::io::Error> {
        if version == GAME_STATE_SNAPSHOT_VERSION {
            return Ok(());
        }

        Err(std::io::Error::other(format!(
            "unsupported snapshot version: {}",
            self.version
        )))
    }

    fn validate_integrity(&self) -> Result<(), std::io::Error> {
        let expected_tiles = self
            .width
            .checked_mul(self.height)
            .ok_or_else(|| std::io::Error::other("snapshot dimensions overflow tile capacity"))?;

        if self.tiles.len() != expected_tiles {
            return Err(std::io::Error::other(format!(
                "snapshot tile count mismatch: expected {expected_tiles}, found {}",
                self.tiles.len()
            )));
        }

        for (index, tile) in self.tiles.iter().enumerate() {
            let expected_x = index % self.width;
            let expected_y = index / self.width;
            if tile.x != expected_x || tile.y != expected_y {
                return Err(std::io::Error::other(format!(
                    "snapshot tile coordinate mismatch at index {index}: expected ({expected_x}, {expected_y}), found ({}, {})",
                    tile.x, tile.y
                )));
            }

            if let Some(unit_id) = tile.unit {
                let unit = self
                    .units
                    .iter()
                    .find(|unit| unit.id == unit_id && unit.alive)
                    .ok_or_else(|| {
                        std::io::Error::other(format!(
                            "tile ({}, {}) references missing live unit {unit_id}",
                            tile.x, tile.y
                        ))
                    })?;
                if unit.x != tile.x || unit.y != tile.y {
                    return Err(std::io::Error::other(format!(
                        "tile ({}, {}) references unit {unit_id} at ({}, {})",
                        tile.x, tile.y, unit.x, unit.y
                    )));
                }
            }

            if let Some(base_id) = tile.base {
                let base = self
                    .bases
                    .iter()
                    .find(|base| base.id == base_id)
                    .ok_or_else(|| {
                        std::io::Error::other(format!(
                            "tile ({}, {}) references missing base {base_id}",
                            tile.x, tile.y
                        ))
                    })?;
                if base.x != tile.x || base.y != tile.y {
                    return Err(std::io::Error::other(format!(
                        "tile ({}, {}) references base {base_id} at ({}, {})",
                        tile.x, tile.y, base.x, base.y
                    )));
                }
            }
        }

        for unit in &self.units {
            if unit.owner >= self.factions.len() {
                return Err(std::io::Error::other(format!(
                    "unit {} has invalid owner {}",
                    unit.id, unit.owner
                )));
            }
            if unit.x >= self.width || unit.y >= self.height {
                return Err(std::io::Error::other(format!(
                    "unit {} is out of bounds at ({}, {})",
                    unit.id, unit.x, unit.y
                )));
            }
            if unit.alive {
                let tile = &self.tiles[unit.y * self.width + unit.x];
                if tile.unit != Some(unit.id) {
                    return Err(std::io::Error::other(format!(
                        "live unit {} is not reciprocally linked from tile ({}, {})",
                        unit.id, unit.x, unit.y
                    )));
                }
            }
        }

        for base in &self.bases {
            if base.owner >= self.factions.len() {
                return Err(std::io::Error::other(format!(
                    "base {} has invalid owner {}",
                    base.id, base.owner
                )));
            }
            if base.x >= self.width || base.y >= self.height {
                return Err(std::io::Error::other(format!(
                    "base {} is out of bounds at ({}, {})",
                    base.id, base.x, base.y
                )));
            }
            let tile = &self.tiles[base.y * self.width + base.x];
            if tile.base != Some(base.id) {
                return Err(std::io::Error::other(format!(
                    "base {} is not reciprocally linked from tile ({}, {})",
                    base.id, base.x, base.y
                )));
            }
        }

        for route in &self.convoy_routes {
            let Some(base_a) = self.bases.iter().find(|base| base.id == route.base_a_id) else {
                return Err(std::io::Error::other(format!(
                    "convoy route references missing base {}",
                    route.base_a_id
                )));
            };
            let Some(base_b) = self.bases.iter().find(|base| base.id == route.base_b_id) else {
                return Err(std::io::Error::other(format!(
                    "convoy route references missing base {}",
                    route.base_b_id
                )));
            };
            if base_a.owner != base_b.owner {
                return Err(std::io::Error::other(format!(
                    "convoy route links bases with different owners: {} and {}",
                    route.base_a_id, route.base_b_id
                )));
            }
            if base_a.id == base_b.id {
                return Err(std::io::Error::other(format!(
                    "convoy route self-links base {}",
                    route.base_a_id
                )));
            }
            if base_a.x.abs_diff(base_b.x) + base_a.y.abs_diff(base_b.y) > 8 {
                return Err(std::io::Error::other(format!(
                    "convoy route between {} and {} exceeds range",
                    route.base_a_id, route.base_b_id
                )));
            }
        }

        for (index, faction) in self.factions.iter().enumerate() {
            if faction.id != index {
                return Err(std::io::Error::other(format!(
                    "faction index mismatch: expected id {index}, found {}",
                    faction.id
                )));
            }
        }

        Ok(())
    }

    fn repair_minor_inconsistencies(mut self) -> Self {
        let mut notes = std::mem::take(&mut self.recovery_notes);
        let expected_tiles = self.width.saturating_mul(self.height);
        if self.tiles.len() == expected_tiles {
            let mut expected_unit_links = vec![None; expected_tiles];
            let mut expected_base_links = vec![None; expected_tiles];

            for unit in self.units.iter().filter(|unit| unit.alive) {
                if unit.x < self.width && unit.y < self.height {
                    expected_unit_links[unit.y * self.width + unit.x] = Some(unit.id);
                }
            }

            for base in &self.bases {
                if base.x < self.width && base.y < self.height {
                    expected_base_links[base.y * self.width + base.x] = Some(base.id);
                }
            }

            let mut rebuilt_links = false;
            for (index, tile) in self.tiles.iter_mut().enumerate() {
                if tile.x != index % self.width || tile.y != index / self.width {
                    notes.push(
                        "Normalized tile coordinates from serialized index order.".to_string(),
                    );
                }
                tile.x = index % self.width;
                tile.y = index / self.width;

                if tile.unit != expected_unit_links[index]
                    || tile.base != expected_base_links[index]
                {
                    rebuilt_links = true;
                }
                tile.unit = expected_unit_links[index];
                tile.base = expected_base_links[index];
            }
            if rebuilt_links {
                notes.push(
                    "Rebuilt tile unit/base links from canonical entity positions.".to_string(),
                );
            }

            for (index, faction) in self.factions.iter_mut().enumerate() {
                if faction.id != index {
                    notes.push("Normalized faction ids to match runtime indices.".to_string());
                }
                faction.id = index;
            }
        }

        if self
            .save_name
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .is_empty()
        {
            notes.push("Filled missing save name with a recovered default.".to_string());
            self.save_name = Some("Recovered Save".to_string());
        }
        if self.saved_turn != self.turn {
            notes.push("Updated saved_turn metadata to match snapshot turn.".to_string());
        }
        let base_index: std::collections::BTreeMap<usize, (usize, usize, usize)> = self
            .bases
            .iter()
            .map(|base| (base.id, (base.owner, base.x, base.y)))
            .collect();
        let original_len = self.convoy_routes.len();
        self.convoy_routes.retain(|route| {
            let Some((owner_a, ax, ay)) = base_index.get(&route.base_a_id).copied() else {
                return false;
            };
            let Some((owner_b, bx, by)) = base_index.get(&route.base_b_id).copied() else {
                return false;
            };
            owner_a == owner_b
                && route.base_a_id != route.base_b_id
                && ax.abs_diff(bx) + ay.abs_diff(by) <= 8
        });
        self.convoy_routes.sort_by_key(|route| {
            (
                route.base_a_id.min(route.base_b_id),
                route.base_a_id.max(route.base_b_id),
                match route.kind {
                    ConvoyRouteKind::Trade => 0,
                    ConvoyRouteKind::Freight => 1,
                    ConvoyRouteKind::MilitarySupply => 2,
                },
            )
        });
        self.convoy_routes.dedup_by_key(|route| {
            (
                route.base_a_id.min(route.base_b_id),
                route.base_a_id.max(route.base_b_id),
                match route.kind {
                    ConvoyRouteKind::Trade => 0,
                    ConvoyRouteKind::Freight => 1,
                    ConvoyRouteKind::MilitarySupply => 2,
                },
            )
        });
        if self.convoy_routes.len() != original_len {
            notes.push("Removed invalid or duplicate convoy routes during recovery.".to_string());
        }
        self.saved_turn = self.turn;
        notes.sort();
        notes.dedup();
        self.recovery_notes = notes;
        self
    }
}

impl SaveSlotMetadata {
    pub fn save_path_for_id(base_dir: impl AsRef<Path>, id: &str) -> PathBuf {
        base_dir
            .as_ref()
            .join(format!("{}.json", normalize_save_id(id)))
    }

    pub fn metadata_path_for_save_path(save_path: &Path) -> PathBuf {
        GameStateSnapshot::metadata_path(save_path)
    }

    pub fn discover_slots(base_dir: impl AsRef<Path>) -> Vec<SaveSlotListing> {
        let base_dir = base_dir.as_ref();
        let mut ids: Vec<String> = fs::read_dir(base_dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| {
                name.strip_suffix(".json")
                    .or_else(|| name.strip_suffix(".json.meta"))
                    .map(|id| id.to_string())
            })
            .collect();

        ids.sort();
        ids.dedup();

        if ids.is_empty() {
            ids.push("slot_1".to_string());
        }

        let next_id = next_available_slot_id(&ids);
        if !ids.contains(&next_id) {
            ids.push(next_id);
        }

        let mut listings: Vec<SaveSlotListing> = ids
            .into_iter()
            .map(|id| {
                let snapshot_path = base_dir.join(format!("{id}.json"));
                let metadata_path = GameStateSnapshot::metadata_path(&snapshot_path);
                let metadata = Self::load_from_path(&snapshot_path).ok();
                SaveSlotListing {
                    category: SaveSlotListing::infer_category(&id, metadata.as_ref()),
                    id,
                    snapshot_path,
                    metadata_path,
                    metadata,
                }
            })
            .collect();
        listings.sort_by(|left, right| {
            let left_time = left
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.last_updated_unix)
                .unwrap_or(0);
            let right_time = right
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.last_updated_unix)
                .unwrap_or(0);
            right_time
                .cmp(&left_time)
                .then_with(|| left.id.cmp(&right.id))
        });
        listings
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let metadata_path = GameStateSnapshot::metadata_path(path);
        let metadata = match fs::read_to_string(&metadata_path) {
            Ok(json) => serde_json::from_str(&json)
                .map_err(|err| std::io::Error::other(format!("deserialize metadata: {err}")))?,
            Err(_) => {
                GameStateSnapshot::load_from_path(path).map(|snapshot| snapshot.metadata())?
            }
        };
        Ok(metadata.with_file_timestamp(path))
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let mut metadata = self.clone();
        if metadata.last_updated_unix.is_none() {
            metadata.last_updated_unix = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
        let json = serde_json::to_string_pretty(&metadata)
            .map_err(|err| std::io::Error::other(format!("serialize metadata: {err}")))?;
        fs::write(path, json)
    }

    fn with_file_timestamp(mut self, snapshot_path: &Path) -> Self {
        if self.last_updated_unix.is_none() {
            self.last_updated_unix = fs::metadata(snapshot_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());
        }
        self
    }
}

impl SaveSlotListing {
    pub fn display_name(&self) -> &str {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.save_name.as_str())
            .unwrap_or("Empty")
    }

    pub fn saved_turn_value(&self) -> i32 {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.saved_turn)
            .unwrap_or(-1)
    }

    pub fn recovery_note_count_value(&self) -> usize {
        self.metadata
            .as_ref()
            .map(|metadata| metadata.recovery_note_count)
            .unwrap_or(0)
    }

    pub fn last_updated_unix_value(&self) -> u64 {
        self.metadata
            .as_ref()
            .and_then(|metadata| metadata.last_updated_unix)
            .unwrap_or(0)
    }

    pub fn rename_to(&self, new_id: &str) -> Result<Self, std::io::Error> {
        self.rename_to_with_options(new_id, false)
    }

    pub fn rename_to_with_options(
        &self,
        new_id: &str,
        overwrite: bool,
    ) -> Result<Self, std::io::Error> {
        let normalized = normalize_save_id(new_id);
        if normalized.is_empty() {
            return Err(std::io::Error::other("save id cannot be empty"));
        }
        if normalized == self.id {
            return Ok(self.clone());
        }

        let parent = self
            .snapshot_path
            .parent()
            .ok_or_else(|| std::io::Error::other("save path has no parent directory"))?;
        let new_snapshot_path = parent.join(format!("{normalized}.json"));
        let new_metadata_path = GameStateSnapshot::metadata_path(&new_snapshot_path);

        if !overwrite && (new_snapshot_path.exists() || new_metadata_path.exists()) {
            return Err(std::io::Error::other("target save already exists"));
        }
        if overwrite {
            if new_snapshot_path.exists() {
                fs::remove_file(&new_snapshot_path)?;
            }
            if new_metadata_path.exists() {
                fs::remove_file(&new_metadata_path)?;
            }
        }

        if self.snapshot_path.exists() {
            fs::rename(&self.snapshot_path, &new_snapshot_path)?;
        }
        if self.metadata_path.exists() {
            fs::rename(&self.metadata_path, &new_metadata_path)?;
        }

        Ok(Self {
            category: SaveSlotListing::infer_category(
                &normalized,
                SaveSlotMetadata::load_from_path(&new_snapshot_path)
                    .ok()
                    .as_ref(),
            ),
            id: normalized.clone(),
            snapshot_path: new_snapshot_path.clone(),
            metadata_path: new_metadata_path,
            metadata: SaveSlotMetadata::load_from_path(&new_snapshot_path).ok(),
        })
    }

    pub fn delete(&self) -> Result<(), std::io::Error> {
        if self.snapshot_path.exists() {
            fs::remove_file(&self.snapshot_path)?;
        }
        if self.metadata_path.exists() {
            fs::remove_file(&self.metadata_path)?;
        }
        Ok(())
    }

    pub fn duplicate_to(&self, new_id: &str) -> Result<Self, std::io::Error> {
        self.duplicate_to_with_options(new_id, false)
    }

    pub fn duplicate_to_with_options(
        &self,
        new_id: &str,
        overwrite: bool,
    ) -> Result<Self, std::io::Error> {
        let normalized = normalize_save_id(new_id);
        if normalized.is_empty() {
            return Err(std::io::Error::other("save id cannot be empty"));
        }

        let parent = self
            .snapshot_path
            .parent()
            .ok_or_else(|| std::io::Error::other("save path has no parent directory"))?;
        let new_snapshot_path = parent.join(format!("{normalized}.json"));
        let new_metadata_path = GameStateSnapshot::metadata_path(&new_snapshot_path);

        if !overwrite && (new_snapshot_path.exists() || new_metadata_path.exists()) {
            return Err(std::io::Error::other("target save already exists"));
        }
        if overwrite {
            if new_snapshot_path.exists() {
                fs::remove_file(&new_snapshot_path)?;
            }
            if new_metadata_path.exists() {
                fs::remove_file(&new_metadata_path)?;
            }
        }

        if self.snapshot_path.exists() {
            fs::copy(&self.snapshot_path, &new_snapshot_path)?;
        }

        let metadata = self
            .metadata
            .clone()
            .or_else(|| SaveSlotMetadata::load_from_path(&self.snapshot_path).ok())
            .unwrap_or(SaveSlotMetadata {
                save_name: normalized.clone(),
                saved_turn: 0,
                recovery_note_count: 0,
                last_updated_unix: None,
                notes: String::new(),
                category: Some(SaveSlotCategory::Imported),
                auto_recovery_base_ids: Vec::new(),
                auto_defense_base_ids: Vec::new(),
                auto_economy_base_ids: Vec::new(),
            });
        metadata.save_to_path(&new_metadata_path)?;

        Ok(Self {
            category: SaveSlotListing::infer_category(
                &normalized,
                SaveSlotMetadata::load_from_path(&new_snapshot_path)
                    .ok()
                    .as_ref(),
            ),
            id: normalized,
            snapshot_path: new_snapshot_path.clone(),
            metadata_path: new_metadata_path,
            metadata: SaveSlotMetadata::load_from_path(&new_snapshot_path).ok(),
        })
    }

    pub fn export_snapshot(
        &self,
        target_path: impl AsRef<Path>,
        overwrite: bool,
    ) -> Result<(), std::io::Error> {
        let target_path = target_path.as_ref();
        if target_path.exists() && !overwrite {
            return Err(std::io::Error::other("target file already exists"));
        }
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let _snapshot = GameStateSnapshot::load_from_path(&self.snapshot_path)?;
        fs::copy(&self.snapshot_path, target_path)?;
        let metadata = self
            .metadata
            .clone()
            .or_else(|| SaveSlotMetadata::load_from_path(&self.snapshot_path).ok())
            .unwrap_or(SaveSlotMetadata {
                save_name: self.id.clone(),
                saved_turn: 0,
                recovery_note_count: 0,
                last_updated_unix: None,
                notes: String::new(),
                category: Some(self.category),
                auto_recovery_base_ids: Vec::new(),
                auto_defense_base_ids: Vec::new(),
                auto_economy_base_ids: Vec::new(),
            });
        metadata.save_to_path(GameStateSnapshot::metadata_path(target_path))
    }

    pub fn import_snapshot(
        source_path: impl AsRef<Path>,
        base_dir: impl AsRef<Path>,
        target_id: &str,
        overwrite: bool,
    ) -> Result<Self, std::io::Error> {
        let source_path = source_path.as_ref();
        let normalized = normalize_save_id(target_id);
        if normalized.is_empty() {
            return Err(std::io::Error::other("save id cannot be empty"));
        }
        let snapshot = GameStateSnapshot::load_from_path(source_path)?;
        let mut metadata =
            SaveSlotMetadata::load_from_path(source_path).unwrap_or_else(|_| snapshot.metadata());
        metadata.category = Some(SaveSlotCategory::Imported);
        let base_dir = base_dir.as_ref();
        fs::create_dir_all(base_dir)?;
        let target_snapshot_path = base_dir.join(format!("{normalized}.json"));
        let target_metadata_path = GameStateSnapshot::metadata_path(&target_snapshot_path);
        if !overwrite && (target_snapshot_path.exists() || target_metadata_path.exists()) {
            return Err(std::io::Error::other("target save already exists"));
        }
        if overwrite {
            if target_snapshot_path.exists() {
                fs::remove_file(&target_snapshot_path)?;
            }
            if target_metadata_path.exists() {
                fs::remove_file(&target_metadata_path)?;
            }
        }
        snapshot.save_to_path(&target_snapshot_path)?;
        metadata.save_to_path(&target_metadata_path)?;
        Ok(Self {
            category: SaveSlotListing::infer_category(&normalized, Some(&metadata)),
            id: normalized,
            snapshot_path: target_snapshot_path.clone(),
            metadata_path: target_metadata_path,
            metadata: SaveSlotMetadata::load_from_path(&target_snapshot_path).ok(),
        })
    }

    fn infer_category(id: &str, metadata: Option<&SaveSlotMetadata>) -> SaveSlotCategory {
        if let Some(category) = metadata.and_then(|metadata| metadata.category) {
            return category;
        }
        if metadata.is_none() {
            return SaveSlotCategory::Empty;
        }
        if id.starts_with("autosave") || id.starts_with("quicksave") {
            SaveSlotCategory::Autosave
        } else if id.starts_with("slot_") || id.starts_with("manual_") {
            SaveSlotCategory::Manual
        } else {
            SaveSlotCategory::Imported
        }
    }
}

pub fn save_sort_label(column: SaveSortColumn) -> &'static str {
    match column {
        SaveSortColumn::FileId => "File",
        SaveSortColumn::Category => "Type",
        SaveSortColumn::SaveName => "Name",
        SaveSortColumn::Turn => "Turn",
        SaveSortColumn::Recovery => "Recovery",
        SaveSortColumn::Updated => "Updated",
    }
}

pub fn save_filter_label(category: SaveFilterCategory) -> &'static str {
    match category {
        SaveFilterCategory::All => "All",
        SaveFilterCategory::Autosave => "Autosaves",
        SaveFilterCategory::Manual => "Manual",
        SaveFilterCategory::Imported => "Imported",
        SaveFilterCategory::Empty => "Empty",
    }
}

pub fn set_save_sort(
    current_column: SaveSortColumn,
    current_descending: bool,
    next_column: SaveSortColumn,
) -> (SaveSortColumn, bool) {
    if current_column == next_column {
        (current_column, !current_descending)
    } else {
        (
            next_column,
            matches!(next_column, SaveSortColumn::Updated | SaveSortColumn::Turn),
        )
    }
}

pub fn matches_save_filters(entry: &SaveSlotListing, query: &SaveBrowserQuery) -> bool {
    let summary = crate::presentation::save_browser_row_summary(entry);
    if !query.filter_text.trim().is_empty() {
        let filter = query.filter_text.trim().to_ascii_lowercase();
        let matches_text = summary.file_id.to_ascii_lowercase().contains(&filter)
            || summary.display_name.to_ascii_lowercase().contains(&filter)
            || summary.category_text.to_ascii_lowercase().contains(&filter)
            || summary.notes_preview.to_ascii_lowercase().contains(&filter);
        if !matches_text {
            return false;
        }
    }
    if query.populated_only && summary.is_empty {
        return false;
    }
    if query.recovered_only
        && entry
            .metadata
            .as_ref()
            .map(|metadata| metadata.recovery_note_count == 0)
            .unwrap_or(true)
    {
        return false;
    }
    match query.filter_category {
        SaveFilterCategory::All => true,
        SaveFilterCategory::Autosave => entry.category == SaveSlotCategory::Autosave,
        SaveFilterCategory::Manual => entry.category == SaveSlotCategory::Manual,
        SaveFilterCategory::Imported => entry.category == SaveSlotCategory::Imported,
        SaveFilterCategory::Empty => entry.category == SaveSlotCategory::Empty,
    }
}

pub fn sort_save_slots(entries: &mut [SaveSlotListing], column: SaveSortColumn, descending: bool) {
    entries.sort_by(|left, right| {
        let ordering = match column {
            SaveSortColumn::FileId => left.id.cmp(&right.id),
            SaveSortColumn::Category => left
                .category
                .cmp(&right.category)
                .then_with(|| left.id.cmp(&right.id)),
            SaveSortColumn::SaveName => left.display_name().cmp(right.display_name()),
            SaveSortColumn::Turn => left
                .saved_turn_value()
                .cmp(&right.saved_turn_value())
                .then_with(|| left.id.cmp(&right.id)),
            SaveSortColumn::Recovery => left
                .recovery_note_count_value()
                .cmp(&right.recovery_note_count_value())
                .then_with(|| left.id.cmp(&right.id)),
            SaveSortColumn::Updated => left
                .last_updated_unix_value()
                .cmp(&right.last_updated_unix_value())
                .then_with(|| left.id.cmp(&right.id)),
        };
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

pub fn save_browser_counts(
    entries: &[SaveSlotListing],
    query: &SaveBrowserQuery,
) -> SaveBrowserCounts {
    SaveBrowserCounts {
        total: entries.len(),
        filtered: entries
            .iter()
            .filter(|entry| matches_save_filters(entry, query))
            .count(),
        autosave_count: entries
            .iter()
            .filter(|entry| entry.category == SaveSlotCategory::Autosave)
            .count(),
        manual_count: entries
            .iter()
            .filter(|entry| entry.category == SaveSlotCategory::Manual)
            .count(),
        imported_count: entries
            .iter()
            .filter(|entry| entry.category == SaveSlotCategory::Imported)
            .count(),
        empty_count: entries
            .iter()
            .filter(|entry| entry.category == SaveSlotCategory::Empty)
            .count(),
    }
}

pub fn save_browser_counts_text(counts: SaveBrowserCounts) -> String {
    format!(
        "Showing {}/{} saves. Autosaves: {}  Manual: {}  Imported: {}  Empty: {}",
        counts.filtered,
        counts.total,
        counts.autosave_count,
        counts.manual_count,
        counts.imported_count,
        counts.empty_count,
    )
}

pub fn save_slot_label(entry: &SaveSlotListing) -> String {
    let summary = crate::presentation::save_browser_row_summary(entry);
    format!(
        "{}: {} (Turn {}) {} {} [{}]",
        summary.file_id,
        summary.display_name,
        summary.turn_text,
        summary.recovery_text,
        summary.updated_text,
        summary.category_text,
    )
}

pub fn current_save_slot_label(entries: &[SaveSlotListing], selected_id: &str) -> String {
    entries
        .iter()
        .find(|entry| entry.id == selected_id)
        .map(save_slot_label)
        .unwrap_or_else(|| format!("{selected_id}: Empty"))
}

pub fn save_sort_button_states(query: &SaveBrowserQuery) -> Vec<SaveSortButtonState> {
    SaveSortColumn::all()
        .into_iter()
        .map(|column| {
            let mut label_text = save_sort_label(column).to_string();
            let selected = query.sort_column == column;
            if selected {
                label_text.push_str(if query.sort_descending { " v" } else { " ^" });
            }
            SaveSortButtonState {
                column,
                label_text,
                selected,
            }
        })
        .collect()
}

pub fn filtered_sorted_save_slots(
    entries: &[SaveSlotListing],
    query: &SaveBrowserQuery,
) -> Vec<SaveSlotListing> {
    let mut filtered_entries: Vec<SaveSlotListing> = entries
        .iter()
        .filter(|entry| matches_save_filters(entry, query))
        .cloned()
        .collect();
    sort_save_slots(
        &mut filtered_entries,
        query.sort_column,
        query.sort_descending,
    );
    filtered_entries
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingSaveConflict {
    Rename {
        new_id: String,
    },
    Duplicate {
        new_id: String,
    },
    Import {
        source_path: PathBuf,
        target_id: String,
    },
    Export {
        target_path: PathBuf,
    },
    Delete {
        id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveConflictDisplayState {
    pub message_text: String,
    pub confirm_button_text: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveManagementDisplayState {
    pub heading_text: &'static str,
    pub target_slot_label: &'static str,
    pub file_id_label: &'static str,
    pub display_name_label: &'static str,
    pub category_label: &'static str,
    pub can_load: bool,
    pub category_options: Vec<(SaveSlotCategory, &'static str)>,
}

pub fn save_management_display_state(
    entries: &[SaveSlotListing],
    selected_id: &str,
) -> SaveManagementDisplayState {
    let can_load = entries.iter().any(|e| e.id == selected_id);
    SaveManagementDisplayState {
        heading_text: "Save & Load",
        target_slot_label: "Target Slot:",
        file_id_label: "File ID:",
        display_name_label: "Display Name:",
        category_label: "Category:",
        can_load,
        category_options: vec![
            (SaveSlotCategory::Manual, "Manual"),
            (SaveSlotCategory::Autosave, "Autosave"),
            (SaveSlotCategory::Imported, "Imported"),
        ],
    }
}

pub fn save_conflict_display_state(conflict: &PendingSaveConflict) -> SaveConflictDisplayState {
    match conflict {
        PendingSaveConflict::Rename { new_id } => SaveConflictDisplayState {
            message_text: format!("Rename would overwrite existing save `{new_id}`."),
            confirm_button_text: "Confirm overwrite",
        },
        PendingSaveConflict::Duplicate { new_id } => SaveConflictDisplayState {
            message_text: format!("Duplicate would overwrite existing save `{new_id}`."),
            confirm_button_text: "Confirm overwrite",
        },
        PendingSaveConflict::Import {
            source_path,
            target_id,
        } => SaveConflictDisplayState {
            message_text: format!(
                "Import from {} would overwrite existing save `{target_id}`.",
                source_path.display()
            ),
            confirm_button_text: "Confirm overwrite",
        },
        PendingSaveConflict::Export { target_path } => SaveConflictDisplayState {
            message_text: format!("Export would overwrite {}.", target_path.display()),
            confirm_button_text: "Confirm overwrite",
        },
        PendingSaveConflict::Delete { id } => SaveConflictDisplayState {
            message_text: format!("Delete save `{id}` from the local browser."),
            confirm_button_text: "Confirm delete",
        },
    }
}

pub fn save_browser_display_state(
    entries: &[SaveSlotListing],
    selected_id: &str,
    query: &SaveBrowserQuery,
) -> SaveBrowserDisplayState {
    let counts = save_browser_counts(entries, query);
    let rows = filtered_sorted_save_slots(entries, query)
        .into_iter()
        .map(|entry| {
            let summary = crate::presentation::save_browser_row_summary(&entry);
            SaveBrowserRowState {
                id: entry.id.clone(),
                selected: entry.id == selected_id,
                category_text: summary.category_text,
                display_name: summary.display_name,
                turn_text: summary.turn_text,
                recovery_text: summary.recovery_text,
                updated_text: summary.updated_text,
                notes_preview: summary.notes_preview,
                file_path_text: format!("Path: {}", entry.snapshot_path.display()),
                delete_action_label: if entry
                    .metadata
                    .as_ref()
                    .map(|m| m.recovery_note_count > 0)
                    .unwrap_or(false)
                {
                    format!("⚠ Delete {}", entry.id)
                } else {
                    format!("Delete {}", entry.id)
                },
                rename_action_label: format!("Rename {}", entry.id),
                duplicate_action_label: format!("Duplicate {}", entry.id),
            }
        })
        .collect();

    SaveBrowserDisplayState {
        current_slot_label: current_save_slot_label(entries, selected_id),
        counts_text: save_browser_counts_text(counts),
        column_headers: vec![
            "Sel", "File", "Type", "Name", "Turn", "Recovery", "Updated", "Notes",
        ],
        global_actions: vec![
            SaveBrowserGlobalActionState {
                label_text: "Rename".to_string(),
                category: None,
                action_type: SaveBrowserGlobalActionType::Rename,
            },
            SaveBrowserGlobalActionState {
                label_text: "Duplicate".to_string(),
                category: None,
                action_type: SaveBrowserGlobalActionType::Duplicate,
            },
            SaveBrowserGlobalActionState {
                label_text: "Save As Manual".to_string(),
                category: Some(SaveSlotCategory::Manual),
                action_type: SaveBrowserGlobalActionType::SaveAs,
            },
            SaveBrowserGlobalActionState {
                label_text: "Save As Autosave".to_string(),
                category: Some(SaveSlotCategory::Autosave),
                action_type: SaveBrowserGlobalActionType::SaveAs,
            },
            SaveBrowserGlobalActionState {
                label_text: "Delete".to_string(),
                category: None,
                action_type: SaveBrowserGlobalActionType::Delete,
            },
            SaveBrowserGlobalActionState {
                label_text: "Import Path".to_string(),
                category: None,
                action_type: SaveBrowserGlobalActionType::Import,
            },
            SaveBrowserGlobalActionState {
                label_text: "Export Path".to_string(),
                category: None,
                action_type: SaveBrowserGlobalActionType::Export,
            },
        ],
        sort_buttons: save_sort_button_states(query),
        rows,
    }
}

fn next_available_slot_id(existing_ids: &[String]) -> String {
    let mut max_slot = 0u32;
    for id in existing_ids {
        if let Some(value) = id
            .strip_prefix("slot_")
            .and_then(|rest| rest.parse::<u32>().ok())
        {
            max_slot = max_slot.max(value);
        }
    }
    format!("slot_{}", max_slot.saturating_add(1).max(1))
}

pub fn normalize_save_id(value: &str) -> String {
    let mut normalized = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '_' | '-' | ' ') {
            normalized.push('_');
        }
    }
    while normalized.contains("__") {
        normalized = normalized.replace("__", "_");
    }
    normalized.trim_matches('_').to_string()
}
