use crate::{
    content, BaseAreaRole, ConvoyOverlayStatus, ConvoyRouteKind, Facility, GovernorMode,
    Improvement, ProductionItem, SaveSlotCategory, SaveSlotListing, SecretProject, Tech, Terrain,
    UnitKind, Yields,
};

pub struct FactionStatusSummary<'a> {
    pub name: &'a str,
    pub leader: Option<&'a str>,
    pub description: Option<&'a str>,
    pub color_hex: &'a str,
    pub base_count: usize,
    pub unit_count: usize,
    pub energy: i32,
    pub research_progress: String,
    pub current_tech: &'a str,
    pub techs_discovered: i32,
    pub unrest_base_count: usize,
    pub recovery_base_count: usize,
    pub frontier_base_count: usize,
    pub rear_area_base_count: usize,
    pub psi_frontier_base_count: usize,
    pub warzone_base_count: usize,
    pub trade_route_count: usize,
    pub freight_route_count: usize,
    pub disrupted_route_count: usize,
    pub intercepted_route_count: usize,
    pub convoy_capacity_used: usize,
    pub convoy_capacity_total: usize,
}

pub struct BasePanelSummary<'a> {
    pub name: &'a str,
    pub owner_name: &'a str,
    pub population: i32,
    pub governor_mode: &'a str,
    pub trade_links: usize,
    pub stability: String,
    pub storage: String,
    pub output: String,
    pub effective_output: String,
    pub production: String,
    pub queue: String,
    pub facilities: String,
}

pub struct UnitPanelSummary<'a> {
    pub unit_name: &'a str,
    pub owner_name: &'a str,
    pub rank: &'a str,
    pub role: &'a str,
    pub location: String,
    pub moves_left: i32,
    pub hp: i32,
}

pub struct TilePanelSummary<'a> {
    pub coordinates: String,
    pub terrain_name: &'a str,
    pub elevation: i32,
    pub moisture: i32,
    pub yield_summary: String,
    pub improvement_name: Option<&'a str>,
}

pub struct SaveBrowserRowSummary {
    pub file_id: String,
    pub display_name: String,
    pub turn_text: String,
    pub recovery_text: String,
    pub updated_text: String,
    pub category_text: String,
    pub notes_preview: String,
    pub is_empty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapOverlay {
    Terrain,
    Yields,
    Ownership,
    Borders,
    Threat,
    FrontierPressure,
    PsiThreat,
    Logistics,
    Trade,
}

impl MapOverlay {
    pub fn all() -> [Self; 9] {
        [
            Self::Terrain,
            Self::Yields,
            Self::Ownership,
            Self::Borders,
            Self::Threat,
            Self::FrontierPressure,
            Self::PsiThreat,
            Self::Logistics,
            Self::Trade,
        ]
    }
}

pub fn map_overlay_label(overlay: MapOverlay) -> &'static str {
    match overlay {
        MapOverlay::Terrain => "Terrain",
        MapOverlay::Yields => "Yields",
        MapOverlay::Ownership => "Ownership",
        MapOverlay::Borders => "Borders",
        MapOverlay::Threat => "Offensive Threat",
        MapOverlay::FrontierPressure => "Frontier Pressure",
        MapOverlay::PsiThreat => "Psi Threat",
        MapOverlay::Logistics => "Logistics",
        MapOverlay::Trade => "Global Trade",
    }
}
pub fn map_overlay_legend(overlay: MapOverlay) -> &'static str {
    match overlay {
        MapOverlay::Terrain => {
            "Terrain: blue ocean, green flat, olive rolling, brown rocky, teal fungus."
        }
        MapOverlay::Yields => "Yields: brighter tiles produce more total output.",
        MapOverlay::Ownership => {
            "Ownership: green player control, red rival control, gray unclaimed."
        }
        MapOverlay::Borders => {
            "Borders: highlighted tiles sit near faction bases and control edges."
        }
        MapOverlay::Threat => {
            "Offensive Threat: deeper red means stronger nearby mobile enemy pressure."
        }
        MapOverlay::FrontierPressure => {
            "Frontier Pressure: amber intensity shows static border and enemy-base exposure."
        }
        MapOverlay::PsiThreat => "Psi Threat: purple intensity shows native/psi danger bands.",
        MapOverlay::Logistics => {
            "Logistics: green active lanes, blue escorted hubs, orange disrupted lanes, red intercepted lanes."
        }
        MapOverlay::Trade => "Trade: Arrows show active trade/freight routes between bases.",
    }
}

pub fn map_overlay_uses_convoy_lines(overlay: MapOverlay) -> bool {
    matches!(overlay, MapOverlay::Logistics | MapOverlay::Trade)
}

pub fn ui_overlay_label() -> &'static str {
    "overlay"
}

pub fn ui_minimap_heading() -> &'static str {
    "Minimap"
}

pub fn terrain_name(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Ocean => "Ocean",
        Terrain::Flat => "Flat",
        Terrain::Rolling => "Rolling",
        Terrain::Rocky => "Rocky",
        Terrain::Fungus => "Xenofungus",
        Terrain::Crater => "Nuclear Crater",
        }
        }

        pub fn terrain_symbol(terrain: Terrain) -> &'static str {
        match terrain {
        Terrain::Ocean => "w",
        Terrain::Flat => ".",
        Terrain::Rolling => "r",
        Terrain::Rocky => "^",
        Terrain::Fungus => "F",
        Terrain::Crater => "X",
        }
        }


pub fn convoy_overlay_status_glyph(status: ConvoyOverlayStatus) -> Option<&'static str> {
    match status {
        ConvoyOverlayStatus::Collapsing => Some("X"),
        ConvoyOverlayStatus::Intercepted => Some("!"),
        ConvoyOverlayStatus::Disrupted => Some("="),
        ConvoyOverlayStatus::Protected => Some("+"),
        ConvoyOverlayStatus::Active => Some("-"),
        ConvoyOverlayStatus::None => None,
    }
}

pub fn convoy_overlay_status_color_hex(status: ConvoyOverlayStatus) -> &'static str {
    match status {
        ConvoyOverlayStatus::Intercepted => "#bc3838",
        ConvoyOverlayStatus::Disrupted => "#c48434",
        ConvoyOverlayStatus::Collapsing => "#dc5628",
        ConvoyOverlayStatus::Protected => "#4694b6",
        ConvoyOverlayStatus::Active => "#489c58",
        ConvoyOverlayStatus::None => "#386048",
    }
}

pub fn improvement_name(improvement: Improvement) -> &'static str {
    match improvement {
        Improvement::Farm => "Farm",
        Improvement::Mine => "Mine",
        Improvement::Solar => "Solar",
        Improvement::Road => "Road",
        Improvement::Condenser => "Condenser",
        Improvement::EchelonMirror => "Echelon Mirror",
        Improvement::Forest => "Forest",
        Improvement::ThermalBorehole => "Thermal Borehole",
    }
}

