use smac_core::GameState;

fn sample_noise(seed: u32, x: i32, y: i32, salt: u32) -> u32 {
    let mut n = seed
        ^ salt
        ^ ((x as u32).wrapping_mul(374_761_393))
        ^ ((y as u32).wrapping_mul(668_265_263));

    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    n ^ (n >> 16)
}

#[test]
fn famine_increases_unrest() {
    let mut game = GameState::new_game(10, 10, 12345);
    let owner = game.player_owner();
    if let Some(faction) = game.faction_mut(owner) {
        faction.energy = 1000;
    }

    // Found a base and force it to have high population but NO nutrients
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;
    game.bases[base_id].minerals_stock = 1000;

    // Step 1: Baseline (Self-sufficient or surplus)
    // Most start with pop 1 or 2 and enough food.
    let baseline_unrest = game.base_unrest(base_id);

    // Step 2: Trigger Famine
    // We'll manually set the faction's food security to a very low level.
    if let Some(faction) = game.faction_mut(owner) {
        faction.food_security = -60; // Should add (60-20)/20 = 2 unrest
    }

    let famine_unrest = game.base_unrest(base_id);
    assert!(famine_unrest > baseline_unrest);
    assert_eq!(famine_unrest, baseline_unrest + 2);
}

#[test]
fn strategic_crises_emit_logs() {
    let mut game = GameState::new_game(10, 10, 12345);
    let owner = game.player_owner();
    if let Some(faction) = game.faction_mut(owner) {
        faction.energy = 1000;
    }

    // Give player a base so they don't disband units from famine and mess up logs
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;
    game.bases[base_id].minerals_stock = 1000;
    game.bases[base_id].population = 100; // Guarantee massive famine

    // Step 1: Trigger Famine Log
    game.end_turn();
    assert!(game
        .log
        .iter()
        .any(|msg| msg.message.contains("GLOBAL FAMINE")));

    // Step 2: Trigger Toxicity Log
    if let Some(faction) = game.faction_mut(owner) {
        faction.planet_toxicity = 60;
    }
    game.end_turn();
    assert!(game
        .log
        .iter()
        .any(|msg| msg.message.contains("ENVIRONMENTAL ALERT")));

    // Step 3: Trigger High AI Dependence Log
    if let Some(faction) = game.faction_mut(owner) {
        faction.ai_dependence = 80;
    }
    game.end_turn();
    assert!(game
        .log
        .iter()
        .any(|msg| msg.message.contains("GOVERNANCE WARNING")));
}

#[test]
fn debris_impact_damages_units() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    if let Some(faction) = game.faction_mut(owner) {
        faction.energy = 1000;
        faction.base_attributes.planet = 0;
    }

    // Setup base
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;
    let base = game.base(base_id).unwrap().clone();
    game.bases[base_id].minerals_stock = 10000;
    game.bases[base_id]
        .facilities
        .push(smac_core::Facility::NetworkNode);
    game.bases[base_id]
        .facilities
        .push(smac_core::Facility::RecyclingTanks);
    game.bases[base_id]
        .facilities
        .push(smac_core::Facility::RecreationCommons);

    // Compute the exact debris target for the next `end_turn()` and place a single
    // sacrificial unit there so the crisis remains deterministic and cheap.
    let next_turn = game.turn + 1;
    let dx = (sample_noise(game.seed, owner as i32, next_turn, 222) % 5) as i32 - 2;
    let dy = (sample_noise(game.seed, owner as i32, next_turn, 333) % 5) as i32 - 2;
    let impact_x = (base.x as i32 + dx).clamp(0, game.width as i32 - 1) as usize;
    let impact_y = (base.y as i32 + dy).clamp(0, game.height as i32 - 1) as usize;

    let scout_id = game
        .units
        .iter()
        .find(|u| u.owner == owner && u.alive && u.id != colony_pod_id)
        .unwrap()
        .id;
    if let Some(unit) = game.units.iter_mut().find(|u| u.id == scout_id) {
        unit.x = impact_x;
        unit.y = impact_y;
        unit.hp = 10;
    }
    game.tiles[impact_y * game.width + impact_x].unit = Some(scout_id);

    if let Some(faction) = game.faction_mut(owner) {
        faction.planet_toxicity = 100;
    }
    game.end_turn();

    assert!(
        game.faction(owner).unwrap().planet_toxicity > 90,
        "planet toxicity should stay in debris-impact range for this scenario"
    );
    assert!(
        game.log
            .iter()
            .any(|msg| msg.message.contains("Debris impact")),
        "debris impact should trigger when toxicity exceeds 90 and valid targets exist"
    );
    assert!(
        game.units
            .iter()
            .any(|unit| unit.id == scout_id && (!unit.alive || unit.hp < 10)),
        "debris impact should damage or destroy at least one unit"
    );
}

#[test]
fn governance_override_forces_machine_polity() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();

    // Setup base with Manual control
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;

    if let Some(base) = game.base_mut(base_id) {
        base.governor_mode = smac_core::GovernorMode::Off;
    }

    if let Some(faction) = game.faction_mut(owner) {
        faction.ai_dependence = 95;
    }

    // Iterate turns until override occurs (15% chance)
    let mut override_occurred = false;
    for _ in 0..100 {
        game.end_turn();
        if game
            .log
            .iter()
            .any(|msg| msg.message.contains("GOVERNANCE OVERRIDE"))
        {
            override_occurred = true;
            break;
        }
    }

    assert!(
        override_occurred,
        "Governance override should eventually trigger at 95 dependence"
    );
    assert_eq!(
        game.base(base_id).unwrap().governor_mode,
        smac_core::GovernorMode::MachinePolity
    );
}
