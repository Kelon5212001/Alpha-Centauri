use smac_core::content_api::{
    ai_attack_bias, ai_colony_base_target, ai_expansion_base_target, ai_exploration_bias,
    ai_native_spawn_noise_salt, ai_preferred_production, base_growth_nutrients_threshold,
    build_runtime_factions, default_starting_scenario, facility_maintenance, facility_name,
    facility_psi_support_bonus, facility_repair_bonus, facility_stability_bonus,
    facility_training_bonus, faction_definition_by_name, forced_land_patch_radius,
    load_facility_definitions, load_faction_definitions, load_production_definitions,
    load_runtime_rules, load_runtime_tech_definitions, load_ui_theme_definition,
    load_unit_definitions, map_flat_moisture_threshold, map_fungus_threshold, map_ocean_threshold,
    map_pod_spawn_threshold, map_rocky_threshold, next_base_name_for_faction, production_cost,
    production_facility, production_name, production_unit_kind,
    runtime_faction_definition_by_owner, runtime_roles, supply_pod_energy_reward, tech_cost,
    tech_description, tech_is_available, tech_name, tech_prerequisites,
    try_default_runtime_faction_setups, try_default_starting_scenario,
    try_runtime_rules_definition, try_runtime_tech_definitions, try_ui_theme_definition,
    try_unit_definition_by_id, ui_app_title, ui_window_title, unit_attack, unit_base_hp,
    unit_defense, unit_max_moves, unit_name, validate_bundled_content,
};
use smac_core::{Facility, ProductionItem, Tech, UnitKind};

#[test]
fn bundled_faction_content_loads() {
    let factions = load_faction_definitions().expect("bundled faction JSON should parse");

    assert!(factions.len() >= 5);
    assert!(factions
        .iter()
        .any(|faction| faction.name == "Gaia's Stepdaughters"));
    assert!(factions
        .iter()
        .any(|faction| faction.name == "University of Planet"));
    assert!(factions
        .iter()
        .any(|faction| faction.name == "Spartan Federation"));
    assert!(factions.iter().any(|faction| faction.name == "Planetmind"));
    assert!(factions
        .iter()
        .find(|faction| faction.name == "University of Planet")
        .and_then(|faction| faction.ai_policy.as_ref())
        .is_some());
    assert!(factions
        .iter()
        .find(|faction| faction.name == "Morgan Industries")
        .and_then(|faction| faction.ai_policy.as_ref())
        .is_some());
}

#[test]
fn bundled_content_validates_end_to_end() {
    validate_bundled_content().unwrap_or_else(|errors| {
        panic!("bundled content validation failed:\n{}", errors.join("\n"))
    });
}

#[test]
fn technology_enables_integrity() {
    let techs = try_runtime_tech_definitions().expect("techs should load");

    // Specific check for secret projects which were recently fixed
    let orbital_mechanics = techs
        .iter()
        .find(|t| t.id == "orbital_mechanics")
        .expect("orbital_mechanics should exist");
    assert!(orbital_mechanics
        .enables
        .secret_projects
        .contains(&"orbital_elevator".to_string()));
    assert!(orbital_mechanics.enables.facilities.is_empty());

    let singularity_physics = techs
        .iter()
        .find(|t| t.id == "singularity_physics")
        .expect("singularity_physics should exist");
    assert!(singularity_physics
        .enables
        .secret_projects
        .contains(&"black_hole_harvester".to_string()));
}

#[test]
fn fallible_unit_definition_lookup_resolves_known_content() {
    let definition = try_unit_definition_by_id("mind_worm")
        .expect("bundled unit definition should resolve through fallible lookup");
    assert_eq!(definition.name, "Mind Worm");
}

#[test]
fn fallible_runtime_faction_setup_loader_resolves_current_bundle() {
    let setups = try_default_runtime_faction_setups()
        .expect("runtime faction bootstrap should resolve for bundled active factions");
    assert_eq!(setups.len(), 3);
    assert_eq!(setups[0].faction_name, "Gaia's Stepdaughters");
    assert_eq!(setups[1].faction_name, "Spartan Federation");
    assert_eq!(setups[2].faction_name, "Planetmind");
}