pub fn improvement_glyph(improvement: Improvement) -> &'static str {
    match improvement {
        Improvement::Farm => "v",
        Improvement::Mine => "X",
        Improvement::Solar => "*",
        Improvement::Road => "=",
        Improvement::Condenser => "C",
        Improvement::EchelonMirror => "M",
        Improvement::Forest => "T",
        Improvement::ThermalBorehole => "B",
    }
}

pub fn unit_name(kind: UnitKind) -> &'static str {
    content::unit_name(kind)
}

pub fn unit_role_summary(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::ColonyPod => "Expansion unit for founding new bases.",
        UnitKind::ScoutPatrol => "Baseline infantry for cheap garrison and early skirmishing.",
        UnitKind::Former => "Terraforming unit for improving yields and roads.",
        UnitKind::Speeder => "Fast line unit for mobility and flexible response.",
        UnitKind::ResonanceLaser => {
            "Advanced shock infantry for breakthrough attacks and base assaults."
        }
        UnitKind::EscortSpeeder => "Fast escort unit for convoy protection and response patrols.",
        UnitKind::RaiderSpeeder => "High-speed striker for aggressive flanks and exposed targets.",
        UnitKind::TranceScout => "Psi-hardened defender against native and psionic threats.",
        UnitKind::GarrisonGuard => "Heavy defensive infantry for frontier bases and chokepoints.",
        UnitKind::PsiSentinel => "Elite psi-defense guard for hostile frontier pressure.",
        UnitKind::MindWorm => "Native psi attacker that threatens exposed units and weak bases.",
        UnitKind::IsleOfTheDeep => "Native sea-borne lifeform for coastal and ocean pressure.",
        UnitKind::Needlejet => "High-speed aircraft for rapid interception and strikes.",
        UnitKind::ProbeTeam => {
            "Covert operatives capable of infiltrating bases and subverting enemy units."
        }
        UnitKind::CustomUnit(_) => "A specialized unit built to faction specifications.",
    }
}

pub fn unit_role_badge(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::ColonyPod => "COL",
        UnitKind::ScoutPatrol => "INF",
        UnitKind::Former => "ENG",
        UnitKind::Speeder => "MOB",
        UnitKind::ResonanceLaser => "SHK",
        UnitKind::EscortSpeeder => "ESC",
        UnitKind::RaiderSpeeder => "RAID",
        UnitKind::TranceScout => "PSI",
        UnitKind::GarrisonGuard => "DEF",
        UnitKind::PsiSentinel => "PSI+",
        UnitKind::MindWorm => "NAT",
        UnitKind::IsleOfTheDeep => "SEA",
        UnitKind::Needlejet => "AIR",
        UnitKind::ProbeTeam => "ESP",
        UnitKind::CustomUnit(_) => "SPEC",
    }
}

pub fn unit_rank_name(experience: i32) -> &'static str {
    match experience {
        i32::MIN..=0 => "Green",
        1 => "Disciplined",
        2 => "Veteran",
        _ => "Elite",
    }
}

pub fn unit_map_symbol(kind: UnitKind, _owner: usize) -> String {
    let symbol = match kind {
        UnitKind::ColonyPod => "○",
        UnitKind::ScoutPatrol => "▲",
        UnitKind::Former => "⚒",
        UnitKind::Speeder => "»",
        UnitKind::ResonanceLaser => "⚡",
        UnitKind::EscortSpeeder => "»",
        UnitKind::RaiderSpeeder => "»",
        UnitKind::TranceScout => "▼",
        UnitKind::GarrisonGuard => "■",
        UnitKind::PsiSentinel => "▣",
        UnitKind::MindWorm => "🪱",
        UnitKind::IsleOfTheDeep => "🦑",
        UnitKind::Needlejet => "✈",
        UnitKind::ProbeTeam => "👁",
        UnitKind::CustomUnit(_) => "✧",
    };

    symbol.to_string()
}

pub fn base_map_symbol(owner: usize, viewer_owner: usize) -> &'static str {
    if owner == viewer_owner {
        "⛫"
    } else {
        "⌬"
    }
}

pub fn format_yields(yields: Yields) -> String {
    format!(
        "{} nutrients / {} minerals / {} energy",
        yields.nutrients, yields.minerals, yields.energy
    )
}

pub fn format_base_storage(nutrients_stock: i32, minerals_stock: i32) -> String {
    format!("{nutrients_stock} nutrients / {minerals_stock} minerals")
}

pub fn format_research_progress(current: i32, cost: i32) -> String {
    format!("{current}/{cost}")
}

pub fn governor_mode_label(mode: GovernorMode) -> &'static str {
    match mode {
        GovernorMode::Off => "Manual",
        GovernorMode::Balanced => "Balanced",
        GovernorMode::Recovery => "Recovery",
        GovernorMode::Defense => "Defense",
        GovernorMode::Economy => "Economy",
        GovernorMode::Logistics => "Logistics",
        GovernorMode::MachinePolity => "Machine Polity",
    }
}

pub fn governor_mode_description(mode: GovernorMode) -> &'static str {
    match mode {
        GovernorMode::Off => "Manual control of production and research focus.",
        GovernorMode::Balanced => "Maintains balanced growth and defense posture.",
        GovernorMode::Recovery => "Prioritizes repair and morale support for stressed bases.",
        GovernorMode::Defense => {
            "Prioritizes garrisons and defensive facilities for frontline bases."
        }
        GovernorMode::Economy => "Prioritizes energy, research, and growth facilities.",
        GovernorMode::Logistics => "Prioritizes convoy capacity and route security.",
        GovernorMode::MachinePolity => "Automated governance optimized for industrial efficiency.",
    }
}

pub fn base_area_role_label(role: BaseAreaRole) -> &'static str {
    match role {
        BaseAreaRole::RearArea => "Rear Area",
        BaseAreaRole::Frontier => "Frontier",
        BaseAreaRole::PsiFrontier => "Psi Frontier",
        BaseAreaRole::Warzone => "Warzone",
    }
}

pub fn format_stability(unrest: i32) -> String {
    if unrest <= 0 {
        "Stable".to_string()
    } else if unrest == 1 {
        "Strained (-1 minerals, -2 energy)".to_string()
    } else {
        format!(
            "Unrest {unrest} (-{unrest} minerals, -{} energy)",
            unrest * 2
        )
    }
}

pub fn format_recovery_status(recovery_note_count: usize) -> String {
    if recovery_note_count == 0 {
        "Clean".to_string()
    } else {
        format!("Recovered ({recovery_note_count})")
    }
}

