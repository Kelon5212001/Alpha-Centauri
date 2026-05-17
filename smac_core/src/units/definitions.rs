use super::components::*;
use crate::content::unit_definition_by_id;
use crate::content_api::UnitDefinition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDesign {
    pub name: String,
    pub chassis: Chassis,
    pub weapon: Weapon,
    pub armor: Armor,
    pub cost: u16,
    pub abilities: Vec<Ability>,
}

impl UnitDesign {
    pub fn attack_strength(&self) -> u8 {
        match &self.weapon {
            Weapon::HandLaser(p)
            | Weapon::ResonanceLaser(p)
            | Weapon::PlasmaBolt(p)
            | Weapon::PlanetBuster(p) => *p,
        }
    }
    pub fn defense_strength(&self) -> u8 {
        match &self.armor {
            Armor::SynthMetal(p)
            | Armor::ResonanceArmor(p)
            | Armor::PlasmaSteel(p)
            | Armor::MonolithArmor(p) => *p,
        }
    }

    pub fn recompute_cost(&mut self) {
        let base_cost = match self.chassis {
            Chassis::Infantry => 10,
            Chassis::Speeder => 20,
            Chassis::Hovertank => 40,
            Chassis::Aircraft => 60,
            Chassis::Sea => 30,
        };
        self.cost =
            base_cost + (self.attack_strength() as u16 * 5) + (self.defense_strength() as u16 * 5);
    }