#[test]
fn fallible_runtime_definition_refs_resolve_current_bundle() {
    let rules = try_runtime_rules_definition()
        .expect("runtime rules should resolve through fallible ref accessor");
    let theme =
        try_ui_theme_definition().expect("ui theme should resolve through fallible ref accessor");
    let techs = try_runtime_tech_definitions()
        .expect("runtime tech definitions should resolve through fallible ref accessor");

    assert_eq!(rules.base_growth_nutrients_threshold, 20);
    assert_eq!(theme.window_title, "SMAC Rust Edition - Planetfall Command");
    assert!(techs.iter().any(|tech| tech.id == "industrial_base"));
}

#[test]
fn runtime_player_faction_is_derived_from_bundled_content() {
    let runtime_factions = build_runtime_factions();
    let roles = runtime_roles();
    let player_faction = runtime_factions
        .iter()
        .find(|faction| faction.id == roles.player)
        .expect("player runtime faction should exist");
    let ai_faction = runtime_factions
        .iter()
        .find(|faction| faction.id == roles.ai)
        .expect("ai runtime faction should exist");

    assert_eq!(player_faction.name, "Gaia's Stepdaughters");
    assert_eq!(ai_faction.name, "Spartan Federation");
}

#[test]
fn default_starting_scenario_contains_expected_opening_units() {
    let scenario = default_starting_scenario(16, 16);
    let roles = runtime_roles();

    assert_eq!(scenario.forced_land_positions.len(), 3);
    assert!(scenario
        .starting_units
        .iter()
        .any(|unit| unit.owner == roles.player && unit.kind == UnitKind::ColonyPod));
    assert!(scenario
        .starting_units
        .iter()
        .any(|unit| unit.owner == roles.player && unit.kind == UnitKind::Former));
    assert!(scenario
        .starting_units
        .iter()
        .any(|unit| unit.owner == roles.ai && unit.kind == UnitKind::ScoutPatrol));
}

#[test]
fn scenario_anchor_offsets_resolve_expected_ai_positions() {
    let scenario = default_starting_scenario(16, 16);
    let roles = runtime_roles();

    assert!(scenario
        .forced_land_positions
        .iter()
        .any(|position| position.x == 8 && position.y == 8));
    assert!(scenario
        .forced_land_positions
        .iter()
        .any(|position| position.x == 11 && position.y == 11));
    assert!(scenario.starting_units.iter().any(|unit| {
        unit.owner == roles.ai
            && unit.kind == UnitKind::ScoutPatrol
            && unit.x == 10
            && unit.y == 11
    }));
}

#[test]
fn fallible_starting_scenario_loader_resolves_current_bundle() {
    let scenario =
        try_default_starting_scenario(16, 16).expect("bundled scenario should resolve cleanly");
    assert_eq!(scenario.forced_land_positions.len(), 3);
    assert_eq!(scenario.starting_units.len(), 5);
}

#[test]
fn content_name_mapping_for_tech_and_units_is_stable() {
    assert_eq!(UnitKind::from_content_id("former"), Some(UnitKind::Former));
    assert_eq!(
        ProductionItem::from_content_id("former"),
        Some(ProductionItem::Former)
    );
    assert_eq!(
        Tech::from_content_name("Industrial Base"),
        Some(Tech::IndustrialBase)
    );
    assert_eq!(
        Tech::from_content_name("Progenitor Psych"),
        Some(Tech::ProgenitorPsych)
    );
    assert_eq!(
        Tech::from_content_name("Field Modulation"),
        Some(Tech::FieldModulation)
    );
    assert_eq!(
        Tech::from_content_name("Planetary Networks"),
        Some(Tech::PlanetaryNetworks)
    );
    assert_eq!(
        Tech::from_content_name("Secrets of the Human Brain"),
        Some(Tech::SecretsOfTheHumanBrain)
    );
    assert_eq!(
        Tech::from_content_name("Gene Splicing"),
        Some(Tech::GeneSplicing)
    );
    assert_eq!(
        UnitKind::from_content_id("isle_of_the_deep"),
        Some(UnitKind::IsleOfTheDeep)
    );
    assert_eq!(
        UnitKind::from_content_id("resonance_laser"),
        Some(UnitKind::ResonanceLaser)
    );
    assert_eq!(
        ProductionItem::from_content_id("research_hospital"),
        Some(ProductionItem::ResearchHospital)
    );
    assert_eq!(
        Facility::from_content_id("bioenhancement_center"),
        Some(Facility::BioenhancementCenter)
    );
}

