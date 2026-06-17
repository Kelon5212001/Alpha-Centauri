use smac_core::{
    Base, DiplomacyStatus, EventLogKind, Facility, GameAction, GameState, GameStateSnapshot,
    GovernorMode, ProbeAction, ProductionItem, Terrain, Unit, UnitKind,
};

fn stage_adjacent_attack(game: &mut GameState, attacker_owner: usize, defender_owner: usize) {
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    game.units.push(Unit {
        id: 0,
        owner: attacker_owner,
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

    game.units.push(Unit {
        id: 1,
        owner: defender_owner,
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
}

fn attack_staged_defender(game: &mut GameState) {
    game.apply_action(GameAction::MoveUnit {
        unit_id: 0,
        target_x: 3,
        target_y: 4,
    })
    .expect("combat move should resolve");
}

fn set_relation(game: &mut GameState, a: usize, b: usize, status: DiplomacyStatus, attitude: i32) {
    game.relations[a][b].status = status;
    game.relations[b][a].status = status;
    game.relations[a][b].attitude = attitude;
    game.relations[b][a].attitude = attitude;
}

#[test]
fn neutral_attack_escalates_to_war_with_first_strike_log() {
    let mut game = GameState::new_game(10, 10, 12345);
    let attacker = game.player_owner();
    let defender = game.ai_owner();
    set_relation(&mut game, attacker, defender, DiplomacyStatus::Truce, 0);
    stage_adjacent_attack(&mut game, attacker, defender);

    attack_staged_defender(&mut game);

    assert_eq!(
        game.relations[attacker][defender].status,
        DiplomacyStatus::War
    );
    assert!(game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::FirstStrikeEscalation));
}

#[test]
fn treaty_attack_escalates_to_war_with_treaty_violation_log() {
    let mut game = GameState::new_game(10, 10, 12345);
    let attacker = game.player_owner();
    let defender = game.ai_owner();
    set_relation(&mut game, attacker, defender, DiplomacyStatus::Treaty, 10);
    stage_adjacent_attack(&mut game, attacker, defender);

    attack_staged_defender(&mut game);

    assert_eq!(
        game.relations[attacker][defender].status,
        DiplomacyStatus::War
    );
    assert!(game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::TreatyViolation));
}

#[test]
fn pact_attack_escalates_to_war_with_betrayal_log_and_larger_penalty() {
    let mut game = GameState::new_game(10, 10, 12345);
    let attacker = game.player_owner();
    let defender = game.ai_owner();
    set_relation(&mut game, attacker, defender, DiplomacyStatus::Pact, 80);
    stage_adjacent_attack(&mut game, attacker, defender);

    attack_staged_defender(&mut game);

    assert_eq!(
        game.relations[attacker][defender].status,
        DiplomacyStatus::War
    );
    assert_eq!(game.relations[defender][attacker].attitude, 10);
    assert!(game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::PactBetrayal));
}

#[test]
fn already_war_attack_does_not_emit_escalation_log() {
    let mut game = GameState::new_game(10, 10, 12345);
    let attacker = game.player_owner();
    let defender = game.ai_owner();
    set_relation(&mut game, attacker, defender, DiplomacyStatus::War, -80);
    stage_adjacent_attack(&mut game, attacker, defender);

    attack_staged_defender(&mut game);

    assert_eq!(
        game.relations[attacker][defender].status,
        DiplomacyStatus::War
    );
    assert!(game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::WartimeCombat));
    assert!(!game.log.iter().any(|entry| {
        matches!(
            entry.kind,
            EventLogKind::FirstStrikeEscalation
                | EventLogKind::TreatyViolation
                | EventLogKind::PactBetrayal
        )
    }));
}

#[test]
fn escalation_survives_snapshot_roundtrip() {
    let mut game = GameState::new_game(10, 10, 12345);
    let attacker = game.player_owner();
    let defender = game.ai_owner();
    set_relation(&mut game, attacker, defender, DiplomacyStatus::Treaty, 25);
    stage_adjacent_attack(&mut game, attacker, defender);

    attack_staged_defender(&mut game);

    let restored = GameStateSnapshot::from(&game).into_game_state();
    assert_eq!(
        restored.relations[attacker][defender].status,
        DiplomacyStatus::War
    );
    assert_eq!(
        restored.relations[defender][attacker].attitude,
        game.relations[defender][attacker].attitude
    );
    assert!(restored
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::TreatyViolation));
}

#[test]
fn tactical_ai_does_not_attack_treaty_or_pact_targets() {
    for status in [DiplomacyStatus::Treaty, DiplomacyStatus::Pact] {
        let mut game = GameState::new_game(10, 10, 12345);
        let attacker = game.ai_owner();
        let defender = game.player_owner();
        set_relation(&mut game, attacker, defender, status, 50);
        stage_adjacent_attack(&mut game, attacker, defender);

        smac_core::run_ai_tactics_for_owner(&mut game, attacker);

        assert_eq!(game.relations[attacker][defender].status, status);
        assert!(game.unit(1).map(|unit| unit.alive).unwrap_or(false));
        assert!(!game.log.iter().any(|entry| {
            matches!(
                entry.kind,
                EventLogKind::FirstStrikeEscalation
                    | EventLogKind::TreatyViolation
                    | EventLogKind::PactBetrayal
                    | EventLogKind::WartimeCombat
            )
        }));
    }
}

