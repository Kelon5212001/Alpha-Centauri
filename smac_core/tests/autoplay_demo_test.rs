use smac_core::{Armor, GameState, UnitKind, Weapon};

#[test]
fn autoplay_mission_year_founds_opening_bases_for_both_sides() {
    let mut game = GameState::new_game(16, 16, 7);

    game.run_autoplay_mission_year();

    assert_eq!(game.turn, 2);
    assert!(!game.bases_for(game.player_owner()).is_empty());
    assert!(!game.bases_for(game.ai_owner()).is_empty());
}

#[test]
fn autoplay_smoke_advances_multiple_turns_without_panicking() {
    let mut game = GameState::new_game(16, 16, 11);

    for _ in 0..20 {
        if game.game_over.is_some() {
            break;
        }
        game.run_autoplay_mission_year();
    }

    assert!(game.turn >= 10);
    assert!(!game.bases_for(game.player_owner()).is_empty());
    assert!(!game.bases_for(game.ai_owner()).is_empty());
}

#[test]
fn autoplay_demo_profile_reaches_one_hundred_turns() {
    let mut game = GameState::new_game(20, 20, 7);

    for _ in 0..100 {
        if game.game_over.is_some() {
            break;
        }
        game.run_autoplay_mission_year();
    }

    // Either we reached turn 101 (loop finished 100 iterations) or someone won
    assert!(
        game.turn >= 50,
        "Game ended suspiciously early at turn {}",
        game.turn
    );
    assert!(game.turn == 101 || game.game_over.is_some());
    assert!(game.bases_for(game.player_owner()).len() >= 1);
    assert!(game.bases_for(game.ai_owner()).len() >= 1);
}

#[test]
fn custom_unit_design_lookup_matches_runtime_designs() {
    let game = GameState::new_game(16, 16, 7);
    let owner = game.player_owner();
    let design = game
        .faction(owner)
        .expect("player faction must exist")
        .unit_designs[0]
        .clone();

    let index = game.find_design_index_for_kind(owner, UnitKind::CustomUnit(design));

    assert_eq!(index, 0);
}

#[test]
fn upgrading_unit_updates_design_index_and_runtime_stats() {
    let mut game = GameState::new_game(16, 16, 7);
    let owner = game.player_owner();
    game.faction_mut(owner)
        .expect("player faction must exist")
        .energy = 500;

    let unit_id = game
        .units
        .iter()
        .find(|unit| {
            unit.owner == owner && unit.alive && matches!(unit.kind, UnitKind::ScoutPatrol)
        })
        .map(|unit| unit.id)
        .expect("player should start with a scout patrol");

    let mut upgraded_design = game
        .faction(owner)
        .expect("player faction must exist")
        .unit_designs[0]
        .clone();
    upgraded_design.name = "Autoplay Test Spear".to_string();
    upgraded_design.weapon = Weapon::ResonanceLaser(6);
    upgraded_design.armor = Armor::ResonanceArmor(2);
    upgraded_design.recompute_cost();

    game.upgrade_unit(unit_id, upgraded_design.clone())
        .expect("upgrade should succeed");

    let stored_index = game
        .faction(owner)
        .and_then(|faction| {
            faction
                .unit_designs
                .iter()
                .position(|design| design == &upgraded_design)
        })
        .expect("upgraded design must be stored on the owning faction");
    let unit = game.unit(unit_id).expect("upgraded unit must exist");

    assert_eq!(unit.design_index, stored_index);
    assert!(matches!(unit.kind, UnitKind::CustomUnit(_)));
    assert_eq!(
        game.unit_attack_strength(unit_id),
        upgraded_design.attack_strength() as i32
    );
    assert_eq!(
        game.unit_defense_strength(unit_id),
        upgraded_design.defense_strength() as i32
    );
}
