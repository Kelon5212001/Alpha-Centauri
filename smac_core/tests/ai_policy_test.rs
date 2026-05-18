use smac_core::{Base, GameState, GovernorMode, ProductionItem, Terrain, Unit, UnitKind};

#[test]
fn ai_turn_switches_base_production_to_policy_preference() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rocky;
        }
    }
    let tile_idx = base_y * game.width + base_x;
    game.tiles[tile_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Test".to_string(),
        x: base_x,
        y: base_y,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ColonyPod,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    // Add dummy bases to saturate expansion target
    for i in 1..8 {
        game.bases.push(Base {
            id: i,
            owner: ai_owner,
            name: format!("Dummy {}", i),
            x: 0,
            y: 0,
            population: 1,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
    }

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::ScoutPatrol
    );
}

#[test]
fn ai_former_uses_terraform_bias_to_improve_its_tile() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();

    let unit_id = game.units.len();
    let x = 10;
    let y = 10;
    let tile_idx = y * game.width + x;
    game.tiles[tile_idx].terrain = Terrain::Flat;
    game.tiles[tile_idx].moisture = 70;
    game.tiles[tile_idx].elevation = 0;
    game.tiles[tile_idx].improvement = None;
    game.tiles[tile_idx].unit = Some(unit_id);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.energy = 100;
    }

    game.units.push(Unit {
        id: unit_id,
        owner: ai_owner,
        kind: UnitKind::Former,
        design_index: 0,
        x,
        y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.tile(x, y).expect("tile should exist").improvement,
        Some(smac_core::Improvement::Farm)
    );
}

#[test]
fn ai_former_chooses_solar_on_dry_tiles() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();

    let unit_id = game.units.len();
    let x = 9;
    let y = 9;
    let tile_idx = y * game.width + x;
    game.tiles[tile_idx].terrain = Terrain::Flat;
    game.tiles[tile_idx].moisture = 10;
    game.tiles[tile_idx].improvement = None;
    game.tiles[tile_idx].unit = Some(unit_id);

    game.units.push(Unit {
        id: unit_id,
        owner: ai_owner,
        kind: UnitKind::Former,
        design_index: 0,
        x,
        y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.tile(x, y).expect("tile should exist").improvement,
        Some(smac_core::Improvement::Solar)
    );
}

#[test]
fn ai_prefers_former_when_energy_and_research_pressure_are_high() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Flat;
            game.tiles[idx].moisture = 10;
            game.tiles[idx].elevation = 20;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Energy Test".to_string(),
        x: base_x,
        y: base_y,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.energy = 0;
        faction.research = 0;
        faction.known_techs.push(smac_core::Tech::CentauriEcology);
    }

    game.bases[0].minerals_stock = 4;
    game.units.clear();

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::Former
    );
}

#[test]
fn ai_prefers_greenhouse_when_food_security_is_thin() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    game.units.clear();
    game.bases.clear();

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rocky;
            game.tiles[idx].moisture = 10;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Food Test".to_string(),
        x: base_x,
        y: base_y,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::Biogenetics);
        faction.energy = 100;
    }

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::Greenhouse
    );
}

#[test]
fn unrest_with_recreation_commons_surfaces_hologram_theatre() {
    let mut game = GameState::new_game(12, 12, 7);
    let ai_owner = game.ai_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Rocky;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Morale Test".to_string(),
        x: 5,
        y: 5,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::RecreationCommons],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::PlanetaryNetworks);
    }

    let plan = game.base_governor_plan(0);
    assert_eq!(
        plan.first().map(|step| step.item),
        Some(ProductionItem::HologramTheatre)
    );
}

#[test]
fn ai_prefers_mineral_refinery_when_extraction_is_thin() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    game.turn = 1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Ocean;
        tile.improvement = None;
    }

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Ocean;
            game.tiles[idx].moisture = 80;
            game.tiles[idx].elevation = 10;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].terrain = Terrain::Flat;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Mineral Test".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::Greenhouse],
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::IndustrialBase);
    }

    assert!(game.is_production_available(ai_owner, ProductionItem::MineralRefinery));
    let yields = game
        .base(0)
        .and_then(|base| game.effective_base_yields(base.id))
        .expect("base yields should exist");
    assert!(yields.minerals <= 3);
    assert!(yields.energy >= yields.minerals);
    assert!(
        game.base_mineral_margin(0)
            .expect("base mineral margin should exist")
            <= 0
    );

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::MineralRefinery
    );
}

#[test]
fn ai_prefers_trade_exchange_when_trade_links_can_boost_energy() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
        tile.moisture = 70;
        tile.improvement = None;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Trade Hub".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.bases.push(Base {
        id: 1,
        owner: ai_owner,
        name: "Sparta Outpost".to_string(),
        x: 9,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 9].base = Some(1);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.energy = 0;
        faction.research = 0;
        faction
            .known_techs
            .push(smac_core::Tech::InformationNetworks);
    }

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::TradeExchange
    );
}