#[test]
fn tactical_ai_escalates_from_truce_only_with_explicit_intent() {
    let mut cautious_game = GameState::new_game(10, 10, 12345);
    let attacker = cautious_game.ai_owner();
    let defender = cautious_game.player_owner();
    set_relation(
        &mut cautious_game,
        attacker,
        defender,
        DiplomacyStatus::Truce,
        -80,
    );
    cautious_game
        .faction_mut(attacker)
        .expect("AI faction should exist")
        .personality
        .aggression = 5;
    stage_adjacent_attack(&mut cautious_game, attacker, defender);

    smac_core::run_ai_tactics_for_owner(&mut cautious_game, attacker);

    assert_eq!(
        cautious_game.relations[attacker][defender].status,
        DiplomacyStatus::Truce
    );
    assert!(cautious_game
        .unit(1)
        .map(|unit| unit.alive)
        .unwrap_or(false));

    let mut hostile_game = GameState::new_game(10, 10, 12345);
    let attacker = hostile_game.ai_owner();
    let defender = hostile_game.player_owner();
    set_relation(
        &mut hostile_game,
        attacker,
        defender,
        DiplomacyStatus::Truce,
        -80,
    );
    hostile_game
        .faction_mut(attacker)
        .expect("AI faction should exist")
        .personality
        .aggression = 10;
    stage_adjacent_attack(&mut hostile_game, attacker, defender);

    smac_core::run_ai_tactics_for_owner(&mut hostile_game, attacker);

    assert_eq!(
        hostile_game.relations[attacker][defender].status,
        DiplomacyStatus::War
    );
    assert!(hostile_game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::FirstStrikeEscalation));
}

#[test]
fn probe_sabotage_removes_highest_value_facility_without_stack_order_dependence() {
    let mut game = GameState::new_game(10, 10, 12345);
    let probe_owner = game.player_owner();
    let target_owner = game.ai_owner();

    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    game.units.push(Unit {
        id: 0,
        owner: probe_owner,
        kind: UnitKind::ProbeTeam,
        design_index: 0,
        x: 4,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 4].unit = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: target_owner,
        name: "Sabotage Target".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter, Facility::RecyclingTanks],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.apply_action(GameAction::PerformProbeAction {
        unit_id: 0,
        target_x: 5,
        target_y: 5,
        action: ProbeAction::SabotageFacility,
    })
    .expect("probe sabotage should succeed");

    let facilities = &game.base(0).expect("target base should exist").facilities;
    assert!(!facilities.contains(&Facility::CommandCenter));
    assert!(facilities.contains(&Facility::RecyclingTanks));
    assert!(!game.unit(0).map(|unit| unit.alive).unwrap_or(false));
    assert!(game
        .log
        .iter()
        .any(|entry| entry.message.contains("sabotaged CommandCenter")));
}

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
    assert_eq!(
        game.relations[owner_a][owner_b].status,
        DiplomacyStatus::War
    );
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
    game.update_diplomacy(owner_a, owner_b, DiplomacyStatus::War)
        .unwrap();

    // C (Pact ally of B) should now also be at War with A!
    assert_eq!(
        game.relations[owner_a][owner_c].status,
        DiplomacyStatus::War
    );
    assert_eq!(
        game.relations[owner_c][owner_a].status,
        DiplomacyStatus::War
    );
    assert!(game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::DefensiveResponse));
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
fn pact_visibility_and_exploration_drop_when_pact_downgrades() {
    let mut game = GameState::new_game(10, 10, 12345);
    let owner_a = 1;
    let owner_b = 2;
    set_relation(&mut game, owner_a, owner_b, DiplomacyStatus::Pact, 80);
    for tile in &mut game.tiles {
        tile.visible_by_owner.clear();
        tile.explored_by_owner.clear();
    }

    let idx = 5 * game.width + 5;
    game.tiles[idx].visible_by_owner.insert(owner_b);
    game.tiles[idx].explored_by_owner.insert(owner_b);

    assert!(game.tile_visible_to_owner(5, 5, owner_a));
    assert!(game.tile_explored_by_owner(5, 5, owner_a));

    game.update_diplomacy(owner_a, owner_b, DiplomacyStatus::Treaty)
        .expect("pact downgrade should succeed");

    assert!(!game.tile_visible_to_owner(5, 5, owner_a));
    assert!(!game.tile_explored_by_owner(5, 5, owner_a));
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
fn damaged_combat_unit_logs_strategic_retreat() {
    let mut game = GameState::new_game(10, 10, 12345);
    let player_owner = game.player_owner();
    let ai_owner = game.ai_owner();

    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    set_relation(&mut game, ai_owner, player_owner, DiplomacyStatus::War, -50);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Fallback Base".to_string(),
        x: 0,
        y: 0,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[0].base = Some(0);

    game.units.push(Unit {
        id: 0,
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 3,
        y: 3,
        moves_left: 1,
        hp: 3,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 3].unit = Some(0);

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

    smac_core::run_ai_tactics_for_owner(&mut game, ai_owner);

    let scout = game.unit(0).unwrap();
    assert!(
        scout.x < 3 || scout.y < 3,
        "damaged scout should fall back toward base"
    );
    assert!(game
        .log
        .iter()
        .any(|entry| entry.kind == EventLogKind::StrategicRetreat));
}

#[test]
fn event_kind_classifies_avoided_hopeless_attack() {
    assert_eq!(
        EventLogKind::classify("AVOIDED ATTACK: Spartans held Scout Patrol back."),
        EventLogKind::AvoidedHopelessAttack
    );
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