#[test]
fn production_definitions_load_and_match_runtime_mapping() {
    let definitions = load_production_definitions().expect("bundled production JSON should parse");

    assert_eq!(definitions.len(), 41);
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "former"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "speeder"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "resonance_laser"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "escort_speeder"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "raider_speeder"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "transit_hub"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "greenhouse"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "mineral_refinery"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "trade_exchange"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "freight_depot"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "patrol_grid"));
    assert_eq!(production_cost(ProductionItem::Former), 18);
    assert_eq!(production_name(ProductionItem::Former), "Former");
    assert_eq!(
        production_unit_kind(ProductionItem::ColonyPod),
        Some(UnitKind::ColonyPod)
    );
    assert_eq!(
        production_unit_kind(ProductionItem::Speeder),
        Some(UnitKind::Speeder)
    );
    assert_eq!(
        production_unit_kind(ProductionItem::ResonanceLaser),
        Some(UnitKind::ResonanceLaser)
    );
    assert_eq!(
        production_unit_kind(ProductionItem::RaiderSpeeder),
        Some(UnitKind::RaiderSpeeder)
    );
    assert_eq!(
        production_unit_kind(ProductionItem::GarrisonGuard),
        Some(UnitKind::GarrisonGuard)
    );
    assert_eq!(
        production_unit_kind(ProductionItem::PsiSentinel),
        Some(UnitKind::PsiSentinel)
    );
    assert_eq!(
        production_facility(ProductionItem::PerimeterDefense),
        Some(Facility::PerimeterDefense)
    );
    assert_eq!(
        production_facility(ProductionItem::CommandCenter),
        Some(Facility::CommandCenter)
    );
    assert_eq!(
        production_facility(ProductionItem::FieldHospital),
        Some(Facility::FieldHospital)
    );
    assert_eq!(
        production_facility(ProductionItem::MilitaryAcademy),
        Some(Facility::MilitaryAcademy)
    );
    assert_eq!(
        production_facility(ProductionItem::SensorArray),
        Some(Facility::SensorArray)
    );
    assert_eq!(
        production_facility(ProductionItem::PsiBeacon),
        Some(Facility::PsiBeacon)
    );
    assert_eq!(
        production_facility(ProductionItem::ForwardDepot),
        Some(Facility::ForwardDepot)
    );
    assert_eq!(
        production_facility(ProductionItem::Greenhouse),
        Some(Facility::Greenhouse)
    );
    assert_eq!(
        production_facility(ProductionItem::MineralRefinery),
        Some(Facility::MineralRefinery)
    );
    assert_eq!(
        production_facility(ProductionItem::TradeExchange),
        Some(Facility::TradeExchange)
    );
    assert_eq!(
        production_facility(ProductionItem::FreightDepot),
        Some(Facility::FreightDepot)
    );
    assert_eq!(
        production_facility(ProductionItem::PatrolGrid),
        Some(Facility::PatrolGrid)
    );
    assert_eq!(
        production_facility(ProductionItem::HologramTheatre),
        Some(Facility::HologramTheatre)
    );
    assert_eq!(
        production_facility(ProductionItem::BioenhancementCenter),
        Some(Facility::BioenhancementCenter)
    );
    assert_eq!(
        production_facility(ProductionItem::ResearchHospital),
        Some(Facility::ResearchHospital)
    );
}