pub fn format_unix_timestamp(last_updated_unix: Option<u64>) -> String {
    let Some(timestamp) = last_updated_unix else {
        return "Unknown".to_string();
    };

    let days = (timestamp / 86_400) as i64;
    let secs_of_day = (timestamp % 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

pub fn save_slot_category_label(category: SaveSlotCategory) -> &'static str {
    match category {
        SaveSlotCategory::Autosave => "Autosave",
        SaveSlotCategory::Manual => "Manual",
        SaveSlotCategory::Imported => "Imported",
        SaveSlotCategory::Empty => "Empty",
    }
}

pub fn save_browser_row_summary(entry: &SaveSlotListing) -> SaveBrowserRowSummary {
    let metadata = entry.metadata.as_ref();
    SaveBrowserRowSummary {
        file_id: entry.id.clone(),
        display_name: metadata
            .map(|metadata| metadata.save_name.clone())
            .unwrap_or_else(|| "Empty".to_string()),
        turn_text: metadata
            .map(|metadata| metadata.saved_turn.to_string())
            .unwrap_or_else(|| "-".to_string()),
        recovery_text: metadata
            .map(|metadata| format_recovery_status(metadata.recovery_note_count))
            .unwrap_or_else(|| "Empty".to_string()),
        updated_text: metadata
            .map(|metadata| format_unix_timestamp(metadata.last_updated_unix))
            .unwrap_or_else(|| "Unknown".to_string()),
        category_text: save_slot_category_label(entry.category).to_string(),
        notes_preview: metadata
            .map(|metadata| {
                let notes = metadata.notes.trim();
                if notes.is_empty() {
                    "No notes".to_string()
                } else if notes.chars().count() > 36 {
                    let preview: String = notes.chars().take(36).collect();
                    format!("{preview}...")
                } else {
                    notes.to_string()
                }
            })
            .unwrap_or_else(|| "No notes".to_string()),
        is_empty: metadata.is_none(),
    }
}

pub fn production_name(item: ProductionItem) -> &'static str {
    content::production_name(item)
}

pub fn production_role_badge(item: ProductionItem) -> &'static str {
    match item {
        ProductionItem::ScoutPatrol => unit_role_badge(UnitKind::ScoutPatrol),
        ProductionItem::ColonyPod => unit_role_badge(UnitKind::ColonyPod),
        ProductionItem::Former => unit_role_badge(UnitKind::Former),
        ProductionItem::Speeder => unit_role_badge(UnitKind::Speeder),
        ProductionItem::ResonanceLaser => unit_role_badge(UnitKind::ResonanceLaser),
        ProductionItem::EscortSpeeder => unit_role_badge(UnitKind::EscortSpeeder),
        ProductionItem::RaiderSpeeder => unit_role_badge(UnitKind::RaiderSpeeder),
        ProductionItem::TranceScout => unit_role_badge(UnitKind::TranceScout),
        ProductionItem::GarrisonGuard => unit_role_badge(UnitKind::GarrisonGuard),
        ProductionItem::PsiSentinel => unit_role_badge(UnitKind::PsiSentinel),
        ProductionItem::RecyclingTanks => "ECO",
        ProductionItem::PerimeterDefense => "DEF",
        ProductionItem::NetworkNode => "RES",
        ProductionItem::RecreationCommons => "CIV",
        ProductionItem::Greenhouse => "AGR",
        ProductionItem::MineralRefinery => "MIN",
        ProductionItem::TradeExchange => "TRD",
        ProductionItem::FreightDepot => "FRT",
        ProductionItem::PatrolGrid => "ESC",
        ProductionItem::CommandCenter => "MIL",
        ProductionItem::FieldHospital => "REC",
        ProductionItem::MilitaryAcademy => "MIL",
        ProductionItem::SensorArray => "SEN",
        ProductionItem::TransitHub => "MOB",
        ProductionItem::PsiBeacon => "PSI",
        ProductionItem::ForwardDepot => "LOG",
        ProductionItem::HologramTheatre => "MRL",
        ProductionItem::BioenhancementCenter => "BIO",
        ProductionItem::ResearchHospital => "MED",
        ProductionItem::WeatherPattern
        | ProductionItem::ClinicalImmortality
        | ProductionItem::EmpathGuild
        | ProductionItem::OrbitalElevator
        | ProductionItem::ManifoldDrive
        | ProductionItem::SingularityContainment
        | ProductionItem::BlackHoleHarvester => "✧WP",
        ProductionItem::ProbeTeam => "ESP",
        ProductionItem::CustomUnit(_) => "✧CST",
        ProductionItem::StockpileEnergy => "✧WLT",
        ProductionItem::SkyHydroponics => "ORB",
        ProductionItem::SolarTransmitter => "ORB",
        ProductionItem::OrbitalDefense => "ORB",
    }
}

pub fn production_role_category(item: ProductionItem) -> &'static str {
    match item {
        ProductionItem::ColonyPod => "Expansion",
        ProductionItem::Former => "Infrastructure",
        ProductionItem::ScoutPatrol
        | ProductionItem::ResonanceLaser
        | ProductionItem::GarrisonGuard
        | ProductionItem::PerimeterDefense
        | ProductionItem::SensorArray => "Defense",
        ProductionItem::Speeder
        | ProductionItem::EscortSpeeder
        | ProductionItem::RaiderSpeeder
        | ProductionItem::TransitHub
        | ProductionItem::ForwardDepot => "Mobility",
        ProductionItem::TranceScout | ProductionItem::PsiSentinel | ProductionItem::PsiBeacon => {
            "Psi"
        }
        ProductionItem::ProbeTeam => "Espionage",
        ProductionItem::NetworkNode => "Research",
        ProductionItem::Greenhouse => "Food",
        ProductionItem::MineralRefinery => "Extraction",
        ProductionItem::TradeExchange => "Trade",
        ProductionItem::FreightDepot => "Logistics",
        ProductionItem::PatrolGrid => "Security",
        ProductionItem::HologramTheatre => "Morale",
        ProductionItem::BioenhancementCenter => "Biotech",
        ProductionItem::ResearchHospital => "Recovery",
        ProductionItem::RecyclingTanks | ProductionItem::RecreationCommons => "Economy",
        ProductionItem::CommandCenter | ProductionItem::MilitaryAcademy => "Command",
        ProductionItem::FieldHospital => "Recovery",
        ProductionItem::WeatherPattern
        | ProductionItem::ClinicalImmortality
        | ProductionItem::EmpathGuild
        | ProductionItem::OrbitalElevator
        | ProductionItem::ManifoldDrive
        | ProductionItem::SingularityContainment
        | ProductionItem::BlackHoleHarvester => "Wonder",
        ProductionItem::CustomUnit(_) => "Custom",
        ProductionItem::StockpileEnergy => "Wealth",
        ProductionItem::SkyHydroponics
        | ProductionItem::SolarTransmitter
        | ProductionItem::OrbitalDefense => "Orbital",
    }
}

