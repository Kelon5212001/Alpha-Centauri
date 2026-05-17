use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chassis {
    Infantry,
    Speeder,
    Hovertank,
    Aircraft,
    Sea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weapon {
    HandLaser(u8),
    ResonanceLaser(u8),
    PlasmaBolt(u8),
    PlanetBuster(u8),
}

impl Weapon {
    pub fn content_id(self) -> &'static str {
        match self {
            Weapon::HandLaser(_) => "hand_laser",
            Weapon::ResonanceLaser(_) => "resonance_laser",
            Weapon::PlasmaBolt(_) => "plasma_bolt",
            Weapon::PlanetBuster(_) => "planet_buster",
        }
    }

    pub fn from_content_id(id: &str, power: u8) -> Option<Self> {
        match id {
            "hand_laser" => Some(Weapon::HandLaser(power)),
            "resonance_laser" => Some(Weapon::ResonanceLaser(power)),
            "plasma_bolt" => Some(Weapon::PlasmaBolt(power)),
            "planet_buster" => Some(Weapon::PlanetBuster(power)),
            _ => None,
        }
    }

    pub fn power(self) -> u8 {
        match self {
            Weapon::HandLaser(p)
            | Weapon::ResonanceLaser(p)
            | Weapon::PlasmaBolt(p)
            | Weapon::PlanetBuster(p) => p,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Armor {
    SynthMetal(u8),
    ResonanceArmor(u8),
    PlasmaSteel(u8),
    MonolithArmor(u8),
}

impl Armor {
    pub fn content_id(self) -> &'static str {
        match self {
            Armor::SynthMetal(_) => "synth_metal",
            Armor::ResonanceArmor(_) => "resonance_armor",
            Armor::PlasmaSteel(_) => "plasma_steel",
            Armor::MonolithArmor(_) => "monolith_armor",
        }
    }

    pub fn from_content_id(id: &str, power: u8) -> Option<Self> {
        match id {
            "synth_metal" => Some(Armor::SynthMetal(power)),
            "resonance_armor" => Some(Armor::ResonanceArmor(power)),
            "plasma_steel" => Some(Armor::PlasmaSteel(power)),
            "monolith_armor" => Some(Armor::MonolithArmor(power)),
            _ => None,
        }
    }

    pub fn power(self) -> u8 {
        match self {
            Armor::SynthMetal(p)
            | Armor::ResonanceArmor(p)
            | Armor::PlasmaSteel(p)
            | Armor::MonolithArmor(p) => p,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ability {
    Trance,
    Amphibious,
    Escort,
    Raid,
    AirSuperiority,
    DeepPressureHull,
    CommJammer,
    NonLethalMethods,
    Artillery,
    DropPod,
    Probe,
    Transport,
}

impl Ability {
    pub fn all() -> [Self; 12] {
        [
            Self::Trance,
            Self::Amphibious,
            Self::Escort,
            Self::Raid,
            Self::AirSuperiority,
            Self::DeepPressureHull,
            Self::CommJammer,
            Self::NonLethalMethods,
            Self::Artillery,
            Self::DropPod,
            Self::Probe,
            Self::Transport,
        ]
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Morale(pub u8);
