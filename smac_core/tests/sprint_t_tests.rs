use smac_core::{DiplomacyStatus, GameState, GameAction, Unit, UnitKind, Facility, ProductionItem, Base, Terrain, GovernorMode};

#[test]
fn test_auto_war_on_attack_ally() {
    let mut game = GameState::new_game(10, 10, 12345);
    let owner_a = game.player_owner();
    let owner_b = game.ai_owner();

    // Clear units and bases
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    // Set A and B to Pact
    game.relations[owner_a][owner_b].status = DiplomacyStatus::Pact;
    game.relations[owner_b][owner_a].status = DiplomacyStatus::Pact;

    // Spawn unit for A at (3, 3)
    game.units.push(Unit {
        id: 0,
        owner: owner_a,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 3,
        y: 3,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 3].unit = Some(0);

    // Spawn unit for B at (3, 4)
    game.units.push(Unit {
        id: 1,
        owner: owner_b,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 3,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 3].unit = Some(1);

    // Attacking should auto-declare war (breaking Pact) and proceed
    let res = game.apply_action(GameAction::MoveUnit {
        unit_id: 0,
        target_x: 3,
        target_y: 4,
    });
    assert!(res.is_ok());

    // Verify they are now at War
    assert_eq!(game.relations[owner_a][owner_b].status, DiplomacyStatus::War);
}

#[test]
fn test_mutual_defense_cascade() {
    let mut game = GameState::new_game(10, 10, 12345);
    // Let's use indices 0, 1, 2 for A, B, C
    let owner_a = 0;
    let owner_b = 1;
    let owner_c = 2;

    // A and B are at Truce, B and C are at Pact
    game.relations[owner_a][owner_b].status = DiplomacyStatus::Truce;
    game.relations[owner_b][owner_a].status = DiplomacyStatus::Truce;
    
    game.relations[owner_b][owner_c].status = DiplomacyStatus::Pact;
    game.relations[owner_c][owner_b].status = DiplomacyStatus::Pact;

    game.relations[owner_a][owner_c].status = DiplomacyStatus::Truce;
    game.relations[owner_c][owner_a].status = DiplomacyStatus::Truce;

    // A declares war on B
    game.update_diplomacy(owner_a, owner_b, DiplomacyStatus::War).unwrap();

    // C (Pact ally of B) should now also be at War with A!
    assert_eq!(game.relations[owner_a][owner_c].status, DiplomacyStatus::War);
    assert_eq!(game.relations[owner_c][owner_a].status, DiplomacyStatus::War);
}

#[test]
fn test_shared_vision_and_exploration() {
    let mut game = GameState::new_game(10, 10, 12345);
    let owner_a = 0;
    let owner_b = 1;

    // A and B are in a Pact
    game.relations[owner_a][owner_b].status = DiplomacyStatus::Pact;
    game.relations[owner_b][owner_a].status = DiplomacyStatus::Pact;

    // Explore tile (5, 5) for B
    let idx = 5 * game.width + 5;
    game.tiles[idx].explored_by_owner.insert(owner_b);

    // A should see it as explored
    assert!(game.tile_explored_by_owner(5, 5, owner_a));
}

#[test]
fn test_non_combat_unit_retreat_threat() {
    let mut game = GameState::new_game(10, 10, 12345);
    let player_owner = game.player_owner();
    let ai_owner = game.ai_owner();

    // Clear units and bases
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    // A and B are at War
    game.relations[ai_owner][player_owner].status = DiplomacyStatus::War;
    game.relations[player_owner][ai_owner].status = DiplomacyStatus::War;

    // Fallback base for AI at (0, 0)
    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Fallback Base".to_string(),
        x: 0,
        y: 0,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[0].base = Some(0);

    // Colony Pod for AI at (3, 3)
    game.units.push(Unit {
        id: 0,
        owner: ai_owner,
        kind: UnitKind::ColonyPod,
        design_index: 0,
        x: 3,
        y: 3,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 3].unit = Some(0);

    // Enemy Scout for Player at (4, 4)
    game.units.push(Unit {
        id: 1,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 4,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 4].unit = Some(1);

    // Run AI tactics for AI owner
    smac_core::run_ai_tactics_for_owner(&mut game, ai_owner);

    // The AI colony pod should have moved away from (4, 4) towards (0, 0)
    let pod = game.unit(0).unwrap();
    let distance_from_enemy = (pod.x as isize - 4).abs() + (pod.y as isize - 4).abs();
    assert!(distance_from_enemy > 2); // It should have moved towards (0, 0), so x and y should be smaller (e.g. 2, 2)
}

#[test]
fn test_fusion_lab_energy() {
    let mut game = GameState::new_game(10, 10, 12345);
    let owner = game.player_owner();

    // Spawn base for player at (5, 5)
    game.bases.push(Base {
        id: 0,
        owner,
        name: "Energy Base".to_string(),
        x: 5,
        y: 5,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::StockpileEnergy,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    // Turn 1 baseline base yields
    let yields_before = game.base_yields(5, 5);

    // Add Fusion Lab to base
    game.bases[0].facilities.push(Facility::FusionLab);

    // Turn 2 yields with Fusion Lab
    let yields_after = game.base_yields(5, 5);

    // Verify it adds exactly 5 energy
    assert_eq!(yields_after.energy, yields_before.energy + 5);
}