pub fn production_role_summary(item: ProductionItem) -> &'static str {
    match item {
        ProductionItem::ScoutPatrol => unit_role_summary(UnitKind::ScoutPatrol),
        ProductionItem::ColonyPod => unit_role_summary(UnitKind::ColonyPod),
        ProductionItem::Former => unit_role_summary(UnitKind::Former),
        ProductionItem::Speeder => unit_role_summary(UnitKind::Speeder),
        ProductionItem::ResonanceLaser => unit_role_summary(UnitKind::ResonanceLaser),
        ProductionItem::EscortSpeeder => unit_role_summary(UnitKind::EscortSpeeder),
        ProductionItem::RaiderSpeeder => unit_role_summary(UnitKind::RaiderSpeeder),
        ProductionItem::TranceScout => unit_role_summary(UnitKind::TranceScout),
        ProductionItem::GarrisonGuard => unit_role_summary(UnitKind::GarrisonGuard),
        ProductionItem::PsiSentinel => unit_role_summary(UnitKind::PsiSentinel),
        ProductionItem::ProbeTeam => unit_role_summary(UnitKind::ProbeTeam),
        ProductionItem::RecyclingTanks => {
            "Economic facility that stabilizes early growth and mineral flow."
        }
        ProductionItem::PerimeterDefense => {
            "Static defense facility for frontline bases under attack."
        }
        ProductionItem::NetworkNode => "Research and energy infrastructure for developing bases.",
        ProductionItem::RecreationCommons => {
            "Stability facility that suppresses unrest and restores output."
        }
        ProductionItem::Greenhouse => {
            "Agriculture facility that improves nutrient flow and steadier population growth."
        }
        ProductionItem::MineralRefinery => {
            "Extraction facility that strengthens mineral throughput and industrial output."
        }
        ProductionItem::TradeExchange => {
            "Trade infrastructure that converts nearby friendly-base links into stronger energy flow."
        }
        ProductionItem::FreightDepot => {
            "Freight logistics facility that converts nearby friendly-base links into stronger mineral throughput."
        }
        ProductionItem::PatrolGrid => {
            "Convoy protection and surveillance facility that increases route capacity and reduces disruption."
        }
        ProductionItem::CommandCenter => "Training, support, and readiness hub for military bases.",
        ProductionItem::FieldHospital => {
            "Recovery facility for damaged garrisons and wounded rotations."
        }
        ProductionItem::MilitaryAcademy => {
            "Advanced training center for higher-quality new troops."
        }
        ProductionItem::SensorArray => {
            "Frontier sensing and coverage facility for threatened borders."
        }
        ProductionItem::TransitHub => {
            "Mobility support facility for fast-response and raider-oriented forces."
        }
        ProductionItem::PsiBeacon => {
            "Psi-support facility for hostile fungus fronts and native-pressure bases."
        }
        ProductionItem::ForwardDepot => {
            "Forward logistics facility for mobile armies, repair rotations, and strike support."
        }
        ProductionItem::HologramTheatre => {
            "Morale and culture facility that suppresses unrest while boosting research-side energy flow."
        }
        ProductionItem::BioenhancementCenter => {
            "Biotech support facility that improves troop readiness, psi resilience, and long-term base performance."
        }
        ProductionItem::ResearchHospital => {
            "Advanced medical complex that accelerates recovery, growth, and scientific throughput."
        }
        ProductionItem::WeatherPattern => {
            "Global wonder that stabilizes planetary conditions, protecting against Dust Fall."
        }
        ProductionItem::ClinicalImmortality => {
            "Global wonder that optimizes biological health, boosting Food Security."
        }
        ProductionItem::EmpathGuild => {
            "Global wonder that strengthens collective oversight, protecting against Governance Override."
        }
        ProductionItem::OrbitalElevator => {
            "Global wonder that provides cheap access to orbit, essential for Space Mastery."
        }
        ProductionItem::ManifoldDrive => {
            "Global wonder that manipulates gravity wells, essential for Space Mastery."
        }
        ProductionItem::SingularityContainment => {
            "Global wonder that stabilizes singularity fields, essential for Black Hole Harvesting."
        }
        ProductionItem::BlackHoleHarvester => {
            "Global wonder that extracts energy from the void, essential for Black Hole Harvesting."
        }
        ProductionItem::CustomUnit(_) => {
            "A unit designed in your workshop with specialized components."
        }
        ProductionItem::StockpileEnergy => {
            "Conversion process that turns excess minerals into immediate energy stock."
        }
        ProductionItem::SkyHydroponics => {
            "Orbital satellite that uses advanced genetics to boost planetary nutrient yields."
        }
        ProductionItem::SolarTransmitter => {
            "Orbital array that transmits focused solar energy to planetary receivers."
        }
        ProductionItem::OrbitalDefense => {
            "Network of defensive pods designed to intercept atmospheric and orbital threats."
        }
    }
}

pub fn production_tooltip_summary(item: ProductionItem) -> String {
    let mut parts = vec![
        production_role_summary(item).to_string(),
        format!("Cost: {}", production_cost(item)),
    ];
    if let Some(required_tech) = content::required_tech_for_production(item) {
        parts.push(format!("Unlock: {}", tech_name(required_tech)));
    }
    if let Some(facility) = item.facility() {
        let psi_support = content::facility_psi_support_bonus(facility);
        if psi_support > 0 {
            parts.push(format!("Psi support: +{psi_support}"));
        }
    }
    if let Some(project) = item.secret_project() {
        parts.push("Global Wonder: Only one can exist in the world.".to_string());
        match project {
            SecretProject::WeatherPattern => {
                parts.push(
                    "Effect: Local bases are immune to Dust Fall nutrient penalties.".to_string(),
                );
            }
            SecretProject::ClinicalImmortality => {
                parts.push("Effect: Provides a global +25% bonus to Food Security.".to_string());
            }
            SecretProject::EmpathGuild => {
                parts.push(
                    "Effect: Significantly reduces the risk of Governance Override.".to_string(),
                );
            }
            SecretProject::OrbitalElevator => {
                parts.push("Effect: Unlocks Space Mastery path. Combined with Manifold Drive, grants Space Transcendence victory.".to_string());
            }
            SecretProject::ManifoldDrive => {
                parts.push("Effect: Enables faster-than-light travel. Combined with Orbital Elevator, grants Space Transcendence victory.".to_string());
            }
            SecretProject::SingularityContainment => {
                parts.push("Effect: Unlocks Black Hole Harvesting path. Combined with Black Hole Harvester, grants Singularity Mastery victory.".to_string());
            }
            SecretProject::BlackHoleHarvester => {
                parts.push("Effect: Draws infinite power from the void. Combined with Singularity Containment, grants Singularity Mastery victory.".to_string());
            }
        }
    }
    parts.join("\n")
}

pub fn production_tooltip_summary_with_status(
    item: ProductionItem,
    available: bool,
    missing_tech: Option<Tech>,
) -> String {
    let mut tooltip = production_tooltip_summary(item);
    if available {
        tooltip.push_str("\nStatus: Available");
    } else if let Some(tech) = missing_tech {
        tooltip.push_str(&format!(
            "\nStatus: Locked\nMissing tech: {}",
            tech_name(tech)
        ));
    } else {
        tooltip.push_str("\nStatus: Unavailable");
    }
    tooltip
}

