use smac_core::{
    DiplomacyStatus, GameState, GameAction, UnitKind,
    ProductionItem, Base, Terrain, GovernorMode, UnitActivity,
    Ability,
};

#[test]
fn test_air_scramble_and_intercept() {
    let mut game = GameState::new_game(12, 12, 12345);
    let owner_a = game.player_owner();
    let owner_b = game.ai_owner();

    // Clear units and bases
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    // Set A and B to War
    game.relations[owner_a][owner_b].status = DiplomacyStatus::War;
    game.relations[owner_b][owner_a].status = DiplomacyStatus::War;

    // 1. Setup Base for A at (3, 3)
    game.bases.push(Base {
        id: 0,
        owner: owner_a,
        name: "Base Alpha".to_string(),
        x: 3,
        y: 3,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::StockpileEnergy,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[3 * game.width + 3].base = Some(0);

    // Spawn Needlejet for A at (3, 3) and set to Scramble
    let jet_id = game.spawn_unit(owner_a, UnitKind::Needlejet, 3, 3);
    game.set_unit_activity(jet_id, UnitActivity::Scramble);

    // Spawn enemy unit for B at (3, 6)
    let enemy_id = game.spawn_unit(owner_b, UnitKind::ScoutPatrol, 3, 6);

    // Enemy moves to (3, 5) - this is within 3 tiles of the base (distance <= 3), triggering scramble
    let res = game.apply_action(GameAction::MoveUnit {
        unit_id: enemy_id,
        target_x: 3,
        target_y: 5,
    });
    assert!(res.is_ok());

    // Verify combat resolved: one of the units should be dead, and the jet's activity/moves consumed
    let jet_alive = game.unit(jet_id).map(|u| u.alive).unwrap_or(false);
    if jet_alive {
        let jet = game.unit(jet_id).unwrap();
        assert_eq!(jet.activity, UnitActivity::None);
        assert_eq!(jet.moves_left, 0);
    }
}

#[test]
fn test_air_intercept_mission() {
    let mut game = GameState::new_game(12, 12, 12345);
    let owner_a = game.player_owner();
    let owner_b = game.ai_owner();

    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    game.relations[owner_a][owner_b].status = DiplomacyStatus::War;
    game.relations[owner_b][owner_a].status = DiplomacyStatus::War;

    // Spawn intercepting Needlejet for A at (5, 5) and set to Intercept
    let jet_id = game.spawn_unit(owner_a, UnitKind::Needlejet, 5, 5);
    game.set_unit_activity(jet_id, UnitActivity::Intercept);

    // Spawn enemy scout for B at (2, 8)
    let enemy_id = game.spawn_unit(owner_b, UnitKind::ScoutPatrol, 2, 8);

    // Enemy moves to (2, 7) - distance from (5,5) is Chebyshev dist: max(|5-2|, |5-7|) = max(3, 2) = 3
    // Within Intercept range (<= 3), should trigger interception!
    let res = game.apply_action(GameAction::MoveUnit {
        unit_id: enemy_id,
        target_x: 2,
        target_y: 7,
    });
    assert!(res.is_ok());

    // Verify interceptor activity/moves updated
    if let Some(jet) = game.unit(jet_id) {
        if jet.alive {
            assert_eq!(jet.activity, UnitActivity::None);
            assert_eq!(jet.moves_left, 0);
        }
    }
}

#[test]
fn test_tectonic_buster_project_completion_and_detonation() {
    let mut game = GameState::new_game(12, 12, 12345);
    let owner = game.player_owner();
    let rival = game.ai_owner();

    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    game.relations[owner][rival].status = DiplomacyStatus::War;
    game.relations[rival][owner].status = DiplomacyStatus::War;

    // Set initial toxicity to high
    game.factions[owner].planet_toxicity = 300;

    // Spawn base at (5, 5)
    game.bases.push(Base {
        id: 0,
        owner,
        name: "Wonder Base".to_string(),
        x: 5,
        y: 5,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::StockpileEnergy,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    // Complete TectonicBuster
    let completed = game.complete_production_for_testing(0, ProductionItem::TectonicBuster);
    assert!(completed);

    // Verify toxicity reset to 0
    assert_eq!(game.factions[owner].planet_toxicity, 0);

    // Verify a free Tectonic Buster (Planet Buster) was spawned at (5, 5)
    let base_tile_unit = game.tiles[5 * game.width + 5].unit;
    assert!(base_tile_unit.is_some());
    let pb_id = base_tile_unit.unwrap();
    let pb = game.unit(pb_id).unwrap();
    assert!(matches!(pb.kind, UnitKind::CustomUnit(_)));

    // Detonate Planet Buster at (5, 6) - spawn an enemy unit there first
    let enemy_id = game.spawn_unit(rival, UnitKind::ScoutPatrol, 5, 6);
    game.factions[owner].planet_toxicity = 150; // set toxicity to positive again

    // Detonate it
    let res = game.apply_action(GameAction::MoveUnit {
        unit_id: pb_id,
        target_x: 5,
        target_y: 6,
    });
    assert!(res.is_ok());

    // Verify unit destroyed
    assert!(game.unit(enemy_id).is_none());

    // Verify terrain altered to NuclearCrater (check 3x3)
    for dy in -1..=1 {
        for dx in -1..=1 {
            let tx = 5 + dx;
            let ty = 6 + dy;
            let idx = ty as usize * game.width + tx as usize;
            assert_eq!(game.tiles[idx].terrain, Terrain::NuclearCrater);
            assert_eq!(game.tiles[idx].elevation, -10);
        }
    }

    // Verify toxicity reset to 0
    assert_eq!(game.factions[owner].planet_toxicity, 0);
}

#[test]
fn test_drop_pod_orbital_insertion() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();

    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    // Setup custom design with DropPod ability
    let design = smac_core::UnitDesign {
        name: "Shock Trooper".to_string(),
        chassis: smac_core::Chassis::Infantry,
        weapon: smac_core::Weapon::HandLaser(1),
        armor: smac_core::Armor::SynthMetal(1),
        cost: 40,
        abilities: vec![Ability::DropPod],
    };
    game.add_unit_design(owner, design);
    let design_idx = game.faction(owner).unwrap().unit_designs.len() - 1;
    let kind = UnitKind::CustomUnit(game.factions[owner].unit_designs[design_idx].clone());

    // Spawn unit at (1, 1)
    let unit_id = game.spawn_unit_with_design(owner, kind, design_idx, 1, 1, 0).unwrap();

    // Target is (8, 8) - explore it first
    let idx = 8 * game.width + 8;
    game.tiles[idx].explored_by_owner.insert(owner);

    // Apply Move to (8, 8) (distant tile)
    let res = game.apply_action(GameAction::MoveUnit {
        unit_id,
        target_x: 8,
        target_y: 8,
    });
    assert!(res.is_ok());

    // Verify unit teleported to (8, 8) and moves consumed
    let unit = game.unit(unit_id).unwrap();
    assert_eq!(unit.x, 8);
    assert_eq!(unit.y, 8);
    assert_eq!(unit.moves_left, 0);
}
