use smac_core::{GameState, SecretProject, Terrain};

#[test]
fn weather_pattern_boosts_fungus_tile_yields() {
    let mut game = GameState::new_game(12, 12, 7);
    let owner = game.player_owner();

    // Create a base at (5, 5)
    game.tiles[5 * game.width + 5].terrain = Terrain::Flat; // 2 nutrients
    game.tiles[4 * game.width + 5].terrain = Terrain::Fungus;
    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "Test Base".to_string(),
        x: 5,
        y: 5,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        facilities: Vec::new(),
        production_queue: Vec::new(),
        governor_mode: smac_core::GovernorMode::Balanced,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    let base_yields = game.base_yields(5, 5);

    // Build Weather Pattern
    game.built_secret_projects
        .push((SecretProject::WeatherPattern, owner));

    let boosted_yields = game.base_yields(5, 5);
    assert_eq!(boosted_yields.nutrients, base_yields.nutrients + 1);
    assert_eq!(boosted_yields.minerals, base_yields.minerals + 1);
    assert_eq!(boosted_yields.energy, base_yields.energy + 1);
}

#[test]
fn clinical_immortality_stability_bonus() {
    let mut game = GameState::new_game(12, 12, 7);
    let owner = game.player_owner();

    // Create a base
    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "Test Base".to_string(),
        x: 5,
        y: 5,
        population: 5,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        facilities: Vec::new(),
        production_queue: Vec::new(),
        governor_mode: smac_core::GovernorMode::Balanced,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    // Unrest = 5 - 2 = 3.
    let base_unrest = game.base_unrest(0);
    assert_eq!(base_unrest, 3);

    // Build Clinical Immortality
    game.built_secret_projects
        .push((SecretProject::ClinicalImmortality, owner));

    let reduced_unrest = game.base_unrest(0);
    assert_eq!(reduced_unrest, 1); // 3 - 2
}

#[test]
fn empath_guild_effects() {
    let mut game = GameState::new_game(12, 12, 7);
    let owner = game.player_owner();

    // Create a base
    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "Test Base".to_string(),
        x: 5,
        y: 5,
        population: 5,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        facilities: Vec::new(),
        production_queue: Vec::new(),
        governor_mode: smac_core::GovernorMode::Balanced,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    let base_unrest = game.base_unrest(0);
    assert_eq!(base_unrest, 3);

    // Build Empath Guild
    game.built_secret_projects
        .push((SecretProject::EmpathGuild, owner));

    let reduced_unrest = game.base_unrest(0);
    assert_eq!(reduced_unrest, 2); // 3 - 1
}
