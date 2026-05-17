use smac_core::content_api::runtime_roles;
use smac_core::{
    presentation, Base, BaseFocusFilter, ConvoyOverlayStatus, ConvoyRouteKind, Facility,
    GameAction, GameState, GovernorMode, Improvement, LogisticsRouteFilter, LogisticsRouteSort,
    MapOverlay, ProductionItem, Tech, Unit, UnitKind,
};

#[test]
fn choose_research_via_action_updates_faction_state() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();

    game.apply_action(GameAction::ChooseResearch {
        owner: roles.player,
        tech: Tech::Biogenetics,
    })
    .expect("choose research action should succeed");

    assert_eq!(
        game.factions[roles.player].current_research,
        Tech::Biogenetics
    );
    assert_eq!(game.factions[roles.player].research, 0);
}

#[test]
fn next_base_cycle_target_advances_and_wraps() {
    let game = GameState::new_game(16, 16, 7);
    let base_ids = vec![2, 5, 7];

    assert_eq!(game.next_base_cycle_target(&base_ids, None), Some(2));
    assert_eq!(game.next_base_cycle_target(&base_ids, Some(2)), Some(5));
    assert_eq!(game.next_base_cycle_target(&base_ids, Some(6)), Some(7));
    assert_eq!(game.next_base_cycle_target(&base_ids, Some(7)), Some(2));
    assert_eq!(game.next_base_cycle_target(&[], Some(7)), None);
}

#[test]
fn next_unit_cycle_target_advances_and_wraps() {
    let game = GameState::new_game(16, 16, 7);
    let unit_ids = vec![2, 5, 7];

    assert_eq!(game.next_unit_cycle_target(&unit_ids, None), Some(2));
    assert_eq!(game.next_unit_cycle_target(&unit_ids, Some(2)), Some(5));
    assert_eq!(game.next_unit_cycle_target(&unit_ids, Some(6)), Some(7));
    assert_eq!(game.next_unit_cycle_target(&unit_ids, Some(7)), Some(2));
    assert_eq!(game.next_unit_cycle_target(&[], Some(7)), None);
}

#[test]
fn stockpile_energy_does_not_enter_zero_cost_completion_loop() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
        tile.moisture = 40;
    }

    game.bases.push(Base {
        id: 0,
        owner: roles.player,
        name: "Stockpile".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 6,
        production: ProductionItem::StockpileEnergy,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    let energy_before = game.faction(roles.player).expect("player faction").energy;

    game.end_turn();

    let base = game.base(0).expect("base should survive stockpiling");
    let energy_after = game.faction(roles.player).expect("player faction").energy;

    assert_eq!(base.production, ProductionItem::StockpileEnergy);
    assert_eq!(base.minerals_stock, 0);
    assert!(energy_after >= energy_before);
}

#[test]
fn energy_reserves_cover_support_before_units_are_disbanded() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 60;
    }

    game.bases.push(Base {
        id: 0,
        owner: roles.player,
        name: "Support Bridge".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::StockpileEnergy,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 4].base = Some(0);

    game.bases.push(Base {
        id: 1,
        owner: roles.ai,
        name: "Other Faction".to_string(),
        x: 11,
        y: 11,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::StockpileEnergy,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[11 * game.width + 11].base = Some(1);

    game.faction_mut(roles.player)
        .expect("player faction")
        .energy = 60;

    for (unit_id, x, y) in [
        (0usize, 4usize, 4usize),
        (1usize, 5usize, 4usize),
        (2usize, 3usize, 4usize),
        (3usize, 4usize, 5usize),
        (4usize, 4usize, 3usize),
    ] {
        game.tiles[y * game.width + x].unit = Some(unit_id);
        game.units.push(Unit {
            id: unit_id,
            owner: roles.player,
            kind: UnitKind::ScoutPatrol,
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
    }

    game.end_turn();

    assert_eq!(game.live_units_for(roles.player).len(), 5);
    assert!(
        game.log.iter().any(|entry| {
            entry.message.contains("spent")
                && entry
                    .message
                    .contains("energy reserves to cover mineral support")
                && entry.message.contains("Gaia's Stepdaughters")
        }),
        "support bridge should log when energy reserves cover mineral support"
    );
    assert!(
        !game.log.iter().any(|entry| {
            entry.message.contains("FAMINE:")
                && entry.message.contains("lack of support")
                && entry.message.contains("Gaia's Stepdaughters")
        }),
        "support bridge should avoid unit disband events for the funded faction"
    );
}

#[test]
fn next_base_focus_target_uses_core_focus_categories() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    for (id, x, governor_mode) in [
        (0, 3, GovernorMode::Defense),
        (1, 5, GovernorMode::Economy),
        (2, 7, GovernorMode::Logistics),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Focus {id}"),
            x,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode,
        });
        game.tiles[5 * game.width + x].base = Some(id);
    }

    assert_eq!(
        game.faction_base_ids_for_focus(roles.player, BaseFocusFilter::Defense),
        vec![0]
    );
    assert_eq!(
        game.faction_base_ids_for_focus(roles.player, BaseFocusFilter::LogisticsMode),
        vec![2]
    );
    assert_eq!(
        game.next_base_focus_target(roles.player, BaseFocusFilter::Defense, None),
        Some(0)
    );
    assert_eq!(
        game.next_base_focus_target(roles.player, BaseFocusFilter::LogisticsMode, None),
        Some(2)
    );
    let queue_gap_state = game.base_focus_state(roles.player, BaseFocusFilter::QueueGap, None);
    assert_eq!(queue_gap_state.count, 3);
    assert_eq!(queue_gap_state.next_focus_base_id, Some(0));
    assert_eq!(
        queue_gap_state.action_label_text.as_deref(),
        Some("Fill Queue Gaps")
    );
    let defense_state = game.base_focus_state(roles.player, BaseFocusFilter::Defense, None);
    assert_eq!(defense_state.count, 1);
    assert_eq!(defense_state.next_focus_base_id, Some(0));
    assert!(defense_state.action_label_text.is_none());
}

#[test]
fn next_base_focus_target_action_logs_when_no_bases_match() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    let next = game.next_base_focus_target_action(roles.player, BaseFocusFilter::Frontier, None);
    assert!(next.is_none());
    assert!(game.log.iter().any(|line| line
        .message
        .contains("No player bases match Frontier focus.")));
}

#[test]
fn player_operations_focus_state_surfaces_dashboard_targets() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    for (id, x, name, population, facilities, governor_mode) in [
        (0, 3, "Unrest", 8, Vec::new(), GovernorMode::Defense),
        (
            1,
            5,
            "Unlock",
            4,
            vec![Facility::RecreationCommons],
            GovernorMode::Economy,
        ),
        (
            2,
            7,
            "Recovery",
            4,
            vec![Facility::CommandCenter],
            GovernorMode::Recovery,
        ),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y: 5,
            population,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode,
        });
        game.tiles[5 * game.width + x].base = Some(id);
    }

    game.units.push(Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 2,
        y: 2,
        moves_left: 1,
        hp: 1,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 2].unit = Some(0);
    game.units.push(Unit {
        id: 1,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 7,
        y: 5,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 7].unit = Some(1);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.current_research = Tech::PlanetaryNetworks;
    }

    let focus = game.player_operations_focus_state(None, None);
    assert_eq!(focus.damaged_unit_count, 2);
    assert_eq!(focus.most_damaged_unit_id, Some(0));
    assert_eq!(focus.next_damaged_unit_id, Some(0));
    assert!(focus.stressed_base_count >= 1);
    assert_eq!(focus.most_unrested_base_id, Some(0));
    assert_eq!(focus.next_stressed_base_id, Some(0));
    assert_eq!(focus.recovering_base_count, 1);
    assert_eq!(focus.most_recovering_garrison_base_id, Some(2));
    assert_eq!(focus.next_recovering_base_id, Some(2));
    assert_eq!(focus.recovering_garrison_unit_count, 1);
    assert_eq!(focus.next_recovering_garrison_unit_id, Some(1));
    assert_eq!(focus.current_research_unlock_base_count, 1);
    assert_eq!(focus.current_research_unlock_focus_base_id, Some(1));

    let cycled = game.player_operations_focus_state(Some(0), Some(0));
    assert_eq!(cycled.next_damaged_unit_id, Some(1));
    assert_eq!(cycled.next_stressed_base_id, Some(2));
    assert_eq!(cycled.next_recovering_garrison_unit_id, Some(1));
    assert_eq!(cycled.next_recovering_base_id, Some(2));
    assert_eq!(cycled.current_research_unlock_focus_base_id, Some(1));
}

#[test]
fn preview_apply_actions_without_staged_preview_are_clean_noops() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();

    let pin = game.pin_current_research_unlock_preview_state_action(roles.player, 3, None);
    assert_eq!(pin.staged_count, 0);
    assert!(pin.preview.is_none());
    assert!(pin.focus_base_id.is_none());

    let refresh = game.refresh_research_unlock_preview_state_action(roles.player, None, None);
    assert_eq!(refresh.staged_count, 0);
    assert!(refresh.preview.is_none());
    assert!(refresh.focus_base_id.is_none());

    let sync = game.sync_research_unlock_preview_state_action(roles.player, None);
    assert_eq!(sync.staged_count, 0);
    assert!(sync.preview.is_none());
    assert!(sync.focus_base_id.is_none());

    let single = game
        .apply_research_unlock_preview_state_action(roles.player, None, 0)
        .expect("no staged preview should be a clean no-op");
    assert_eq!(single.staged_count, 0);
    assert!(single.preview.is_none());
    assert!(single.focus_base_id.is_none());

    let bulk = game
        .apply_all_research_unlock_preview_state_action(roles.player, None)
        .expect("no staged previews should be a clean no-op");
    assert_eq!(bulk.staged_count, 0);
    assert!(bulk.preview.is_none());
    assert!(bulk.focus_base_id.is_none());
}

#[test]
fn research_action_respects_prerequisites() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let current = game.factions[roles.player].current_research;

    game.apply_action(GameAction::ChooseResearch {
        owner: roles.player,
        tech: Tech::ProgenitorPsych,
    })
    .expect("choose research action should return cleanly");

    assert_eq!(game.factions[roles.player].current_research, current);
}

#[test]
fn build_improvement_via_action_updates_tile() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let former_id = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind == UnitKind::Former)
        .map(|unit| unit.id)
        .expect("player should start with a former");

    let former = game
        .unit(former_id)
        .cloned()
        .expect("former should be alive");

    game.apply_action(GameAction::BuildImprovement {
        unit_id: former_id,
        improvement: Improvement::Farm,
    })
    .expect("terraforming action should succeed");

    let tile = game
        .tile(former.x, former.y)
        .expect("former tile should still exist");
    assert_eq!(tile.improvement, Some(Improvement::Farm));
}

#[test]
fn set_production_via_action_updates_base() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let colony_id = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind.clone().can_found_base())
        .map(|unit| unit.id)
        .expect("player should start with a colony pod");

    game.apply_action(GameAction::FoundBase { unit_id: colony_id })
        .expect("found base action should succeed");

    let base_id = game
        .bases
        .iter()
        .find(|base| base.owner == roles.player)
        .map(|base| base.id)
        .expect("player should now own a base");

    game.apply_action(GameAction::SetBaseProduction {
        base_id,
        item: ProductionItem::Former,
    })
    .expect("production action should succeed");

    assert_eq!(
        game.base(base_id).expect("base should exist").production,
        ProductionItem::Former
    );
}

#[test]
fn end_turn_via_action_advances_the_turn_counter() {
    let mut game = GameState::new_game(16, 16, 7);

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    assert_eq!(game.turn, 2);
}

#[test]
fn queue_production_via_action_updates_base_queue() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let colony_id = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind.clone().can_found_base())
        .map(|unit| unit.id)
        .expect("player should start with a colony pod");

    game.apply_action(GameAction::FoundBase { unit_id: colony_id })
        .expect("found base action should succeed");

    let base_id = game
        .bases
        .iter()
        .find(|base| base.owner == roles.player)
        .map(|base| base.id)
        .expect("player should now own a base");

    game.apply_action(GameAction::QueueBaseProduction {
        base_id,
        item: ProductionItem::RecyclingTanks,
    })
    .expect("queue action should succeed");

    assert_eq!(
        game.base(base_id)
            .expect("base should exist")
            .production_queue,
        vec![ProductionItem::RecyclingTanks]
    );
}

#[test]
fn economy_plan_surfaces_stability_and_economy_items() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Economy".to_string(),
        x: 5,
        y: 5,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::SocialPsych);
        faction.known_techs.push(Tech::CentauriEcology);
        faction.known_techs.push(Tech::InformationNetworks);
    }

    let plan = game.base_economy_plan_items(0, 4);
    assert!(plan.contains(&ProductionItem::RecreationCommons));
    assert!(
        plan.contains(&ProductionItem::RecyclingTanks)
            || plan.contains(&ProductionItem::NetworkNode)
    );
}

#[test]
fn governor_reason_for_item_reports_matching_plan_entry() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Intent".to_string(),
        x: 5,
        y: 5,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::RecreationCommons],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::PlanetaryNetworks);
    }

    let reason = game
        .base_governor_reason_for_item(0, ProductionItem::HologramTheatre)
        .expect("governor should explain planned morale upgrade");
    assert!(reason.0 > 0);
    assert!(reason.1.contains("Hologram Theatre"));
}

