use smac_core::{Economics, GameState, Politics};

#[test]
fn efficiency_affects_base_expansion_limit_and_unrest() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().name = "Test Faction".to_string();
    game.faction_mut(owner).unwrap().base_attributes.efficiency = 0;

    // 1. Initial State (Frontier: 0 Efficiency)
    // Map factor: (16*16)/128 = 2. Limit: 4 + 2 + (0*2) = 6.
    assert_eq!(game.base_expansion_limit(owner), 6);

    // 2. Switch to Police State (-2 Efficiency)
    // Limit: 4 + 2 + (-2*2) = 2.
    game.choose_social_engineering(owner, Some(Politics::Police), None, None, None)
        .unwrap();
    assert_eq!(game.base_expansion_limit(owner), 2);

    // 3. Create 4 bases
    // Base 0 already exists from new_game? No, new_game doesn't create bases.
    // Wait, new_game might create a starting unit but not a base.
    game.bases.clear();
    for i in 0..4 {
        game.bases.push(smac_core::Base {
            id: i,
            owner,
            name: format!("Base {}", i),
            x: i,
            y: 0,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: smac_core::ProductionItem::ScoutPatrol,
            production_queue: vec![],
            facilities: vec![],
            governor_mode: smac_core::GovernorMode::Off,
        });
        game.tiles[0 * game.width + i].base = Some(i);
    }

    // With 4 bases and limit 2, bureaucracy unrest = (4-2)/2 = 1.
    // Base unrest = pop 2 - 2 + bureaucracy 1 = 1.
    assert_eq!(game.base_unrest(0), 1);

    // 4. Switch to Democratic (+2 Efficiency)
    // Efficiency 0 + 2 = 2.
    // Limit: 4 + 2 + (2*2) = 10.
    game.choose_social_engineering(owner, Some(Politics::Democratic), None, None, None)
        .unwrap();
    assert_eq!(game.base_expansion_limit(owner), 10);

    // With 4 bases and limit 10, bureaucracy unrest = 0.
    // Base unrest = pop 2 - 2 + 0 = 0.
    assert_eq!(game.base_unrest(0), 0);
}

#[test]
fn efficiency_affects_energy_waste() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().name = "Test Faction".to_string();
    game.faction_mut(owner).unwrap().base_attributes.efficiency = 0;
    game.bases.clear();

    // 1. Setup HQ and a distant base
    game.bases.push(smac_core::Base {
        id: 0,
        owner,
        name: "HQ".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        production_queue: vec![],
        facilities: vec![],
        governor_mode: smac_core::GovernorMode::Off,
    });
    game.tiles[0].base = Some(0);
    game.faction_mut(owner).unwrap().headquarters_base_id = Some(0);

    game.bases.push(smac_core::Base {
        id: 1,
        owner,
        name: "Colony".to_string(),
        x: 10,
        y: 10,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: smac_core::ProductionItem::ScoutPatrol,
        production_queue: vec![],
        facilities: vec![],
        governor_mode: smac_core::GovernorMode::Off,
    });
    game.tiles[10 * game.width + 10].base = Some(1);

    // Distance = 20.
    // Efficiency 0. Efficiency Score = 10 + 0 = 10.
    // Waste = (20 * 2) / 10 = 4%.
    assert_eq!(game.energy_waste_pct(1), 4);

    // 2. Switch to Planned Economics (-2 Efficiency)
    // Efficiency -2. Efficiency Score = 10 + (-4) = 6.
    // Waste = (20 * 2) / 6 = 6%.
    game.choose_social_engineering(owner, None, Some(Economics::Planned), None, None)
        .unwrap();
    assert_eq!(game.energy_waste_pct(1), 6);

    // 3. Switch to Free Market (+2 Efficiency)
    // Efficiency +2. Efficiency Score = 10 + 4 = 14.
    // Waste = (20 * 2) / 14 = 2%.
    game.choose_social_engineering(owner, None, Some(Economics::FreeMarket), None, None)
        .unwrap();
    assert_eq!(game.energy_waste_pct(1), 2);
}
