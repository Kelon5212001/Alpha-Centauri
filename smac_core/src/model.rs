use crate::units::definitions::UnitDesign;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const PLAYER_ID: usize = 0;
pub const AI_ID: usize = 1;
pub const NATIVE_ID: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Ocean,
    Flat,
    Rolling,
    Rocky,
    Fungus,
    Crater,
}

impl Terrain {
    pub fn yields(self) -> Yields {
        match self {
            Terrain::Ocean => Yields {
                nutrients: 1,
                minerals: 0,
                energy: 2,
            },
            Terrain::Flat => Yields {
                nutrients: 2,
                minerals: 1,
                energy: 1,
            },
            Terrain::Rolling => Yields {
                nutrients: 1,
                minerals: 2,
                energy: 1,
            },
            Terrain::Rocky => Yields {
                nutrients: 0,
                minerals: 3,
                energy: 0,
            },
            Terrain::Fungus => Yields {
                nutrients: 1,
                minerals: 0,
                energy: 1,
            },
            Terrain::Crater => Yields {
                nutrients: 0,
                minerals: 0,
                energy: 0,
            },
        }
    }

    pub fn is_land(self) -> bool {
        self != Terrain::Ocean && self != Terrain::Crater
    }

    pub fn is_ocean(self) -> bool {
        self == Terrain::Ocean || self == Terrain::Crater
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Improvement {
    Farm,
    Mine,
    Solar,
    Road,
    Condenser,
    EchelonMirror,
    Forest,
    ThermalBorehole,
}

impl Improvement {
    pub fn all() -> [Self; 8] {
        [
            Self::Farm,
            Self::Mine,
            Self::Solar,
            Self::Road,
            Self::Condenser,
            Self::EchelonMirror,
            Self::Forest,
            Self::ThermalBorehole,
        ]
    }

    pub fn content_id(self) -> &'static str {
        match self {
            Improvement::Farm => "farm",
            Improvement::Mine => "mine",
            Improvement::Solar => "solar_collector",
            Improvement::Road => "road",
            Improvement::Condenser => "condenser",
            Improvement::EchelonMirror => "echelon_mirror",
            Improvement::Forest => "forest",
            Improvement::ThermalBorehole => "thermal_borehole",
        }
    }

    pub fn from_content_id(value: &str) -> Option<Self> {
        match value {
            "farm" => Some(Improvement::Farm),
            "mine" => Some(Improvement::Mine),
            "solar_collector" => Some(Improvement::Solar),
            "road" => Some(Improvement::Road),
            "condenser" => Some(Improvement::Condenser),
            "echelon_mirror" => Some(Improvement::EchelonMirror),
            "forest" => Some(Improvement::Forest),
            "thermal_borehole" => Some(Improvement::ThermalBorehole),
            _ => None,
        }
    }

    pub fn yields(self) -> Yields {
        match self {
            Improvement::Farm => Yields {
                nutrients: 1,
                minerals: 0,
                energy: 0,
            },
            Improvement::Mine => Yields {
                nutrients: 0,
                minerals: 1,
                energy: 0,
            },
            Improvement::Solar => Yields {
                nutrients: 0,
                minerals: 0,
                energy: 2,
            },
            Improvement::Road => Yields {
                nutrients: 0,
                minerals: 0,
                energy: 0,
            },
            Improvement::Condenser => Yields {
                nutrients: 2,
                minerals: 0,
                energy: 0,
            },
            Improvement::EchelonMirror => Yields {
                nutrients: 0,
                minerals: 0,
                energy: 3,
            },
            Improvement::Forest => Yields {
                nutrients: 1,
                minerals: 2,
                energy: 1,
            },
            Improvement::ThermalBorehole => Yields {
                nutrients: 0,
                minerals: 6,
                energy: 6,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Yields {
    pub nutrients: i32,
    pub minerals: i32,
    pub energy: i32,
}

impl Yields {
    pub fn add(self, other: Yields) -> Yields {
        Yields {
            nutrients: self.nutrients + other.nutrients,
            minerals: self.minerals + other.minerals,
            energy: self.energy + other.energy,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub terrain: Terrain,
    pub elevation: i32,
    pub moisture: i32,
    pub unit: Option<usize>,
    pub base: Option<usize>,
    pub pod: bool,
    pub improvement: Option<Improvement>,
    pub explored_by_owner: BTreeSet<usize>,
    pub visible_by_owner: BTreeSet<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind {
    ColonyPod,
    SeaColonyPod,
    ScoutPatrol,
    Former,
    Speeder,
    ResonanceLaser,
    EscortSpeeder,
    RaiderSpeeder,
    TranceScout,
    GarrisonGuard,
    PsiSentinel,
    MindWorm,
    IsleOfTheDeep,
    Needlejet,
    ProbeTeam,
    SeaTransport,
    CustomUnit(UnitDesign),
}

impl UnitKind {
    pub fn content_id(self) -> &'static str {
        match self {
            UnitKind::ColonyPod => "colony_pod",
            UnitKind::SeaColonyPod => "sea_colony_pod",
            UnitKind::ScoutPatrol => "scout_patrol",
            UnitKind::Former => "former",
            UnitKind::Speeder => "speeder",
            UnitKind::ResonanceLaser => "resonance_laser",
            UnitKind::EscortSpeeder => "escort_speeder",
            UnitKind::RaiderSpeeder => "raider_speeder",
            UnitKind::TranceScout => "trance_scout",
            UnitKind::GarrisonGuard => "garrison_guard",
            UnitKind::PsiSentinel => "psi_sentinel",
            UnitKind::MindWorm => "mind_worm",
            UnitKind::IsleOfTheDeep => "isle_of_the_deep",
            UnitKind::Needlejet => "needlejet",
            UnitKind::ProbeTeam => "probe_team",
            UnitKind::SeaTransport => "sea_transport",
            UnitKind::CustomUnit(_) => "custom_unit",
        }
    }

    pub fn all() -> [UnitKind; 16] {
        [
            UnitKind::ColonyPod,
            UnitKind::SeaColonyPod,
            UnitKind::ScoutPatrol,
            UnitKind::Former,
            UnitKind::Speeder,
            UnitKind::ResonanceLaser,
            UnitKind::EscortSpeeder,
            UnitKind::RaiderSpeeder,
            UnitKind::TranceScout,
            UnitKind::GarrisonGuard,
            UnitKind::PsiSentinel,
            UnitKind::MindWorm,
            UnitKind::IsleOfTheDeep,
            UnitKind::Needlejet,
            UnitKind::ProbeTeam,
            UnitKind::SeaTransport,
        ]
    }

    pub fn max_moves(self) -> i32 {
        crate::content::unit_max_moves(self)
    }

    pub fn attack(self) -> i32 {
        crate::content::unit_attack(self)
    }

    pub fn defense(self) -> i32 {
        crate::content::unit_defense(self)
    }

    pub fn can_found_base(self) -> bool {
        self == UnitKind::ColonyPod || self == UnitKind::SeaColonyPod
    }

    pub fn can_terraform(self) -> bool {
        self == UnitKind::Former
    }

    pub fn from_content_id(value: &str) -> Option<Self> {
        match value {
            "colony_pod" => Some(UnitKind::ColonyPod),
            "sea_colony_pod" => Some(UnitKind::SeaColonyPod),
            "scout_patrol" => Some(UnitKind::ScoutPatrol),
            "former" => Some(UnitKind::Former),
            "speeder" => Some(UnitKind::Speeder),
            "resonance_laser" => Some(UnitKind::ResonanceLaser),
            "escort_speeder" => Some(UnitKind::EscortSpeeder),
            "raider_speeder" => Some(UnitKind::RaiderSpeeder),
            "trance_scout" => Some(UnitKind::TranceScout),
            "garrison_guard" => Some(UnitKind::GarrisonGuard),
            "psi_sentinel" => Some(UnitKind::PsiSentinel),
            "mind_worm" => Some(UnitKind::MindWorm),
            "isle_of_the_deep" => Some(UnitKind::IsleOfTheDeep),
            "needlejet" => Some(UnitKind::Needlejet),
            "probe_team" => Some(UnitKind::ProbeTeam),
            "sea_transport" => Some(UnitKind::SeaTransport),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnitActivity {
    #[default]
    None,
    Sentry,
    Patrol,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: usize,
    pub owner: usize,
    pub kind: UnitKind,
    pub design_index: usize,
    pub x: usize,
    pub y: usize,
    pub moves_left: i32,
    pub hp: i32,
    #[serde(default)]
    pub experience: i32,
    #[serde(default)]
    pub cargo_unit_ids: Vec<usize>,
    #[serde(default)]
    pub activity: UnitActivity,
    pub alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionItem {
    ScoutPatrol,
    ColonyPod,
    SeaColonyPod,
    Former,
    Speeder,
    ResonanceLaser,
    EscortSpeeder,
    RaiderSpeeder,
    TranceScout,
    GarrisonGuard,
    PsiSentinel,
    RecyclingTanks,
    PerimeterDefense,
    NetworkNode,
    RecreationCommons,
    Greenhouse,
    MineralRefinery,
    TradeExchange,
    FreightDepot,
    PatrolGrid,
    CommandCenter,
    FieldHospital,
    MilitaryAcademy,
    SensorArray,
    TransitHub,
    PsiBeacon,
    ForwardDepot,
    HologramTheatre,
    BioenhancementCenter,
    ResearchHospital,
    WeatherPattern,
    ClinicalImmortality,
    EmpathGuild,
    OrbitalElevator,
    ManifoldDrive,
    SingularityContainment,
    BlackHoleHarvester,
    ProbeTeam,
    SeaTransport,
    CustomUnit(usize),
    StockpileEnergy,
    SkyHydroponics,
    SolarTransmitter,
    OrbitalDefense,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecretProject {
    WeatherPattern,
    ClinicalImmortality,
    EmpathGuild,
    OrbitalElevator,
    ManifoldDrive,
    SingularityContainment,
    BlackHoleHarvester,
}

impl ProductionItem {
    pub fn content_id(self) -> &'static str {
        match self {
            ProductionItem::ScoutPatrol => "scout_patrol",
            ProductionItem::ColonyPod => "colony_pod",
            ProductionItem::SeaColonyPod => "sea_colony_pod",
            ProductionItem::Former => "former",
            ProductionItem::Speeder => "speeder",
            ProductionItem::ResonanceLaser => "resonance_laser",
            ProductionItem::EscortSpeeder => "escort_speeder",
            ProductionItem::RaiderSpeeder => "raider_speeder",
            ProductionItem::TranceScout => "trance_scout",
            ProductionItem::GarrisonGuard => "garrison_guard",
            ProductionItem::PsiSentinel => "psi_sentinel",
            ProductionItem::RecyclingTanks => "recycling_tanks",
            ProductionItem::PerimeterDefense => "perimeter_defense",
            ProductionItem::NetworkNode => "network_node",
            ProductionItem::RecreationCommons => "recreation_commons",
            ProductionItem::Greenhouse => "greenhouse",
            ProductionItem::MineralRefinery => "mineral_refinery",
            ProductionItem::TradeExchange => "trade_exchange",
            ProductionItem::FreightDepot => "freight_depot",
            ProductionItem::PatrolGrid => "patrol_grid",
            ProductionItem::CommandCenter => "command_center",
            ProductionItem::FieldHospital => "field_hospital",
            ProductionItem::MilitaryAcademy => "military_academy",
            ProductionItem::SensorArray => "sensor_array",
            ProductionItem::TransitHub => "transit_hub",
            ProductionItem::PsiBeacon => "psi_beacon",
            ProductionItem::ForwardDepot => "forward_depot",
            ProductionItem::HologramTheatre => "hologram_theatre",
            ProductionItem::BioenhancementCenter => "bioenhancement_center",
            ProductionItem::ResearchHospital => "research_hospital",
            ProductionItem::WeatherPattern => "weather_pattern",
            ProductionItem::ClinicalImmortality => "clinical_immortality",
            ProductionItem::EmpathGuild => "empath_guild",
            ProductionItem::OrbitalElevator => "orbital_elevator",
            ProductionItem::ManifoldDrive => "manifold_drive",
            ProductionItem::SingularityContainment => "singularity_containment",
            ProductionItem::BlackHoleHarvester => "black_hole_harvester",
            ProductionItem::ProbeTeam => "probe_team",
            ProductionItem::SeaTransport => "sea_transport",
            ProductionItem::CustomUnit(_) => "custom_unit",
            ProductionItem::StockpileEnergy => "stockpile_energy",
            ProductionItem::SkyHydroponics => "sky_hydroponics",
            ProductionItem::SolarTransmitter => "solar_transmitter",
            ProductionItem::OrbitalDefense => "orbital_defense",
        }
    }

    pub fn facility(self) -> Option<Facility> {
        match self {
            ProductionItem::RecyclingTanks => Some(Facility::RecyclingTanks),
            ProductionItem::PerimeterDefense => Some(Facility::PerimeterDefense),
            ProductionItem::NetworkNode => Some(Facility::NetworkNode),
            ProductionItem::RecreationCommons => Some(Facility::RecreationCommons),
            ProductionItem::Greenhouse => Some(Facility::Greenhouse),
            ProductionItem::MineralRefinery => Some(Facility::MineralRefinery),
            ProductionItem::TradeExchange => Some(Facility::TradeExchange),
            ProductionItem::FreightDepot => Some(Facility::FreightDepot),
            ProductionItem::PatrolGrid => Some(Facility::PatrolGrid),
            ProductionItem::CommandCenter => Some(Facility::CommandCenter),
            ProductionItem::FieldHospital => Some(Facility::FieldHospital),
            ProductionItem::MilitaryAcademy => Some(Facility::MilitaryAcademy),
            ProductionItem::SensorArray => Some(Facility::SensorArray),
            ProductionItem::TransitHub => Some(Facility::TransitHub),
            ProductionItem::PsiBeacon => Some(Facility::PsiBeacon),
            ProductionItem::ForwardDepot => Some(Facility::ForwardDepot),
            ProductionItem::HologramTheatre => Some(Facility::HologramTheatre),
            ProductionItem::BioenhancementCenter => Some(Facility::BioenhancementCenter),
            ProductionItem::ResearchHospital => Some(Facility::ResearchHospital),
            _ => None,
        }
    }

    pub fn all() -> [ProductionItem; 41] {
        [
            ProductionItem::ScoutPatrol,
            ProductionItem::ColonyPod,
            ProductionItem::Former,
            ProductionItem::Speeder,
            ProductionItem::ResonanceLaser,
            ProductionItem::EscortSpeeder,
            ProductionItem::RaiderSpeeder,
            ProductionItem::TranceScout,
            ProductionItem::GarrisonGuard,
            ProductionItem::PsiSentinel,
            ProductionItem::RecyclingTanks,
            ProductionItem::PerimeterDefense,
            ProductionItem::NetworkNode,
            ProductionItem::RecreationCommons,
            ProductionItem::Greenhouse,
            ProductionItem::MineralRefinery,
            ProductionItem::TradeExchange,
            ProductionItem::FreightDepot,
            ProductionItem::PatrolGrid,
            ProductionItem::CommandCenter,
            ProductionItem::FieldHospital,
            ProductionItem::MilitaryAcademy,
            ProductionItem::SensorArray,
            ProductionItem::TransitHub,
            ProductionItem::PsiBeacon,
            ProductionItem::ForwardDepot,
            ProductionItem::HologramTheatre,
            ProductionItem::BioenhancementCenter,
            ProductionItem::ResearchHospital,
            ProductionItem::WeatherPattern,
            ProductionItem::ClinicalImmortality,
            ProductionItem::EmpathGuild,
            ProductionItem::OrbitalElevator,
            ProductionItem::ManifoldDrive,
            ProductionItem::SingularityContainment,
            ProductionItem::BlackHoleHarvester,
            ProductionItem::ProbeTeam,
            ProductionItem::StockpileEnergy,
            ProductionItem::SkyHydroponics,
            ProductionItem::SolarTransmitter,
            ProductionItem::OrbitalDefense,
        ]
    }

    pub fn secret_project(self) -> Option<SecretProject> {
        match self {
            ProductionItem::WeatherPattern => Some(SecretProject::WeatherPattern),
            ProductionItem::ClinicalImmortality => Some(SecretProject::ClinicalImmortality),
            ProductionItem::EmpathGuild => Some(SecretProject::EmpathGuild),
            ProductionItem::OrbitalElevator => Some(SecretProject::OrbitalElevator),
            ProductionItem::ManifoldDrive => Some(SecretProject::ManifoldDrive),
            ProductionItem::SingularityContainment => Some(SecretProject::SingularityContainment),
            ProductionItem::BlackHoleHarvester => Some(SecretProject::BlackHoleHarvester),
            _ => None,
        }
    }

    pub fn from_facility(facility: Facility) -> Option<Self> {
        match facility {
            Facility::RecyclingTanks => Some(ProductionItem::RecyclingTanks),
            Facility::PerimeterDefense => Some(ProductionItem::PerimeterDefense),
            Facility::NetworkNode => Some(ProductionItem::NetworkNode),
            Facility::RecreationCommons => Some(ProductionItem::RecreationCommons),
            Facility::Greenhouse => Some(ProductionItem::Greenhouse),
            Facility::MineralRefinery => Some(ProductionItem::MineralRefinery),
            Facility::TradeExchange => Some(ProductionItem::TradeExchange),
            Facility::FreightDepot => Some(ProductionItem::FreightDepot),
            Facility::PatrolGrid => Some(ProductionItem::PatrolGrid),
            Facility::CommandCenter => Some(ProductionItem::CommandCenter),
            Facility::FieldHospital => Some(ProductionItem::FieldHospital),
            Facility::MilitaryAcademy => Some(ProductionItem::MilitaryAcademy),
            Facility::SensorArray => Some(ProductionItem::SensorArray),
            Facility::TransitHub => Some(ProductionItem::TransitHub),
            Facility::PsiBeacon => Some(ProductionItem::PsiBeacon),
            Facility::ForwardDepot => Some(ProductionItem::ForwardDepot),
            Facility::HologramTheatre => Some(ProductionItem::HologramTheatre),
            Facility::BioenhancementCenter => Some(ProductionItem::BioenhancementCenter),
            Facility::ResearchHospital => Some(ProductionItem::ResearchHospital),
        }
    }

    pub fn from_content_id(value: &str) -> Option<Self> {
        match value {
            "scout_patrol" => Some(ProductionItem::ScoutPatrol),
            "colony_pod" => Some(ProductionItem::ColonyPod),
            "sea_colony_pod" => Some(ProductionItem::SeaColonyPod),
            "former" => Some(ProductionItem::Former),
            "speeder" => Some(ProductionItem::Speeder),
            "resonance_laser" => Some(ProductionItem::ResonanceLaser),
            "escort_speeder" => Some(ProductionItem::EscortSpeeder),
            "raider_speeder" => Some(ProductionItem::RaiderSpeeder),
            "trance_scout" => Some(ProductionItem::TranceScout),
            "garrison_guard" => Some(ProductionItem::GarrisonGuard),
            "psi_sentinel" => Some(ProductionItem::PsiSentinel),
            "recycling_tanks" => Some(ProductionItem::RecyclingTanks),
            "perimeter_defense" => Some(ProductionItem::PerimeterDefense),
            "network_node" => Some(ProductionItem::NetworkNode),
            "recreation_commons" => Some(ProductionItem::RecreationCommons),
            "greenhouse" => Some(ProductionItem::Greenhouse),
            "mineral_refinery" => Some(ProductionItem::MineralRefinery),
            "trade_exchange" => Some(ProductionItem::TradeExchange),
            "freight_depot" => Some(ProductionItem::FreightDepot),
            "patrol_grid" => Some(ProductionItem::PatrolGrid),
            "command_center" => Some(ProductionItem::CommandCenter),
            "field_hospital" => Some(ProductionItem::FieldHospital),
            "military_academy" => Some(ProductionItem::MilitaryAcademy),
            "sensor_array" => Some(ProductionItem::SensorArray),
            "transit_hub" => Some(ProductionItem::TransitHub),
            "psi_beacon" => Some(ProductionItem::PsiBeacon),
            "forward_depot" => Some(ProductionItem::ForwardDepot),
            "hologram_theatre" => Some(ProductionItem::HologramTheatre),
            "bioenhancement_center" => Some(ProductionItem::BioenhancementCenter),
            "research_hospital" => Some(ProductionItem::ResearchHospital),
            "weather_pattern" => Some(ProductionItem::WeatherPattern),
            "clinical_immortality" => Some(ProductionItem::ClinicalImmortality),
            "empath_guild" => Some(ProductionItem::EmpathGuild),
            "orbital_elevator" => Some(ProductionItem::OrbitalElevator),
            "manifold_drive" => Some(ProductionItem::ManifoldDrive),
            "singularity_containment" => Some(ProductionItem::SingularityContainment),
            "black_hole_harvester" => Some(ProductionItem::BlackHoleHarvester),
            "probe_team" => Some(ProductionItem::ProbeTeam),
            "sea_transport" => Some(ProductionItem::SeaTransport),
            "stockpile_energy" => Some(ProductionItem::StockpileEnergy),
            "sky_hydroponics" => Some(ProductionItem::SkyHydroponics),
            "solar_transmitter" => Some(ProductionItem::SolarTransmitter),
            "orbital_defense" => Some(ProductionItem::OrbitalDefense),
            _ => None,
        }
    }
}

impl SecretProject {
    pub fn content_id(self) -> &'static str {
        match self {
            SecretProject::WeatherPattern => "weather_pattern",
            SecretProject::ClinicalImmortality => "clinical_immortality",
            SecretProject::EmpathGuild => "empath_guild",
            SecretProject::OrbitalElevator => "orbital_elevator",
            SecretProject::ManifoldDrive => "manifold_drive",
            SecretProject::SingularityContainment => "singularity_containment",
            SecretProject::BlackHoleHarvester => "black_hole_harvester",
        }
    }

    pub fn from_content_id(value: &str) -> Option<Self> {
        match value {
            "weather_pattern" => Some(SecretProject::WeatherPattern),
            "clinical_immortality" => Some(SecretProject::ClinicalImmortality),
            "empath_guild" => Some(SecretProject::EmpathGuild),
            "orbital_elevator" => Some(SecretProject::OrbitalElevator),
            "manifold_drive" => Some(SecretProject::ManifoldDrive),
            "singularity_containment" => Some(SecretProject::SingularityContainment),
            "black_hole_harvester" => Some(SecretProject::BlackHoleHarvester),
            _ => None,
        }
    }

    pub fn all() -> [SecretProject; 7] {
        [
            SecretProject::WeatherPattern,
            SecretProject::ClinicalImmortality,
            SecretProject::EmpathGuild,
            SecretProject::OrbitalElevator,
            SecretProject::ManifoldDrive,
            SecretProject::SingularityContainment,
            SecretProject::BlackHoleHarvester,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Facility {
    RecyclingTanks,
    PerimeterDefense,
    NetworkNode,
    RecreationCommons,
    Greenhouse,
    MineralRefinery,
    TradeExchange,
    FreightDepot,
    PatrolGrid,
    CommandCenter,
    FieldHospital,
    MilitaryAcademy,
    SensorArray,
    TransitHub,
    PsiBeacon,
    ForwardDepot,
    HologramTheatre,
    BioenhancementCenter,
    ResearchHospital,
}

impl Facility {
    pub fn content_id(self) -> &'static str {
        match self {
            Facility::RecyclingTanks => "recycling_tanks",
            Facility::PerimeterDefense => "perimeter_defense",
            Facility::NetworkNode => "network_node",
            Facility::RecreationCommons => "recreation_commons",
            Facility::Greenhouse => "greenhouse",
            Facility::MineralRefinery => "mineral_refinery",
            Facility::TradeExchange => "trade_exchange",
            Facility::FreightDepot => "freight_depot",
            Facility::PatrolGrid => "patrol_grid",
            Facility::CommandCenter => "command_center",
            Facility::FieldHospital => "field_hospital",
            Facility::MilitaryAcademy => "military_academy",
            Facility::SensorArray => "sensor_array",
            Facility::TransitHub => "transit_hub",
            Facility::PsiBeacon => "psi_beacon",
            Facility::ForwardDepot => "forward_depot",
            Facility::HologramTheatre => "hologram_theatre",
            Facility::BioenhancementCenter => "bioenhancement_center",
            Facility::ResearchHospital => "research_hospital",
        }
    }

    pub fn all() -> [Facility; 19] {
        [
            Facility::RecyclingTanks,
            Facility::PerimeterDefense,
            Facility::NetworkNode,
            Facility::RecreationCommons,
            Facility::Greenhouse,
            Facility::MineralRefinery,
            Facility::TradeExchange,
            Facility::FreightDepot,
            Facility::PatrolGrid,
            Facility::CommandCenter,
            Facility::FieldHospital,
            Facility::MilitaryAcademy,
            Facility::SensorArray,
            Facility::TransitHub,
            Facility::PsiBeacon,
            Facility::ForwardDepot,
            Facility::HologramTheatre,
            Facility::BioenhancementCenter,
            Facility::ResearchHospital,
        ]
    }

    pub fn from_content_id(value: &str) -> Option<Self> {
        match value {
            "recycling_tanks" => Some(Facility::RecyclingTanks),
            "perimeter_defense" => Some(Facility::PerimeterDefense),
            "network_node" => Some(Facility::NetworkNode),
            "recreation_commons" => Some(Facility::RecreationCommons),
            "greenhouse" => Some(Facility::Greenhouse),
            "mineral_refinery" => Some(Facility::MineralRefinery),
            "trade_exchange" => Some(Facility::TradeExchange),
            "freight_depot" => Some(Facility::FreightDepot),
            "patrol_grid" => Some(Facility::PatrolGrid),
            "command_center" => Some(Facility::CommandCenter),
            "field_hospital" => Some(Facility::FieldHospital),
            "military_academy" => Some(Facility::MilitaryAcademy),
            "sensor_array" => Some(Facility::SensorArray),
            "transit_hub" => Some(Facility::TransitHub),
            "psi_beacon" => Some(Facility::PsiBeacon),
            "forward_depot" => Some(Facility::ForwardDepot),
            "hologram_theatre" => Some(Facility::HologramTheatre),
            "bioenhancement_center" => Some(Facility::BioenhancementCenter),
            "research_hospital" => Some(Facility::ResearchHospital),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tech {
    CentauriEcology,
    SocialPsych,
    ProgenitorPsych,
    FieldModulation,
    PlanetaryNetworks,
    SecretsOfTheHumanBrain,
    GeneSplicing,
    IndustrialBase,
    DoctrineMobility,
    DoctrineInitiative,
    DoctrineAirPower,
    InformationNetworks,
    Biogenetics,
    SecretsOfPlanet,
    OrbitalMechanics,
    AdvancedFieldTheory,
    SingularityPhysics,
}

impl Tech {
    pub fn content_id(self) -> &'static str {
        match self {
            Tech::CentauriEcology => "centauri_ecology",
            Tech::SocialPsych => "social_psych",
            Tech::ProgenitorPsych => "progenitor_psych",
            Tech::FieldModulation => "field_modulation",
            Tech::PlanetaryNetworks => "planetary_networks",
            Tech::SecretsOfTheHumanBrain => "secrets_of_the_human_brain",
            Tech::GeneSplicing => "gene_splicing",
            Tech::IndustrialBase => "industrial_base",
            Tech::DoctrineMobility => "doctrine_mobility",
            Tech::DoctrineInitiative => "doctrine_initiative",
            Tech::DoctrineAirPower => "doctrine_air_power",
            Tech::InformationNetworks => "information_networks",
            Tech::Biogenetics => "biogenetics",
            Tech::SecretsOfPlanet => "secrets_of_planet",
            Tech::OrbitalMechanics => "orbital_mechanics",
            Tech::AdvancedFieldTheory => "advanced_field_theory",
            Tech::SingularityPhysics => "singularity_physics",
        }
    }

    pub fn all() -> [Tech; 15] {
        [
            Tech::CentauriEcology,
            Tech::SocialPsych,
            Tech::ProgenitorPsych,
            Tech::FieldModulation,
            Tech::PlanetaryNetworks,
            Tech::SecretsOfTheHumanBrain,
            Tech::GeneSplicing,
            Tech::IndustrialBase,
            Tech::DoctrineMobility,
            Tech::InformationNetworks,
            Tech::Biogenetics,
            Tech::SecretsOfPlanet,
            Tech::OrbitalMechanics,
            Tech::AdvancedFieldTheory,
            Tech::SingularityPhysics,
        ]
    }

    pub fn from_content_name(value: &str) -> Option<Self> {
        match value {
            "Centauri Ecology" => Some(Tech::CentauriEcology),
            "Social Psych" => Some(Tech::SocialPsych),
            "Progenitor Psych" => Some(Tech::ProgenitorPsych),
            "Field Modulation" => Some(Tech::FieldModulation),
            "Planetary Networks" => Some(Tech::PlanetaryNetworks),
            "Secrets of the Human Brain" => Some(Tech::SecretsOfTheHumanBrain),
            "Gene Splicing" => Some(Tech::GeneSplicing),
            "Industrial Base" => Some(Tech::IndustrialBase),
            "Doctrine: Mobility" => Some(Tech::DoctrineMobility),
            "Information Networks" => Some(Tech::InformationNetworks),
            "Biogenetics" => Some(Tech::Biogenetics),
            "Secrets of Planet" => Some(Tech::SecretsOfPlanet),
            "Orbital Mechanics" => Some(Tech::OrbitalMechanics),
            "Advanced Field Theory" => Some(Tech::AdvancedFieldTheory),
            "Singularity Physics" => Some(Tech::SingularityPhysics),
            _ => None,
        }
    }

    pub fn from_content_id(value: &str) -> Option<Self> {
        match value {
            "centauri_ecology" => Some(Tech::CentauriEcology),
            "social_psych" => Some(Tech::SocialPsych),
            "progenitor_psych" => Some(Tech::ProgenitorPsych),
            "field_modulation" => Some(Tech::FieldModulation),
            "planetary_networks" => Some(Tech::PlanetaryNetworks),
            "secrets_of_the_human_brain" => Some(Tech::SecretsOfTheHumanBrain),
            "gene_splicing" => Some(Tech::GeneSplicing),
            "industrial_base" => Some(Tech::IndustrialBase),
            "doctrine_mobility" => Some(Tech::DoctrineMobility),
            "doctrine_initiative" => Some(Tech::DoctrineInitiative),
            "doctrine_air_power" => Some(Tech::DoctrineAirPower),
            "information_networks" => Some(Tech::InformationNetworks),
            "biogenetics" => Some(Tech::Biogenetics),
            "secrets_of_planet" => Some(Tech::SecretsOfPlanet),
            "orbital_mechanics" => Some(Tech::OrbitalMechanics),
            "advanced_field_theory" => Some(Tech::AdvancedFieldTheory),
            "singularity_physics" => Some(Tech::SingularityPhysics),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernorMode {
    #[default]
    Off,
    Balanced,
    Recovery,
    Defense,
    Economy,
    Logistics,
    MachinePolity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseAreaRole {
    RearArea,
    Frontier,
    PsiFrontier,
    Warzone,
}

impl GovernorMode {
    pub fn all() -> [GovernorMode; 7] {
        [
            GovernorMode::Off,
            GovernorMode::Balanced,
            GovernorMode::Recovery,
            GovernorMode::Defense,
            GovernorMode::Economy,
            GovernorMode::Logistics,
            GovernorMode::MachinePolity,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Base {
    pub id: usize,
    pub owner: usize,
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub population: i32,
    pub nutrients_stock: i32,
    pub minerals_stock: i32,
    pub production: ProductionItem,
    #[serde(default)]
    pub production_queue: Vec<ProductionItem>,
    #[serde(default)]
    pub facilities: Vec<Facility>,
    #[serde(default)]
    pub governor_mode: GovernorMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConvoyRouteKind {
    Trade,
    Freight,
    MilitarySupply,
}

impl Default for ConvoyRouteKind {
    fn default() -> Self {
        Self::Trade
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvoyRoute {
    pub base_a_id: usize,
    pub base_b_id: usize,
    #[serde(default)]
    pub kind: ConvoyRouteKind,
    #[serde(default = "default_convoy_integrity")]
    pub integrity: u8,
}

fn default_convoy_integrity() -> u8 {
    3
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvoyRouteSummary {
    pub base_a_id: usize,
    pub base_b_id: usize,
    pub base_a_name: String,
    pub base_b_name: String,
    pub kind: ConvoyRouteKind,
    pub disrupted: bool,
    pub intercepted: bool,
    pub integrity: u8,
    pub protected: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactionAttributes {
    pub growth: i32,
    pub industry: i32,
    pub economy: i32,
    pub efficiency: i32,
    pub support: i32,
    pub morale: i32,
    pub police: i32,
    pub research: i32,
    pub probe: i32,
    pub planet: i32,
    #[serde(default)]
    pub free_facilities: Vec<Facility>,
    #[serde(default)]
    pub facility_equivalents: Vec<Facility>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Politics {
    Frontier,
    Police,
    Democratic,
    Fundamentalist,
}

impl Default for Politics {
    fn default() -> Self {
        Self::Frontier
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Economics {
    Simple,
    FreeMarket,
    Planned,
    Green,
}

impl Default for Economics {
    fn default() -> Self {
        Self::Simple
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Values {
    Survival,
    Wealth,
    Knowledge,
    Power,
}

impl Default for Values {
    fn default() -> Self {
        Self::Survival
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FutureSociety {
    None,
    Cybernetic,
    ThoughtControl,
    Eudaimonic,
}

impl Default for FutureSociety {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SocialEngineering {
    pub politics: Politics,
    pub economics: Economics,
    pub values: Values,
    pub future: FutureSociety,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactionPersonality {
    pub aggression: i32,
    pub tech_bias: i32,
    pub diplomatic_tone: i32,
    pub expansion_bias: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertPriority {
    Critical,
    High,
    Medium,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionableAlert {
    pub priority: AlertPriority,
    pub message: String,
    pub location: Option<(usize, usize)>,
    pub base_id: Option<usize>,
    pub unit_id: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnSummary {
    pub turn: i32,
    pub alerts: Vec<ActionableAlert>,
    pub event_highlights: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub id: usize,
    pub name: String,
    pub energy: i32,
    pub research: i32,
    pub techs_discovered: i32,
    pub is_ai: bool,
    pub known_techs: Vec<Tech>,
    pub current_research: Tech,
    #[serde(default)]
    pub unit_designs: Vec<UnitDesign>,
    #[serde(default)]
    pub food_security: i32,
    #[serde(default)]
    pub ai_dependence: i32,
    #[serde(default)]
    pub orbital_index: i32,
    #[serde(default)]
    pub sky_hydroponics: i32,
    #[serde(default)]
    pub solar_transmitters: i32,
    #[serde(default)]
    pub orbital_defenses: i32,
    #[serde(default)]
    pub planet_toxicity: i32,
    #[serde(default)]
    pub base_attributes: FactionAttributes,
    #[serde(default)]
    pub social_engineering: SocialEngineering,
    #[serde(default)]
    pub personality: FactionPersonality,
    pub headquarters_base_id: Option<usize>,
}

impl Faction {
    pub fn total_population(&self, state: &GameState) -> i32 {
        state
            .bases
            .iter()
            .filter(|b| b.owner == self.id)
            .map(|b| b.population)
            .sum()
    }

    pub fn effective_attributes(&self) -> FactionAttributes {
        let mut total = self.base_attributes.clone();

        match self.social_engineering.politics {
            Politics::Frontier => {}
            Politics::Police => {
                total.support += 2;
                total.police += 2;
                total.efficiency -= 2;
            }
            Politics::Democratic => {
                total.growth += 2;
                total.efficiency += 2;
                total.support -= 2;
            }
            Politics::Fundamentalist => {
                total.morale += 2;
                total.probe += 2;
                total.research -= 2;
            }
        }

        match self.social_engineering.economics {
            Economics::Simple => {}
            Economics::FreeMarket => {
                total.economy += 2;
                total.efficiency += 2;
                total.planet -= 2;
            }
            Economics::Planned => {
                total.industry += 1;
                total.growth += 1;
                total.efficiency -= 2;
            }
            Economics::Green => {
                total.efficiency += 2;
                total.planet += 1;
                total.growth -= 2;
            }
        }

        match self.social_engineering.values {
            Values::Survival => {}
            Values::Wealth => {
                total.economy += 1;
                total.industry += 1;
                total.morale -= 1;
            }
            Values::Knowledge => {
                total.research += 2;
                total.efficiency += 1;
                total.probe -= 1;
            }
            Values::Power => {
                total.morale += 2;
                total.support += 1;
                total.industry -= 1;
            }
        }

        match self.social_engineering.future {
            FutureSociety::None => {}
            FutureSociety::Cybernetic => {
                total.efficiency += 2;
                total.research += 2;
                total.planet += 1;
                total.police -= 1;
            }
            FutureSociety::ThoughtControl => {
                total.police += 2;
                total.morale += 2;
                total.probe += 2;
                total.research -= 1;
            }
            FutureSociety::Eudaimonic => {
                total.economy += 2;
                total.efficiency += 2;
                total.growth += 2;
                total.industry -= 1;
            }
        }

        total
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOver {
    PlayerWonConquest,
    PlayerWonEconomic,
    PlayerWonTranscendence,
    PlayerWonSpaceTranscendence,
    PlayerWonBlackHoleHarvesting,
    AiWonConquest,
    AiWonEconomic,
    AiWonTranscendence,
    AiWonSpaceTranscendence,
    AiWonBlackHoleHarvesting,
    DiplomaticVictory,
    CouncilGovernorElected,
    PlanetUnited,
    PlayerLost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiplomacyStatus {
    War,
    Truce,
    Treaty,
    Pact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiplomaticRelation {
    pub status: DiplomacyStatus,
    pub attitude: i32, // -100 to 100
}

impl Default for DiplomaticRelation {
    fn default() -> Self {
        Self {
            status: DiplomacyStatus::Truce, // Start at Truce until they meet
            attitude: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventCategory {
    General,
    Crisis,
    SecretProject,
    Diplomacy,
    Economics,
    Narrative,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventLogEntry {
    pub category: EventCategory,
    pub message: String,
    pub turn: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandCenterTurnTrace {
    pub turn: i32,
    pub owner: usize,
    pub base_id: usize,
    pub base_name: String,
    pub post_production_stock: i32,
    pub post_interdiction_stock: i32,
    pub upkeep_drain: i32,
    pub upkeep_order_index: Option<usize>,
    pub end_stock: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
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
    pub relations: Vec<Vec<DiplomaticRelation>>,
    #[serde(default)]
    pub built_secret_projects: Vec<(SecretProject, usize)>,
    pub log: Vec<EventLogEntry>,
    #[serde(default)]
    pub pending_diplomacy_offers: Vec<(usize, usize, DiplomacyStatus)>,
    #[serde(default)]
    pub pending_tech_trades: Vec<(usize, usize, Tech, Tech)>,
    #[serde(default)]
    pub pending_demands: Vec<(usize, usize, DemandKind)>,
    #[serde(default)]
    pub triggered_narratives: BTreeSet<String>,
    #[serde(default)]
    pub council: CouncilState,
    pub game_over: Option<GameOver>,
    #[serde(default, skip)]
    pub command_center_turn_traces: Vec<CommandCenterTurnTrace>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CouncilVote {
    pub faction_id: usize,
    pub candidate_id: usize,
    pub weight: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CouncilState {
    pub is_active: bool,
    pub governor_id: Option<usize>,
    pub last_meeting_turn: i32,
    pub pending_votes: Vec<CouncilVote>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeAction {
    StealTech,
    SabotageFacility,
    SubvertUnit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemandKind {
    Technology(Tech),
    Energy(i32),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameAction {
    MoveUnit {
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    },
    FoundBase {
        unit_id: usize,
    },
    BuildImprovement {
        unit_id: usize,
        improvement: Improvement,
    },
    SetBaseProduction {
        base_id: usize,
        item: ProductionItem,
    },
    QueueBaseProduction {
        base_id: usize,
        item: ProductionItem,
    },
    ChooseResearch {
        owner: usize,
        tech: Tech,
    },
    DesignUnit {
        owner: usize,
        design: UnitDesign,
    },
    UpdateDiplomacy {
        faction_a: usize,
        faction_b: usize,
        status: DiplomacyStatus,
    },
    ProposeDiplomacy {
        proposer: usize,
        receiver: usize,
        status: DiplomacyStatus,
    },
    RespondDiplomacy {
        proposer: usize,
        receiver: usize,
        status: DiplomacyStatus,
        accept: bool,
    },
    ProposeTechTrade {
        proposer: usize,
        receiver: usize,
        offered_tech: Tech,
        requested_tech: Tech,
    },
    RespondTechTrade {
        proposer: usize,
        receiver: usize,
        offered_tech: Tech,
        requested_tech: Tech,
        accept: bool,
    },
    MakeDemand {
        proposer: usize,
        receiver: usize,
        demand: DemandKind,
    },
    RespondDemand {
        proposer: usize,
        receiver: usize,
        demand: DemandKind,
        accept: bool,
    },
    LoadUnit {
        unit_id: usize,
        transport_id: usize,
    },
    UnloadUnit {
        unit_id: usize,
        transport_id: usize,
        target_x: usize,
        target_y: usize,
    },
    SetUnitActivity {
        unit_id: usize,
        activity: UnitActivity,
    },
    RushBuild {
        base_id: usize,
    },
    UpgradeUnit {
        unit_id: usize,
        new_design: UnitDesign,
    },
    CallCouncil,
    VoteForGovernor {
        voter_id: usize,
        candidate_id: usize,
    },
    VoteForSupremeLeader {
        voter_id: usize,
        candidate_id: usize,
    },
    ChooseSocialEngineering {
        owner: usize,
        politics: Option<Politics>,
        economics: Option<Economics>,
        values: Option<Values>,
        future: Option<FutureSociety>,
    },
    PerformProbeAction {
        unit_id: usize,
        target_x: usize,
        target_y: usize,
        action: ProbeAction,
    },
    EndTurn,
}
