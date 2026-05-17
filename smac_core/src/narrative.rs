use crate::model::Tech;

pub struct NarrativeTrigger {
    pub message: &'static str,
    pub condition: NarrativeCondition,
}

pub enum NarrativeCondition {
    Toxicity(i32),
    Technology(Tech),
    Turn(i32),
}

pub fn get_voice_of_planet_triggers() -> Vec<NarrativeTrigger> {
    vec![
        NarrativeTrigger {
            message: "I am the Planet. I feel your machines like parasites upon my skin.",
            condition: NarrativeCondition::Toxicity(30),
        },
        NarrativeTrigger {
            message: "The fungus spreads. It is my white blood cells, rushing to the site of infection.",
            condition: NarrativeCondition::Toxicity(60),
        },
        NarrativeTrigger {
            message: "You think you can master me? I was here before your sun was born.",
            condition: NarrativeCondition::Toxicity(90),
        },
        NarrativeTrigger {
            message: "Information networks... you weave a web of glass across the world. I see your thoughts.",
            condition: NarrativeCondition::Technology(Tech::InformationNetworks),
        },
        NarrativeTrigger {
            message: "Planetary networks... you are beginning to hear my voice. Do not look away.",
            condition: NarrativeCondition::Technology(Tech::PlanetaryNetworks),
        },
        NarrativeTrigger {
            message: "The first century on this world ends. You have survived. For now.",
            condition: NarrativeCondition::Turn(100),
        },
    ]
}