    pub fn display_state(&self) -> WorkshopDisplayState {
        WorkshopDisplayState {
            chassis_text: crate::presentation::chassis_label(self.chassis).to_string(),
            weapon_text: crate::presentation::weapon_label(self.weapon),
            armor_text: crate::presentation::armor_label(self.armor),
            attack_text: format!("Attack Rating: {}", self.attack_strength()),
            defense_text: format!("Defense Rating: {}", self.defense_strength()),
            cost_text: format!("Production Cost: {}", self.cost),
            chassis_options: vec![
                (
                    crate::Chassis::Infantry,
                    crate::presentation::chassis_label(crate::Chassis::Infantry),
                ),
                (
                    crate::Chassis::Speeder,
                    crate::presentation::chassis_label(crate::Chassis::Speeder),
                ),
                (
                    crate::Chassis::Hovertank,
                    crate::presentation::chassis_label(crate::Chassis::Hovertank),
                ),
            ],
            weapon_options: vec![
                (
                    crate::Weapon::HandLaser(1),
                    crate::presentation::weapon_label(crate::Weapon::HandLaser(1)),
                ),
                (
                    crate::Weapon::ResonanceLaser(6),
                    crate::presentation::weapon_label(crate::Weapon::ResonanceLaser(6)),
                ),
            ],
            armor_options: vec![
                (
                    crate::Armor::SynthMetal(1),
                    crate::presentation::armor_label(crate::Armor::SynthMetal(1)),
                ),
                (
                    crate::Armor::ResonanceArmor(2),
                    crate::presentation::armor_label(crate::Armor::ResonanceArmor(2)),
                ),
            ],
            ability_options: vec![
                (
                    crate::Ability::Trance,
                    "Trance (Bonus against psionic threats)",
                ),
                (
                    crate::Ability::Amphibious,
                    "Amphibious (No penalty for sea attack)",
                ),
                (
                    crate::Ability::Escort,
                    "Escort (Bonus to convoy protection)",
                ),
                (crate::Ability::Raid, "Raid (Bonus against exposed targets)"),
                (
                    crate::Ability::AirSuperiority,
                    "Air Superiority (Bonus against aircraft)",
                ),
                (
                    crate::Ability::DeepPressureHull,
                    "Deep Pressure Hull (Bonus at sea)",
                ),
                (
                    crate::Ability::CommJammer,
                    "Comm Jammer (Bonus against fast units)",
                ),
                (
                    crate::Ability::NonLethalMethods,
                    "Non-Lethal Methods (Police bonus)",
                ),
                (crate::Ability::Artillery, "Artillery (Can bombard bases)"),
                (crate::Ability::DropPod, "Drop Pod (Can deploy anywhere)"),
                (crate::Ability::Probe, "Probe (Covert operations)"),
                (crate::Ability::Transport, "Transport (Can carry units)"),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkshopDisplayState {
    pub chassis_text: String,
    pub weapon_text: String,
    pub armor_text: String,
    pub attack_text: String,
    pub defense_text: String,
    pub cost_text: String,
    pub chassis_options: Vec<(crate::Chassis, &'static str)>,
    pub weapon_options: Vec<(crate::Weapon, String)>,
    pub armor_options: Vec<(crate::Armor, String)>,
    pub ability_options: Vec<(crate::Ability, &'static str)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitDesignError {
    UnknownChassis {
        unit_id: String,
        chassis: String,
    },
    UnknownWeapon {
        unit_id: String,
        weapon_kind: String,
    },
    UnknownArmor {
        unit_id: String,
        armor_kind: String,
    },
    UnknownAbility {
        unit_id: String,
        ability: String,
    },
}

impl std::fmt::Display for UnitDesignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitDesignError::UnknownChassis { unit_id, chassis } => {
                write!(f, "unit '{unit_id}' uses unknown chassis '{chassis}'")
            }
            UnitDesignError::UnknownWeapon {
                unit_id,
                weapon_kind,
            } => write!(
                f,
                "unit '{unit_id}' uses unknown weapon kind '{weapon_kind}'"
            ),
            UnitDesignError::UnknownArmor {
                unit_id,
                armor_kind,
            } => write!(f, "unit '{unit_id}' uses unknown armor kind '{armor_kind}'"),
            UnitDesignError::UnknownAbility { unit_id, ability } => {
                write!(f, "unit '{unit_id}' uses unknown ability '{ability}'")
            }
        }
    }
}

impl std::error::Error for UnitDesignError {}

pub struct UnitPrototype;

impl UnitPrototype {
    pub fn scout_patrol() -> UnitDesign {
        try_design_from_content("scout_patrol")
            .expect("bundled unit content must decode scout_patrol")
    }
    pub fn colony_pod() -> UnitDesign {
        try_design_from_content("colony_pod").expect("bundled unit content must decode colony_pod")
    }
    pub fn former() -> UnitDesign {
        try_design_from_content("former").expect("bundled unit content must decode former")
    }
}

pub fn try_design_from_content(id: &str) -> Result<UnitDesign, UnitDesignError> {
    let definition = unit_definition_by_id(id);
    design_from_definition(id, definition)
}

pub(crate) fn design_from_definition(
    id: &str,
    definition: &UnitDefinition,
) -> Result<UnitDesign, UnitDesignError> {
    Ok(UnitDesign {
        name: definition.name.clone(),
        chassis: match definition.chassis.as_str() {
            "infantry" => Chassis::Infantry,
            "speeder" => Chassis::Speeder,
            "hovertank" => Chassis::Hovertank,
            "sea" => Chassis::Sea,
            "aircraft" => Chassis::Aircraft,
            other => {
                return Err(UnitDesignError::UnknownChassis {
                    unit_id: id.to_string(),
                    chassis: other.to_string(),
                })
            }
        },
        weapon: match Weapon::from_content_id(
            &definition.weapon_kind,
            definition.weapon_power,
        ) {
            Some(w) => w,
            None => {
                return Err(UnitDesignError::UnknownWeapon {
                    unit_id: id.to_string(),
                    weapon_kind: definition.weapon_kind.clone(),
                })
            }
        },
        armor: match Armor::from_content_id(
            &definition.armor_kind,
            definition.armor_power,
        ) {
            Some(a) => a,
            None => {
                return Err(UnitDesignError::UnknownArmor {
                    unit_id: id.to_string(),
                    armor_kind: definition.armor_kind.clone(),
                })
            }
        },
        cost: definition.cost,
        abilities: definition
            .abilities
            .iter()
            .map(|ability| match ability.as_str() {
                "trance" => Ok(Ability::Trance),
                "amphibious" => Ok(Ability::Amphibious),
                "escort" => Ok(Ability::Escort),
                "raid" => Ok(Ability::Raid),
                "air_superiority" => Ok(Ability::AirSuperiority),
                "deep_pressure_hull" => Ok(Ability::DeepPressureHull),
                "comm_jammer" => Ok(Ability::CommJammer),
                "non_lethal_methods" => Ok(Ability::NonLethalMethods),
                "artillery" => Ok(Ability::Artillery),
                "drop_pod" => Ok(Ability::DropPod),
                "probe" => Ok(Ability::Probe),
                "transport" => Ok(Ability::Transport),
                other => Err(UnitDesignError::UnknownAbility {
                    unit_id: id.to_string(),
                    ability: other.to_string(),
                }),
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}