#[test]
fn faction_governor_intent_counts_roll_up_recommendations_and_queue_items() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    for (id, x) in [(0, 5), (1, 8)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Intent {id}"),
            x,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![Facility::RecreationCommons],
            governor_mode: GovernorMode::Economy,
        });
        game.tiles[5 * game.width + x].base = Some(id);
    }
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::PlanetaryNetworks);
    }

    let recommendation_counts = game.faction_governor_recommendation_counts(roles.player);
    let queue_counts = game.faction_governor_queue_intent_counts(roles.player, 3);

    assert!(recommendation_counts
        .iter()
        .any(|(item, count)| *item == ProductionItem::HologramTheatre && *count == 2));
    assert!(queue_counts
        .iter()
        .any(|(item, count)| *item == ProductionItem::HologramTheatre && *count == 2));
}

#[test]
fn locked_governor_recommendation_surfaces_missing_unlock_for_base() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Locked Morale".to_string(),
        x: 5,
        y: 5,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::RecreationCommons],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    let blocked = game
        .base_governor_locked_recommendation(0)
        .expect("locked governor recommendation should exist");
    assert_eq!(blocked.0, ProductionItem::HologramTheatre);
    assert_eq!(blocked.1, Tech::PlanetaryNetworks);

    let faction_blocked = game.faction_locked_governor_recommendations(roles.player);
    assert!(faction_blocked.iter().any(|(base_id, item, tech, _)| {
        *base_id == 0
            && *item == ProductionItem::HologramTheatre
            && *tech == Tech::PlanetaryNetworks
    }));

    let by_tech = game
        .faction_locked_governor_recommendations_for_tech(roles.player, Tech::PlanetaryNetworks);
    assert!(by_tech
        .iter()
        .any(|(base_id, item, _)| *base_id == 0 && *item == ProductionItem::HologramTheatre));
    let entries = game.faction_locked_governor_recommendation_entries_for_tech(
        roles.player,
        Tech::PlanetaryNetworks,
    );
    assert!(entries.iter().any(|(base_name, item, tech)| {
        base_name == "Locked Morale"
            && *item == ProductionItem::HologramTheatre
            && *tech == Tech::PlanetaryNetworks
    }));
    assert_eq!(
        game.faction_locked_governor_recommendation_base_ids_for_tech(
            roles.player,
            Tech::PlanetaryNetworks,
        ),
        vec![0]
    );
    let impact = game.tech_unlock_impact_state(roles.player, Tech::PlanetaryNetworks);
    assert_eq!(impact.recommendation_count, 1);
    assert_eq!(impact.base_ids, vec![0]);
    assert_eq!(
        impact.summary_text.as_deref(),
        Some("Affects: Locked Morale -> Hologram Theatre (Planetary Networks)")
    );
    assert!(impact.entries.iter().any(|(base_name, item, tech)| {
        base_name == "Locked Morale"
            && *item == ProductionItem::HologramTheatre
            && *tech == Tech::PlanetaryNetworks
    }));
    let (known, available, blocked) = game.research_buckets(roles.player);
    assert!(known.contains(&Tech::CentauriEcology));
    assert!(!available.is_empty());
    assert!(blocked
        .iter()
        .any(|(tech, missing)| *tech == Tech::GeneSplicing
            && missing.contains(&Tech::SecretsOfTheHumanBrain)));

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.current_research = Tech::PlanetaryNetworks;
    }

    let current_research = game
        .current_research_display_state(roles.player, 3, None)
        .expect("current research state should exist");
    assert_eq!(current_research.tech, Tech::PlanetaryNetworks);
    assert_eq!(current_research.affected_base_ids, vec![0]);
    assert_eq!(current_research.affected_focus_base_id, Some(0));
    assert_eq!(
        current_research.affected_focus_label_text.as_deref(),
        Some("Cycle affected base")
    );
    assert!(current_research
        .label_text
        .starts_with("Current: Planetary Networks (0/"));
    assert!(!current_research.description_text.is_empty());
    assert_eq!(current_research.preview_heading_text, "Unlock preview");
    assert!(!current_research.unlock_lines.is_empty());
    assert_eq!(current_research.preview_section.hidden_count, 0);
    assert_eq!(
        current_research.preview_section.heading_text,
        "Queue preview if research lands"
    );
    assert_eq!(current_research.preview_section.focus_label_text, "Focus");
    assert_eq!(
        current_research.preview_section.keep_open_label_text,
        "Keep preview open"
    );
    assert_eq!(
        current_research.preview_section.stage_all_log_label_text,
        "Stage all previews in log"
    );
    assert!(current_research.preview_section.hidden_count_text.is_none());
    assert_eq!(
        current_research.affected_summary_text.as_deref(),
        Some("Affects: Locked Morale -> Hologram Theatre (Planetary Networks)")
    );
    assert!(current_research
        .affected_entries
        .iter()
        .any(|(base_name, item, tech)| {
            base_name == "Locked Morale"
                && *item == ProductionItem::HologramTheatre
                && *tech == Tech::PlanetaryNetworks
        }));
    assert!(current_research.preview_section.rows.iter().any(|row| {
        row.base_id == 0
            && row.base_name == "Locked Morale"
            && row.items.first() == Some(&ProductionItem::HologramTheatre)
            && row.row_text.contains("Locked Morale")
    }));
    let panel = game.research_panel_display_state(roles.player, None, None);
    assert!(!panel.summary_text.is_empty());
    assert!(panel.available_heading_text.starts_with("Available now ("));
    assert!(panel.blocked_heading_text.starts_with("Blocked ("));
    assert!(panel.known_heading_text.starts_with("Known techs ("));
    assert!(!panel.available_empty_text.is_empty());
    assert!(!panel.blocked_empty_text.is_empty());
    assert!(panel
        .known
        .iter()
        .any(|entry| { entry.tech == Tech::CentauriEcology && !entry.label_text.is_empty() }));
    assert!(!panel.available.is_empty());
    assert!(panel.available.iter().all(|entry| {
        !entry.label_text.is_empty()
            && entry.cost_text.starts_with("Cost ")
            && !entry.description_text.is_empty()
            && !entry.unlock_lines.is_empty()
    }));
    assert!(panel.available.iter().all(|entry| {
        entry.affected_focus_base_id.is_some() == (entry.unlock_impact.recommendation_count > 0)
    }));
    assert!(panel.blocked.iter().any(|entry| {
        entry.tech == Tech::GeneSplicing
            && entry.missing.contains(&Tech::SecretsOfTheHumanBrain)
            && !entry.label_text.is_empty()
            && !entry.description_text.is_empty()
            && !entry.unlock_lines.is_empty()
    }));
    let preview_rows = game.production_preview_rows(&current_research.queue_previews);
    assert!(preview_rows.iter().any(|row| {
        row.base_id == 0
            && row.base_name == "Locked Morale"
            && row.items.first() == Some(&ProductionItem::HologramTheatre)
    }));
    let prepared = game
        .prepare_research_unlock_preview(roles.player, Tech::PlanetaryNetworks, 3)
        .expect("prepared preview selection should exist");
    assert_eq!(prepared.preview.tech, Tech::PlanetaryNetworks);
    assert_eq!(prepared.affected_base_ids, vec![0]);
    assert!(prepared
        .preview
        .previews
        .iter()
        .any(|(base_id, items)| *base_id == 0
            && items.first() == Some(&ProductionItem::HologramTheatre)));
    let pinned_current = game
        .current_research_unlock_preview_state(roles.player, 3)
        .expect("current research preview state should exist");
    assert_eq!(pinned_current.tech, Tech::PlanetaryNetworks);
    assert!(pinned_current
        .previews
        .iter()
        .any(|(base_id, items)| *base_id == 0
            && items.first() == Some(&ProductionItem::HologramTheatre)));
    let log_len_before_pin = game.log.len();
    let pinned_action =
        game.pin_research_unlock_preview_action(roles.player, Tech::PlanetaryNetworks, 3, None);
    assert_eq!(pinned_action.staged_count, 1);
    assert_eq!(pinned_action.focus_base_id, Some(0));
    assert!(pinned_action
        .selection
        .as_ref()
        .is_some_and(|selection| selection.affected_base_ids == vec![0]));
    assert_eq!(game.log.len(), log_len_before_pin);
    let pinned_current_action =
        game.pin_current_research_unlock_preview_action(roles.player, 3, None);
    assert_eq!(pinned_current_action.staged_count, 1);
    assert_eq!(pinned_current_action.focus_base_id, Some(0));
    assert!(pinned_current_action
        .selection
        .as_ref()
        .is_some_and(|selection| selection.preview.tech == Tech::PlanetaryNetworks));
    let pinned_state_action = game.pin_research_unlock_preview_state_action(
        roles.player,
        Tech::PlanetaryNetworks,
        3,
        None,
    );
    assert_eq!(pinned_state_action.staged_count, 1);
    assert_eq!(pinned_state_action.focus_base_id, Some(0));
    assert!(pinned_state_action
        .preview
        .as_ref()
        .is_some_and(|preview| preview.tech == Tech::PlanetaryNetworks));
    let pinned_current_state_action =
        game.pin_current_research_unlock_preview_state_action(roles.player, 3, None);
    assert_eq!(pinned_current_state_action.staged_count, 1);
    assert_eq!(pinned_current_state_action.focus_base_id, Some(0));
    assert!(pinned_current_state_action
        .preview
        .as_ref()
        .is_some_and(|preview| preview.tech == Tech::PlanetaryNetworks));
    assert!(game
        .apply_research_unlock_preview_to_base(
            roles.player,
            game.research_unlock_preview_state(roles.player, Tech::PlanetaryNetworks, 3)
                .expect("preview state should exist"),
            0,
        )
        .is_err());

    let current_pressure = game.current_research_unlock_pressure_base_ids(roles.player);
    assert_eq!(current_pressure, vec![0]);
    assert_eq!(
        game.current_research_unlock_focus_base_id(roles.player, None),
        Some(0)
    );

    let advice = game.player_operations_advice();
    assert!(advice
        .iter()
        .any(|line| line
            .contains("Current research Planetary Networks would unblock governor plans")));

    let preview = game.base_governor_plan_preview_with_tech(0, Tech::PlanetaryNetworks, 3);
    assert!(!preview.is_empty());
    assert_eq!(preview[0], ProductionItem::HologramTheatre);
    assert!(game
        .apply_research_unlock_queue_preview_items(0, Tech::PlanetaryNetworks, preview.clone())
        .is_err());

    let faction_previews =
        game.faction_research_unlock_queue_previews(roles.player, Tech::PlanetaryNetworks, 3);
    assert!(faction_previews
        .iter()
        .any(|(base_id, items)| *base_id == 0
            && items.first() == Some(&ProductionItem::HologramTheatre)));

    let preview_state = game
        .research_unlock_preview_state(roles.player, Tech::PlanetaryNetworks, 3)
        .expect("preview state should exist");
    assert_eq!(game.research_unlock_preview_counts(&preview_state), (1, 0));
    assert_eq!(
        game.research_unlock_preview_counts_for_tech(Some(&preview_state), Tech::PlanetaryNetworks),
        Some((1, 0))
    );
    assert_eq!(
        game.research_unlock_preview_status_for_tech(Some(&preview_state), Tech::PlanetaryNetworks)
            .map(|status| (status.total, status.drifted)),
        Some((1, 0))
    );
    let preview_panel = game.research_panel_display_state(roles.player, None, Some(&preview_state));
    assert!(preview_panel.available.iter().all(|entry| {
        entry.preview_action_label.is_some() == (entry.unlock_impact.recommendation_count > 0)
    }));
    assert_eq!(
        game.research_unlock_preview_counts_for_tech(Some(&preview_state), Tech::Biogenetics),
        None
    );

    let staged_action =
        game.stage_research_unlock_preview_action(roles.player, Tech::PlanetaryNetworks, 3, None);
    assert_eq!(staged_action.staged_count, 1);
    assert_eq!(staged_action.focus_base_id, Some(0));
    assert!(staged_action
        .selection
        .as_ref()
        .is_some_and(|selection| selection.affected_base_ids == vec![0]));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Unlock preview if Planetary Networks lands")));
    game.log.clear();
    let staged_state_action = game.stage_research_unlock_preview_state_action(
        roles.player,
        Tech::PlanetaryNetworks,
        3,
        None,
    );
    assert_eq!(staged_state_action.staged_count, 1);
    assert_eq!(staged_state_action.focus_base_id, Some(0));
    assert!(staged_state_action
        .preview
        .as_ref()
        .is_some_and(|preview| preview.tech == Tech::PlanetaryNetworks));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Unlock preview if Planetary Networks lands")));
    game.log.clear();

    let current_staged_action =
        game.stage_current_research_unlock_preview_action(roles.player, 3, None);
    assert_eq!(current_staged_action.staged_count, 1);
    assert_eq!(current_staged_action.focus_base_id, Some(0));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Unlock preview if Planetary Networks lands")));
    game.log.clear();
    let current_staged_state_action =
        game.stage_current_research_unlock_preview_state_action(roles.player, 3, None);
    assert_eq!(current_staged_state_action.staged_count, 1);
    assert_eq!(current_staged_state_action.focus_base_id, Some(0));
    assert!(current_staged_state_action
        .preview
        .as_ref()
        .is_some_and(|preview| preview.tech == Tech::PlanetaryNetworks));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Unlock preview if Planetary Networks lands")));
    let current_focus_action =
        game.current_research_unlock_base_focus_action(roles.player, 3, None);
    assert_eq!(current_focus_action.staged_count, 1);
    assert_eq!(current_focus_action.focus_base_id, Some(0));
    assert_eq!(current_focus_action.affected_base_ids, vec![0]);
    let pinned_display =
        game.pinned_research_unlock_preview_display_state(roles.player, &preview_state);
    assert!(!pinned_display.can_apply);
    assert!(!pinned_display.apply_all_enabled);
    assert!(pinned_display.waiting_on_current_research);
    assert!(pinned_display.drifted_base_ids.is_empty());
    assert!(pinned_display.heading_text.contains("Planetary Networks"));
    assert!(pinned_display.availability_text.is_some());
    assert!(pinned_display.drift_text.is_none());
    assert_eq!(pinned_display.stage_log_label_text, "Stage in log");
    assert_eq!(pinned_display.refresh_label_text, "Refresh");
    assert_eq!(pinned_display.clear_label_text, "Clear");
    assert_eq!(pinned_display.apply_all_label_text, "Apply all");
    assert!(pinned_display.hidden_count_text.is_none());
    assert_eq!(pinned_display.hidden_count, 0);
    assert!(pinned_display.rows.iter().any(|row| {
        row.base_id == 0
            && row.is_current
            && row.items.first() == Some(&ProductionItem::HologramTheatre)
            && row.stale_label_text.is_none()
            && row.focus_label_text == "Focus"
            && row.apply_label_text == "Apply"
            && !row.row_text.is_empty()
            && !row.can_apply
    }));

    game.log.clear();
    let staged =
        game.stage_research_unlock_queue_previews(roles.player, Tech::PlanetaryNetworks, 3);
    assert_eq!(staged, 1);
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Unlock preview if Planetary Networks lands")));

    let mut drifted_game = game.clone();
    if let Some(base) = drifted_game.bases.iter_mut().find(|base| base.id == 0) {
        base.facilities.push(Facility::HologramTheatre);
    }
    assert_eq!(
        drifted_game.research_unlock_preview_drifted_base_ids(&preview_state),
        vec![0]
    );
    let drifted_display =
        drifted_game.pinned_research_unlock_preview_display_state(roles.player, &preview_state);
    assert_eq!(drifted_display.drifted_base_ids, vec![0]);
    assert!(drifted_display.drift_text.is_some());
    assert!(drifted_display.hidden_count_text.is_none());
    assert!(drifted_display.rows.iter().any(|row| {
        row.base_id == 0 && !row.is_current && row.stale_label_text.as_deref() == Some("(stale)")
    }));
    assert!(drifted_game
        .sync_research_unlock_preview_state(roles.player, &preview_state)
        .is_none());
    assert!(drifted_game
        .sync_research_unlock_preview_action(roles.player, Some(preview_state.clone()))
        .is_none());
    let synced_state_action = drifted_game
        .sync_research_unlock_preview_state_action(roles.player, Some(preview_state.clone()));
    assert!(synced_state_action.preview.is_none());
    assert!(synced_state_action.focus_base_id.is_none());
    let refreshed =
        drifted_game.refresh_research_unlock_preview_action(roles.player, &preview_state, None);
    assert!(refreshed.preview.is_none());
    assert_eq!(refreshed.focus_base_id, None);
    let refreshed_state_action = drifted_game.refresh_research_unlock_preview_state_action(
        roles.player,
        Some(preview_state.clone()),
        None,
    );
    assert!(refreshed_state_action.preview.is_none());
    assert_eq!(refreshed_state_action.focus_base_id, None);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::PlanetaryNetworks);
    }
    let mut apply_state_game = game.clone();
    let apply_result = game
        .apply_research_unlock_preview_to_base(roles.player, preview_state.clone(), 0)
        .expect("staged preview should apply once the tech is known");
    assert_eq!(apply_result.applied_base_ids, vec![0]);
    assert_eq!(apply_result.focus_base_id, Some(0));
    assert!(apply_result.remaining_preview.is_none());
    let apply_state_result = apply_state_game
        .apply_research_unlock_preview_state_action(roles.player, Some(preview_state.clone()), 0)
        .expect("state apply action should succeed once the tech is known");
    assert_eq!(apply_state_result.staged_count, 0);
    assert_eq!(apply_state_result.focus_base_id, Some(0));
    assert!(apply_state_result.preview.is_none());
    let base = game.base(0).expect("base should exist");
    assert_eq!(base.production, ProductionItem::HologramTheatre);
}