pub fn production_dependency_text(item: ProductionItem) -> String {
    if let Some(required_tech) = content::required_tech_for_production(item) {
        format!("Requires {}", tech_name(required_tech))
    } else {
        "No tech prerequisite".to_string()
    }
}

pub fn build_availability_summary(available: usize, locked: usize) -> String {
    format!("Build availability: {available} available / {locked} locked")
}

pub fn governor_mode_mix_summary(
    balanced: usize,
    defense: usize,
    recovery: usize,
    economy: usize,
    logistics: usize,
) -> String {
    format!(
        "{balanced} balanced / {defense} defense / {recovery} recovery / {economy} economy / {logistics} logistics"
    )
}

pub fn summarize_named_counts(entries: &[(String, usize)], empty: &str, limit: usize) -> String {
    if entries.is_empty() {
        return empty.to_string();
    }

    let mut sorted = entries.to_vec();
    sorted.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    sorted
        .into_iter()
        .take(limit)
        .map(|(name, count)| format!("{name} x{count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn summarize_production_item_counts(
    entries: &[(ProductionItem, usize)],
    empty: &str,
    limit: usize,
) -> String {
    if entries.is_empty() {
        return empty.to_string();
    }

    let mut sorted = entries.to_vec();
    sorted.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| production_name(left.0).cmp(production_name(right.0)))
    });
    sorted
        .into_iter()
        .take(limit)
        .map(|(item, count)| format!("{} x{count}", production_name(item)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn summarize_base_names(names: &[String], empty: &str, limit: usize) -> String {
    if names.is_empty() {
        return empty.to_string();
    }

    names
        .iter()
        .take(limit)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn summarize_base_unlock_blocks(
    entries: &[(String, ProductionItem, Tech)],
    empty: &str,
    limit: usize,
) -> String {
    if entries.is_empty() {
        return empty.to_string();
    }

    entries
        .iter()
        .take(limit)
        .map(|(base_name, item, tech)| {
            format!(
                "{base_name} -> {} ({})",
                production_name(*item),
                tech_name(*tech)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn production_sequence_summary(items: &[ProductionItem]) -> String {
    if items.is_empty() {
        return "None".to_string();
    }

    items
        .iter()
        .map(|item| production_name(*item))
        .collect::<Vec<_>>()
        .join(" -> ")
}

pub fn research_current_label(tech: Tech, current: i32, cost: i32) -> String {
    format!(
        "Current: {} ({})",
        tech_name(tech),
        format_research_progress(current, cost)
    )
}

pub fn available_research_label(tech: Tech, is_selected: bool) -> String {
    if is_selected {
        format!("Selected: {}", tech_name(tech))
    } else {
        tech_name(tech).to_string()
    }
}

pub fn unlock_preview_heading(tech: Tech) -> String {
    format!("Pinned unlock preview: {}", tech_name(tech))
}

pub fn unlock_preview_section_heading() -> &'static str {
    "Queue preview if research lands"
}

pub fn unlock_preview_row_text(base_name: &str, items: &[ProductionItem]) -> String {
    format!("{base_name}: {}", production_sequence_summary(items))
}

pub fn unlock_preview_stale_label() -> &'static str {
    "(stale)"
}

pub fn unlock_preview_focus_label() -> &'static str {
    "Focus"
}

pub fn unlock_preview_apply_label() -> &'static str {
    "Apply"
}

pub fn unlock_preview_keep_open_label() -> &'static str {
    "Keep preview open"
}

pub fn unlock_preview_stage_log_label() -> &'static str {
    "Stage in log"
}

pub fn unlock_preview_stage_all_log_label() -> &'static str {
    "Stage all previews in log"
}

pub fn unlock_preview_refresh_label() -> &'static str {
    "Refresh"
}

pub fn unlock_preview_apply_all_label() -> &'static str {
    "Apply all"
}

pub fn unlock_preview_clear_label() -> &'static str {
    "Clear"
}

pub fn research_cycle_affected_base_label() -> &'static str {
    "Cycle affected base"
}

pub fn research_focus_affected_label() -> &'static str {
    "Focus affected"
}

pub fn research_unlock_impact_text(count: usize) -> String {
    format!("Unblocks {count} base plan(s)")
}

pub fn research_unlock_affects_text(summary: &str) -> String {
    format!("Affects: {summary}")
}

pub fn unlock_preview_more_bases_text(hidden: usize) -> String {
    format!("{hidden} more base(s) not shown")
}

pub fn research_available_empty_text() -> &'static str {
    "No new techs are currently selectable."
}

pub fn research_blocked_missing_text(missing: &[Tech]) -> String {
    if missing.is_empty() {
        "Progress blocked".to_string()
    } else {
        format!(
            "Missing: {}",
            missing
                .iter()
                .copied()
                .map(tech_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub fn research_blocked_label(tech: Tech, missing: &[Tech]) -> String {
    format!(
        "{} [{}]",
        tech_name(tech),
        research_blocked_missing_text(missing)
    )
}

pub fn research_blocked_empty_text() -> &'static str {
    "No blocked techs remain."
}

pub fn unlock_preview_action_label(has_pinned_preview: bool, drifted: usize) -> &'static str {
    if drifted > 0 {
        "Refresh preview"
    } else if has_pinned_preview {
        "Pinned preview"
    } else {
        "Preview queue"
    }
}

pub fn unlock_preview_status_text(total: usize, drifted: usize) -> String {
    if drifted > 0 {
        format!("Pinned preview: {drifted}/{total} stale")
    } else {
        format!("Pinned preview: {total} current")
    }
}

pub fn unlock_preview_availability_text(tech: Tech, waiting_on_current_research: bool) -> String {
    if waiting_on_current_research {
        format!(
            "This preview becomes actionable after current research completes: {}.",
            tech_name(tech)
        )
    } else {
        format!(
            "This preview becomes actionable once {} is known.",
            tech_name(tech)
        )
    }
}

pub fn unlock_preview_drift_text(drifted: usize) -> String {
    format!("{drifted} staged base preview(s) drifted from current governor intent.")
}

pub fn unlock_preview_apply_tooltip(can_apply: bool, is_current: bool, tech: Tech) -> String {
    if can_apply && is_current {
        "Apply this staged queue to the base now.".to_string()
    } else if !is_current {
        "This staged queue drifted from current governor intent. Refresh the preview first."
            .to_string()
    } else {
        format!(
            "Unlock {} before applying this staged queue.",
            tech_name(tech)
        )
    }
}

pub fn unlock_preview_apply_all_tooltip(can_apply: bool, has_drifted: bool, tech: Tech) -> String {
    if can_apply && !has_drifted {
        "Apply all staged queues that are still valid.".to_string()
    } else if has_drifted {
        "One or more staged queues drifted from current governor intent. Refresh the preview first."
            .to_string()
    } else {
        format!("Unlock {} before applying staged queues.", tech_name(tech))
    }
}

pub fn convoy_route_kind_label(kind: ConvoyRouteKind) -> &'static str {
    match kind {
        ConvoyRouteKind::Trade => "Trade",
        ConvoyRouteKind::Freight => "Freight",
        ConvoyRouteKind::MilitarySupply => "Military Supply",
    }
}

pub fn production_cost(item: ProductionItem) -> i32 {
    content::production_cost(item)
}

pub fn facility_name(facility: Facility) -> &'static str {
    content::facility_name(facility)
}

