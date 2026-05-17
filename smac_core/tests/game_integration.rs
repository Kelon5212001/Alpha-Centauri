use smac_core::units::native::NativeType;
use smac_core::units::{try_design_from_content, UnitPrototype};
use smac_core::TechnologyTree;
use std::collections::HashSet;

#[test]
fn tech_progression() {
    let tree = TechnologyTree::new();
    let mut researched = HashSet::new();
    researched.insert("centauri_ecology".into());
    researched.insert("social_psych".into());
    let available = tree.get_available_technologies(&researched);
    assert!(available.iter().any(|t| t.id == "progenitor_psych"));
}

#[test]
fn native_stats() {
    let worm = NativeType::MindWorm.design();
    assert_eq!(worm.attack_strength(), 3);
}

#[test]
fn unit_prototype_costs() {
    assert!(UnitPrototype::colony_pod().cost > UnitPrototype::scout_patrol().cost);
}

#[test]
fn fallible_unit_design_loading_decodes_known_units() {
    let unit = try_design_from_content("isle_of_the_deep")
        .expect("bundled unit content should decode known units");
    assert_eq!(unit.attack_strength(), 4);
}

#[test]
fn advanced_resonance_unit_design_decodes_from_content() {
    let unit = try_design_from_content("resonance_laser")
        .expect("bundled advanced unit content should decode known units");
    assert_eq!(unit.attack_strength(), 4);
    assert_eq!(unit.defense_strength(), 2);
}