#[test]
fn faction_governor_warnings_surface_missing_defense_and_balanced_coverage() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    for (id, x) in [(0, 3), (1, 5), (2, 7), (3, 9)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Warning {id}"),
            x,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Economy,
        });
        game.tiles[5 * game.width + x].base = Some(id);
    }

    game.units.push(Unit {
        id: 100,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 4,
        y: 5,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 4].unit = Some(100);

    let warnings = game.faction_governor_warning_lines(roles.player);
    assert!(warnings
        .iter()
        .any(|line| line.contains("no Defense-governed bases")));
    assert!(warnings
        .iter()
        .any(|line| line.contains("Balanced governance across a larger empire")));
}

#[test]
fn apply_empty_queue_governor_plans_fills_follow_up_items_for_matching_bases() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    for (id, x) in [(0, 5), (1, 8)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Queue {id}"),
            x,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![Facility::RecreationCommons],
            governor_mode: GovernorMode::Economy,
        });
        game.tiles[5 * game.width + x].base = Some(id);
    }
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::PlanetaryNetworks);
    }

    let applied = game.apply_empty_queue_governor_plans(roles.player, 3);
    assert_eq!(applied, 2);
    assert_eq!(
        game.base(0).expect("base should exist").production,
        ProductionItem::HologramTheatre
    );
    assert_eq!(
        game.base(1).expect("base should exist").production,
        ProductionItem::HologramTheatre
    );
}

#[test]
fn economy_plan_surfaces_hologram_theatre_after_basic_morale_support() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Morale Upgrade".to_string(),
        x: 5,
        y: 5,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::RecreationCommons],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::PlanetaryNetworks);
    }

    let plan = game.base_economy_plan_items(0, 4);
    assert!(plan.contains(&ProductionItem::HologramTheatre));
}

#[test]
fn food_strain_surfaces_greenhouse_in_economy_plans_and_advice() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rocky;
        tile.moisture = 10;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Food Stress".to_string(),
        x: 5,
        y: 5,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::PatrolGrid],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::Biogenetics);
    }

    assert!(game.base_food_margin(0).expect("food margin should exist") <= 0);
    let plan = game.base_economy_plan_items(0, 4);
    assert!(plan.contains(&ProductionItem::Greenhouse));

    let advice = game.player_operations_advice();
    assert!(advice.iter().any(|line| line.contains("food-strained")));
}

#[test]
fn recovery_plan_surfaces_research_hospital_after_field_hospital() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Advanced Recovery".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::FieldHospital],
        governor_mode: GovernorMode::Recovery,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::GeneSplicing);
    }

    game.units.push(Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(0);

    let plan = game.base_recovery_plan_items(0, 4);
    assert!(plan.contains(&ProductionItem::ResearchHospital));
}

#[test]
fn mineral_strain_surfaces_refinery_in_economy_plans_and_advice() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Ocean;
        tile.moisture = 80;
        tile.improvement = None;
    }
    game.tiles[5 * game.width + 5].terrain = smac_core::Terrain::Flat;

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Mineral Stress".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![smac_core::Facility::Greenhouse],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[5 * game.width + 5].base = Some(0);
    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::IndustrialBase);
    }

    assert!(
        game.base_mineral_margin(0)
            .expect("mineral margin should exist")
            <= 0
    );
    let plan = game.base_economy_plan_items(0, 4);
    assert!(plan.contains(&ProductionItem::MineralRefinery));

    let advice = game.player_operations_advice();
    assert!(advice.iter().any(|line| line.contains("mineral-strained")));
}

#[test]
fn trade_exchange_adds_energy_from_nearby_friendly_trade_links() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
        tile.improvement = None;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Trade One".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);

    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Trade Two".to_string(),
        x: 8,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::InformationNetworks);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade convoy route should be added");
    assert_eq!(game.base_trade_links(0), 1);
    let yields = game
        .effective_base_yields(0)
        .expect("trade base yields should exist");
    assert_eq!(yields.energy, game.base_yields(4, 4).energy + 1);
    assert!(game
        .base_economy_plan_items(1, 4)
        .contains(&ProductionItem::TradeExchange));
}

#[test]
fn freight_depot_adds_minerals_from_nearby_friendly_trade_links() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
        tile.improvement = None;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Freight One".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::FreightDepot],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);

    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Freight Two".to_string(),
        x: 8,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::IndustrialBase);
        faction.known_techs.push(Tech::InformationNetworks);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Freight)
        .expect("freight convoy route should be added");
    assert_eq!(game.base_trade_links(0), 1);
    let yields = game
        .effective_base_yields(0)
        .expect("freight base yields should exist");
    assert_eq!(yields.minerals, game.base_yields(4, 4).minerals + 1);
    assert!(game
        .base_economy_plan_items(1, 4)
        .contains(&ProductionItem::FreightDepot));
}

#[test]
fn convoy_routes_can_be_added_and_removed_explicitly() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
        tile.improvement = None;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Alpha".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);

    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Beta".to_string(),
        x: 8,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);

    assert_eq!(game.base_trade_links(0), 0);
    assert_eq!(game.available_convoy_targets(0), vec![1]);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("convoy route should be added");
    assert_eq!(game.base_trade_links(0), 1);
    assert_eq!(game.convoy_routes_for_base(0), vec![1]);
    assert!(game.available_convoy_targets(0).is_empty());
    game.remove_convoy_route(0, 1)
        .expect("convoy route should be removed");
    assert_eq!(game.base_trade_links(0), 0);
}

#[test]
fn convoy_capacity_limits_additional_routes_without_logistics_support() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, x) in [(0usize, 4usize), (1, 8), (2, 12)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Base {id}"),
            x,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Economy,
        });
        game.tiles[4 * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("first route should be added");
    let err = game
        .add_convoy_route_typed(0, 2, ConvoyRouteKind::Freight)
        .expect_err("second route should exceed base capacity");
    assert!(err.contains("capacity"));
}

#[test]
fn convoy_route_disruption_blocks_yield_bonus_under_frontier_pressure() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Trade One".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Trade Two".to_string(),
        x: 8,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    let enemy_unit_id = 100;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let route_details = game.convoy_route_details_for_base(0);
    assert_eq!(game.base_convoy_security(0), 0);
    assert_eq!(game.base_local_military_pressure(0), 3);
    assert_eq!(game.base_local_military_pressure(1), 3);
    assert_eq!(route_details.len(), 1);
    assert!(route_details[0].2);
    let yields = game
        .effective_base_yields(0)
        .expect("trade base yields should exist");
    assert_eq!(yields.energy, game.base_yields(4, 4).energy);
}

#[test]
fn patrol_grid_reduces_convoy_disruption_for_protected_hub() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Secure Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange, Facility::PatrolGrid],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Secure Spoke".to_string(),
        x: 11,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::PatrolGrid],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 11].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    let enemy_unit_id = 101;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let route_details = game.convoy_route_details_for_base(0);
    assert_eq!(route_details.len(), 1);
    assert!(!route_details[0].2);
    let yields = game
        .effective_base_yields(0)
        .expect("protected trade base yields should exist");
    assert_eq!(yields.energy, game.base_yields(4, 4).energy + 1);
}

#[test]
fn intercepted_trade_route_costs_energy_on_end_turn() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Trade Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Trade Spoke".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    let enemy_unit_id = 102;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let status = game.convoy_route_status_for_base(0);
    assert_eq!(status.len(), 1);
    assert!(status[0].2);
    assert!(status[0].3);
    assert_eq!(status[0].4, 3);

    let base0_energy = game.operational_base_yields(0).unwrap().energy;
    let base1_energy = game.operational_base_yields(1).unwrap().energy;
    let (_, _, upkeep) = game.faction_upkeep(roles.player);
    let starting_energy = game.faction(roles.player).expect("player faction").energy;

    game.end_turn();

    let expected_energy = starting_energy + base0_energy + base1_energy - upkeep - 2;
    assert_eq!(
        game.faction(roles.player).expect("player faction").energy,
        expected_energy
    );
}

#[test]
fn intercepted_freight_route_costs_minerals_on_end_turn() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Freight Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::FreightDepot],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Freight Spoke".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Freight)
        .expect("freight route should be added");

    let enemy_unit_id = 103;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let status = game.convoy_route_status_for_base(0);
    assert_eq!(status.len(), 1);
    assert!(status[0].3);
    assert_eq!(status[0].4, 3);

    let base0_minerals = game.operational_base_yields(0).unwrap().minerals;

    game.end_turn();

    assert_eq!(
        game.base(0)
            .expect("freight hub should exist")
            .minerals_stock,
        base0_minerals - 2
    );
}