pub fn tech_name(tech: Tech) -> &'static str {
    content::tech_name(tech)
}

pub fn tech_description(tech: Tech) -> &'static str {
    content::tech_description(tech)
}

pub fn tech_cost(tech: Tech) -> i32 {
    content::tech_cost(tech)
}

pub fn tech_unlock_summary(tech: Tech) -> String {
    let units = content::tech_enabled_unit_names(tech);
    let facilities = content::tech_enabled_facility_names(tech);
    let prerequisites = content::tech_prerequisites(tech);
    let mut parts = Vec::new();
    if !prerequisites.is_empty() {
        parts.push(format!(
            "Requires: {}",
            prerequisites
                .into_iter()
                .map(tech_name)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !units.is_empty() {
        parts.push(format!("Units: {}", units.join(", ")));
    }
    if !facilities.is_empty() {
        parts.push(format!("Facilities: {}", facilities.join(", ")));
    }
    if parts.is_empty() {
        "No direct prototype unlocks".to_string()
    } else {
        parts.join(" | ")
    }
}

pub fn tech_unlock_lines(tech: Tech) -> Vec<String> {
    let mut lines = Vec::new();
    let prerequisites = content::tech_prerequisites(tech);
    if !prerequisites.is_empty() {
        lines.push(format!(
            "Requires: {}",
            prerequisites
                .into_iter()
                .map(tech_name)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let units = content::tech_enabled_unit_names(tech);
    if !units.is_empty() {
        lines.push(format!("Units: {}", units.join(", ")));
    }

    let facilities = content::tech_enabled_facility_names(tech);
    if !facilities.is_empty() {
        lines.push(format!("Facilities: {}", facilities.join(", ")));
    }

    if lines.is_empty() {
        lines.push("No direct prototype unlocks".to_string());
    }

    lines
}

pub fn research_state_summary(known: usize, available: usize, blocked: usize) -> String {
    format!("Research state: {known} known / {available} available now / {blocked} blocked")
}

pub fn faction_leader_name(faction_name: &str) -> Option<&'static str> {
    content::runtime_faction_definition(faction_name).map(|definition| definition.leader.as_str())
}

pub fn faction_description(faction_name: &str) -> Option<&'static str> {
    content::runtime_faction_definition(faction_name)
        .map(|definition| definition.description.as_str())
}

pub fn faction_color_hex(faction_name: &str) -> Option<&'static str> {
    content::runtime_faction_definition(faction_name)
        .map(|definition| definition.color_hex.as_str())
}

pub fn faction_status_summary<'a>(
    faction_name: &'a str,
    base_count: usize,
    unit_count: usize,
    energy: i32,
    research: i32,
    current_tech: Tech,
    techs_discovered: i32,
    unrest_base_count: usize,
    recovery_base_count: usize,
    frontier_base_count: usize,
    rear_area_base_count: usize,
    psi_frontier_base_count: usize,
    warzone_base_count: usize,
    trade_route_count: usize,
    freight_route_count: usize,
    disrupted_route_count: usize,
    intercepted_route_count: usize,
    convoy_capacity_used: usize,
    convoy_capacity_total: usize,
) -> FactionStatusSummary<'a> {
    FactionStatusSummary {
        name: faction_name,
        leader: faction_leader_name(faction_name),
        description: faction_description(faction_name),
        color_hex: faction_color_hex(faction_name).unwrap_or("#d0d0d0"),
        base_count,
        unit_count,
        energy,
        research_progress: format_research_progress(research, tech_cost(current_tech)),
        current_tech: tech_name(current_tech),
        techs_discovered,
        unrest_base_count,
        recovery_base_count,
        frontier_base_count,
        rear_area_base_count,
        psi_frontier_base_count,
        warzone_base_count,
        trade_route_count,
        freight_route_count,
        disrupted_route_count,
        intercepted_route_count,
        convoy_capacity_used,
        convoy_capacity_total,
    }
}

pub fn format_yield_breakdown(yields: crate::Yields) -> String {
    format!(
        "N: {} | M: {} | E: {}",
        yields.nutrients, yields.minerals, yields.energy
    )
}

pub fn base_panel_summary<'a>(
    name: &'a str,
    owner_name: &'a str,
    population: i32,
    governor_mode: GovernorMode,
    trade_links: usize,
    unrest: i32,
    nutrients_stock: i32,
    minerals_stock: i32,
    raw_output: Yields,
    effective_output: Yields,
    production: ProductionItem,
    production_progress: i32,
    queue: &[ProductionItem],
    facilities: &[Facility],
) -> BasePanelSummary<'a> {
    BasePanelSummary {
        name,
        owner_name,
        population,
        governor_mode: governor_mode_label(governor_mode),
        trade_links,
        stability: format_stability(unrest),
        storage: format_base_storage(nutrients_stock, minerals_stock),
        output: format_yields(raw_output),
        effective_output: format_yields(effective_output),
        production: format!(
            "{} ({}/{})",
            production_name(production),
            production_progress,
            production_cost(production)
        ),
        queue: if queue.is_empty() {
            "Empty".to_string()
        } else {
            queue
                .iter()
                .map(|item| production_name(*item))
                .collect::<Vec<_>>()
                .join(" -> ")
        },
        facilities: if facilities.is_empty() {
            "None".to_string()
        } else {
            facilities
                .iter()
                .map(|facility| facility_name(*facility))
                .collect::<Vec<_>>()
                .join(", ")
        },
    }
}

pub fn unit_panel_summary<'a>(
    kind: UnitKind,
    unit_name: &'a str,
    owner_name: &'a str,
    experience: i32,
    x: usize,
    y: usize,
    moves_left: i32,
    hp: i32,
) -> UnitPanelSummary<'a> {
    UnitPanelSummary {
        unit_name,
        owner_name,
        rank: unit_rank_name(experience),
        role: unit_role_summary(kind),
        location: format!("{x}, {y}"),
        moves_left,
        hp,
    }
}

pub fn tile_panel_summary<'a>(
    terrain: Terrain,
    x: usize,
    y: usize,
    elevation: i32,
    moisture: i32,
    yields: Yields,
    improvement: Option<Improvement>,
) -> TilePanelSummary<'a> {
    TilePanelSummary {
        coordinates: format!("{x}, {y}"),
        terrain_name: terrain_name(terrain),
        elevation,
        moisture,
        yield_summary: format_yields(yields),
        improvement_name: improvement.map(improvement_name),
    }
}