#[test]
fn facility_definitions_load_and_match_runtime_mapping() {
    let definitions = load_facility_definitions().expect("bundled facility JSON should parse");

    assert_eq!(definitions.len(), 19);
    assert_eq!(facility_name(Facility::NetworkNode), "Network Node");
    assert_eq!(facility_maintenance(Facility::PerimeterDefense), 2);
    assert_eq!(
        facility_name(Facility::RecreationCommons),
        "Recreation Commons"
    );
    assert_eq!(facility_name(Facility::FieldHospital), "Field Hospital");
    assert_eq!(facility_name(Facility::MilitaryAcademy), "Military Academy");
    assert_eq!(facility_name(Facility::SensorArray), "Sensor Array");
    assert_eq!(facility_name(Facility::TransitHub), "Transit Hub");
    assert_eq!(facility_name(Facility::PsiBeacon), "Psi Beacon");
    assert_eq!(facility_name(Facility::ForwardDepot), "Forward Depot");
    assert_eq!(facility_name(Facility::Greenhouse), "Greenhouse");
    assert_eq!(facility_name(Facility::MineralRefinery), "Mineral Refinery");
    assert_eq!(facility_name(Facility::TradeExchange), "Trade Exchange");
    assert_eq!(facility_name(Facility::FreightDepot), "Freight Depot");
    assert_eq!(facility_name(Facility::PatrolGrid), "Patrol Grid");
    assert_eq!(facility_name(Facility::HologramTheatre), "Hologram Theatre");
    assert_eq!(
        facility_name(Facility::BioenhancementCenter),
        "Bioenhancement Center"
    );
    assert_eq!(
        facility_name(Facility::ResearchHospital),
        "Research Hospital"
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "transit_hub")
            .expect("transit hub definition should exist")
            .mobility_bonus,
        1
    );
    assert_eq!(facility_psi_support_bonus(Facility::PsiBeacon), 2);
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "forward_depot")
            .expect("forward depot definition should exist")
            .mobility_bonus,
        1
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "recreation_commons")
            .expect("recreation commons definition should exist")
            .stability_bonus,
        2
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "greenhouse")
            .expect("greenhouse definition should exist")
            .yield_bonus
            .nutrients,
        2
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "mineral_refinery")
            .expect("mineral refinery definition should exist")
            .yield_bonus
            .minerals,
        2
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "trade_exchange")
            .expect("trade exchange definition should exist")
            .yield_bonus
            .energy,
        2
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "freight_depot")
            .expect("freight depot definition should exist")
            .yield_bonus
            .minerals,
        1
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "patrol_grid")
            .expect("patrol grid definition should exist")
            .convoy_security_bonus,
        2
    );
    assert_eq!(
        definitions
            .iter()
            .find(|definition| definition.id == "military_academy")
            .expect("military academy definition should exist")
            .training_bonus,
        2
    );
    assert_eq!(facility_stability_bonus(Facility::HologramTheatre), 2);
    assert_eq!(facility_training_bonus(Facility::BioenhancementCenter), 1);
    assert_eq!(
        facility_psi_support_bonus(Facility::BioenhancementCenter),
        1
    );
    assert_eq!(facility_repair_bonus(Facility::ResearchHospital), 2);
}

#[test]
fn runtime_tech_definitions_load_and_match_runtime_mapping() {
    let definitions =
        load_runtime_tech_definitions().expect("bundled runtime tech JSON should parse");

    assert!(definitions.len() >= 12);
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "industrial_base"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "social_psych"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "progenitor_psych"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "field_modulation"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "planetary_networks"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "secrets_of_the_human_brain"));
    assert!(definitions
        .iter()
        .any(|definition| definition.id == "gene_splicing"));
    assert_eq!(tech_cost(Tech::Biogenetics), 50);
    assert_eq!(tech_name(Tech::DoctrineMobility), "Doctrine: Mobility");
    assert_eq!(
        tech_description(Tech::SecretsOfPlanet),
        "Reveals deeper native-life behavior."
    );
    assert_eq!(
        Tech::from_content_id("industrial_base"),
        Some(Tech::IndustrialBase)
    );
    assert_eq!(
        Tech::from_content_id("field_modulation"),
        Some(Tech::FieldModulation)
    );
    assert_eq!(
        Tech::from_content_id("planetary_networks"),
        Some(Tech::PlanetaryNetworks)
    );
    assert_eq!(
        Tech::from_content_id("secrets_of_the_human_brain"),
        Some(Tech::SecretsOfTheHumanBrain)
    );
    assert_eq!(
        Tech::from_content_id("gene_splicing"),
        Some(Tech::GeneSplicing)
    );
    assert_eq!(
        tech_prerequisites(Tech::ProgenitorPsych),
        vec![Tech::CentauriEcology, Tech::SocialPsych]
    );
    assert!(tech_is_available(
        &[Tech::CentauriEcology, Tech::SocialPsych],
        Tech::ProgenitorPsych
    ));
}