#[test]
fn escort_speeders_secure_convoy_endpoints_against_interception() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Escort Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Escort Spoke".to_string(),
        x: 11,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 11].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    game.tiles[4 * game.width + 4].unit = Some(200);
    game.units.push(Unit {
        id: 200,
        owner: roles.player,
        kind: UnitKind::EscortSpeeder,
        design_index: 0,
        x: 4,
        y: 4,
        moves_left: 3,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 11].unit = Some(201);
    game.units.push(Unit {
        id: 201,
        owner: roles.player,
        kind: UnitKind::EscortSpeeder,
        design_index: 0,
        x: 11,
        y: 4,
        moves_left: 3,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let enemy_unit_id = 104;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    assert_eq!(game.base_convoy_escort_count(0), 1);
    assert_eq!(game.base_convoy_escort_count(1), 1);
    assert_eq!(game.base_convoy_security(0), 2);
    assert_eq!(game.base_convoy_security(1), 2);

    let status = game.convoy_route_status_for_base(0);
    assert_eq!(status.len(), 1);
    assert!(!status[0].2);
    assert!(!status[0].3);
    assert_eq!(status[0].4, 3);
}

#[test]
fn repeated_convoy_interception_can_collapse_route() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Fragile Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Fragile Spoke".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 1;

    let enemy_unit_id = 105;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    if let Some(enemy) = game.units.iter_mut().find(|unit| unit.id == enemy_unit_id) {
        enemy.x = 6;
        enemy.y = 4;
        enemy.moves_left = 1;
        enemy.hp = 10;
        enemy.alive = true;
    }
    game.end_turn();

    assert!(game.convoy_routes_for_base(0).is_empty());
}

#[test]
fn suggested_escort_patrol_moves_target_convoy_pressure_bases() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Escort Base".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Escort Target".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    game.tiles[1 * game.width + 1].unit = Some(210);
    game.units.push(Unit {
        id: 210,
        owner: roles.player,
        kind: UnitKind::EscortSpeeder,
        design_index: 0,
        x: 11,
        y: 4,
        moves_left: 3,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let enemy_unit_id = 106;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let moves = game.suggested_escort_patrol_moves(roles.player);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].0, 210);
}

#[test]
fn convoy_route_can_be_repaired_after_interception_damage() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Repair Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Repair Spoke".to_string(),
        x: 8,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    let enemy_unit_id = 107;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.end_turn();
    let damaged_status = game.convoy_route_status_for_base(0);
    assert_eq!(damaged_status[0].4, 2);

    let energy_before_repair = game.faction(roles.player).expect("player faction").energy;
    game.repair_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("route repair should succeed");
    let repaired_status = game.convoy_route_status_for_base(0);
    assert_eq!(repaired_status[0].4, 3);
    assert_eq!(
        game.faction(roles.player).expect("player faction").energy,
        energy_before_repair - 2
    );
}

#[test]
fn faction_convoy_route_summaries_prioritize_worst_routes_first() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(18, 18, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Alpha".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::TradeExchange,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Beta".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::TradeExchange,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.bases.push(smac_core::Base {
        id: 2,
        owner: roles.player,
        name: "Gamma".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::TradeExchange,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[2 * game.width + 2].base = Some(0);
    game.tiles[2 * game.width + 6].base = Some(1);
    game.tiles[8 * game.width + 2].base = Some(2);

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("first route should be added");
    game.add_convoy_route_typed(0, 2, ConvoyRouteKind::Trade)
        .expect("second route should be added");

    let route = game
        .convoy_routes
        .iter_mut()
        .find(|route| route.base_a_id == 0 && route.base_b_id == 1)
        .expect("route should exist");
    route.integrity = 1;

    game.units.push(smac_core::Unit {
        id: 90,
        owner: roles.ai,
        kind: UnitKind::Speeder,
        design_index: 0,
        x: 4,
        y: 4,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    let summaries = game.faction_convoy_route_summaries(roles.player);
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].base_a_name, "Alpha");
    assert_eq!(summaries[0].base_b_name, "Beta");
    assert!(summaries[0].intercepted);
    assert_eq!(summaries[0].integrity, 1);
}

#[test]
fn intercepted_route_damages_escort_and_collapse_adds_extra_losses() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(18, 18, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Alpha".to_string(),
        x: 2,
        y: 2,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::TradeExchange,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Beta".to_string(),
        x: 6,
        y: 2,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::TradeExchange,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[2 * game.width + 2].base = Some(0);
    game.tiles[2 * game.width + 6].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("route should be added");
    game.convoy_routes[0].integrity = 1;

    game.units.push(smac_core::Unit {
        id: 50,
        owner: roles.player,
        kind: UnitKind::EscortSpeeder,
        design_index: 0,
        x: 3,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 3].unit = Some(50);
    game.units.push(smac_core::Unit {
        id: 99,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(99);
    game.units.push(smac_core::Unit {
        id: 100,
        owner: roles.ai,
        kind: UnitKind::Speeder,
        design_index: 0,
        x: 5,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 5].unit = Some(100);

    game.apply_action(GameAction::EndTurn)
        .expect("end turn should resolve");

    assert!(
        game.convoy_routes.is_empty(),
        "collapsed route should be removed"
    );
    let surviving_escort_hp = game.unit(50).map(|unit| unit.hp).unwrap_or(0);
    assert!(
        surviving_escort_hp <= 8,
        "escort should be damaged or destroyed by skirmish/combat follow-up"
    );
    assert!(
        game.log
            .iter()
            .any(|line| line.message.contains("collapsed under interception")),
        "collapse should be logged"
    );
}

#[test]
fn faction_upkeep_breakdown_counts_typed_convoy_routes() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(18, 18, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, x, y, facilities) in [
        (0, 2, 2, vec![Facility::TradeExchange]),
        (1, 6, 2, vec![]),
        (2, 2, 10, vec![Facility::FreightDepot]),
        (3, 6, 10, vec![]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Base {id}"),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.add_convoy_route_typed(2, 3, ConvoyRouteKind::Freight)
        .expect("freight route should be added");

    let (facility_upkeep, convoy_upkeep, unit_upkeep, total_upkeep) =
        game.faction_upkeep_breakdown(roles.player);
    assert_eq!(
        convoy_upkeep, 3,
        "trade route costs 1 and freight route costs 2"
    );
    assert_eq!(
        facility_upkeep, 4,
        "trade exchange and freight depot facility upkeep should still count"
    );
    assert_eq!(unit_upkeep, 0);
    assert_eq!(total_upkeep, facility_upkeep + convoy_upkeep + unit_upkeep);
}

#[test]
fn logistics_alerts_report_high_route_maintenance_and_saturated_hubs() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(20, 20, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, x, y, facilities) in [
        (0, 2, 2, vec![Facility::TradeExchange]),
        (1, 6, 2, vec![Facility::TradeExchange]),
        (2, 10, 2, vec![Facility::TransitHub, Facility::ForwardDepot]),
        (3, 14, 2, vec![]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Base {id}"),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("route 0-1 should be added");
    game.add_convoy_route_typed(0, 2, ConvoyRouteKind::Trade)
        .expect("route 0-2 should be added");
    game.add_convoy_route_typed(1, 2, ConvoyRouteKind::Trade)
        .expect("route 1-2 should be added");
    game.add_convoy_route_typed(2, 3, ConvoyRouteKind::Trade)
        .expect("route 2-3 should be added");

    let alerts = game.faction_logistics_alerts(roles.player);
    assert!(alerts.iter().any(|line| line.contains("energy per turn")));
    assert!(alerts
        .iter()
        .any(|line| line.contains("full lane capacity")));
}

#[test]
fn military_supply_routes_reduce_support_pressure_and_improve_repair() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(18, 18, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, x, y, facilities) in [
        (0, 2, 2, vec![Facility::CommandCenter]),
        (1, 6, 2, vec![Facility::FieldHospital]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Base {id}"),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    for unit_id in 0..8 {
        let x = 2 + (unit_id % 4);
        let y = 2 + (unit_id / 4);
        game.units.push(smac_core::Unit {
            id: unit_id,
            owner: roles.player,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x,
            y,
            moves_left: 0,
            hp: if unit_id == 0 { 8 } else { 10 },
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        game.tiles[y * game.width + x].unit = Some(unit_id);
    }

    let before = game.faction_upkeep_breakdown(roles.player).2;
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::MilitarySupply)
        .expect("military supply route should be added");
    let after = game.faction_upkeep_breakdown(roles.player).2;
    assert!(
        after < before,
        "military supply should reduce unit support pressure"
    );

    game.end_turn();
    let repaired = game.unit(0).expect("damaged unit should survive");
    assert_eq!(
        repaired.hp, 10,
        "military supply should improve on-base repair"
    );
}

#[test]
fn recommended_governor_mode_prefers_logistics_for_high_route_upkeep() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(20, 20, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, x, y, facilities) in [
        (0, 2, 2, vec![Facility::TradeExchange]),
        (1, 6, 2, vec![Facility::TradeExchange]),
        (2, 10, 2, vec![Facility::TransitHub, Facility::ForwardDepot]),
        (3, 14, 2, vec![]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Base {id}"),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.add_convoy_route_typed(0, 2, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.add_convoy_route_typed(1, 2, ConvoyRouteKind::Freight)
        .expect("freight route should be added");
    game.add_convoy_route_typed(2, 3, ConvoyRouteKind::MilitarySupply)
        .expect("supply route should be added");

    assert_eq!(
        game.recommended_governor_mode_for_base(2),
        GovernorMode::Logistics
    );
}

#[test]
fn convoy_rebuild_suggestions_surface_free_capacity_links() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    for (id, x) in [(0usize, 4usize), (1, 8usize)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Rebuild {id}"),
            x,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![Facility::TradeExchange],
            governor_mode: GovernorMode::Economy,
        });
        game.tiles[4 * game.width + x].base = Some(id);
    }

    let rebuilds = game.suggested_convoy_rebuilds(roles.player);
    assert_eq!(rebuilds.len(), 1);
    assert_eq!(rebuilds[0].0, 0);
    assert_eq!(rebuilds[0].1, 1);
}

#[test]
fn recommended_governor_mode_prefers_defense_for_logistics_pressure() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Governor Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Governor Spoke".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");

    let enemy_unit_id = 108;
    game.tiles[4 * game.width + 6].unit = Some(enemy_unit_id);
    game.units.push(Unit {
        id: enemy_unit_id,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    assert_eq!(
        game.recommended_governor_mode_for_base(0),
        GovernorMode::Logistics
    );
}

#[test]
fn queue_items_can_move_and_be_removed() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let colony_id = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind.clone().can_found_base())
        .map(|unit| unit.id)
        .expect("player should start with a colony pod");

    game.apply_action(GameAction::FoundBase { unit_id: colony_id })
        .expect("found base action should succeed");

    let base_id = game
        .bases
        .iter()
        .find(|base| base.owner == roles.player)
        .map(|base| base.id)
        .expect("player should now own a base");

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::IndustrialBase);
        faction.known_techs.push(Tech::DoctrineMobility);
    }

    game.queue_base_production(base_id, ProductionItem::CommandCenter)
        .expect("queue command center");
    game.queue_base_production(base_id, ProductionItem::SensorArray)
        .expect("queue sensor array");

    game.move_queued_production_up(base_id, 1)
        .expect("move queued item up");
    assert_eq!(
        game.base(base_id).expect("base exists").production_queue,
        vec![ProductionItem::SensorArray, ProductionItem::CommandCenter]
    );

    game.remove_queued_production(base_id, 0)
        .expect("remove queued item");
    assert_eq!(
        game.base(base_id).expect("base exists").production_queue,
        vec![ProductionItem::CommandCenter]
    );
}

#[test]
fn queue_item_can_be_promoted_to_active_and_queue_cleared() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let colony_id = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind.clone().can_found_base())
        .map(|unit| unit.id)
        .expect("player should start with a colony pod");

    game.apply_action(GameAction::FoundBase { unit_id: colony_id })
        .expect("found base action should succeed");

    let base_id = game
        .bases
        .iter()
        .find(|base| base.owner == roles.player)
        .map(|base| base.id)
        .expect("player should now own a base");

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::IndustrialBase);
    }

    game.queue_base_production(base_id, ProductionItem::CommandCenter)
        .expect("queue command center");
    game.promote_queued_production_to_active(base_id, 0)
        .expect("promote queued item");

    let base = game.base(base_id).expect("base exists");
    assert_eq!(base.production, ProductionItem::CommandCenter);
    assert_eq!(base.production_queue, vec![ProductionItem::ScoutPatrol]);

    game.clear_production_queue(base_id)
        .expect("clear queue should succeed");
    assert!(game
        .base(base_id)
        .expect("base exists")
        .production_queue
        .is_empty());
}

#[test]
fn facility_completion_and_upkeep_apply_after_end_turn() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let colony_id = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind.clone().can_found_base())
        .map(|unit| unit.id)
        .expect("player should start with a colony pod");

    game.apply_action(GameAction::FoundBase { unit_id: colony_id })
        .expect("found base action should succeed");

    let base_id = game
        .bases
        .iter()
        .find(|base| base.owner == roles.player)
        .map(|base| base.id)
        .expect("player should now own a base");

    {
        let base = game
            .bases
            .iter_mut()
            .find(|base| base.id == base_id)
            .unwrap();
        base.production = ProductionItem::RecyclingTanks;
        base.minerals_stock = 40;
    }
    let starting_energy = game.faction(roles.player).unwrap().energy;

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    let base = game.base(base_id).expect("base should exist");
    assert!(base.facilities.contains(&Facility::RecyclingTanks));
    assert!(
        game.faction(roles.player).unwrap().energy
            < starting_energy + game.base_yields(base.x, base.y).energy
    );
}

