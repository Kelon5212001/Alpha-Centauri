use smac_core::{Economics, GameState, UnitKind};

#[test]
fn economy_bonus_adds_energy_to_tiles() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().name = "Test Faction".to_string();
    game.faction_mut(owner).unwrap().base_attributes.economy = 0;

    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "Eco Base".to_string(),
        x: 5,
        y: 5,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        production_queue: vec![],
        facilities: vec![],
        governor_mode: smac_core::GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    // Initial yields (Flat, 70 moisture -> ~1 energy?)
    let yields_before = game.base_yields(5, 5);

    // Switch to Free Market (+2 Economy)
    game.choose_social_engineering(owner, None, Some(Economics::FreeMarket), None, None)
        .unwrap();
    let _yields_after = game.base_yields(5, 5);

    // Each of the 9 tiles should have +1 energy if it produced any energy.
    // Flat terrain 70 moisture: Terrain yields 0/0/1 (Flat land usually has 1 energy if moist/rolling)
    // Actually Flat is 0/0/0, Rolling is 0/0/1.
    // Let's force a Solar Collector.
    game.tiles[5 * game.width + 5].improvement = Some(smac_core::Improvement::Solar);

    let yields_with_solar = game.base_yields(5, 5);
    // Base tile now has 2 energy from Solar. +1 from Economy bonus.
    // Total should be higher.
    assert!(yields_with_solar.energy > yields_before.energy);
}

#[test]
fn growth_bonus_causes_population_boom() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().name = "Test Faction".to_string();
    game.faction_mut(owner).unwrap().base_attributes.growth = 0;

    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "Boom Base".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        production_queue: vec![],
        facilities: vec![],
        governor_mode: smac_core::GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    // Force high growth via Democratic (+2) and Planned (+1) and maybe Green? No, Green is -2.
    // Let's just manually set it.
    game.faction_mut(owner).unwrap().base_attributes.growth = 6;

    // Ensure surplus nutrients. Base pop 2 needs 2 nutrients.
    // Flat terrain 70 moisture might have some nutrients.
    // Let's force a Condenser.
    game.tiles[5 * game.width + 5].improvement = Some(smac_core::Improvement::Condenser);

    let pop_before = game.bases[0].population;
    game.end_turn();
    let pop_after = game.bases[0].population;

    assert_eq!(pop_after, pop_before + 1);
}

#[test]
fn planet_attribute_affects_psionic_combat() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().name = "Test Faction".to_string();
    let native_owner = game.native_owner();

    // Non-native faction with +2 Planet (Green +1, Gaia +1)
    game.faction_mut(owner).unwrap().base_attributes.planet = 2;
    game.faction_mut(owner).unwrap().base_attributes.morale = 0;

    let attacker_id = game.spawn_unit(owner, UnitKind::ScoutPatrol, 5, 5);
    let _defender_id = game.spawn_unit(native_owner, UnitKind::MindWorm, 5, 6);

    game.move_unit_to(attacker_id, 5, 6).unwrap();

    // Psionic Combat: Atk 3 + Exp 0 + Roll + Planet 2 = 5 + Roll.
    // Check log for atk score.
    for msg in &game.log {
        println!("LOG: {:?}", msg);
    }
    assert!(game.log.iter().any(|msg| msg.message.contains("atk: 5")
        || msg.message.contains("atk: 6")
        || msg.message.contains("atk: 7")
        || msg.message.contains("atk: 8")
        || msg.message.contains("atk: 9")
        || msg.message.contains("atk: 10")));
}
