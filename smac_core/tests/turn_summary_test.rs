use smac_core::{AlertPriority, GameState};

#[test]
fn turn_summary_reports_empty_queue() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();

    // Found a base and clear its queue
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;
    game.bases[base_id].production_queue.clear();

    let summary = game.generate_turn_summary(owner);

    let empty_queue_alert = summary
        .alerts
        .iter()
        .find(|a| a.message.contains("Production queue empty"));
    assert!(empty_queue_alert.is_some());
    assert_eq!(empty_queue_alert.unwrap().priority, AlertPriority::High);
    assert_eq!(empty_queue_alert.unwrap().base_id, Some(base_id));
}

#[test]
fn turn_summary_reports_famine() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();

    // Found a base and force famine
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;

    // Set population high and stocks low to trigger famine (needs nutrient margin < 0)
    // base_food_margin uses operational_base_yields which uses tile_total_yields
    // We'll just check if it detects it when we mock the condition.
    // Actually base_food_margin is real, so we need to ensure the base is in a bad spot.
    game.bases[base_id].population = 100;

    let summary = game.generate_turn_summary(owner);

    let famine_alert = summary
        .alerts
        .iter()
        .find(|a| a.message.contains("FOOD SHORTAGE"));
    assert!(famine_alert.is_some());
    assert_eq!(famine_alert.unwrap().priority, AlertPriority::Critical);
}

#[test]
fn turn_summary_reports_financial_crisis() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();

    // Add expensive facilities to increase upkeep
    let colony_pod_id = game.live_units_for(owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(owner)[0].id;
    game.bases[base_id]
        .facilities
        .push(smac_core::Facility::NetworkNode);
    game.bases[base_id]
        .facilities
        .push(smac_core::Facility::RecreationCommons);

    if let Some(faction) = game.faction_mut(owner) {
        faction.energy = 0; // Broke
    }

    let summary = game.generate_turn_summary(owner);

    let energy_alert = summary
        .alerts
        .iter()
        .find(|a| a.message.contains("FINANCIAL CRISIS"));
    assert!(energy_alert.is_some());
    assert_eq!(energy_alert.unwrap().priority, AlertPriority::Critical);
}

#[test]
fn turn_summary_reports_pending_diplomacy() {
    let mut game = GameState::new_game(16, 16, 12345);
    let player_owner = game.player_owner();
    let ai_owner = game.ai_owner();

    game.pending_diplomacy_offers.push((
        ai_owner,
        player_owner,
        smac_core::DiplomacyStatus::Treaty,
    ));

    let summary = game.generate_turn_summary(player_owner);

    let diplomacy_alert = summary
        .alerts
        .iter()
        .find(|a| a.message.contains("PENDING OFFER"));
    assert!(diplomacy_alert.is_some());
}

#[test]
fn turn_summary_reports_threats() {
    let mut game = GameState::new_game(16, 16, 12345);
    let player_owner = game.player_owner();
    let ai_owner = game.ai_owner();

    // Found a base
    let colony_pod_id = game.live_units_for(player_owner)[0].id;
    game.found_base(colony_pod_id).unwrap();
    let base_id = game.bases_for(player_owner)[0].id;
    let base_x = game.bases[base_id].x;
    let base_y = game.bases[base_id].y;

    // Spawn many enemy units nearby to trigger high pressure
    for _ in 0..5 {
        game.spawn_unit(
            ai_owner,
            smac_core::UnitKind::ScoutPatrol,
            base_x.saturating_sub(1),
            base_y.saturating_sub(1),
        );
    }

    let summary = game.generate_turn_summary(player_owner);

    let threat_alert = summary
        .alerts
        .iter()
        .find(|a| a.message.contains("IMMINENT THREAT"));
    assert!(threat_alert.is_some());
}