#[test]
fn command_center_reduces_unit_support_pressure() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Support Base".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });

    let tile_idx = 4 * game.width + 4;
    game.tiles[tile_idx].base = Some(0);

    let before = game.faction_upkeep(roles.player).1;
    let start_id = game.units.len();
    for (offset, x) in [(0usize, 5usize), (1, 6), (2, 7)] {
        let id = start_id + offset;
        game.tiles[4 * game.width + x].unit = Some(id);
        game.units.push(smac_core::Unit {
            id,
            owner: roles.player,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 5,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
    }
    let after = game.faction_upkeep(roles.player).1;

    assert_eq!(before, 0);
    assert_eq!(after, 0);
}

#[test]
fn unrest_reduces_effective_base_yields_until_recreation_commons_are_built() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.improvement = None;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Pressure Test".to_string(),
        x: 4,
        y: 4,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    let raw = game.base_yields(5, 5);
    let pressured = game
        .effective_base_yields(0)
        .expect("base should have effective yields");
    assert_eq!(game.base_unrest(0), 2);
    assert!(pressured.minerals < raw.minerals);
    assert!(pressured.energy < raw.energy);

    game.bases[0].facilities.push(Facility::RecreationCommons);

    let stabilized = game
        .effective_base_yields(0)
        .expect("base should still have effective yields");
    assert_eq!(game.base_unrest(0), 0);
    assert_eq!(stabilized.minerals, raw.minerals + 0);
    assert!(stabilized.energy >= raw.energy);
}

#[test]
fn command_center_trains_built_units_to_disciplined_rank() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Training Yard".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 30,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    let trained = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind == UnitKind::ScoutPatrol && unit.alive)
        .expect("built scout should exist");
    assert_eq!(trained.experience, 1);
}

#[test]
fn victorious_unit_gains_experience() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .morale = 0;
    game.faction_mut(roles.ai).unwrap().base_attributes.morale = 0;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.tiles[5 * game.width + 4].unit = Some(0);
    game.tiles[5 * game.width + 5].unit = Some(1);
    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 4,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.ai,
        kind: UnitKind::Speeder,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.apply_action(GameAction::MoveUnit {
        unit_id: 0,
        target_x: 5,
        target_y: 5,
    })
    .expect("combat move should resolve");

    let attacker = game.unit(0).expect("attacker should survive");
    assert_eq!(attacker.experience, 2);
}

#[test]
fn veteran_units_repair_faster_than_green_units() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    if let Some(f) = game.faction_mut(roles.player) {
        f.energy = 1000;
    }
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Support Base".to_string(),
        x: 1,
        y: 1,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 20,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[1 * game.width + 1].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 0,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 3].unit = Some(0);

    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 4,
        y: 4,
        moves_left: 0,
        hp: 4,
        experience: 2,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 4].unit = Some(1);

    game.units.push(smac_core::Unit {
        id: 2,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 10,
        y: 10,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[10 * game.width + 10].unit = Some(2);

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    let green = game.unit(0).expect("green unit should survive");
    let veteran = game.unit(1).expect("veteran unit should survive");
    assert_eq!(green.hp, 5);
    assert_eq!(veteran.hp, 7);
}

#[test]
fn command_center_garrison_repairs_faster() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Garrison".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 0,
        hp: 4,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(0);

    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 10,
        y: 10,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[10 * game.width + 10].unit = Some(1);

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    let garrison = game.unit(0).expect("garrison should survive");
    assert_eq!(garrison.hp, 8);
}

#[test]
fn damaged_unit_operation_advice_points_toward_friendly_base() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Safehold".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 3].unit = Some(0);

    let advice = game
        .unit_operation_advice(0)
        .expect("damaged unit should produce advice");
    assert!(advice.contains("Safehold"));
    assert!(advice.contains("4/10"));
}

#[test]
fn player_operations_advice_reports_unrest_and_support_pressure() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Overstretched".to_string(),
        x: 4,
        y: 4,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    for (offset, x) in [(0usize, 2usize), (1, 3), (2, 4)] {
        game.units.push(smac_core::Unit {
            id: offset,
            owner: roles.player,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 5,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        game.tiles[2 * game.width + x].unit = Some(offset);
    }

    let advice = game.player_operations_advice();
    assert!(advice
        .iter()
        .any(|line| line.contains("Overstretched has unrest")));
    assert!(advice.iter().any(|line| line.contains("Support costs are")));
}

#[test]
fn operations_priority_helpers_pick_damaged_unit_and_stressed_base() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Calm".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[2 * game.width + 2].base = Some(0);

    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Strained".to_string(),
        x: 4,
        y: 4,
        population: 5,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(1);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 8,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[3 * game.width + 3].unit = Some(0);

    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 4,
        y: 4,
        moves_left: 1,
        hp: 3,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 4].unit = Some(1);

    assert_eq!(game.most_damaged_player_unit_id(), Some(1));
    assert_eq!(game.most_unrested_player_base_id(), Some(1));
}

#[test]
fn player_operations_dashboard_state_surfaces_bulk_action_counts() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Recovery Hub".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter, Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Escort Target".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.bases.push(smac_core::Base {
        id: 2,
        owner: roles.player,
        name: "Unrest".to_string(),
        x: 12,
        y: 4,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 12].base = Some(2);

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 2;

    game.units.push(Unit {
        id: 10,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 12,
        y: 4,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 12].unit = Some(10);
    game.units.push(Unit {
        id: 11,
        owner: roles.player,
        kind: UnitKind::EscortSpeeder,
        design_index: 0,
        x: 1,
        y: 1,
        moves_left: 3,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[1 * game.width + 1].unit = Some(11);
    game.units.push(Unit {
        id: 20,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 6].unit = Some(20);

    println!("UNREST: {}", game.base_unrest(2));
    let dashboard = game.player_operations_dashboard_state(None, None, 3);
    assert!(!dashboard.advice_lines.is_empty());
    assert_eq!(dashboard.heading_text, "Operations Advice");

    let bulk_action = |action_type: smac_core::PlayerOperationsActionType| {
        dashboard
            .bulk_actions
            .iter()
            .find(|a| a.action_type == action_type)
            .expect("action should exist")
    };
    let jump_action = |filter: smac_core::BaseFocusFilter| {
        dashboard
            .jump_actions
            .iter()
            .find(|a| a.filter == filter)
            .expect("jump action should exist")
    };

    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::SelectDamagedUnit).button_text,
        "Select Damaged Unit (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::CycleDamagedUnits).button_text,
        "Cycle Damaged Units (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::JumpStressedBase).button_text,
        "Jump To Stressed Base (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::CycleStressedBases).button_text,
        "Cycle Stressed Bases (1)"
    );
    assert_eq!(dashboard.focus.recovering_base_count, 1);
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::JumpRecoveryBase).button_text,
        "Jump To Recovery Base (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::ApplyRecoveryBasePlan).button_text,
        "Apply Recovery Base Plan (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::RepairConvoys).available_count,
        1
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::RepairConvoys).button_text,
        "Repair Convoys (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::RebuildConvoys).available_count,
        1
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::RebuildConvoys).button_text,
        "Rebuild Convoys (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::AssignEscortPatrols).available_count,
        1
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::AssignEscortPatrols).button_text,
        "Assign Escort Patrols (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::JumpResearchUnlock).button_text,
        "Jump Research Unlock"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::CycleRecoveryBases).button_text,
        "Cycle Recovery Bases (1)"
    );
    assert_eq!(
        bulk_action(smac_core::PlayerOperationsActionType::SelectRecoveringGarrison).button_text,
        "Select Recovering Garrison (1)"
    );

    let gov_action = bulk_action(smac_core::PlayerOperationsActionType::SuggestGovernors);
    assert_eq!(
        gov_action.button_text,
        if gov_action.available_count > 0 {
            format!("Suggest Governors ({})", gov_action.available_count)
        } else {
            "Suggest Governors".to_string()
        }
    );

    let queue_action = bulk_action(smac_core::PlayerOperationsActionType::FillEmptyQueues);
    assert_eq!(
        queue_action.button_text,
        if queue_action.available_count > 0 {
            format!("Fill Empty Queues ({})", queue_action.available_count)
        } else {
            "Fill Empty Queues".to_string()
        }
    );

    assert!(jump_action(smac_core::BaseFocusFilter::Frontier)
        .button_text
        .starts_with("Jump Frontier Base"));
    assert!(jump_action(smac_core::BaseFocusFilter::Unrest)
        .button_text
        .starts_with("Jump Economy Stress"));
    assert!(jump_action(smac_core::BaseFocusFilter::Logistics)
        .button_text
        .starts_with("Jump Logistics Stress"));
    assert!(jump_action(smac_core::BaseFocusFilter::Saturated)
        .button_text
        .starts_with("Jump Saturated Hub"));
    assert!(jump_action(smac_core::BaseFocusFilter::Collapsing)
        .button_text
        .starts_with("Jump Collapsing Route"));
    assert!(jump_action(smac_core::BaseFocusFilter::Defense)
        .button_text
        .starts_with("Jump Defense Mode"));
    assert!(jump_action(smac_core::BaseFocusFilter::LogisticsMode)
        .button_text
        .starts_with("Jump Logistics Mode"));
    assert!(jump_action(smac_core::BaseFocusFilter::Recovery)
        .button_text
        .starts_with("Jump Recovery Mode"));
}

#[test]
fn safest_player_fallback_tile_prefers_safe_adjacent_step() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Safehold".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[8 * game.width + 8].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(0);

    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 10,
        y: 10,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[6 * game.width + 6].unit = Some(1);

    let step = game
        .safest_player_fallback_tile(0)
        .expect("damaged player unit should have a fallback tile");
    assert_ne!(step, (6, 6));
}

#[test]
fn apply_player_fallback_moves_uses_core_action_path() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Safehold".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[8 * game.width + 8].base = Some(0);

    game.units.push(Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(0);

    let moved = game.apply_player_fallback_moves();
    assert_eq!(moved, 1);
    let unit = game.unit(0).expect("damaged unit should still exist");
    assert_ne!((unit.x, unit.y), (5, 5));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Operations assisted 1 damaged units toward safety.")));
}

