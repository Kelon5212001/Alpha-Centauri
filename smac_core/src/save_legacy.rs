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