pub fn ui_window_title() -> &'static str {
    content::ui_window_title()
}

pub fn ui_app_title() -> &'static str {
    content::ui_app_title()
}

pub fn ui_command_console_heading() -> String {
    content::ui_command_console_heading()
}

pub fn ui_gameplay_loop_heading() -> &'static str {
    "Gameplay loop:"
}

pub fn ui_gameplay_loop_steps() -> [&'static str; 6] {
    [
        "1. Found a base with C.",
        "2. Move S scouts into fog.",
        "3. Pop ? supply pods.",
        "4. Use F formers to terraform.",
        "5. Choose base production and research.",
        "6. End turn and survive Planet.",
    ]
}

pub fn ui_selection_heading() -> String {
    content::ui_selection_heading()
}

pub fn ui_research_heading() -> String {
    content::ui_research_heading()
}

pub fn ui_factions_heading() -> String {
    content::ui_factions_heading()
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::{
        available_research_label, build_availability_summary, format_recovery_status,
        format_unix_timestamp, governor_mode_mix_summary, production_dependency_text,
        production_role_badge, production_role_category, production_sequence_summary,
        production_tooltip_summary_with_status, research_available_empty_text,
        research_blocked_empty_text, research_blocked_label, research_blocked_missing_text,
        research_current_label, research_cycle_affected_base_label, research_focus_affected_label,
        research_state_summary, research_unlock_affects_text, research_unlock_impact_text,
        summarize_base_names, summarize_base_unlock_blocks, summarize_named_counts,
        summarize_production_item_counts, tech_unlock_lines, unlock_preview_action_label,
        unlock_preview_apply_all_label, unlock_preview_apply_all_tooltip,
        unlock_preview_apply_label, unlock_preview_apply_tooltip, unlock_preview_availability_text,
        unlock_preview_clear_label, unlock_preview_drift_text, unlock_preview_focus_label,
        unlock_preview_heading, unlock_preview_keep_open_label, unlock_preview_more_bases_text,
        unlock_preview_refresh_label, unlock_preview_row_text, unlock_preview_section_heading,
        unlock_preview_stage_all_log_label, unlock_preview_stage_log_label,
        unlock_preview_stale_label, unlock_preview_status_text,
    };
    use crate::{ProductionItem, SaveSlotCategory, SaveSlotListing, Tech};

    #[test]
    fn unix_timestamp_is_formatted_for_display() {
        assert_eq!(format_unix_timestamp(Some(0)), "1970-01-01 00:00 UTC");
    }

    #[test]
    fn recovery_status_is_human_readable() {
        assert_eq!(format_recovery_status(0), "Clean");
        assert_eq!(format_recovery_status(2), "Recovered (2)");
    }

    #[test]
    fn save_browser_summary_formats_category_and_empty_state() {
        let entry = SaveSlotListing {
            id: "autosave_1".to_string(),
            snapshot_path: "saves/autosave_1.json".into(),
            metadata_path: "saves/autosave_1.json.meta".into(),
            metadata: None,
            category: SaveSlotCategory::Empty,
        };
        let summary = super::save_browser_row_summary(&entry);
        assert_eq!(summary.display_name, "Empty");
        assert_eq!(summary.category_text, "Empty");
        assert!(summary.is_empty);
    }

    #[test]
    fn production_tooltip_reports_locked_status() {
        let tooltip = production_tooltip_summary_with_status(
            ProductionItem::ForwardDepot,
            false,
            Some(Tech::DoctrineMobility),
        );
        assert!(tooltip.contains("Status: Locked"));
        assert!(tooltip.contains("Missing tech: Doctrine: Mobility"));
    }

    #[test]
    fn production_role_helpers_are_stable() {
        assert_eq!(production_role_badge(ProductionItem::RaiderSpeeder), "RAID");
        assert_eq!(production_role_category(ProductionItem::PsiBeacon), "Psi");
        assert_eq!(
            production_dependency_text(ProductionItem::ForwardDepot),
            "Requires Doctrine: Mobility"
        );
    }

    #[test]
    fn tech_unlock_lines_surface_named_unlock_buckets() {
        let lines = tech_unlock_lines(Tech::GeneSplicing);
        assert!(lines
            .iter()
            .any(|line| line.contains("Facilities: Research Hospital")));
    }

    #[test]
    fn status_summary_helpers_are_stable() {
        assert_eq!(
            build_availability_summary(3, 5),
            "Build availability: 3 available / 5 locked"
        );
        assert_eq!(
            governor_mode_mix_summary(1, 2, 3, 4, 5),
            "1 balanced / 2 defense / 3 recovery / 4 economy / 5 logistics"
        );
        assert_eq!(
            research_state_summary(2, 4, 6),
            "Research state: 2 known / 4 available now / 6 blocked"
        );
        assert_eq!(
            summarize_named_counts(
                &[("Economy".to_string(), 2), ("Defense".to_string(), 1)],
                "None",
                4
            ),
            "Economy x2, Defense x1"
        );
        assert_eq!(
            summarize_production_item_counts(
                &[
                    (ProductionItem::Former, 2),
                    (ProductionItem::ScoutPatrol, 1)
                ],
                "None",
                4
            ),
            "Former x2, Scout Patrol x1"
        );
        assert_eq!(
            summarize_base_names(
                &["New Apogee".to_string(), "UN Headquarters".to_string()],
                "None",
                2
            ),
            "New Apogee, UN Headquarters"
        );
        assert_eq!(
            summarize_base_unlock_blocks(
                &[(
                    "New Apogee".to_string(),
                    ProductionItem::HologramTheatre,
                    Tech::PlanetaryNetworks
                )],
                "None",
                2
            ),
            "New Apogee -> Hologram Theatre (Planetary Networks)"
        );
        assert_eq!(
            production_sequence_summary(&[
                ProductionItem::HologramTheatre,
                ProductionItem::NetworkNode
            ]),
            "Hologram Theatre -> Network Node"
        );
        assert_eq!(
            research_current_label(Tech::PlanetaryNetworks, 3, 20),
            "Current: Planetary Networks (3/20)"
        );
        assert_eq!(
            available_research_label(Tech::PlanetaryNetworks, true),
            "Selected: Planetary Networks"
        );
        assert_eq!(
            unlock_preview_heading(Tech::PlanetaryNetworks),
            "Pinned unlock preview: Planetary Networks"
        );
        assert_eq!(
            unlock_preview_section_heading(),
            "Queue preview if research lands"
        );
        assert_eq!(
            unlock_preview_row_text("New Apogee", &[ProductionItem::HologramTheatre]),
            "New Apogee: Hologram Theatre"
        );
        assert_eq!(unlock_preview_stale_label(), "(stale)");
        assert_eq!(unlock_preview_focus_label(), "Focus");
        assert_eq!(unlock_preview_apply_label(), "Apply");
        assert_eq!(unlock_preview_keep_open_label(), "Keep preview open");
        assert_eq!(unlock_preview_stage_log_label(), "Stage in log");
        assert_eq!(
            unlock_preview_stage_all_log_label(),
            "Stage all previews in log"
        );
        assert_eq!(unlock_preview_refresh_label(), "Refresh");
        assert_eq!(unlock_preview_apply_all_label(), "Apply all");
        assert_eq!(unlock_preview_clear_label(), "Clear");
        assert_eq!(research_cycle_affected_base_label(), "Cycle affected base");
        assert_eq!(research_focus_affected_label(), "Focus affected");
        assert_eq!(research_unlock_impact_text(2), "Unblocks 2 base plan(s)");
        assert_eq!(
            research_unlock_affects_text("New Apogee -> Hologram Theatre (Planetary Networks)"),
            "Affects: New Apogee -> Hologram Theatre (Planetary Networks)"
        );
        assert_eq!(
            unlock_preview_more_bases_text(3),
            "3 more base(s) not shown"
        );
        assert_eq!(
            research_available_empty_text(),
            "No new techs are currently selectable."
        );
        assert_eq!(research_blocked_missing_text(&[]), "Progress blocked");
        assert_eq!(
            research_blocked_label(Tech::PlanetaryNetworks, &[Tech::SocialPsych]),
            "Planetary Networks [Missing: Social Psych]"
        );
        assert_eq!(research_blocked_empty_text(), "No blocked techs remain.");
        assert_eq!(unlock_preview_action_label(false, 0), "Preview queue");
        assert_eq!(unlock_preview_action_label(true, 0), "Pinned preview");
        assert_eq!(unlock_preview_action_label(true, 1), "Refresh preview");
        assert_eq!(
            unlock_preview_status_text(3, 0),
            "Pinned preview: 3 current"
        );
        assert_eq!(
            unlock_preview_status_text(3, 1),
            "Pinned preview: 1/3 stale"
        );
        assert_eq!(
            unlock_preview_availability_text(Tech::PlanetaryNetworks, true),
            "This preview becomes actionable after current research completes: Planetary Networks."
        );
        assert_eq!(
            unlock_preview_drift_text(2),
            "2 staged base preview(s) drifted from current governor intent."
        );
        assert_eq!(
            unlock_preview_apply_tooltip(true, true, Tech::PlanetaryNetworks),
            "Apply this staged queue to the base now."
        );
        assert_eq!(
            unlock_preview_apply_all_tooltip(false, true, Tech::PlanetaryNetworks),
            "One or more staged queues drifted from current governor intent. Refresh the preview first."
        );
    }
}