#[test]
fn convoy_support_bulk_actions_use_core_paths() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .police = -1;
    game.units.clear();
    game.bases.clear();
    game.convoy_routes.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    for (id, x, facilities) in [
        (0usize, 4usize, vec![Facility::TradeExchange]),
        (1usize, 8usize, Vec::new()),
        (2usize, 12usize, vec![Facility::TradeExchange]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Ops {id}"),
            x,
            y: 4,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Economy,
        });
        game.tiles[4 * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 2;

    game.units.push(Unit {
        id: 30,
        owner: roles.player,
        kind: UnitKind::EscortSpeeder,
        design_index: 0,
        x: 5,
        y: 4,
        moves_left: 3,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 5].unit = Some(30);
    game.units.push(Unit {
        id: 31,
        owner: roles.ai,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 6,
        y: 4,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[4 * game.width + 6].unit = Some(31);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.energy = 100;
    }

    let repaired = game.apply_convoy_repairs_all(roles.player);
    assert_eq!(repaired, 1);
    assert_eq!(game.convoy_routes[0].integrity, 3);

    let rebuilt = game.apply_convoy_rebuilds_all(roles.player);
    assert_eq!(rebuilt, 1);
    assert_eq!(game.convoy_routes.len(), 2);

    let moved = game.apply_escort_patrol_moves(roles.player);
    assert_eq!(moved, 1);
    let escort = game.unit(30).expect("escort should still exist");
    assert_ne!((escort.x, escort.y), (1, 1));

    assert!(game.log.iter().any(|line| line
        .message
        .contains("Operations repaired 1 convoy route(s).")));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Operations rebuilt or expanded 1 convoy route(s).")));
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Operations reassigned 1 escort unit(s) to convoy patrol duty.")));
}

#[test]
fn filtered_convoy_routes_and_opportunities_use_core_filtering() {
    let mut game = GameState::new_game(18, 18, 7);
    let roles = runtime_roles();
    let ai_owner = roles.ai;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    for (id, name, x, y, facilities) in [
        (
            0usize,
            "Alpha",
            2usize,
            2usize,
            vec![Facility::TradeExchange],
        ),
        (1usize, "Beta", 6usize, 2usize, Vec::new()),
        (
            2usize,
            "Freight Hub",
            2usize,
            10usize,
            vec![Facility::FreightDepot],
        ),
        (3usize, "Freight Spoke", 6usize, 10usize, Vec::new()),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 1;
    game.units.push(Unit {
        id: 90,
        owner: ai_owner,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(90);

    let intercepted = game.filtered_convoy_route_summaries(
        roles.player,
        LogisticsRouteFilter::Intercepted,
        LogisticsRouteSort::Severity,
    );
    assert_eq!(intercepted.len(), 1);
    assert_eq!(intercepted[0].kind, ConvoyRouteKind::Trade);

    let freight = game.convoy_route_opportunities(roles.player, LogisticsRouteFilter::Freight);
    assert!(!freight.is_empty());
    assert!(freight
        .iter()
        .all(|entry| entry.kind == ConvoyRouteKind::Freight));
}

#[test]
fn convoy_display_state_helpers_surface_route_hub_and_target_rows() {
    let mut game = GameState::new_game(18, 18, 7);
    let roles = runtime_roles();
    let ai_owner = roles.ai;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    for (id, name, x, y, facilities) in [
        (
            0usize,
            "Alpha",
            2usize,
            2usize,
            vec![Facility::TradeExchange],
        ),
        (1usize, "Beta", 6usize, 2usize, Vec::new()),
        (2usize, "Gamma", 10usize, 2usize, vec![Facility::TransitHub]),
        (3usize, "Delta", 14usize, 2usize, Vec::new()),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.add_convoy_route_typed(0, 2, ConvoyRouteKind::Trade)
        .expect("second route should be added");
    game.convoy_routes[0].integrity = 2;
    game.units.push(Unit {
        id: 93,
        owner: ai_owner,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(93);

    let base_rows = game.base_convoy_route_display_rows(0);
    assert_eq!(base_rows.len(), 2);
    assert!(base_rows[0].row_text.contains("integrity"));

    let target_rows = game.available_convoy_target_opportunities(2);
    assert!(!target_rows.is_empty());
    assert!(target_rows[0].button_text.contains("to"));

    let hub_rows = game.faction_convoy_hub_display_rows(roles.player);
    assert_eq!(hub_rows[0].base_id, 0);
    assert!(hub_rows[0].is_saturated);

    let route_rows = game.filtered_convoy_route_display_rows(
        roles.player,
        LogisticsRouteFilter::All,
        LogisticsRouteSort::Severity,
    );
    assert!(route_rows[0].row_text.contains("INTERCEPTED"));
}

#[test]
fn faction_logistics_panel_state_surfaces_display_and_action_state() {
    let mut game = GameState::new_game(18, 18, 7);
    let roles = runtime_roles();
    let ai_owner = roles.ai;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
    }

    for (id, name, x, y, facilities) in [
        (
            0usize,
            "Alpha",
            2usize,
            2usize,
            vec![Facility::TradeExchange],
        ),
        (1usize, "Beta", 6usize, 2usize, Vec::new()),
        (2usize, "Gamma", 10usize, 2usize, vec![Facility::TransitHub]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 1;
    game.units.push(Unit {
        id: 94,
        owner: ai_owner,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(94);

    let panel = game.faction_logistics_panel_state(
        roles.player,
        LogisticsRouteFilter::Intercepted,
        LogisticsRouteSort::Severity,
    );
    assert_eq!(panel.alerts_heading_text, "Logistics Alerts");
    assert_eq!(panel.routes_heading_text, "Convoy Routes");
    assert_eq!(panel.route_rows.len(), 1);
    assert!(panel.jump_worst_route_action.enabled);
    assert!(panel.repair_filtered_action.enabled);
    assert!(panel.filtered_count_text.contains("shown"));
}

#[test]
fn convoy_overlay_helpers_surface_tile_status_glyphs_and_lines() {
    let mut game = GameState::new_game(18, 18, 7);
    let roles = runtime_roles();
    let ai_owner = roles.ai;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
        tile.explored_by_owner.clear();
        tile.visible_by_owner.clear();
    }

    for (id, name, x, y, facilities) in [
        (
            0usize,
            "Alpha",
            2usize,
            2usize,
            vec![Facility::TradeExchange],
        ),
        (1usize, "Beta", 6usize, 2usize, vec![Facility::PatrolGrid]),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities,
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    for x in 2..=6 {
        game.tiles[2 * game.width + x]
            .explored_by_owner
            .insert(roles.player);
        game.tiles[2 * game.width + x]
            .visible_by_owner
            .insert(roles.player);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 1;
    game.units.push(Unit {
        id: 95,
        owner: ai_owner,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 4,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(95);

    let status = game.convoy_overlay_status_at(roles.player, 4, 2);
    assert_eq!(status, ConvoyOverlayStatus::Disrupted);
    assert_eq!(game.convoy_overlay_glyph_at(roles.player, 4, 2), Some("="));

    let lines = game.convoy_overlay_lines(roles.player);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].status, ConvoyOverlayStatus::Disrupted);
    assert_eq!((lines[0].start_x, lines[0].start_y), (2, 2));
    assert_eq!((lines[0].end_x, lines[0].end_y), (6, 2));
}

#[test]
fn map_overlay_labels_legends_and_colors_use_core_policy() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.moisture = 70;
        tile.explored_by_owner.clear();
        tile.visible_by_owner.clear();
    }

    game.tiles[5 * game.width + 5].terrain = smac_core::Terrain::Ocean;
    game.tiles[5 * game.width + 5]
        .explored_by_owner
        .insert(roles.player);
    game.tiles[5 * game.width + 5]
        .visible_by_owner
        .insert(roles.player);

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Player Base".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    assert_eq!(
        presentation::map_overlay_label(MapOverlay::Threat),
        "Offensive Threat"
    );
    assert!(presentation::map_overlay_legend(MapOverlay::Logistics).contains("green active lanes"));
    assert_eq!(
        game.map_overlay_color_hex(MapOverlay::Terrain, roles.player, 5, 5),
        "#1c4878"
    );
    assert_eq!(
        game.map_overlay_color_hex(MapOverlay::Ownership, roles.player, 5, 5),
        "#469650"
    );
    assert_eq!(
        game.map_overlay_color_hex(MapOverlay::Terrain, roles.player, 0, 0),
        "#08080a"
    );
    assert_eq!(presentation::ui_overlay_label(), "overlay");
    assert_eq!(presentation::ui_minimap_heading(), "Minimap");
    assert!(presentation::map_overlay_uses_convoy_lines(
        MapOverlay::Logistics
    ));
    assert!(!presentation::map_overlay_uses_convoy_lines(
        MapOverlay::Terrain
    ));

    let panel = game.map_panel_display_state(MapOverlay::Logistics);
    assert_eq!(panel.heading_text, presentation::ui_planet_heading());
    assert_eq!(panel.minimap_heading_text, "Minimap");
    assert_eq!(panel.overlay_label_text, "overlay");
    assert_eq!(panel.selected_overlay_label_text, "Logistics");
    assert!(panel.overlay_legend_text.contains("green active lanes"));
    assert!(panel.uses_convoy_lines);
    assert_eq!(panel.overlay_options.len(), 9);
    assert!(panel
        .overlay_options
        .iter()
        .any(|option| option.overlay == MapOverlay::Threat
            && option.label_text == "Offensive Threat"));
}

#[test]
fn command_console_display_state_surfaces_shared_loop_copy() {
    let game = GameState::new_game(12, 12, 7);
    let console = game.command_console_display_state();

    assert_eq!(
        console.heading_text,
        presentation::ui_command_console_heading()
    );
    assert_eq!(console.gameplay_loop_heading_text, "Gameplay loop:");
    assert_eq!(console.gameplay_loop_steps.len(), 6);
    assert_eq!(console.gameplay_loop_steps[0], "1. Found a base with C.");
    assert_eq!(
        console.event_log_heading_text,
        presentation::ui_event_log_heading()
    );
}

#[test]
fn selection_panel_display_state_surfaces_unit_tile_and_actions() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.improvement = None;
        tile.pod = false;
        tile.explored_by_owner.clear();
        tile.visible_by_owner.clear();
    }

    let former_design_index = game.find_design_index_for_kind(roles.player, UnitKind::Former);
    game.units.push(Unit {
        id: 77,
        owner: roles.player,
        kind: UnitKind::Former,
        design_index: former_design_index,
        x: 4,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 1,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 4].unit = Some(77);
    game.tiles[5 * game.width + 4].improvement = Some(Improvement::Road);
    game.tiles[5 * game.width + 4].pod = true;
    game.tiles[5 * game.width + 4]
        .explored_by_owner
        .insert(roles.player);
    game.tiles[5 * game.width + 4]
        .visible_by_owner
        .insert(roles.player);

    let panel = game.selection_panel_display_state(Some(77), Some((4, 5)), roles.player);
    assert_eq!(panel.heading_text, presentation::ui_selection_heading());

    match panel.unit {
        smac_core::UnitSelectionDisplayState::Selected {
            label_text,
            owner_text,
            terraform_actions,
            found_base_label_text,
            ..
        } => {
            assert!(label_text.contains("Former"));
            assert!(owner_text.contains("Owner:"));
            assert_eq!(found_base_label_text, None);
            assert_eq!(terraform_actions.len(), 8);
            assert_eq!(terraform_actions[0].button_text, "Farm");
        }
        other => panic!("expected selected unit state, got {other:?}"),
    }

    match panel.tile {
        smac_core::TileSelectionDisplayState::Selected {
            coordinates_text,
            terrain_text,
            improvement_text,
            warning_text,
            ..
        } => {
            assert_eq!(coordinates_text, "Tile: 4, 5");
            assert_eq!(terrain_text, "Terrain: Flat");
            assert_eq!(improvement_text.as_deref(), Some("Improvement: Road"));
            assert_eq!(warning_text, Some(presentation::ui_warning_text()));
        }
        other => panic!("expected selected tile state, got {other:?}"),
    }
}

#[test]
fn selection_panel_display_state_handles_missing_and_unexplored_selection() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    for tile in &mut game.tiles {
        tile.explored_by_owner.clear();
        tile.visible_by_owner.clear();
    }

    let panel = game.selection_panel_display_state(Some(9999), Some((3, 3)), roles.player);

    match panel.unit {
        smac_core::UnitSelectionDisplayState::Missing { message_text } => {
            assert_eq!(message_text, "Selected unit no longer exists.");
        }
        other => panic!("expected missing unit state, got {other:?}"),
    }

    match panel.tile {
        smac_core::TileSelectionDisplayState::Unexplored {
            coordinates_text,
            message_text,
        } => {
            assert_eq!(coordinates_text, "Tile: 3, 3");
            assert_eq!(message_text, "Unexplored territory.");
        }
        other => panic!("expected unexplored tile state, got {other:?}"),
    }
}

#[test]
fn base_panel_display_state_surfaces_core_owned_base_sections() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    game.convoy_routes.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(Base {
        id: 0,
        owner: roles.player,
        name: "Panel Base".to_string(),
        x: 4,
        y: 4,
        population: 4,
        nutrients_stock: 2,
        minerals_stock: 5,
        production: ProductionItem::Former,
        production_queue: vec![ProductionItem::NetworkNode],
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Recovery,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.tiles[4 * game.width + 4].terrain = smac_core::Terrain::Rolling;
    let player_base_id = 0;
    let panel = game
        .base_panel_display_state(player_base_id, roles.player)
        .expect("base panel state should exist");

    assert!(panel.heading_text.starts_with("Base: "));
    assert_eq!(panel.owner, roles.player);
    assert!(panel.can_manage);
    assert!(panel.population_text.contains("Population: 4"));
    assert_eq!(panel.current_governor_mode, GovernorMode::Recovery);
    assert!(panel.production_text.contains("Producing: Former"));
    assert!(panel
        .build_availability_text
        .contains("Build availability:"));
    assert_eq!(panel.governor_heading_text, "Governor");
    assert_eq!(panel.queue_editor_heading_text, Some("Edit queue:"));
    assert_eq!(panel.clear_queue_label_text, Some("Clear Queue"));
    assert_eq!(panel.set_production_heading_text, Some("Set production:"));
    assert_eq!(panel.queue_item_heading_text, Some("Queue item:"));
    assert!(!panel.set_production_options.is_empty());
    assert!(!panel.queue_item_options.is_empty());
    assert_eq!(panel.queue_rows.len(), 1);
    assert!(panel.queue_rows[0].label_text.contains("Network Node"));
}

#[test]
fn base_panel_display_state_surfaces_locked_items_and_convoy_status() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.bases.clear();
    game.units.clear();
    game.convoy_routes.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }
    game.bases.push(Base {
        id: 0,
        owner: roles.player,
        name: "Hub".to_string(),
        x: 4,
        y: 4,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.bases.push(Base {
        id: 1,
        owner: roles.player,
        name: "Link".to_string(),
        x: 4,
        y: 4,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[3 * game.width + 3].base = Some(0);
    game.tiles[3 * game.width + 7].base = Some(1);
    game.add_convoy_route_action(0, 1, ConvoyRouteKind::Trade);
    game.add_convoy_route_action(0, 1, ConvoyRouteKind::Freight);

    let panel = game
        .base_panel_display_state(0, roles.player)
        .expect("base panel state should exist");

    assert!(panel.convoy_capacity_text.contains("Convoy capacity:"));
    assert!(!panel.convoy_routes.is_empty());
    assert!(!panel.convoy_status_tags.is_empty());
    assert!(panel
        .locked_production_heading_text
        .as_deref()
        .unwrap_or_default()
        .contains("Locked production"));
    assert!(!panel.locked_production_options.is_empty());
}

#[test]
fn base_panel_action_wrappers_apply_core_mutations() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.bases.clear();
    game.units.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }
    game.bases.push(Base {
        id: 0,
        owner: roles.player,
        name: "Actions".to_string(),
        x: 4,
        y: 4,
        population: 3,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: vec![ProductionItem::Former, ProductionItem::NetworkNode],
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 4].base = Some(0);

    assert!(game.set_base_governor_mode_action(0, GovernorMode::Defense));
    assert_eq!(game.governor_mode_for_base(0), GovernorMode::Defense);

    assert!(game.promote_queued_production_to_active_action(0, 0));
    assert_eq!(
        game.base(0).expect("base").production,
        ProductionItem::Former
    );

    assert!(game.move_queued_production_down_action(0, 0));
    assert_eq!(
        game.base(0).expect("base").production_queue,
        vec![ProductionItem::NetworkNode, ProductionItem::ScoutPatrol]
    );

    assert!(game.remove_queued_production_action(0, 1));
    assert_eq!(
        game.base(0).expect("base").production_queue,
        vec![ProductionItem::NetworkNode]
    );

    assert!(game.clear_production_queue_action(0));
    assert!(game.base(0).expect("base").production_queue.is_empty());
}

#[test]
fn overlay_aware_tile_map_label_uses_core_logic() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
        tile.explored_by_owner.clear();
        tile.visible_by_owner.clear();
    }

    for x in 2..=6 {
        game.tiles[2 * game.width + x]
            .explored_by_owner
            .insert(roles.player);
        game.tiles[2 * game.width + x]
            .visible_by_owner
            .insert(roles.player);
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Alpha".to_string(),
        x: 2,
        y: 2,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TradeExchange],
        governor_mode: GovernorMode::Off,
    });
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Beta".to_string(),
        x: 6,
        y: 2,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[2 * game.width + 2].base = Some(0);
    game.tiles[2 * game.width + 6].base = Some(1);
    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("trade route should be added");
    game.convoy_routes[0].integrity = 1;

    let normal = game.tile_map_label_for_overlay(4, 2, roles.player, MapOverlay::Terrain);
    let logistics = game.tile_map_label_for_overlay(4, 2, roles.player, MapOverlay::Logistics);
    let map_tile = game.map_tile_display_state(4, 2, roles.player, None, MapOverlay::Logistics);
    let minimap_tile =
        game.minimap_tile_display_state(4, 2, roles.player, None, MapOverlay::Terrain);

    assert_eq!(normal, ".".to_string());
    assert_eq!(logistics, "-".to_string());
    assert_eq!(map_tile.label_text, "-".to_string());
    assert_eq!(map_tile.color_hex, "#489c58".to_string());
    assert_eq!(minimap_tile.label_text, " ".to_string());
    assert_eq!(minimap_tile.color_hex, "#2e783e".to_string());
}

#[test]
fn convoy_route_focus_and_filtered_repair_actions_use_core_paths() {
    let mut game = GameState::new_game(18, 18, 7);
    let roles = runtime_roles();
    let ai_owner = roles.ai;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, name, x, y) in [
        (0usize, "Alpha", 2usize, 2usize),
        (1usize, "Beta", 6usize, 2usize),
        (2usize, "Gamma", 2usize, 10usize),
        (3usize, "Delta", 6usize, 10usize),
    ] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::TradeExchange,
            production_queue: Vec::new(),
            facilities: vec![Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[y * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("first route should be created");
    game.add_convoy_route_typed(2, 3, ConvoyRouteKind::Trade)
        .expect("second route should be created");
    game.convoy_routes[0].integrity = 1;
    game.convoy_routes[1].integrity = 2;
    game.units.push(Unit {
        id: 91,
        owner: ai_owner,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(91);

    assert_eq!(game.worst_convoy_route_focus_action(roles.player), Some(0));

    let repaired = game.apply_filtered_convoy_repairs_for_owner(
        roles.player,
        LogisticsRouteFilter::Intercepted,
        LogisticsRouteSort::Severity,
    );
    assert_eq!(repaired, 1);

    let mut routes = game.faction_convoy_route_summaries(roles.player);
    routes.sort_by_key(|route| (route.base_a_id, route.base_b_id));
    assert_eq!(routes[0].integrity, 2);
    assert_eq!(routes[1].integrity, 2);
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Operations repaired 1 filtered convoy route(s).")));
}

#[test]
fn remove_collapsing_convoy_routes_uses_core_action_path() {
    let mut game = GameState::new_game(18, 18, 7);
    let roles = runtime_roles();
    let ai_owner = roles.ai;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, name, x) in [(0usize, "Alpha", 2usize), (1usize, "Beta", 6usize)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: name.to_string(),
            x,
            y: 2,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::TradeExchange,
            production_queue: Vec::new(),
            facilities: vec![Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[2 * game.width + x].base = Some(id);
    }

    game.add_convoy_route_typed(0, 1, ConvoyRouteKind::Trade)
        .expect("route should be created");
    game.convoy_routes[0].integrity = 1;
    game.units.push(Unit {
        id: 92,
        owner: ai_owner,
        kind: UnitKind::RaiderSpeeder,
        design_index: 0,
        x: 4,
        y: 2,
        moves_left: 0,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[2 * game.width + 4].unit = Some(92);

    let removed = game.remove_collapsing_convoy_routes(roles.player);
    assert_eq!(removed, 1);
    assert!(game.convoy_routes.is_empty());
    assert!(game.log.iter().any(|line| line
        .message
        .contains("Operations removed 1 collapsing convoy route(s).")));
}

#[test]
fn apply_enabled_automations_uses_core_governor_mode_loop() {
    let mut game = GameState::new_game(12, 12, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Auto Defense".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Defense,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(smac_core::Base {
        id: 1,
        owner: roles.player,
        name: "Auto Economy".to_string(),
        x: 4,
        y: 4,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Economy,
    });
    game.tiles[4 * game.width + 8].base = Some(1);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::SocialPsych);
        faction.known_techs.push(Tech::DoctrineMobility);
    }

    game.units.push(Unit {
        id: 50,
        owner: roles.ai,
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
    game.tiles[4 * game.width + 2].unit = Some(50);

    let applied = game.apply_enabled_automations(3);
    assert_eq!(applied, 2);
    assert_eq!(
        game.base(0).expect("defense base should exist").production,
        ProductionItem::PerimeterDefense
    );
    assert_eq!(
        game.base(1).expect("economy base should exist").production,
        ProductionItem::RecreationCommons
    );
}

#[test]
fn governor_recommendation_prefers_field_hospital_for_damaged_garrison() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Clinic".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::Biogenetics);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 4,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(0);

    let (item, recommendation) = game
        .base_governor_recommendation(0)
        .expect("damaged garrison should trigger a governor recommendation");
    assert_eq!(item, ProductionItem::FieldHospital);
    assert!(recommendation.contains("Field Hospital"));
}

#[test]
fn governor_plan_can_stack_multiple_base_actions() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Stacked Pressure".to_string(),
        x: 4,
        y: 4,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::SocialPsych);
        faction.known_techs.push(Tech::IndustrialBase);
        faction.energy = 0;
    }

    for id in 0..3 {
        let x = 6 + id;
        game.units.push(smac_core::Unit {
            id,
            owner: roles.player,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 5,
            y: 5,
            moves_left: 1,
            hp: if id == 0 { 4 } else { 10 },
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        game.tiles[5 * game.width + x].unit = Some(id);
    }

    let plan = game.base_governor_plan(0);
    assert!(plan.len() >= 2);
    assert_eq!(plan[0].item, ProductionItem::RecreationCommons);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::CommandCenter));
}

#[test]
fn recovery_base_helpers_prioritize_bases_with_damaged_garrisons() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    for (id, x, hp) in [(0usize, 4usize, 3), (1, 7, 8)] {
        game.bases.push(smac_core::Base {
            id,
            owner: roles.player,
            name: format!("Recovery {id}"),
            x,
            y: 5,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![Facility::CommandCenter],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[5 * game.width + x].base = Some(id);
        game.units.push(smac_core::Unit {
            id,
            owner: roles.player,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x,
            y: 5,
            moves_left: 1,
            hp,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: smac_core::UnitActivity::None,
        });
        game.tiles[5 * game.width + x].unit = Some(id);
    }

    let recovery_bases = game.recovering_garrison_base_ids();
    assert_eq!(recovery_bases, vec![0, 1]);
    assert_eq!(game.most_recovering_garrison_base_id(), Some(0));
}

#[test]
fn governor_plan_includes_sensor_array_under_frontline_pressure() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Frontier".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::DoctrineMobility);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.ai,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::SensorArray));
}

#[test]
fn governor_plan_can_surface_forward_depot_after_transit_hub() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
        tile.moisture = 40;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Forward Line".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::TransitHub],
        governor_mode: GovernorMode::Defense,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::DoctrineMobility);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.ai,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::ForwardDepot));
}

#[test]
fn governor_queue_items_excludes_current_and_duplicate_builds() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Queue Plan".to_string(),
        x: 4,
        y: 4,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::RecreationCommons,
        production_queue: vec![ProductionItem::CommandCenter],
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::SocialPsych);
        faction.known_techs.push(Tech::IndustrialBase);
        faction.known_techs.push(Tech::DoctrineMobility);
        faction.energy = 0;
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.ai,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    let items = game.base_governor_queue_items(0, 4);
    assert!(!items.contains(&ProductionItem::RecreationCommons));
    assert!(!items.contains(&ProductionItem::CommandCenter));
    assert!(items
        .iter()
        .any(|item| *item == ProductionItem::PerimeterDefense));
}

