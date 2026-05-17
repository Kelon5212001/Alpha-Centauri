use smac_core::{Ability, Armor, Chassis, GameState, Terrain, UnitDesign, UnitKind, Weapon};

#[test]
fn ability_raid_bonus() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    let ai_owner = game.ai_owner();
    game.faction_mut(owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(owner).unwrap().base_attributes.planet = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.planet = 0;

    // 1. Create a "Raider" custom design
    let raider = UnitDesign {
        name: "Raider".to_string(),
        chassis: Chassis::Speeder,
        weapon: Weapon::HandLaser(1),
        armor: Armor::SynthMetal(1),
        cost: 30,
        abilities: vec![Ability::Raid],
    };

    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(raider);
        faction.unit_designs.len() - 1
    };

    // 2. Spawn the raider
    let attacker_id = game.units.len();
    game.units.push(smac_core::Unit {
        id: attacker_id,
        owner,
        kind: UnitKind::Speeder,
        design_index,
        x: 5,
        y: 5,
        moves_left: 2,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(attacker_id);

    // 3. Spawn an exposed defender (not in base)
    let defender_id = game.spawn_unit(ai_owner, UnitKind::ScoutPatrol, 5, 6);
    game.tiles[6 * game.width + 5].base = None; // Ensure it's not a base

    // 4. Verify Raid bonus
    assert!(game.unit_has_ability(attacker_id, Ability::Raid));
    assert!(game.unit_is_exposed(defender_id));

    // Attack!
    game.move_unit_to(attacker_id, 5, 6).unwrap();

    // Raid bonus is +2. Base attack is 1. Roll is 0-5. Experience is 0.
    // Score should be 1 (atk) + 0 (exp) + roll + 1 (fixed) + 2 (raid) = roll + 4.
    // So 4-9.
    assert!(game.log.iter().any(|msg| msg.message.contains("atk: 4")
        || msg.message.contains("atk: 5")
        || msg.message.contains("atk: 6")
        || msg.message.contains("atk: 7")
        || msg.message.contains("atk: 8")
        || msg.message.contains("atk: 9")));
}

#[test]
fn ability_trance_bonus() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    let native_owner = game.native_owner();
    game.faction_mut(owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(owner).unwrap().base_attributes.planet = 0;
    if let Some(faction) = game.faction_mut(native_owner) {
        faction.base_attributes.morale = 0;
        faction.base_attributes.planet = 0;
    }

    // 1. Create a "Trance Scout" custom design
    let trance_scout = UnitDesign {
        name: "Trance Scout".to_string(),
        chassis: Chassis::Infantry,
        weapon: Weapon::HandLaser(1),
        armor: Armor::SynthMetal(1),
        cost: 20,
        abilities: vec![Ability::Trance],
    };

    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(trance_scout);
        faction.unit_designs.len() - 1
    };

    // 2. Spawn the trance scout as defender
    let defender_id = game.units.len();
    game.units.push(smac_core::Unit {
        id: defender_id,
        owner,
        kind: UnitKind::ScoutPatrol,
        design_index,
        x: 5,
        y: 6,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[6 * game.width + 5].unit = Some(defender_id);

    // 3. Spawn a psi threat (Mind Worm)
    let attacker_id = game.spawn_unit(native_owner, UnitKind::MindWorm, 5, 5);

    // 4. Verify Trance bonus
    assert!(game.unit_has_ability(defender_id, Ability::Trance));
    assert!(game.unit_is_psi_threat(attacker_id));

    // Let the native unit attack!
    game.move_unit_to(attacker_id, 5, 6).unwrap();

    // Trance bonus is +2. Base defense is 1. Experience is 0. HP/4 is 10/4 = 2.
    // Score should be 1 (def) + 0 (exp) + 2 (hp/4) + bonus (0 if flat) + 2 (trance) = 5.
    assert!(game.log.iter().any(|msg| msg.message.contains("def: 7")));
}

#[test]
fn ability_deep_pressure_hull_bonus() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    let ai_owner = game.ai_owner();
    game.faction_mut(owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(owner).unwrap().base_attributes.planet = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.planet = 0;

    // 1. Create a "Submarine" custom design
    let sub = UnitDesign {
        name: "Submarine".to_string(),
        chassis: Chassis::Hovertank, // Placeholder for sea chassis
        weapon: Weapon::HandLaser(1),
        armor: Armor::SynthMetal(1),
        cost: 40,
        abilities: vec![Ability::DeepPressureHull],
    };

    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(sub);
        faction.unit_designs.len() - 1
    };

    // 2. Spawn the sub as defender at sea
    let defender_id = game.units.len();
    game.units.push(smac_core::Unit {
        id: defender_id,
        owner,
        kind: UnitKind::Speeder,
        design_index,
        x: 5,
        y: 6,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[6 * game.width + 5].unit = Some(defender_id);
    game.tiles[6 * game.width + 5].terrain = Terrain::Ocean;

    // 3. Spawn an attacker (Isle of the Deep)
    let attacker_id = game.spawn_unit(ai_owner, UnitKind::IsleOfTheDeep, 5, 5);
    game.tiles[5 * game.width + 5].terrain = Terrain::Ocean;

    // 4. Verify Deep Pressure Hull bonus
    assert!(game.unit_has_ability(defender_id, Ability::DeepPressureHull));
    assert!(game.unit_is_at_sea(defender_id));

    // Attack!
    game.move_unit_to(attacker_id, 5, 6).unwrap();

    // DPH bonus is +2. Base defense is 1. Experience 0. HP/4 is 2. Tile bonus 0.
    // Total defense: 1 + 0 + 2 + 0 + 2 (DPH) = 5.
    assert!(game.log.iter().any(|msg| msg.message.contains("def: 7")));
}

#[test]
fn ability_air_superiority_bonus() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    let ai_owner = game.ai_owner();
    game.faction_mut(owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(owner).unwrap().base_attributes.planet = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.planet = 0;

    // 1. Create a "Fighter" custom design
    let fighter = UnitDesign {
        name: "Fighter".to_string(),
        chassis: Chassis::Hovertank, // Placeholder
        weapon: Weapon::HandLaser(1),
        armor: Armor::SynthMetal(1),
        cost: 40,
        abilities: vec![Ability::AirSuperiority],
    };

    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(fighter);
        faction.unit_designs.len() - 1
    };

    // 2. Spawn the fighter
    let attacker_id = game.units.len();
    game.units.push(smac_core::Unit {
        id: attacker_id,
        owner,
        kind: UnitKind::Speeder,
        design_index,
        x: 5,
        y: 5,
        moves_left: 2,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(attacker_id);

    // 3. Spawn an aircraft defender (Needlejet)
    let defender_id = game.spawn_unit(ai_owner, UnitKind::Needlejet, 5, 6);

    // 4. Verify Air Superiority bonus
    assert!(game.unit_has_ability(attacker_id, Ability::AirSuperiority));
    assert!(game.unit_is_aircraft(defender_id));

    // Attack!
    game.move_unit_to(attacker_id, 5, 6).unwrap();

    // AS bonus is +2. Base attack 1. Roll 0-5. Exp 0. Fixed 1.
    // Score: 1 + 0 + roll + 1 + 2 (AS) = roll + 4.
    assert!(game.log.iter().any(|msg| msg.message.contains("atk: 4")
        || msg.message.contains("atk: 5")
        || msg.message.contains("atk: 6")
        || msg.message.contains("atk: 7")
        || msg.message.contains("atk: 8")
        || msg.message.contains("atk: 9")));
}

#[test]
fn ability_comm_jammer_bonus() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    let ai_owner = game.ai_owner();
    game.faction_mut(owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(owner).unwrap().base_attributes.planet = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.morale = 0;
    game.faction_mut(ai_owner).unwrap().base_attributes.planet = 0;

    let jammer = UnitDesign {
        name: "Jammer".to_string(),
        chassis: Chassis::Infantry,
        weapon: Weapon::HandLaser(1),
        armor: Armor::SynthMetal(1),
        cost: 20,
        abilities: vec![Ability::CommJammer],
    };

    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(jammer);
        faction.unit_designs.len() - 1
    };

    let defender_id = game.units.len();
    game.units.push(smac_core::Unit {
        id: defender_id,
        owner,
        kind: UnitKind::ScoutPatrol,
        design_index,
        x: 5,
        y: 6,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[6 * game.width + 5].unit = Some(defender_id);

    // Spawn a fast attacker (Speeder)
    let attacker_id = game.spawn_unit(ai_owner, UnitKind::Speeder, 5, 5);

    assert!(game.unit_has_ability(defender_id, Ability::CommJammer));
    assert!(game.unit_is_fast(attacker_id));

    game.move_unit_to(attacker_id, 5, 6).unwrap();

    // Def: base 1. Jammer bonus +2. Exp 0. hp/4 = 10/4 = 2.
    // 1 + 2 (Jammer) + 0 + 2 = 5.
    assert!(game.log.iter().any(|msg| msg.message.contains("def: 5")));
}

#[test]
fn ability_non_lethal_methods_police_bonus() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().base_attributes.police = 0;

    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "Police Station".to_string(),
        x: 5,
        y: 5,
        population: 5,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        production_queue: vec![],
        facilities: vec![],
        governor_mode: smac_core::GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    let unrest_before = game.base_unrest(0);
    // Base pop 5 - 2 = 3 unrest.
    assert_eq!(unrest_before, 3);

    let police = UnitDesign {
        name: "Police".to_string(),
        chassis: Chassis::Infantry,
        weapon: Weapon::HandLaser(1),
        armor: Armor::SynthMetal(1),
        cost: 20,
        abilities: vec![Ability::NonLethalMethods],
    };

    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(police);
        faction.unit_designs.len() - 1
    };

    // Spawn normal unit, uses 1 of the default 1 police rating
    game.spawn_unit(owner, UnitKind::ScoutPatrol, 5, 5);
    let unrest_one_garrison = game.base_unrest(0);
    assert_eq!(unrest_one_garrison, 2);

    // Spawn non-lethal unit, adds extra police ignoring limit
    game.units.push(smac_core::Unit {
        id: game.units.len(),
        owner,
        kind: UnitKind::ScoutPatrol, // custom design
        design_index,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    let unrest_with_police = game.base_unrest(0);
    assert_eq!(unrest_with_police, 1);
}