#[test]
fn ai_prefers_freight_depot_when_trade_links_can_boost_minerals() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
        tile.moisture = 70;
        tile.improvement = None;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Freight Hub".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![
            smac_core::Facility::TradeExchange,
            smac_core::Facility::MineralRefinery,
        ],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.bases.push(Base {
        id: 1,
        owner: ai_owner,
        name: "Sparta Freight Link".to_string(),
        x: 9,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 9].base = Some(1);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::IndustrialBase);
    }

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::FreightDepot
    );
}

#[test]
fn convoy_pressure_surfaces_patrol_grid_in_governor_planning() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = smac_core::content_api::runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
        tile.moisture = 70;
        tile.improvement = None;
    }

    game.bases.push(Base {
        id: 0,
        owner: roles.player,
        name: "Logistics Hub".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.bases.push(Base {
        id: 1,
        owner: roles.player,
        name: "Logistics Spoke".to_string(),
        x: 9,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 9].base = Some(1);
    game.add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    let enemy_id = 200;
    game.tiles[5 * game.width + 7].unit = Some(enemy_id);
    game.units.push(Unit {
        id: enemy_id,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 7,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(smac_core::Tech::DoctrineMobility);
    }

    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::EscortSpeeder));
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::PatrolGrid));
}

#[test]
fn ai_repairs_damaged_convoy_routes_during_economy_phase() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = smac_core::content_api::runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(Base {
        id: 0,
        owner: roles.ai,
        name: "AI Hub".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    game.bases.push(Base {
        id: 1,
        owner: roles.ai,
        name: "AI Spoke".to_string(),
        x: 9,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 9].base = Some(1);
    game.add_convoy_route_typed(0, 1, smac_core::ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 2;

    game.end_turn();

    assert_eq!(game.convoy_routes[0].integrity, 3);
}

#[test]
fn ai_prefers_scouts_when_player_military_pressure_is_nearby() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Flat;
            game.tiles[idx].moisture = 55;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Frontline".to_string(),
        x: base_x,
        y: base_y,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: game.units.len(),
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: base_x,
        y: base_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[center_idx].unit = Some(0);

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::PerimeterDefense
    );
}

#[test]
fn ai_can_choose_garrison_guard_under_frontline_pressure_without_facility_need() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rolling;
            game.tiles[idx].moisture = 35;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Guard Test".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: vec![
            smac_core::Facility::PerimeterDefense,
            smac_core::Facility::SensorArray,
            smac_core::Facility::TransitHub,
        ],
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: game.units.len(),
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: base_x,
        y: base_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[center_idx].unit = Some(0);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::IndustrialBase);
    }

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::GarrisonGuard
    );
}

#[test]
fn ai_prefers_raider_speeders_when_mobility_and_attack_pressure_are_available() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Flat;
            game.tiles[idx].moisture = 35;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Speeder Test".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: vec![
            smac_core::Facility::PerimeterDefense,
            smac_core::Facility::SensorArray,
            smac_core::Facility::TransitHub,
        ],
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: game.units.len(),
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: base_x,
        y: base_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[center_idx].unit = Some(0);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::DoctrineMobility);
    }

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::RaiderSpeeder
    );
}

#[test]
fn ai_prefers_standard_speeders_when_terrain_is_poor_for_raiding() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rocky;
            game.tiles[idx].moisture = 20;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Mobility Test".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::PerimeterDefense],
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: game.units.len(),
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: base_x,
        y: base_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[center_idx].unit = Some(0);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::DoctrineMobility);
    }

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::Speeder
    );
}

#[test]
fn frontline_pressure_surfaces_military_academy_in_governor_planning() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rocky;
            game.tiles[idx].moisture = 35;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Academy Test".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::IndustrialBase);
    }

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::MilitaryAcademy));
}

#[test]
fn frontline_pressure_can_surface_resonance_laser_after_basic_defenses() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Rolling;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Shock Test".to_string(),
        x: 10,
        y: 10,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![
            smac_core::Facility::PerimeterDefense,
            smac_core::Facility::SensorArray,
            smac_core::Facility::MilitaryAcademy,
            smac_core::Facility::TransitHub,
            smac_core::Facility::ForwardDepot,
        ],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[10 * game.width + 10].base = Some(0);

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::DoctrineMobility);
        faction.known_techs.push(smac_core::Tech::FieldModulation);
    }

    game.units.push(Unit {
        id: 0,
        owner: game.player_owner(),
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 8,
        y: 10,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[10 * game.width + 8].unit = Some(0);

    let defense_items = game.base_defense_plan_items(0, 5);
    assert!(defense_items.contains(&ProductionItem::ResonanceLaser));
}

#[test]
fn progenitor_psych_unlocks_psi_sentinel_in_governor_planning() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    let native_owner = game.native_owner();

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rolling;
            game.tiles[idx].moisture = 40;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Psionic Front".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::ProgenitorPsych);
    }

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    let native_unit_id = game.units.len();
    let native_x = base_x;
    let native_y = base_y.saturating_sub(2);
    let native_idx = native_y * game.width + native_x;
    game.tiles[native_idx].unit = Some(native_unit_id);
    game.units.push(Unit {
        id: native_unit_id,
        owner: native_owner,
        kind: UnitKind::MindWorm,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::PsiSentinel));
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::PsiBeacon));
}