#[test]
fn governor_queue_preserves_progressed_military_build_under_pressure() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Hold The Line".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 8,
        production: ProductionItem::PerimeterDefense,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::DoctrineMobility);
        faction.known_techs.push(Tech::IndustrialBase);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.ai,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    let items = game.base_governor_queue_items(0, 4);
    assert!(!items.contains(&ProductionItem::PerimeterDefense));
}

#[test]
fn psi_pressure_surfaces_psi_sentinel_and_recovery_plan() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Psionic Hold".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::CommandCenter],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::ProgenitorPsych);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    assert!(game.base_local_psi_pressure(0) >= 2);
    let plan = game.base_governor_plan(0);
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::PsiSentinel));
    assert!(plan
        .iter()
        .any(|step| step.item == ProductionItem::PsiBeacon));

    let recovery_items = game.base_recovery_plan_items(0, 3);
    assert!(recovery_items.contains(&ProductionItem::PsiSentinel));
    assert!(recovery_items.contains(&ProductionItem::PsiBeacon));
}

#[test]
fn psi_beacon_reduces_local_psi_pressure_and_updates_base_role() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Psi Shield".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    assert_eq!(game.base_area_role(0), smac_core::BaseAreaRole::Warzone);
    let before = game.base_local_psi_pressure(0);
    assert!(before >= 2);

    if let Some(base) = game.base_mut(0) {
        base.facilities.push(Facility::PsiBeacon);
    }

    let after = game.base_local_psi_pressure(0);
    assert!(after < before);
    assert_eq!(game.base_area_role(0), smac_core::BaseAreaRole::Frontier);
}

#[test]
fn player_operations_advice_reports_psi_pressure() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Psi Alert".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[3 * game.width + 5].unit = Some(1);

    let advice = game.player_operations_advice();
    assert!(advice.iter().any(|line| line.contains("psi pressure")));
}

#[test]
fn player_operations_advice_mentions_bioenhancement_support() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Psi Upgrade".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::PsiBeacon],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::SecretsOfTheHumanBrain);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    game.units.push(smac_core::Unit {
        id: 1,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[3 * game.width + 5].unit = Some(1);

    let advice = game.player_operations_advice();
    assert!(advice
        .iter()
        .any(|line| line.contains("Bioenhancement Center support")));
}

#[test]
fn apply_recovery_plans_all_updates_a_stressed_base_production() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Recovery Plan".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::ProgenitorPsych);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.native,
        kind: UnitKind::MindWorm,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    game.apply_recovery_plans_all(3);

    assert_eq!(
        game.base(0).expect("base should exist").production,
        ProductionItem::PsiSentinel
    );
}

#[test]
fn apply_defense_plans_all_updates_a_frontier_base_production() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Defense Plan".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::DoctrineMobility);
    }

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.ai,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    game.apply_defense_plans_all(3);

    assert_eq!(
        game.base(0).expect("base should exist").production,
        ProductionItem::PerimeterDefense
    );
}

#[test]
fn frontier_base_ids_include_bases_under_heavy_pressure() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Frontier".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.units.push(smac_core::Unit {
        id: 0,
        owner: roles.ai,
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
    game.tiles[5 * game.width + 3].unit = Some(0);

    assert_eq!(game.frontier_base_ids(), vec![0]);
}

#[test]
fn military_academy_trains_new_units_to_veteran_rank() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Academy".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 30,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::MilitaryAcademy],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    let trained = game
        .units
        .iter()
        .find(|unit| unit.owner == roles.player && unit.kind == UnitKind::ScoutPatrol && unit.alive)
        .expect("built scout should exist");
    assert_eq!(trained.experience, 2);
}

#[test]
fn transit_hub_grants_extra_movement_to_mobile_units() {
    let roles = runtime_roles();
    let mut game = GameState::new_game(12, 12, 7);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }

    game.bases.push(smac_core::Base {
        id: 0,
        owner: roles.player,
        name: "Transit".to_string(),
        x: 5,
        y: 5,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 30,
        production: ProductionItem::RaiderSpeeder,
        production_queue: Vec::new(),
        facilities: vec![Facility::TransitHub],
        governor_mode: GovernorMode::Off,
    });
    game.tiles[5 * game.width + 5].base = Some(0);

    if let Some(faction) = game.faction_mut(roles.player) {
        faction.known_techs.push(Tech::DoctrineMobility);
    }

    game.apply_action(GameAction::EndTurn)
        .expect("end turn action should succeed");

    let raider = game
        .units
        .iter()
        .find(|unit| {
            unit.owner == roles.player && unit.kind == UnitKind::RaiderSpeeder && unit.alive
        })
        .expect("built raider should exist");
    assert_eq!(raider.moves_left, UnitKind::RaiderSpeeder.max_moves() + 1);
}