pub fn ui_event_log_heading() -> String {
    content::ui_event_log_heading()
}

pub fn ui_planet_heading() -> String {
    content::ui_planet_heading()
}

pub fn ui_victory_text() -> &'static str {
    content::ui_victory_text()
}

pub fn ui_defeat_text() -> &'static str {
    content::ui_defeat_text()
}

pub fn ui_warning_text() -> &'static str {
    content::ui_warning_text()
}

pub fn ui_accent_hex() -> &'static str {
    content::ui_accent_hex()
}

pub fn ui_secret_project_hex() -> &'static str {
    "#c8c8ff"
}

pub fn chassis_label(chassis: crate::Chassis) -> &'static str {
    match chassis {
        crate::Chassis::Infantry => "Infantry (Base)",
        crate::Chassis::Speeder => "Speeder (Fast)",
        crate::Chassis::Hovertank => "Hovertank (Heavy)",
        crate::Chassis::Aircraft => "Needlejet (Air)",
        crate::Chassis::Sea => "Foil (Sea)",
    }
}

pub fn weapon_label(weapon: crate::Weapon) -> String {
    match weapon {
        crate::Weapon::HandLaser(p) => format!("Hand Laser ({p})"),
        crate::Weapon::ResonanceLaser(p) => format!("Resonance Laser ({p})"),
        crate::Weapon::PlasmaBolt(p) => format!("Plasma Bolt ({p})"),
        crate::Weapon::PlanetBuster(p) => format!("Planet Buster ({p})"),
    }
}

pub fn armor_label(armor: crate::Armor) -> String {
    match armor {
        crate::Armor::SynthMetal(p) => format!("Synthmetal Armor ({p})"),
        crate::Armor::ResonanceArmor(p) => format!("Resonance Armor ({p})"),
        crate::Armor::PlasmaSteel(p) => format!("Plasma Steel ({p})"),
        crate::Armor::MonolithArmor(p) => format!("Monolith Armor ({p})"),
    }
}

pub fn base_status_tag_color_hex(kind: crate::game_state::BaseStatusTagKind) -> &'static str {
    match kind {
        crate::game_state::BaseStatusTagKind::Warning => ui_warning_hex(),
        crate::game_state::BaseStatusTagKind::Danger => ui_danger_hex(),
        crate::game_state::BaseStatusTagKind::Frontier => "#dc6e3c",
        crate::game_state::BaseStatusTagKind::Psi => "#aa5aa2",
        crate::game_state::BaseStatusTagKind::Saturated => "#dc6e3c",
        crate::game_state::BaseStatusTagKind::Tight => ui_warning_hex(),
    }
}

pub fn ui_warning_hex() -> &'static str {
    content::ui_warning_hex()
}

pub fn ui_danger_hex() -> &'static str {
    content::ui_danger_hex()
}

pub fn ui_success_hex() -> &'static str {
    content::ui_success_hex()
}

pub fn ui_panel_fill_hex() -> &'static str {
    content::ui_panel_fill_hex()
}

pub fn diplomacy_status_text(status: crate::DiplomacyStatus) -> &'static str {
    match status {
        crate::DiplomacyStatus::War => "WAR",
        crate::DiplomacyStatus::Truce => "Truce",
        crate::DiplomacyStatus::Treaty => "Treaty",
        crate::DiplomacyStatus::Pact => "PACT",
    }
}

pub fn diplomacy_status_color_hex(status: crate::DiplomacyStatus) -> &'static str {
    match status {
        crate::DiplomacyStatus::War => content::ui_danger_hex(),
        crate::DiplomacyStatus::Truce => content::ui_warning_hex(),
        crate::DiplomacyStatus::Treaty => "#46c8c8", // Teal
        crate::DiplomacyStatus::Pact => content::ui_success_hex(),
    }
}

pub fn diplomacy_attitude_text(attitude: i32) -> &'static str {
    if attitude <= -80 {
        "Vendetta"
    } else if attitude <= -40 {
        "Hostile"
    } else if attitude <= -10 {
        "Cold"
    } else if attitude < 10 {
        "Neutral"
    } else if attitude < 40 {
        "Cordial"
    } else if attitude < 80 {
        "Friendly"
    } else {
        "Inseparable"
    }
}