#[test]
fn ai_prefers_psi_sentinel_when_native_psi_pressure_is_nearby() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let native_owner = game.native_owner();
    game.units.clear();

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rolling;
            game.tiles[idx].moisture = 45;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Psi Test".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::PerimeterDefense],
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::ProgenitorPsych);
    }

    let native_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(native_unit_id);
    game.units.push(Unit {
        id: native_unit_id,
        owner: native_owner,
        kind: UnitKind::MindWorm,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    assert_eq!(
        game.base(0).expect("ai base should exist").production,
        ProductionItem::PsiSentinel
    );
}

#[test]
fn governor_plan_can_surface_forward_depot_for_mobile_frontline_support() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    let base_x: usize = 11;
    let base_y: usize = 11;
    for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
        for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
            let idx = y * game.width + x;
            game.tiles[idx].terrain = Terrain::Rolling;
            game.tiles[idx].moisture = 45;
        }
    }
    let center_idx = base_y * game.width + base_x;
    game.tiles[center_idx].base = Some(0);

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Sparta Forward Depot".to_string(),
        x: base_x,
        y: base_y,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: vec![
            smac_core::Facility::PerimeterDefense,
            smac_core::Facility::SensorArray,
            smac_core::Facility::TransitHub,
        ],
        governor_mode: GovernorMode::Off,
    });

    if let Some(faction) = game.faction_mut(ai_owner) {
        faction.known_techs.push(smac_core::Tech::DoctrineMobility);
    }

    let player_unit_id = game.units.len();
    let pressure_x = base_x.saturating_sub(2);
    let pressure_y = base_y;
    let pressure_idx = pressure_y * game.width + pressure_x;
    game.tiles[pressure_idx].unit = Some(player_unit_id);

    // Add a dummy base for the player so their unit isn't disbanded due to famine
    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.units.push(Unit {
        id: player_unit_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: pressure_x,
        y: pressure_y,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::ForwardDepot));
}

#[test]
fn damaged_ai_unit_retreats_toward_friendly_base() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Fallback".to_string(),
        x: 11,
        y: 11,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[11 * game.width + 11].base = Some(0);

    game.units.push(Unit {
        id: 0,
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 8,
        y: 8,
        moves_left: 1,
        hp: 4,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[8 * game.width + 8].unit = Some(0);

    game.units.push(Unit {
        id: 1,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 7,
        y: 8,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[8 * game.width + 7].unit = Some(1);

    game.end_turn();

    let ai_unit = game.unit(0).expect("ai unit should survive");
    assert_eq!((ai_unit.x, ai_unit.y), (9, 9));
}

#[test]
fn damaged_ai_unit_stays_on_friendly_base_to_heal() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Hospital".to_string(),
        x: 11,
        y: 11,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[11 * game.width + 11].base = Some(0);

    game.units.push(Unit {
        id: 0,
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 11,
        y: 11,
        moves_left: 1,
        hp: 4,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[11 * game.width + 11].unit = Some(0);

    game.end_turn();

    let ai_unit = game.unit(0).expect("ai unit should survive");
    assert_eq!((ai_unit.x, ai_unit.y), (11, 11));
    assert!(ai_unit.hp > 4);
}

#[test]
fn damaged_ai_unit_avoids_enemy_adjacent_retreat_step_when_falling_back() {
    let mut game = GameState::new_game(16, 16, 7);
    let ai_owner = game.ai_owner();
    let player_owner = game.player_owner();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = Terrain::Flat;
    }

    game.bases.push(Base {
        id: 0,
        owner: ai_owner,
        name: "Fallback".to_string(),
        x: 11,
        y: 11,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[11 * game.width + 11].base = Some(0);

    game.units.push(Unit {
        id: 0,
        owner: ai_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 8,
        y: 8,
        moves_left: 1,
        hp: 4,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[8 * game.width + 8].unit = Some(0);

    game.units.push(Unit {
        id: 1,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 9,
        y: 9,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[9 * game.width + 9].unit = Some(1);

    game.tiles[0].base = Some(game.bases.len());
    game.bases.push(Base {
        id: game.bases.len(),
        owner: player_owner,
        name: "Dummy Player Base".to_string(),
        x: 0,
        y: 0,
        population: 1,
        nutrients_stock: 0,
        minerals_stock: 100,
        production: ProductionItem::Former,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });

    game.end_turn();

    let ai_unit = game.unit(0).expect("ai unit should survive");
    assert_ne!((ai_unit.x, ai_unit.y), (9, 9));
    let distance_from_enemy = (ai_unit.x as isize - 9).abs() + (ai_unit.y as isize - 9).abs();
    assert!(distance_from_enemy >= 2);
}