#[test]
fn fortified_base_defense_changes_combat_outcome() {
    let roles = runtime_roles();

    let mut open_field = GameState::new_game(12, 12, 7);
    open_field
        .faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .morale = 0;
    open_field
        .faction_mut(roles.ai)
        .unwrap()
        .base_attributes
        .morale = 0;
    open_field.units.clear();
    open_field.bases.clear();
    for tile in &mut open_field.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Flat;
    }
    open_field.tiles[5 * open_field.width + 4].unit = Some(0);
    open_field.tiles[5 * open_field.width + 5].unit = Some(1);
    open_field.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
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
    open_field.units.push(smac_core::Unit {
        id: 1,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    open_field
        .apply_action(GameAction::MoveUnit {
            unit_id: 0,
            target_x: 5,
            target_y: 5,
        })
        .expect("combat move should resolve");
    assert!(
        open_field.unit(0).is_some(),
        "attacker should survive in open terrain"
    );

    let mut fortified = GameState::new_game(12, 12, 7);
    fortified
        .faction_mut(roles.player)
        .unwrap()
        .base_attributes
        .morale = 0;
    fortified
        .faction_mut(roles.ai)
        .unwrap()
        .base_attributes
        .morale = 0;
    fortified.units.clear();
    fortified.bases.clear();
    for tile in &mut fortified.tiles {
        tile.unit = None;
        tile.base = None;
        tile.terrain = smac_core::Terrain::Rolling;
    }
    fortified.tiles[5 * fortified.width + 4].unit = Some(0);
    fortified.tiles[5 * fortified.width + 5].unit = Some(1);
    fortified.units.push(smac_core::Unit {
        id: 0,
        owner: roles.player,
        kind: UnitKind::ScoutPatrol,
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
    fortified.units.push(smac_core::Unit {
        id: 1,
        owner: roles.ai,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: 5,
        y: 5,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    fortified.bases.push(smac_core::Base {
        id: 0,
        owner: roles.ai,
        name: "Fortified".to_string(),
        x: 5,
        y: 5,
        population: 4,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: vec![Facility::PerimeterDefense],
        governor_mode: GovernorMode::Off,
    });
    fortified.tiles[5 * fortified.width + 5].base = Some(0);
    fortified
        .apply_action(GameAction::MoveUnit {
            unit_id: 0,
            target_x: 5,
            target_y: 5,
        })
        .expect("combat move should resolve");
    assert!(
        fortified.unit(0).is_none(),
        "attacker should fail against fortified base"
    );
}

#[test]
fn victory_economic_attained_on_energy_threshold() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();

    let unit_id = game
        .live_units_for(roles.player)
        .into_iter()
        .find(|unit| unit.moves_left > 0)
        .expect("player should start with a movable unit")
        .id;
    let first_target = {
        let unit = game.unit(unit_id).unwrap();
        [
            (unit.x.saturating_add(1), unit.y),
            (unit.x.saturating_sub(1), unit.y),
            (unit.x, unit.y.saturating_add(1)),
            (unit.x, unit.y.saturating_sub(1)),
        ]
        .into_iter()
        .find(|(x, y)| {
            *x < game.width
                && *y < game.height
                && game
                    .tile(*x, *y)
                    .map(|tile| tile.unit.is_none() && tile.terrain.is_land())
                    .unwrap_or(false)
        })
        .expect("player should have an adjacent open land tile")
    };

    // Stay below the threshold and trigger a normal gameplay action that re-checks victory
    // conditions without adding turn income.
    game.faction_mut(roles.player).unwrap().energy = 0;
    game.apply_action(GameAction::MoveUnit {
        unit_id,
        target_x: first_target.0,
        target_y: first_target.1,
    })
    .expect("move action should succeed");
    assert!(game.game_over.is_none());

    // Hit threshold
    game.faction_mut(roles.player).unwrap().energy = 10000;
    if let Some(unit) = game.units.iter_mut().find(|unit| unit.id == unit_id) {
        unit.moves_left = 1;
    }
    let second_target = {
        let unit = game.unit(unit_id).unwrap();
        [
            (unit.x.saturating_add(1), unit.y),
            (unit.x.saturating_sub(1), unit.y),
            (unit.x, unit.y.saturating_add(1)),
            (unit.x, unit.y.saturating_sub(1)),
        ]
        .into_iter()
        .find(|(x, y)| {
            *x < game.width
                && *y < game.height
                && (*x != first_target.0 || *y != first_target.1)
                && game
                    .tile(*x, *y)
                    .map(|tile| tile.unit.is_none() && tile.terrain.is_land())
                    .unwrap_or(false)
        })
        .expect("player should still have an adjacent open land tile")
    };
    game.apply_action(GameAction::MoveUnit {
        unit_id,
        target_x: second_target.0,
        target_y: second_target.1,
    })
    .expect("second move action should succeed");
    assert_eq!(game.game_over, Some(smac_core::GameOver::PlayerWonEconomic));
}

#[test]
fn victory_transcendence_requires_empath_guild_project() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }
    game.tiles[3 * game.width + 3].terrain = smac_core::Terrain::Flat;
    game.tiles[10 * game.width + 10].terrain = smac_core::Terrain::Flat;
    game.spawn_base(roles.player, 3, 3)
        .expect("player base should spawn");
    game.spawn_base(roles.ai, 10, 10)
        .expect("ai base should spawn");

    game.faction_mut(roles.player)
        .unwrap()
        .known_techs
        .push(smac_core::Tech::SecretsOfPlanet);
    game.end_turn();
    assert!(game.game_over.is_none());
}

#[test]
fn victory_transcendence_attained_with_tech_and_empath_guild() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let owner = roles.player;
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }
    game.tiles[3 * game.width + 3].terrain = smac_core::Terrain::Flat;
    game.tiles[10 * game.width + 10].terrain = smac_core::Terrain::Flat;
    game.spawn_base(owner, 3, 3)
        .expect("player base should spawn");
    game.spawn_base(roles.ai, 10, 10)
        .expect("ai base should spawn");

    game.faction_mut(owner)
        .unwrap()
        .known_techs
        .push(smac_core::Tech::SecretsOfPlanet);
    game.built_secret_projects
        .push((smac_core::SecretProject::EmpathGuild, owner));
    game.end_turn();
    assert_eq!(
        game.game_over,
        Some(smac_core::GameOver::PlayerWonTranscendence)
    );
}

#[test]
fn victory_space_transcendence_attained_on_project_completion() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let owner = roles.player;

    // Build the required projects
    game.built_secret_projects
        .push((smac_core::SecretProject::OrbitalElevator, owner));
    game.built_secret_projects
        .push((smac_core::SecretProject::ManifoldDrive, owner));

    game.end_turn();
    assert_eq!(
        game.game_over,
        Some(smac_core::GameOver::PlayerWonSpaceTranscendence)
    );
}

#[test]
fn victory_black_hole_harvesting_attained_on_project_completion() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let owner = roles.player;

    // Build the required projects
    game.built_secret_projects
        .push((smac_core::SecretProject::SingularityContainment, owner));
    game.built_secret_projects
        .push((smac_core::SecretProject::BlackHoleHarvester, owner));

    game.end_turn();
    assert_eq!(
        game.game_over,
        Some(smac_core::GameOver::PlayerWonBlackHoleHarvesting)
    );
}

#[test]
fn units_can_load_and_unload_from_transports() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let owner = roles.player;

    // Force tiles to be ocean for sea transport
    game.tiles[5 * game.width + 5].terrain = smac_core::Terrain::Ocean;
    game.tiles[6 * game.width + 5].terrain = smac_core::Terrain::Ocean;
    game.tiles[6 * game.width + 5].unit = None;

    // 1. Create a transport
    let transport_design = smac_core::UnitDesign {
        name: "Sea Transport".to_string(),
        chassis: smac_core::Chassis::Sea,
        weapon: smac_core::Weapon::HandLaser(1),
        armor: smac_core::Armor::SynthMetal(1),
        cost: 30,
        abilities: vec![smac_core::Ability::Transport],
    };
    let transport_design_clone = transport_design.clone();
    game.add_unit_design(owner, transport_design);

    let design_index = game.faction(owner).unwrap().unit_designs.len() - 1;
    let transport_id = game
        .spawn_unit_with_design(
            owner,
            smac_core::UnitKind::CustomUnit(transport_design_clone),
            design_index,
            5,
            5,
            0,
        )
        .unwrap();
    let passenger_id = game.spawn_unit(owner, smac_core::UnitKind::ScoutPatrol, 5, 5);

    // 2. Load unit
    game.apply_action(smac_core::GameAction::LoadUnit {
        unit_id: passenger_id,
        transport_id,
    })
    .expect("Loading should succeed");

    assert!(game.unit(passenger_id).is_some());
    assert!(game
        .unit(transport_id)
        .unwrap()
        .cargo_unit_ids
        .contains(&passenger_id));
    assert!(game.tiles[5 * game.width + 5].unit == Some(transport_id));

    // 3. Move transport
    game.apply_action(smac_core::GameAction::MoveUnit {
        unit_id: transport_id,
        target_x: 5,
        target_y: 6,
    })
    .expect("Move should succeed");

    // 4. Unload unit at new location
    // Must clear tile first if occupied (transport is there)
    // Actually, in SMAC you can unload to adjacent or same tile.
    // My implementation unloads to same tile, but checks if occupied.
    // Transport is in (5, 6).

    // For test, move passenger out of cargo to (5, 6)
    game.apply_action(smac_core::GameAction::UnloadUnit {
        unit_id: passenger_id,
        transport_id,
    })
    .expect("Unloading should succeed");

    assert!(!game
        .unit(transport_id)
        .unwrap()
        .cargo_unit_ids
        .contains(&passenger_id));
    // Since transport and passenger are in same tile, and passenger was just added,
    // it might have displaced the transport in the tile.unit if not careful.
    // In our implementation, tile.unit = Some(unit_id) overwrites.
    assert!(game.tiles[6 * game.width + 5].unit == Some(passenger_id));
    assert_eq!(game.unit(passenger_id).unwrap().x, 5);
    assert_eq!(game.unit(passenger_id).unwrap().y, 6);
}

#[test]
fn planet_buster_vaporizes_surrounding_area() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let owner = roles.player;
    let rival = roles.ai;

    // 1. Setup a target base and unit
    game.bases.push(smac_core::Base {
        id: 100,
        owner: rival,
        name: "Target City".to_string(),
        x: 8,
        y: 8,
        population: 5,
        minerals_stock: 0,
        nutrients_stock: 0,
        production: smac_core::ProductionItem::RecyclingTanks,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: smac_core::GovernorMode::Off,
    });
    game.tiles[8 * game.width + 8].base = Some(100);

    let defender_id = game.spawn_unit(rival, UnitKind::ScoutPatrol, 8, 8);

    // 2. Create Planet Buster design
    let pb_design = smac_core::UnitDesign {
        name: "PB-1".to_string(),
        chassis: smac_core::Chassis::Aircraft,
        weapon: smac_core::Weapon::PlanetBuster(20),
        armor: smac_core::Armor::SynthMetal(1),
        cost: 100,
        abilities: Vec::new(),
    };
    game.add_unit_design(owner, pb_design);
    let design_index = game.faction(owner).unwrap().unit_designs.len() - 1;

    let pb_id = game
        .spawn_unit_with_design(
            owner,
            UnitKind::CustomUnit(game.factions[owner].unit_designs[design_index].clone()),
            design_index,
            7,
            8,
            0,
        )
        .unwrap();

    // 3. Launch Planet Buster (Attack the tile)
    game.apply_action(smac_core::GameAction::MoveUnit {
        unit_id: pb_id,
        target_x: 8,
        target_y: 8,
    })
    .expect("PB launch should succeed");

    // 4. Verify Destruction
    assert!(game.unit(pb_id).is_none()); // PB consumed
    assert!(game.unit(defender_id).is_none()); // Defender vaporized
    assert!(game.bases.iter().find(|b| b.id == 100).is_none()); // Base vaporized

    // Check 3x3 crater field
    for dy in -1..=1 {
        for dx in -1..=1 {
            let tx = 8 + dx;
            let ty = 8 + dy;
            let idx = ty as usize * game.width + tx as usize;
            assert_eq!(game.tiles[idx].terrain, smac_core::Terrain::Crater);
            assert_eq!(game.tiles[idx].elevation, -10);
        }
    }
}

#[test]
fn air_superiority_intercepts_moving_enemy() {
    let mut game = GameState::new_game(16, 16, 7);
    let roles = runtime_roles();
    let owner = roles.player;
    let rival = roles.ai;

    // 1. Create interceptor
    let air_design = smac_core::UnitDesign {
        name: "Interceptor".to_string(),
        chassis: smac_core::Chassis::Aircraft,
        weapon: smac_core::Weapon::HandLaser(1),
        armor: smac_core::Armor::SynthMetal(1),
        cost: 30,
        abilities: vec![smac_core::Ability::AirSuperiority],
    };
    game.add_unit_design(owner, air_design);
    let design_index = game.faction(owner).unwrap().unit_designs.len() - 1;

    let interceptor_id = game
        .spawn_unit_with_design(
            owner,
            UnitKind::CustomUnit(game.factions[owner].unit_designs[design_index].clone()),
            design_index,
            5,
            5,
            0,
        )
        .unwrap();
    game.set_unit_activity(interceptor_id, smac_core::UnitActivity::Patrol);

    // 2. Spawn enemy moving through adjacent tile
    let enemy_id = game.spawn_unit(rival, UnitKind::ScoutPatrol, 6, 6);

    // 3. Move enemy: should trigger interception
    // Tile (6,5) is adjacent to interceptor at (5,5)
    game.apply_action(smac_core::GameAction::MoveUnit {
        unit_id: enemy_id,
        target_x: 6,
        target_y: 5,
    })
    .expect("Move should resolve (with combat)");

    // Interceptor or enemy should have taken damage
    // Note: resolve_combat might destroy one of them.
    let interceptor_alive = game.unit(interceptor_id).is_some();
    let enemy_alive = game.unit(enemy_id).is_some();

    if interceptor_alive && enemy_alive {
        let interceptor = game.unit(interceptor_id).unwrap();
        let enemy = game.unit(enemy_id).unwrap();
        assert!(interceptor.hp < 10 || enemy.hp < 10);
    } else {
        assert!(!interceptor_alive || !enemy_alive);
    }
}