#[test]
fn runtime_rules_and_unit_runtime_definitions_load_and_match_mappings() {
    let runtime_rules = load_runtime_rules().expect("bundled runtime rules JSON should parse");
    let unit_definitions = load_unit_definitions().expect("bundled unit JSON should parse");

    assert_eq!(runtime_rules.base_growth_nutrients_threshold, 20);
    assert_eq!(base_growth_nutrients_threshold(), 20);
    assert_eq!(supply_pod_energy_reward(), 25);
    assert_eq!(ai_colony_base_target(), 2);
    assert_eq!(ai_native_spawn_noise_salt(), 404);
    assert_eq!(map_ocean_threshold(), 18);
    assert_eq!(map_fungus_threshold(), 86);
    assert_eq!(map_rocky_threshold(), 76);
    assert_eq!(map_flat_moisture_threshold(), 52);
    assert_eq!(map_pod_spawn_threshold(), 94);
    assert_eq!(forced_land_patch_radius(), 3);
    assert_eq!(unit_definitions.len(), 14);
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "mind_worm"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "isle_of_the_deep"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "speeder"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "resonance_laser"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "escort_speeder"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "raider_speeder"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "garrison_guard"));
    assert!(unit_definitions
        .iter()
        .any(|definition| definition.id == "psi_sentinel"));
    assert_eq!(unit_name(UnitKind::MindWorm), "Mind Worm");
    assert_eq!(unit_name(UnitKind::IsleOfTheDeep), "Isle of the Deep");
    assert_eq!(unit_name(UnitKind::ResonanceLaser), "Resonance Laser");
    assert_eq!(unit_name(UnitKind::GarrisonGuard), "Garrison Guard");
    assert_eq!(unit_name(UnitKind::PsiSentinel), "Psi Sentinel");
    assert_eq!(unit_max_moves(UnitKind::Former), 1);
    assert_eq!(unit_max_moves(UnitKind::Speeder), 2);
    assert_eq!(unit_attack(UnitKind::MindWorm), 3);
    assert_eq!(unit_attack(UnitKind::ResonanceLaser), 6);
    assert_eq!(unit_defense(UnitKind::ScoutPatrol), 2);
    assert_eq!(unit_defense(UnitKind::GarrisonGuard), 4);
    assert_eq!(unit_defense(UnitKind::PsiSentinel), 5);
    assert_eq!(unit_defense(UnitKind::TranceScout), 3);
    assert_eq!(unit_base_hp(UnitKind::ColonyPod), 10);
}

#[test]
fn faction_and_theme_presentation_content_is_available() {
    let gaia = faction_definition_by_name("Gaia's Stepdaughters");
    let theme = load_ui_theme_definition().expect("bundled ui theme JSON should parse");

    assert_eq!(gaia.color_hex, "#5f9d58");
    assert_eq!(gaia.leader, "Lady Deirdre Skye");
    assert_eq!(theme.window_title, "SMAC Rust Edition - Planetfall Command");
    assert_eq!(ui_window_title(), "SMAC Rust Edition - Planetfall Command");
    assert_eq!(ui_app_title(), "SMAC Rust Edition - Planetfall Command");
    assert_eq!(gaia.runtime_role.as_deref(), Some("player"));
    assert_eq!(gaia.base_names[0], "Landing Point");
}

#[test]
fn runtime_faction_setup_and_base_names_are_content_driven() {
    let roles = runtime_roles();
    let player =
        runtime_faction_definition_by_owner(roles.player).expect("player faction should exist");
    let ai = runtime_faction_definition_by_owner(roles.ai).expect("ai faction should exist");

    assert_eq!(player.name, "Gaia's Stepdaughters");
    assert_eq!(ai.name, "Spartan Federation");
    assert_eq!(player.known_tech_ids, vec!["centauri_ecology"]);
    assert_eq!(ai.current_research_id.as_deref(), Some("industrial_base"));
    assert_eq!(ai_expansion_base_target(roles.ai), 3);
    assert_eq!(ai_attack_bias(roles.ai), 9);
    assert_eq!(ai_exploration_bias(roles.ai), 3);
    assert_eq!(
        ai_preferred_production(roles.ai),
        ProductionItem::ScoutPatrol
    );
    assert_eq!(ai_colony_base_target(), 2);
    assert_eq!(
        next_base_name_for_faction("Gaia's Stepdaughters", 2),
        "Greenhouse Gate"
    );
    assert_eq!(
        next_base_name_for_faction("Spartan Federation", 4),
        "Spartan Base 4"
    );
}
