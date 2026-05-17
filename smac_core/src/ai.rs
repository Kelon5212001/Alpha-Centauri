use crate::{
    content, presentation, Ability, Armor, Chassis, GameAction, GameState, Tech, Terrain,
    UnitDesign, UnitKind, Weapon, Yields,
};

#[derive(Clone, Copy, Debug, Default)]
struct AiEconomySignals {
    expansion_pressure: bool,
    military_pressure: i32,
    infrastructure_pressure: bool,
    energy_pressure: bool,
    mineral_pressure: bool,
    support_pressure: bool,
    support_deficit: i32,
}

#[derive(Clone, Copy, Debug, Default)]
struct AiTacticalSignals {
    attack_bias: i32,
    exploration_bias: i32,
}

pub fn run_non_player_turns(state: &mut GameState) {
    let start = std::time::Instant::now();
    let ai_owner = state.ai_owner();
    state.reset_moves_for_owner(ai_owner);
    spawn_native_life(state);
    run_native_life_turn(state);
    run_ai_strategy(state);
    run_ai_economy(state);
    update_ai_modernization(state, ai_owner);
    run_ai_tactics(state);
    let duration = start.elapsed();
    state.push_log(format!("AI TURN PROFILER: Finished in {:?}", duration));
}

pub fn run_autoplay_turn_for_owner(state: &mut GameState, owner: usize) {
    state.reset_moves_for_owner(owner);
    run_autoplay_turn_for_owner_internal(state, owner);
}

fn run_autoplay_turn_for_owner_internal(state: &mut GameState, owner: usize) {
    run_ai_strategy_for_owner(state, owner);
    run_ai_economy_for_owner(state, owner);
    update_ai_modernization(state, owner);
    run_ai_tactics_for_owner(state, owner);
}

pub fn run_ai_strategy(state: &mut GameState) {
    run_ai_strategy_for_owner(state, state.ai_owner());
}

fn run_ai_strategy_for_owner(state: &mut GameState, owner: usize) {
    update_ai_research(state, owner);
    update_ai_social_engineering(state, owner);
    update_ai_diplomacy(state, owner);

    let ai_colonies: Vec<usize> = state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == owner && u.kind == UnitKind::ColonyPod)
        .map(|u| u.id)
        .collect();

    if state.bases_for(owner).is_empty() {
        for colony_id in ai_colonies {
            let _ = state.apply_action(GameAction::FoundBase { unit_id: colony_id });
            break;
        }
    }

    update_ai_social_engineering(state, owner);
    update_ai_diplomacy(state, owner);
}

fn update_ai_research(state: &mut GameState, owner: usize) {
    let Some(faction) = state.faction(owner) else {
        return;
    };
    let current_research = faction.current_research;
    let support = state.faction_support_summary(owner);
    let base_count = state.bases_for(owner).len();
    let live_units = state.live_units_for(owner).len();
    let underexpanded = base_count < 4;
    let support_strained = support.supported_units > 0 || support.unit_upkeep > 0;
    let has_any_project = state
        .built_secret_projects
        .iter()
        .any(|(_, project_owner)| *project_owner == owner);

    // Only pick new research if current is not available or we want to re-evaluate
    if state.is_research_available(owner, current_research) && state.turn % 20 != 0 {
        return;
    }

    let tech_tree = crate::technology_tree::TechnologyTree::new();
    let personality = &faction.personality;

    let mut best_tech = None;
    let mut best_score = -1;

    for tech_variant in crate::Tech::all() {
        if !state.is_research_available(owner, tech_variant) {
            continue;
        }

        let Some(tech_def) = tech_tree.get_technology(tech_variant.content_id()) else {
            continue;
        };

        let mut score = 10; // Base score

        match tech_def.category {
            crate::technology_tree::TechCategory::Conquer => score += personality.aggression * 2,
            crate::technology_tree::TechCategory::Discover => score += personality.tech_bias * 2,
            crate::technology_tree::TechCategory::Explore => {
                score += personality.expansion_bias * 2
            }
            crate::technology_tree::TechCategory::Build => score += personality.diplomatic_tone * 2,
        }

        // Bonus for lower level techs (easier to complete)
        score += (5 - tech_def.level as i32) * 5;

        let unlocks_support_recovery = tech_def.enables.facilities.iter().any(|facility| {
            matches!(
                facility.as_str(),
                "command_center"
                    | "recycling_tanks"
                    | "greenhouse"
                    | "recreation_commons"
                    | "transit_hub"
                    | "trade_exchange"
                    | "field_hospital"
                    | "network_node"
                    | "mineral_refinery"
            )
        });

        let unlocks_expansion_mobility = tech_def.enables.units.iter().any(|unit| {
            matches!(
                unit.as_str(),
                "former" | "speeder" | "escort_speeder" | "raider_speeder"
            )
        }) || tech_def
            .enables
            .facilities
            .iter()
            .any(|facility| matches!(facility.as_str(), "transit_hub" | "freight_depot"));

        if underexpanded {
            if tech_def.level <= 2 {
                score += 12;
            }
            if unlocks_support_recovery {
                score += 20;
            }
            if unlocks_expansion_mobility {
                score += 10;
            }
            if tech_def.level >= 4 {
                score -= 8;
            }
        }

        if support_strained && unlocks_support_recovery {
            score += 25;
        }

        if tech_variant == crate::Tech::SecretsOfPlanet {
            if underexpanded
                || support_strained
                || live_units < base_count.max(2)
                || !has_any_project
            {
                score -= 80;
            } else {
                score += personality.tech_bias * 4;
            }
        }

        if score > best_score {
            best_score = score;
            best_tech = Some(tech_variant);
        }
    }

    if let Some(tech) = best_tech {
        if tech != current_research {
            let _ = state.apply_action(GameAction::ChooseResearch { owner, tech });
        }
    }
}

fn update_ai_social_engineering(state: &mut GameState, owner: usize) {
    if state.turn % 10 != 0 {
        return;
    }

    let faction = match state.faction(owner) {
        Some(f) => f,
        None => return,
    };

    let known_tech_ids: std::collections::HashSet<String> = faction
        .known_techs
        .iter()
        .map(|t| t.content_id().to_string())
        .collect();
    let tech_tree = crate::technology_tree::TechnologyTree::new();

    let is_unlocked = |option_id: &str| {
        if option_id == "frontier"
            || option_id == "simple"
            || option_id == "survival"
            || option_id == "none"
        {
            return true;
        }
        tech_tree.all_technologies().iter().any(|tech| {
            known_tech_ids.contains(tech.id.as_str())
                && tech
                    .enables
                    .social_engineering
                    .contains(&option_id.to_string())
        })
    };

    let mut politics = faction.social_engineering.politics;
    let mut economics = faction.social_engineering.economics;
    let mut values = faction.social_engineering.values;
    let mut future = faction.social_engineering.future;

    let personality = &faction.personality;
    let support = state.faction_support_summary(owner);
    let has_unrest = !state.unrest_base_ids(owner).is_empty();
    let support_strained = support.supported_units > 0;
    let low_expansion = state.bases_for(owner).len() < 3;

    // Politics
    if is_unlocked("police") && (support_strained || has_unrest) {
        politics = crate::model::Politics::Police;
    } else if is_unlocked("democratic")
        && personality.aggression < 3
        && faction.base_attributes.growth < 10
    {
        politics = crate::model::Politics::Democratic;
    } else if is_unlocked("fundamentalist") && personality.aggression > 5 {
        politics = crate::model::Politics::Fundamentalist;
    } else if is_unlocked("police") && personality.expansion_bias > 5 {
        politics = crate::model::Politics::Police;
    }

    // Economics
    if is_unlocked("planned") && (support_strained || has_unrest || faction.food_security < 0) {
        economics = crate::model::Economics::Planned;
    } else if is_unlocked("free_market") && faction.energy < 200 && !support_strained && !has_unrest
    {
        economics = crate::model::Economics::FreeMarket;
    } else if is_unlocked("planned") && faction.base_attributes.industry < 10 {
        economics = crate::model::Economics::Planned;
    } else if is_unlocked("green") && personality.tech_bias > 5 && !support_strained {
        economics = crate::model::Economics::Green;
    }

    // Values
    if is_unlocked("knowledge") && personality.tech_bias > 5 && !support_strained && !has_unrest {
        values = crate::model::Values::Knowledge;
    } else if is_unlocked("power") && personality.aggression > 5 {
        values = crate::model::Values::Power;
    } else if is_unlocked("wealth") && (faction.energy < 100 || low_expansion) {
        values = crate::model::Values::Wealth;
    }

    // Future
    if is_unlocked("cybernetic") && personality.tech_bias > 7 && !support_strained && !has_unrest {
        future = crate::model::FutureSociety::Cybernetic;
    } else if is_unlocked("thought_control") && personality.aggression > 7 {
        future = crate::model::FutureSociety::ThoughtControl;
    } else if is_unlocked("eudaimonic") && faction.base_attributes.growth > 20 {
        future = crate::model::FutureSociety::Eudaimonic;
    }

    let action = GameAction::ChooseSocialEngineering {
        owner,
        politics: Some(politics),
        economics: Some(economics),
        values: Some(values),
        future: Some(future),
    };
    let _ = state.apply_action(action);
}

fn update_ai_diplomacy(state: &mut GameState, owner: usize) {
    if state.turn % 5 != 0 {
        return;
    }

    let (personality, our_known_techs) = match state.faction(owner) {
        Some(f) => (f.personality.clone(), f.known_techs.clone()),
        None => return,
    };

    // Check pending offers
    let offers = state.pending_diplomacy_offers.clone();
    for &(proposer, receiver, status) in &offers {
        if receiver == owner {
            let mut accept = false;
            // Aggressive AIs only accept Pacts if high diplomatic tone
            if status == crate::DiplomacyStatus::Pact && personality.aggression < 5 {
                accept = true;
            } else if status == crate::DiplomacyStatus::Treaty && personality.diplomatic_tone > 0 {
                accept = true;
            }

            let _ = state.apply_action(GameAction::RespondDiplomacy {
                proposer,
                receiver: owner,
                status,
                accept,
            });
        }
    }

    // Check pending tech trades
    let trades = state.pending_tech_trades.clone();
    for &(proposer, receiver, offered, requested) in &trades {
        if receiver == owner {
            let mut accept = false;
            let relation = &state.relations[owner][proposer];

            let offered_cost = content::tech_cost(offered);
            let requested_cost = content::tech_cost(requested);

            // Accept trade if attitude is neutral or better, personality is not too aggressive,
            // and the trade is fair (or we really like them)
            let is_fair = requested_cost <= offered_cost + 20;
            let is_friendly = relation.attitude >= 40;

            if relation.attitude >= 0 && personality.aggression < 8 && (is_fair || is_friendly) {
                accept = true;
            }

            let _ = state.apply_action(GameAction::RespondTechTrade {
                proposer,
                receiver: owner,
                offered_tech: offered,
                requested_tech: requested,
                accept,
            });
        }
    }

    // Check pending demands
    let demands = state.pending_demands.clone();
    for (proposer, receiver, demand) in demands {
        if receiver == owner {
            let mut accept = false;
            let relation = &state.relations[owner][proposer];

            // Fear-based acceptance: if they have much more population
            let our_pop: i32 = state
                .bases
                .iter()
                .filter(|b| b.owner == owner)
                .map(|b| b.population)
                .sum();
            let their_pop: i32 = state
                .bases
                .iter()
                .filter(|b| b.owner == proposer)
                .map(|b| b.population)
                .sum();

            if their_pop > our_pop * 2 || (relation.attitude < -40 && personality.aggression < 5) {
                accept = true; // Cower/Appease
            }

            let _ = state.apply_action(GameAction::RespondDemand {
                proposer,
                receiver: owner,
                demand,
                accept,
            });
        }
    }

    // Proactive offers
    let faction_count = state.factions.len();
    for other_id in 0..faction_count {
        if other_id == owner {
            continue;
        }

        let (current_status, attitude) = {
            let relation = &state.relations[owner][other_id];
            (relation.status, relation.attitude)
        };

        // Proactive Demands: Extort weaker neighbors if aggressive
        if personality.aggression >= 7
            && state.turn % 20 == 0
            && current_status != crate::DiplomacyStatus::Pact
        {
            let our_pop: i32 = state
                .bases
                .iter()
                .filter(|b| b.owner == owner)
                .map(|b| b.population)
                .sum();
            let their_pop: i32 = state
                .bases
                .iter()
                .filter(|b| b.owner == other_id)
                .map(|b| b.population)
                .sum();

            if our_pop > their_pop * 2 && attitude <= 10 {
                // Try to find a tech they have
                let other_techs = &state.factions[other_id].known_techs;
                let they_have_we_dont: Vec<_> = other_techs
                    .iter()
                    .filter(|t| !our_known_techs.contains(t))
                    .collect();

                let demand = if !they_have_we_dont.is_empty() {
                    crate::model::DemandKind::Technology(*they_have_we_dont[0])
                } else {
                    crate::model::DemandKind::Energy(100)
                };

                let _ = state.apply_action(GameAction::MakeDemand {
                    proposer: owner,
                    receiver: other_id,
                    demand,
                });
            }
        }

        // Propose tech trade if we have a Treaty or Pact and high attitude
        if (current_status == crate::DiplomacyStatus::Treaty
            || current_status == crate::DiplomacyStatus::Pact)
            && attitude >= 40
            && state.turn % 10 == 0
        {
            // Find a tech we have that they don't, and vice versa
            let other_known_techs = &state.factions[other_id].known_techs;

            let mut we_have_they_dont: Vec<_> = our_known_techs
                .iter()
                .filter(|t| !other_known_techs.contains(t))
                .collect();
            let mut they_have_we_dont: Vec<_> = other_known_techs
                .iter()
                .filter(|t| !our_known_techs.contains(t))
                .collect();

            // Sort so we offer the cheapest tech we have, and ask for the most expensive tech they have
            we_have_they_dont.sort_by_key(|t| content::tech_cost(**t));
            they_have_we_dont.sort_by_key(|t| std::cmp::Reverse(content::tech_cost(**t)));

            if !we_have_they_dont.is_empty() && !they_have_we_dont.is_empty() {
                let _ = state.apply_action(GameAction::ProposeTechTrade {
                    proposer: owner,
                    receiver: other_id,
                    offered_tech: *we_have_they_dont[0],
                    requested_tech: *they_have_we_dont[0],
                });
            }
        }

        // If neutral or better, consider proposing a Treaty
        if current_status == crate::DiplomacyStatus::War
            || current_status == crate::DiplomacyStatus::Truce
        {
            if attitude >= 20 || (personality.diplomatic_tone > 5 && attitude >= 0) {
                // Check if already proposed recently
                if !state.pending_diplomacy_offers.iter().any(|&(p, r, s)| {
                    p == owner && r == other_id && s == crate::DiplomacyStatus::Treaty
                }) {
                    let _ = state.apply_action(GameAction::ProposeDiplomacy {
                        proposer: owner,
                        receiver: other_id,
                        status: crate::DiplomacyStatus::Treaty,
                    });
                }
            }
        }

        // If already have a Treaty, consider proposing a Pact
        if current_status == crate::DiplomacyStatus::Treaty {
            if attitude >= 60 || (personality.diplomatic_tone > 8 && attitude >= 40) {
                if !state.pending_diplomacy_offers.iter().any(|&(p, r, s)| {
                    p == owner && r == other_id && s == crate::DiplomacyStatus::Pact
                }) {
                    let _ = state.apply_action(GameAction::ProposeDiplomacy {
                        proposer: owner,
                        receiver: other_id,
                        status: crate::DiplomacyStatus::Pact,
                    });
                }
            } else if attitude <= -20 && personality.aggression > 5 {
                // Break Treaty
                let _ = state.apply_action(GameAction::UpdateDiplomacy {
                    faction_a: owner,
                    faction_b: other_id,
                    status: crate::DiplomacyStatus::Truce,
                });
            }
        } else if current_status == crate::DiplomacyStatus::Pact {
            if attitude <= 0 {
                // Break Pact
                let _ = state.apply_action(GameAction::UpdateDiplomacy {
                    faction_a: owner,
                    faction_b: other_id,
                    status: crate::DiplomacyStatus::Treaty,
                });
            }
        } else if current_status == crate::DiplomacyStatus::Truce {
            if attitude <= -40 && personality.aggression > 6 {
                // Declare War
                let _ = state.apply_action(GameAction::UpdateDiplomacy {
                    faction_a: owner,
                    faction_b: other_id,
                    status: crate::DiplomacyStatus::War,
                });
            }
        }
    }
}

fn update_ai_unit_designs(state: &mut GameState, owner: usize) {
    if state.turn % 15 != 0 {
        return;
    }

    let faction = match state.faction(owner) {
        Some(f) => f,
        None => return,
    };

    let known_techs = &faction.known_techs;

    // Find best weapon
    let mut best_weapon_id = "hand_laser".to_string();
    let mut best_weapon_power = 1;

    for &tech in known_techs {
        for weapon_id in content::tech_enabled_weapon_ids(tech) {
            let power = content::weapon_power_from_id(&weapon_id);
            if power > best_weapon_power {
                best_weapon_power = power;
                best_weapon_id = weapon_id;
            }
        }
    }

    // Find best armor
    let mut best_armor_id = "synth_metal".to_string();
    let mut best_armor_power = 1;

    for &tech in known_techs {
        for armor_id in content::tech_enabled_armor_ids(tech) {
            let power = content::armor_power_from_id(&armor_id);
            if power > best_armor_power {
                best_armor_power = power;
                best_armor_id = armor_id;
            }
        }
    }

    // Propose new designs if better than current best for each chassis
    let mut chassis_types = vec![Chassis::Infantry, Chassis::Speeder, Chassis::Hovertank];
    if known_techs.contains(&Tech::DoctrineInitiative) {
        chassis_types.push(Chassis::Sea);
    }
    if known_techs.contains(&Tech::DoctrineAirPower) {
        chassis_types.push(Chassis::Aircraft);
    }

    for chassis in chassis_types {
        let is_planet_buster = chassis == Chassis::Aircraft && best_weapon_id == "planet_buster";

        let weapon = Weapon::from_content_id(&best_weapon_id, best_weapon_power).unwrap();
        let armor = Armor::from_content_id(&best_armor_id, best_armor_power).unwrap();
        let mut abilities = Vec::new();

        if chassis == Chassis::Sea {
            abilities.push(Ability::Transport);
        }
        if chassis == Chassis::Aircraft && !is_planet_buster {
            abilities.push(Ability::AirSuperiority);
        }

        let mut new_design = UnitDesign {
            name: format!(
                "AI {} {}-{}{}",
                presentation::chassis_label(chassis),
                best_weapon_power,
                best_armor_power,
                if is_planet_buster {
                    " (PB)"
                } else if abilities.contains(&Ability::Transport) {
                    " (TRP)"
                } else if abilities.contains(&Ability::AirSuperiority) {
                    " (AIR)"
                } else {
                    ""
                }
            ),
            chassis,
            weapon,
            armor,
            cost: 0,
            abilities,
        };
        new_design.recompute_cost();

        let _ = state.apply_action(GameAction::DesignUnit {
            owner,
            design: new_design,
        });
    }
}

fn update_ai_modernization(state: &mut GameState, owner: usize) {
    update_ai_unit_designs(state, owner);

    if state.turn % 15 != 0 {
        return;
    }

    let designs = match state.faction(owner) {
        Some(f) => f.unit_designs.clone(),
        None => return,
    };

    let unit_ids: Vec<usize> = state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == owner)
        .map(|u| u.id)
        .collect();

    for unit_id in unit_ids {
        let current_design_index = {
            let Some(unit) = state.unit(unit_id) else {
                continue;
            };
            unit.design_index
        };

        let current_design = match designs.get(current_design_index) {
            Some(d) => d,
            None => continue,
        };

        // Find a better design with the same chassis that is currently available to build
        let mut best_design = None;
        for design in &designs {
            if design.chassis != current_design.chassis {
                continue;
            }

            // Only consider designs we have the tech for
            let is_available = crate::model::ProductionItem::all().into_iter().any(|item| {
                let matches_name = if let Some(kind) = crate::content::production_unit_kind(item) {
                    if let crate::model::UnitKind::CustomUnit(d) = kind {
                        d.name == design.name
                    } else {
                        crate::content::unit_name(kind) == design.name
                    }
                } else {
                    false
                };
                matches_name && state.is_production_available(owner, item)
            });

            // If we can't find a standard matching production item, we assume it's a custom design
            // For custom designs we need to check weapon/armor tech requirements (which we don't strictly have right now)
            // But since all starting designs are standard, this check suffices for now.
            let actually_available =
                is_available || (design.name.starts_with("Custom")/* placeholder */);
            if !actually_available {
                continue;
            }

            if design.attack_strength() > current_design.attack_strength()
                || design.defense_strength() > current_design.defense_strength()
            {
                best_design = Some(design.clone());
            }
        }

        if let Some(new_design) = best_design {
            let energy = state.faction(owner).map(|f| f.energy).unwrap_or(0);
            if energy < 50 {
                break;
            }

            let _ = state.apply_action(GameAction::UpgradeUnit {
                unit_id,
                new_design,
            });
        }
    }
}

pub fn run_ai_economy(state: &mut GameState) {
    run_ai_economy_for_owner(state, state.ai_owner());
}

fn run_ai_economy_for_owner(state: &mut GameState, owner: usize) {
    for (base_a_id, base_b_id, kind) in state.suggested_convoy_repairs(owner) {
        let _ = state.repair_convoy_route_typed(base_a_id, base_b_id, kind);
    }
    for (base_a_id, base_b_id, kind) in state.suggested_convoy_rebuilds(owner) {
        let _ = state.add_convoy_route_typed(base_a_id, base_b_id, kind);
    }

    let base_ids: Vec<usize> = state
        .bases_for(owner)
        .into_iter()
        .map(|base| base.id)
        .collect();

    for base_id in &base_ids {
        maybe_assign_ai_convoy_route(state, *base_id, owner);
    }

    for base_id in base_ids {
        let Some(base) = state.base(base_id) else {
            continue;
        };
        let desired_item = choose_ai_production_for_base(state, base.id, owner);

        if base.production != desired_item {
            // Only switch if we haven't made significant progress (e.g., 40%)
            // Or if the current production is a basic unit and we need a critical facility
            let current_cost = state.production_cost(owner, base.production).max(1);
            let progress_pct = (base.minerals_stock * 100) / current_cost;

            let is_critical_switch = match desired_item {
                crate::ProductionItem::Greenhouse => {
                    state.base_food_margin(base.id).unwrap_or(0) < 0
                }
                crate::ProductionItem::RecreationCommons => state.base_unrest(base.id) > 1,
                crate::ProductionItem::ColonyPod => {
                    state.bases_for(owner).len() <= 2 && base.population >= 4
                }
                _ => false,
            };

            if progress_pct < 40 || is_critical_switch {
                let _ = state.apply_action(GameAction::SetBaseProduction {
                    base_id,
                    item: desired_item,
                });
            }
        }

        if let Some(base) = state.base(base_id) {
            if base.production_queue.is_empty() {
                let queue_item = choose_ai_queue_follow_up(state, base.id, owner);
                if queue_item != base.production {
                    let _ = state.apply_action(GameAction::QueueBaseProduction {
                        base_id,
                        item: queue_item,
                    });
                }
            }
        }
    }
}

fn maybe_assign_ai_convoy_route(state: &mut GameState, base_id: usize, owner: usize) {
    let Some(base) = state.base(base_id) else {
        return;
    };
    if base.owner != owner {
        return;
    }
    let should_route = base.facilities.contains(&crate::Facility::TradeExchange)
        || base.facilities.contains(&crate::Facility::FreightDepot)
        || base.facilities.contains(&crate::Facility::CommandCenter)
        || base.facilities.contains(&crate::Facility::FieldHospital)
        || base.facilities.contains(&crate::Facility::MilitaryAcademy)
        || base.facilities.contains(&crate::Facility::ForwardDepot)
        || state.base_potential_trade_links(base_id) >= 1 && state.bases_for(owner).len() >= 2;
    if !should_route {
        return;
    }
    let faction_support_pressure = state.faction_support_summary(owner).supported_units > 0;
    let has_military_support_infra = base.facilities.contains(&crate::Facility::CommandCenter)
        || base.facilities.contains(&crate::Facility::FieldHospital)
        || base.facilities.contains(&crate::Facility::MilitaryAcademy)
        || base.facilities.contains(&crate::Facility::ForwardDepot);
    let kind = if has_military_support_infra
        && (state.base_local_military_pressure(base_id) >= 1
            || state.damaged_garrison_count_for_base(base_id) > 0
            || faction_support_pressure)
    {
        crate::ConvoyRouteKind::MilitarySupply
    } else if base.facilities.contains(&crate::Facility::FreightDepot)
        || state.base_mineral_margin(base_id).unwrap_or_default() <= 0
    {
        crate::ConvoyRouteKind::Freight
    } else {
        crate::ConvoyRouteKind::Trade
    };
    if let Some(target_id) = state
        .available_convoy_targets_for_kind(base_id, kind)
        .into_iter()
        .next()
    {
        let _ = state.add_convoy_route_typed(base_id, target_id, kind);
    }
}

fn choose_ai_recovery_production(
    state: &GameState,
    base: &crate::Base,
    owner: usize,
    military_pressure: i32,
) -> Option<crate::ProductionItem> {
    let unrest = state.base_unrest(base.id);
    let damaged_garrisons = state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner == owner)
        .filter(|unit| {
            let max_hp = content::unit_base_hp(unit.kind.clone());
            let on_base = state
                .tile(unit.x, unit.y)
                .and_then(|tile| tile.base)
                .map(|id| id == base.id)
                .unwrap_or(false);
            on_base && unit.hp < max_hp
        })
        .count();
    let severe_unrest = unrest >= 2 || (unrest >= 1 && base.population >= 6);

    if unrest > 0
        && state.is_production_available(owner, crate::ProductionItem::RecreationCommons)
        && !base
            .facilities
            .contains(&crate::Facility::RecreationCommons)
    {
        return Some(crate::ProductionItem::RecreationCommons);
    }

    if severe_unrest
        && state.is_production_available(owner, crate::ProductionItem::HologramTheatre)
        && base
            .facilities
            .contains(&crate::Facility::RecreationCommons)
        && !base.facilities.contains(&crate::Facility::HologramTheatre)
    {
        return Some(crate::ProductionItem::HologramTheatre);
    }

    if damaged_garrisons > 0
        && state.is_production_available(owner, crate::ProductionItem::FieldHospital)
        && !base.facilities.contains(&crate::Facility::FieldHospital)
    {
        return Some(crate::ProductionItem::FieldHospital);
    }

    if military_pressure == 0
        && (damaged_garrisons > 0 || severe_unrest)
        && state.is_production_available(owner, crate::ProductionItem::ResearchHospital)
        && base.facilities.contains(&crate::Facility::FieldHospital)
        && !base.facilities.contains(&crate::Facility::ResearchHospital)
    {
        return Some(crate::ProductionItem::ResearchHospital);
    }

    None
}

fn choose_ai_support_production(
    state: &GameState,
    base: &crate::Base,
    owner: usize,
    yields: Yields,
    trade_links: usize,
) -> Option<crate::ProductionItem> {
    let support = state.faction_support_summary(owner);
    if support.supported_units <= 0 {
        return None;
    }

    if state.is_production_available(owner, crate::ProductionItem::CommandCenter)
        && !base.facilities.contains(&crate::Facility::CommandCenter)
    {
        return Some(crate::ProductionItem::CommandCenter);
    }

    let base_count = state.bases_for(owner).len() as i32;
    let severe_support_pressure =
        support.supported_units >= base_count.max(1) || support.unit_upkeep >= 2;

    if severe_support_pressure
        && state.is_production_available(owner, crate::ProductionItem::TransitHub)
        && !base.facilities.contains(&crate::Facility::TransitHub)
        && base.facilities.contains(&crate::Facility::CommandCenter)
        && yields.minerals + yields.energy >= 4
    {
        return Some(crate::ProductionItem::TransitHub);
    }

    if severe_support_pressure
        && state.is_production_available(owner, crate::ProductionItem::RecyclingTanks)
        && !base.facilities.contains(&crate::Facility::RecyclingTanks)
    {
        return Some(crate::ProductionItem::RecyclingTanks);
    }

    if severe_support_pressure
        && trade_links >= 1
        && state.is_production_available(owner, crate::ProductionItem::TradeExchange)
        && !base.facilities.contains(&crate::Facility::TradeExchange)
    {
        return Some(crate::ProductionItem::TradeExchange);
    }

    if severe_support_pressure
        && state.is_production_available(owner, crate::ProductionItem::NetworkNode)
        && !base.facilities.contains(&crate::Facility::NetworkNode)
        && yields.energy >= yields.minerals
    {
        return Some(crate::ProductionItem::NetworkNode);
    }

    if severe_support_pressure
        && state.is_production_available(owner, crate::ProductionItem::StockpileEnergy)
    {
        return Some(crate::ProductionItem::StockpileEnergy);
    }

    None
}

fn choose_ai_support_relief_fallback(
    state: &GameState,
    base: &crate::Base,
    owner: usize,
    yields: Yields,
    trade_links: usize,
) -> Option<crate::ProductionItem> {
    if !base.facilities.contains(&crate::Facility::RecyclingTanks)
        && state.is_production_available(owner, crate::ProductionItem::RecyclingTanks)
    {
        return Some(crate::ProductionItem::RecyclingTanks);
    }

    if trade_links >= 1
        && !base.facilities.contains(&crate::Facility::TradeExchange)
        && state.is_production_available(owner, crate::ProductionItem::TradeExchange)
    {
        return Some(crate::ProductionItem::TradeExchange);
    }

    if !base.facilities.contains(&crate::Facility::NetworkNode)
        && state.is_production_available(owner, crate::ProductionItem::NetworkNode)
        && yields.energy >= yields.minerals
    {
        return Some(crate::ProductionItem::NetworkNode);
    }

    if !base.facilities.contains(&crate::Facility::Greenhouse)
        && state.is_production_available(owner, crate::ProductionItem::Greenhouse)
        && yields.nutrients <= base.population.max(1) + 1
    {
        return Some(crate::ProductionItem::Greenhouse);
    }

    if state.is_production_available(owner, crate::ProductionItem::Former)
        && yields.nutrients >= yields.minerals
    {
        return Some(crate::ProductionItem::Former);
    }

    state
        .is_production_available(owner, crate::ProductionItem::StockpileEnergy)
        .then_some(crate::ProductionItem::StockpileEnergy)
}

fn choose_ai_production_for_base(
    state: &GameState,
    base_id: usize,
    owner: usize,
) -> crate::ProductionItem {
    let Some(base) = state.base(base_id) else {
        return content::ai_preferred_production(owner);
    };

    let yields = state
        .operational_base_yields(base.id)
        .unwrap_or_else(|| state.base_yields(base.x, base.y));
    let preferred = content::ai_preferred_production(owner);
    let signals = economy_signals_for_base(state, owner, base.id, base.x, base.y, yields);
    let prefer_raider = should_prefer_raider_speeder(state, owner, base.x, base.y);
    let psi_pressure = state.base_local_psi_pressure(base.id);
    let trade_links = state.base_potential_trade_links(base.id);
    let active_colony_pods = live_colony_pod_count(state, owner);
    let active_formers = live_former_count(state, owner);
    let owned_bases = state.bases_for(owner).len();
    let expansion_frontier = desired_ai_expansion_target(state, owner).min(4).max(2);
    let underexpanded = owned_bases + active_colony_pods < expansion_frontier;
    let colony_support_tolerance = if owned_bases == 2 {
        2
    } else if owned_bases == 1 {
        1
    } else if underexpanded {
        1
    } else {
        0
    };
    let can_push_colony_pod = signals.support_deficit <= colony_support_tolerance;
    let convoy_pressure = trade_links >= 1
        && ((state.base_local_military_pressure(base.id) >= 1 || psi_pressure >= 1)
            || state
                .convoy_route_status_for_base(base.id)
                .into_iter()
                .any(|(_, _, disrupted, intercepted, _)| disrupted || intercepted));

    let attributes = state
        .faction(owner)
        .map(|f| f.effective_attributes())
        .unwrap_or_default();
    let unrest = state.base_unrest(base.id);
    let faction = match state.faction(owner) {
        Some(f) => f,
        None => return content::ai_preferred_production(owner),
    };

    // CRITICAL DEFENSE: If base has no garrison and is under threat
    let local_units = state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == owner && u.x == base.x && u.y == base.y)
        .count();
    if local_units == 0 && signals.military_pressure >= 1 {
        if state.is_production_available(owner, crate::ProductionItem::GarrisonGuard) {
            return crate::ProductionItem::GarrisonGuard;
        }
    }

    // FORCED EXPANSION: Single-base factions must expand once they are either
    // large enough or obviously hoarding tech/energy without building a second base.
    if owned_bases == 1
        && base.population > 3
        && signals.military_pressure < 2
        && psi_pressure < 2
        && active_colony_pods < 1
        && can_push_colony_pod
    {
        if state.is_production_available(owner, crate::ProductionItem::ColonyPod)
            && state.base_food_margin(base.id).unwrap_or(0) >= -1
        {
            return crate::ProductionItem::ColonyPod;
        }
    }

    if owned_bases == 1
        && base.population >= 2
        && state.turn >= 30
        && faction.energy >= 100
        && faction.known_techs.len() >= 3
        && signals.military_pressure < 2
        && psi_pressure < 2
        && active_colony_pods < 1
        && can_push_colony_pod
    {
        if state.is_production_available(owner, crate::ProductionItem::ColonyPod)
            && state.base_food_margin(base.id).unwrap_or(0) >= -1
        {
            return crate::ProductionItem::ColonyPod;
        }
    }

    if owned_bases <= 2
        && underexpanded
        && base.population >= 6
        && state.turn >= 35
        && signals.military_pressure < 2
        && active_colony_pods < 2
        && can_push_colony_pod
    {
        if state.is_production_available(owner, crate::ProductionItem::ColonyPod)
            && state.base_food_margin(base.id).unwrap_or(0) >= -1
        {
            return crate::ProductionItem::ColonyPod;
        }
    }

    if owned_bases <= 2
        && underexpanded
        && base.population >= 3
        && state.turn >= 30
        && faction.energy >= 35
        && faction.known_techs.len() >= 2
        && signals.military_pressure < 2
        && active_colony_pods < 2
        && can_push_colony_pod
    {
        if state.is_production_available(owner, crate::ProductionItem::ColonyPod)
            && state.base_food_margin(base.id).unwrap_or(0) >= -1
        {
            return crate::ProductionItem::ColonyPod;
        }
    }

    if owned_bases == 2
        && underexpanded
        && base.population >= 2
        && state.turn >= 50
        && signals.military_pressure < 3
        && psi_pressure < 3
        && active_colony_pods < 2
    {
        if state.is_production_available(owner, crate::ProductionItem::ColonyPod)
            && state.base_food_margin(base.id).unwrap_or(0) >= -1
        {
            return crate::ProductionItem::ColonyPod;
        }
    }

    // CRITICAL SURVIVAL: Famine or extreme unrest prevention
    if (yields.nutrients < base.population as i32
        || (yields.nutrients <= base.population as i32 + 1 && signals.expansion_pressure))
        && state.is_production_available(owner, crate::ProductionItem::Greenhouse)
        && !base.facilities.contains(&crate::Facility::Greenhouse)
    {
        return crate::ProductionItem::Greenhouse;
    }

    if (unrest > 1 || (unrest > 0 && yields.energy >= 5))
        && state.is_production_available(owner, crate::ProductionItem::RecreationCommons)
        && !base
            .facilities
            .contains(&crate::Facility::RecreationCommons)
    {
        return crate::ProductionItem::RecreationCommons;
    }

    if faction.energy < 0
        && state.is_production_available(owner, crate::ProductionItem::StockpileEnergy)
    {
        return crate::ProductionItem::StockpileEnergy;
    }

    if signals.mineral_pressure
        && state.is_production_available(owner, crate::ProductionItem::RecyclingTanks)
        && !base.facilities.contains(&crate::Facility::RecyclingTanks)
    {
        return crate::ProductionItem::RecyclingTanks;
    }

    if signals.military_pressure >= 1 {
        if state.is_production_available(owner, crate::ProductionItem::PerimeterDefense)
            && !base.facilities.contains(&crate::Facility::PerimeterDefense)
            && (yields.minerals >= 4 || signals.military_pressure >= 2)
        {
            return crate::ProductionItem::PerimeterDefense;
        }
        if state.is_production_available(owner, crate::ProductionItem::GarrisonGuard)
            && (yields.minerals >= yields.energy || signals.military_pressure >= 2)
        {
            return crate::ProductionItem::GarrisonGuard;
        }
    }

    if let Some(recovery_item) =
        choose_ai_recovery_production(state, base, owner, signals.military_pressure)
    {
        return recovery_item;
    }

    if underexpanded
        && active_colony_pods < if owned_bases <= 2 { 2 } else { 1 }
        && signals.support_deficit > 0
        && signals.military_pressure < 2
        && psi_pressure < 2
        && can_push_colony_pod
        && base.population >= 2
        && state.base_food_margin(base.id).unwrap_or(0) >= -1
        && state.is_production_available(owner, crate::ProductionItem::ColonyPod)
    {
        return crate::ProductionItem::ColonyPod;
    }

    if let Some(support_item) =
        choose_ai_support_production(state, base, owner, yields, trade_links)
    {
        return support_item;
    }

    if state.is_production_available(owner, crate::ProductionItem::RecreationCommons)
        && !base
            .facilities
            .contains(&crate::Facility::RecreationCommons)
        && (base.population >= 3 || (unrest > 0 && yields.energy >= 4))
    {
        return crate::ProductionItem::RecreationCommons;
    }

    if yields.nutrients <= base.population.max(1) + 1
        && state.is_production_available(owner, crate::ProductionItem::Greenhouse)
        && !base.facilities.contains(&crate::Facility::Greenhouse)
    {
        return crate::ProductionItem::Greenhouse;
    }

    if state.base_mineral_margin(base.id).unwrap_or_default() <= 0
        && state.is_production_available(owner, crate::ProductionItem::MineralRefinery)
        && !base.facilities.contains(&crate::Facility::MineralRefinery)
    {
        return crate::ProductionItem::MineralRefinery;
    }

    if signals.energy_pressure
        && trade_links >= 1
        && state.is_production_available(owner, crate::ProductionItem::TradeExchange)
        && !base.facilities.contains(&crate::Facility::TradeExchange)
    {
        return crate::ProductionItem::TradeExchange;
    }

    if trade_links >= 1
        && state.base_convoy_escort_count(base.id) == 0
        && state.is_production_available(owner, crate::ProductionItem::EscortSpeeder)
        && convoy_pressure
    {
        return crate::ProductionItem::EscortSpeeder;
    }

    if trade_links >= 1
        && state.is_production_available(owner, crate::ProductionItem::PatrolGrid)
        && !base.facilities.contains(&crate::Facility::PatrolGrid)
        && (state.base_local_military_pressure(base.id) >= 1
            || state.base_local_psi_pressure(base.id) >= 1)
    {
        return crate::ProductionItem::PatrolGrid;
    }

    if trade_links >= 1
        && state.is_production_available(owner, crate::ProductionItem::FreightDepot)
        && !base.facilities.contains(&crate::Facility::FreightDepot)
        && (base.facilities.contains(&crate::Facility::TradeExchange)
            || state.base_mineral_margin(base.id).unwrap_or(1) <= 1)
    {
        return crate::ProductionItem::FreightDepot;
    }

    if signals.energy_pressure && signals.infrastructure_pressure && signals.military_pressure == 0
    {
        if state.is_production_available(owner, crate::ProductionItem::NetworkNode)
            && !base.facilities.contains(&crate::Facility::NetworkNode)
        {
            return crate::ProductionItem::NetworkNode;
        }
        return crate::ProductionItem::Former;
    }

    if signals.expansion_pressure
        && signals.military_pressure < 2
        && psi_pressure < 2
        && active_colony_pods < 2
        && can_push_colony_pod
    {
        return crate::ProductionItem::ColonyPod;
    }

    if (active_formers < 2
        || (active_formers < state.bases_for(owner).len() && signals.infrastructure_pressure))
        && state.is_production_available(owner, crate::ProductionItem::Former)
        && signals.military_pressure < 2
        && (!signals.support_pressure || active_formers == 0)
    {
        return crate::ProductionItem::Former;
    }

    if signals.military_pressure >= 1 {
        if psi_pressure >= 2
            && state.is_production_available(owner, crate::ProductionItem::PsiSentinel)
        {
            return crate::ProductionItem::PsiSentinel;
        }
        if state.is_production_available(owner, crate::ProductionItem::PsiSentinel)
            && yields.energy >= yields.minerals
        {
            return crate::ProductionItem::PsiSentinel;
        }
        if psi_pressure >= 2
            && state.is_production_available(owner, crate::ProductionItem::PsiBeacon)
            && !base.facilities.contains(&crate::Facility::PsiBeacon)
            && yields.energy >= 3
        {
            return crate::ProductionItem::PsiBeacon;
        }
        if state.is_production_available(owner, crate::ProductionItem::SensorArray)
            && !base.facilities.contains(&crate::Facility::SensorArray)
            && yields.energy >= 3
        {
            return crate::ProductionItem::SensorArray;
        }
        if content::ai_attack_bias(owner) >= 1
            && state.is_production_available(owner, crate::ProductionItem::TransitHub)
            && !base.facilities.contains(&crate::Facility::TransitHub)
            && yields.energy >= 3
        {
            return crate::ProductionItem::TransitHub;
        }
        if state.is_production_available(owner, crate::ProductionItem::MilitaryAcademy)
            && !base.facilities.contains(&crate::Facility::MilitaryAcademy)
            && base.facilities.contains(&crate::Facility::CommandCenter)
        {
            return crate::ProductionItem::MilitaryAcademy;
        }
        if content::ai_attack_bias(owner) >= 1
            && state.is_production_available(owner, crate::ProductionItem::RaiderSpeeder)
            && (yields.minerals >= 3 || signals.military_pressure >= 2)
            && prefer_raider
        {
            return crate::ProductionItem::RaiderSpeeder;
        }
        if state.is_production_available(owner, crate::ProductionItem::Speeder)
            && (yields.minerals >= 3 || signals.military_pressure >= 2)
        {
            return crate::ProductionItem::Speeder;
        }
        if content::ai_attack_bias(owner) >= 1
            && state.is_production_available(owner, crate::ProductionItem::ForwardDepot)
            && base.facilities.contains(&crate::Facility::TransitHub)
            && base.facilities.contains(&crate::Facility::SensorArray)
            && !base.facilities.contains(&crate::Facility::ForwardDepot)
            && yields.energy > yields.minerals
            && yields.minerals + yields.energy >= 7
        {
            return crate::ProductionItem::ForwardDepot;
        }

        // ORBITAL ECONOMY
        if signals.military_pressure <= 1 && psi_pressure < 2 {
            // Priority 1: Food if margin is tight
            if state.base_food_margin(base.id).unwrap_or(1) <= 1
                && state.is_production_available(owner, crate::ProductionItem::SkyHydroponics)
            {
                return crate::ProductionItem::SkyHydroponics;
            }
            // Priority 2: Energy if needed
            if signals.energy_pressure
                && state.is_production_available(owner, crate::ProductionItem::SolarTransmitter)
            {
                return crate::ProductionItem::SolarTransmitter;
            }
            // Priority 3: Defense if we have satellites to protect
            if (faction.sky_hydroponics + faction.solar_transmitters) > 3
                && faction.orbital_defenses
                    < (faction.sky_hydroponics + faction.solar_transmitters) / 2
                && state.is_production_available(owner, crate::ProductionItem::OrbitalDefense)
            {
                return crate::ProductionItem::OrbitalDefense;
            }
        }

        // Only build defensive scouts if we have room for support
        if signals.support_pressure && signals.military_pressure <= 1 {
            if let Some(relief_item) =
                choose_ai_support_relief_fallback(state, base, owner, yields, trade_links)
            {
                return relief_item;
            }
        }

        if !signals.mineral_pressure
            && state.is_production_available(owner, crate::ProductionItem::ScoutPatrol)
        {
            return crate::ProductionItem::ScoutPatrol;
        }
    }

    // High-tier/Mineral Pressure facilities
    if (attributes.industry >= 2 || signals.mineral_pressure)
        && state.is_production_available(owner, crate::ProductionItem::MineralRefinery)
        && !base.facilities.contains(&crate::Facility::MineralRefinery)
        && yields.minerals >= 4
    {
        return crate::ProductionItem::MineralRefinery;
    }
    if signals.mineral_pressure
        && state.is_production_available(owner, crate::ProductionItem::RecyclingTanks)
        && !base.facilities.contains(&crate::Facility::RecyclingTanks)
    {
        return crate::ProductionItem::RecyclingTanks;
    }
    if attributes.growth >= 2
        && state.is_production_available(owner, crate::ProductionItem::Greenhouse)
        && !base.facilities.contains(&crate::Facility::Greenhouse)
        && yields.nutrients >= 2
    {
        return crate::ProductionItem::Greenhouse;
    }

    if state.is_production_available(owner, crate::ProductionItem::RecyclingTanks)
        && !base.facilities.contains(&crate::Facility::RecyclingTanks)
        && base.population <= 2
    {
        return crate::ProductionItem::RecyclingTanks;
    }

    if state.is_production_available(owner, crate::ProductionItem::CommandCenter)
        && !base.facilities.contains(&crate::Facility::CommandCenter)
        && yields.minerals >= yields.energy
    {
        return crate::ProductionItem::CommandCenter;
    }

    // Check for Secret Projects
    if base.population >= 3
        && yields.minerals >= 12
        && !signals.expansion_pressure
        && signals.military_pressure < 1
    {
        for project in crate::model::SecretProject::all() {
            let item = match project {
                crate::model::SecretProject::WeatherPattern => {
                    crate::ProductionItem::WeatherPattern
                }
                crate::model::SecretProject::ClinicalImmortality => {
                    crate::ProductionItem::ClinicalImmortality
                }
                crate::model::SecretProject::EmpathGuild => crate::ProductionItem::EmpathGuild,
                crate::model::SecretProject::OrbitalElevator => {
                    crate::ProductionItem::OrbitalElevator
                }
                crate::model::SecretProject::ManifoldDrive => crate::ProductionItem::ManifoldDrive,
                crate::model::SecretProject::SingularityContainment => {
                    crate::ProductionItem::SingularityContainment
                }
                crate::model::SecretProject::BlackHoleHarvester => {
                    crate::ProductionItem::BlackHoleHarvester
                }
            };

            if state.is_production_available(owner, item) {
                return item;
            }
        }
    }

    let damaged_garrisons = state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner == owner)
        .filter(|unit| {
            let max_hp = content::unit_base_hp(unit.kind.clone());
            let on_base = state
                .tile(unit.x, unit.y)
                .and_then(|tile| tile.base)
                .and_then(|id| state.base(id))
                .map(|b| b.id == base.id)
                .unwrap_or(false);
            on_base && unit.hp < max_hp
        })
        .count();
    if damaged_garrisons > 0
        && state.is_production_available(owner, crate::ProductionItem::FieldHospital)
        && !base.facilities.contains(&crate::Facility::FieldHospital)
    {
        return crate::ProductionItem::FieldHospital;
    }

    if yields.minerals >= yields.nutrients + 2 && psi_pressure < 2 {
        if signals.support_pressure {
            if let Some(relief_item) =
                choose_ai_support_relief_fallback(state, base, owner, yields, trade_links)
            {
                return relief_item;
            }
        }
        return crate::ProductionItem::ScoutPatrol;
    }

    if signals.infrastructure_pressure
        && yields.nutrients > yields.minerals
        && signals.military_pressure == 0
    {
        return crate::ProductionItem::Former;
    }

    if psi_pressure_near_base(state, base.x, base.y, owner) >= 2
        && state.is_production_available(owner, crate::ProductionItem::PsiSentinel)
    {
        return crate::ProductionItem::PsiSentinel;
    }

    if signals.military_pressure >= 1
        && state.is_production_available(owner, crate::ProductionItem::ProbeTeam)
        && state
            .units
            .iter()
            .filter(|u| u.alive && u.owner == owner && u.kind == crate::UnitKind::ProbeTeam)
            .count()
            < 2
    {
        return crate::ProductionItem::ProbeTeam;
    }

    if signals.support_pressure {
        if let Some(relief_item) =
            choose_ai_support_relief_fallback(state, base, owner, yields, trade_links)
        {
            return relief_item;
        }
    }

    preferred
}

fn choose_ai_queue_follow_up(
    state: &GameState,
    base_id: usize,
    owner: usize,
) -> crate::ProductionItem {
    let Some(base) = state.base(base_id) else {
        return crate::ProductionItem::ScoutPatrol;
    };
    let yields = state
        .operational_base_yields(base.id)
        .unwrap_or_else(|| state.base_yields(base.x, base.y));
    let pressure = military_pressure_near_base(state, base.x, base.y, owner);
    let psi_pressure = state.base_local_psi_pressure(base.id);
    let prefer_raider = should_prefer_raider_speeder(state, owner, base.x, base.y);
    let trade_links = state.base_potential_trade_links(base.id);
    let support_pressure = state.faction_support_summary(owner).supported_units > 0;
    let convoy_pressure = trade_links >= 1
        && ((pressure >= 1 || psi_pressure >= 1)
            || state
                .convoy_route_status_for_base(base.id)
                .into_iter()
                .any(|(_, _, disrupted, intercepted, _)| disrupted || intercepted));

    if let Some(recovery_item) = choose_ai_recovery_production(state, base, owner, pressure) {
        return recovery_item;
    }

    if let Some(support_item) =
        choose_ai_support_production(state, base, owner, yields, trade_links)
    {
        return support_item;
    }

    if pressure >= 2 {
        if content::ai_attack_bias(owner) >= 1
            && state.is_production_available(owner, crate::ProductionItem::RaiderSpeeder)
            && prefer_raider
        {
            return crate::ProductionItem::RaiderSpeeder;
        }
        if state.is_production_available(owner, crate::ProductionItem::Speeder) {
            return crate::ProductionItem::Speeder;
        }
        return crate::ProductionItem::ScoutPatrol;
    }
    if !base.facilities.contains(&crate::Facility::NetworkNode)
        && state.is_production_available(owner, crate::ProductionItem::NetworkNode)
        && yields.energy >= yields.minerals
    {
        return crate::ProductionItem::NetworkNode;
    }
    if !base.facilities.contains(&crate::Facility::TradeExchange)
        && state.is_production_available(owner, crate::ProductionItem::TradeExchange)
        && trade_links >= 1
        && yields.energy >= yields.minerals
    {
        return crate::ProductionItem::TradeExchange;
    }
    if state.base_convoy_escort_count(base.id) == 0
        && state.is_production_available(owner, crate::ProductionItem::EscortSpeeder)
        && convoy_pressure
    {
        return crate::ProductionItem::EscortSpeeder;
    }
    if !base.facilities.contains(&crate::Facility::PatrolGrid)
        && state.is_production_available(owner, crate::ProductionItem::PatrolGrid)
        && trade_links >= 1
        && (pressure >= 1 || psi_pressure >= 1)
    {
        return crate::ProductionItem::PatrolGrid;
    }
    if !base.facilities.contains(&crate::Facility::FreightDepot)
        && state.is_production_available(owner, crate::ProductionItem::FreightDepot)
        && trade_links >= 1
        && (base.facilities.contains(&crate::Facility::TradeExchange)
            || yields.minerals <= yields.energy)
    {
        return crate::ProductionItem::FreightDepot;
    }
    if !base.facilities.contains(&crate::Facility::CommandCenter)
        && state.is_production_available(owner, crate::ProductionItem::CommandCenter)
        && yields.minerals >= yields.nutrients
    {
        return crate::ProductionItem::CommandCenter;
    }
    if !base.facilities.contains(&crate::Facility::SensorArray)
        && state.is_production_available(owner, crate::ProductionItem::SensorArray)
        && pressure >= 1
        && yields.energy >= yields.nutrients
    {
        return crate::ProductionItem::SensorArray;
    }
    if !base.facilities.contains(&crate::Facility::ForwardDepot)
        && base.facilities.contains(&crate::Facility::TransitHub)
        && state.is_production_available(owner, crate::ProductionItem::ForwardDepot)
        && pressure >= 1
        && yields.energy > yields.minerals
        && yields.minerals + yields.energy >= 7
    {
        return crate::ProductionItem::ForwardDepot;
    }
    if !base.facilities.contains(&crate::Facility::MilitaryAcademy)
        && base.facilities.contains(&crate::Facility::CommandCenter)
        && state.is_production_available(owner, crate::ProductionItem::MilitaryAcademy)
        && pressure >= 1
    {
        return crate::ProductionItem::MilitaryAcademy;
    }
    if !base.facilities.contains(&crate::Facility::PsiBeacon)
        && state.is_production_available(owner, crate::ProductionItem::PsiBeacon)
        && psi_pressure >= 1
        && yields.energy >= yields.nutrients
    {
        return crate::ProductionItem::PsiBeacon;
    }
    if !base.facilities.contains(&crate::Facility::RecyclingTanks)
        && state.is_production_available(owner, crate::ProductionItem::RecyclingTanks)
    {
        return crate::ProductionItem::RecyclingTanks;
    }

    if pressure >= 1
        && state.is_production_available(owner, crate::ProductionItem::ProbeTeam)
        && state
            .units
            .iter()
            .filter(|u| u.alive && u.owner == owner && u.kind == crate::UnitKind::ProbeTeam)
            .count()
            < 2
    {
        return crate::ProductionItem::ProbeTeam;
    }
    if !base
        .facilities
        .contains(&crate::Facility::RecreationCommons)
        && state.is_production_available(owner, crate::ProductionItem::RecreationCommons)
        && yields.nutrients >= yields.energy
    {
        return crate::ProductionItem::RecreationCommons;
    }
    if !base.facilities.contains(&crate::Facility::Greenhouse)
        && state.is_production_available(owner, crate::ProductionItem::Greenhouse)
        && yields.nutrients <= yields.minerals
    {
        return crate::ProductionItem::Greenhouse;
    }
    if !base.facilities.contains(&crate::Facility::MineralRefinery)
        && state.is_production_available(owner, crate::ProductionItem::MineralRefinery)
        && yields.minerals <= yields.energy
    {
        return crate::ProductionItem::MineralRefinery;
    }
    if pressure >= 1 && state.is_production_available(owner, crate::ProductionItem::GarrisonGuard) {
        return crate::ProductionItem::GarrisonGuard;
    }
    if support_pressure {
        if let Some(relief_item) =
            choose_ai_support_relief_fallback(state, base, owner, yields, trade_links)
        {
            return relief_item;
        }
    }
    if yields.nutrients >= yields.minerals {
        crate::ProductionItem::Former
    } else {
        crate::ProductionItem::ScoutPatrol
    }
}

fn should_prefer_raider_speeder(state: &GameState, owner: usize, x: usize, y: usize) -> bool {
    if content::ai_attack_bias(owner) < 1 {
        return false;
    }

    let open_terrain_tiles = (-1isize..=1)
        .flat_map(|dy| (-1isize..=1).map(move |dx| (dx, dy)))
        .filter_map(|(dx, dy)| {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 {
                return None;
            }
            state
                .tile(nx as usize, ny as usize)
                .map(|tile| tile.terrain)
        })
        .filter(|terrain| matches!(terrain, Terrain::Flat | Terrain::Rolling))
        .count();

    let enemy_base_distance = state
        .bases
        .iter()
        .filter(|base| base.owner != owner)
        .map(|base| manhattan(x, y, base.x, base.y))
        .min();
    let enemy_unit_distance = state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner != owner)
        .map(|unit| manhattan(x, y, unit.x, unit.y))
        .min();

    enemy_base_distance.map(|d| d <= 6).unwrap_or(false)
        || open_terrain_tiles >= 5 && enemy_unit_distance.map(|d| d <= 4).unwrap_or(false)
}

fn economy_signals_for_base(
    state: &GameState,
    owner: usize,
    base_id: usize,
    x: usize,
    y: usize,
    yields: Yields,
) -> AiEconomySignals {
    let preferred = content::ai_preferred_production(owner);
    let expansion_target = desired_ai_expansion_target(state, owner);
    let owned_bases = state.bases_for(owner).len();
    let Some(faction) = state.faction(owner) else {
        return AiEconomySignals {
            infrastructure_pressure: preferred == crate::ProductionItem::Former,
            ..AiEconomySignals::default()
        };
    };
    let research_cost = content::tech_cost(faction.current_research);
    let research_gap = (research_cost - faction.research).max(0);
    let terraform_bias = content::ai_terraform_bias(owner);
    let unrest = state.base_unrest(base_id);
    let food_margin = state
        .base(base_id)
        .map(|base| yields.nutrients - base.population.max(1))
        .unwrap_or_default();
    let support = state.faction_support_summary(owner);
    let minerals_stock = state.base(base_id).map(|b| b.minerals_stock).unwrap_or(0);
    let local_military_pressure = frontline_military_pressure_near_base(state, x, y, owner);

    AiEconomySignals {
        expansion_pressure: owned_bases < expansion_target
            && support.supported_units <= owned_bases as i32
            && (food_margin >= 0
                || (owned_bases < 5 && food_margin >= -2)
                || (owned_bases == 1
                    && state.base(base_id).map(|b| b.population).unwrap_or(0) > 5)),
        military_pressure: local_military_pressure,
        infrastructure_pressure: unrest > 0
            || terraform_bias >= 4 && yields.energy >= 8
            || terraform_bias >= 5 && yields.nutrients > yields.minerals,
        energy_pressure: faction.energy <= 10
            || research_gap >= yields.energy.max(1) * 10
            || faction.energy < 0,
        mineral_pressure: support.unit_upkeep > (owned_bases as i32 * 2) || minerals_stock <= 2,
        support_pressure: support.supported_units > 0,
        support_deficit: support.supported_units,
    }
}

pub(crate) fn military_pressure_near_base(
    state: &GameState,
    x: usize,
    y: usize,
    owner: usize,
) -> i32 {
    let mut pressure = 0;

    for unit in state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner != owner)
    {
        let distance = manhattan(x, y, unit.x, unit.y);
        if distance <= 3 {
            pressure += 2;
        } else if distance <= 6 {
            pressure += 1;
        }
    }

    for base in state.bases.iter().filter(|base| base.owner != owner) {
        let distance = manhattan(x, y, base.x, base.y);
        if distance <= 4 {
            pressure += 1;
        }
    }

    let own_units = state.live_units_for(owner).len();
    let enemy_units = state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner != owner)
        .count();
    if enemy_units > 0 && own_units <= enemy_units / 2 {
        pressure += 1;
    }

    pressure
}

fn frontline_military_pressure_near_base(
    state: &GameState,
    x: usize,
    y: usize,
    owner: usize,
) -> i32 {
    let mut pressure = 0;

    for unit in state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner != owner)
    {
        if matches!(
            unit.kind,
            UnitKind::MindWorm
                | UnitKind::TranceScout
                | UnitKind::PsiSentinel
                | UnitKind::ColonyPod
                | UnitKind::Former
        ) {
            continue;
        }
        let distance = manhattan(x, y, unit.x, unit.y);
        if distance <= 3 {
            pressure += 2;
        } else if distance <= 6 {
            pressure += 1;
        }
    }

    for base in state.bases.iter().filter(|base| base.owner != owner) {
        let distance = manhattan(x, y, base.x, base.y);
        if distance <= 4 {
            pressure += 1;
        }
    }

    pressure
}

pub(crate) fn psi_pressure_near_base(state: &GameState, x: usize, y: usize, owner: usize) -> i32 {
    let mut pressure = 0;

    for unit in state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner != owner)
    {
        let is_psi_threat = matches!(
            unit.kind,
            UnitKind::MindWorm | UnitKind::TranceScout | UnitKind::PsiSentinel
        );
        if !is_psi_threat {
            continue;
        }

        let distance = manhattan(x, y, unit.x, unit.y);
        if distance <= 3 {
            pressure += 2;
        } else if distance <= 6 {
            pressure += 1;
        }
    }
    pressure
}

pub fn run_ai_tactics(state: &mut GameState) {
    run_ai_tactics_for_owner(state, state.ai_owner());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AiObjective {
    Assemble(usize, usize),
    AssembleNaval(usize, usize),
    AttackBase(usize),
    DefendBase(usize),
    SupportColony(usize),
    BoardTransport(usize),
    NavalInvasion(usize),
    LaunchPlanetBuster(usize, usize),
}

struct AiBattleGroup {
    objective: AiObjective,
    unit_ids: Vec<usize>,
}

fn run_ai_tactics_for_owner(state: &mut GameState, owner: usize) {
    let mut battle_groups: Vec<AiBattleGroup> = Vec::new();

    // 1. Identify all available combat units (excluding those in cargo)
    let cargo_ids: std::collections::HashSet<usize> = state
        .units
        .iter()
        .flat_map(|u| u.cargo_unit_ids.iter().cloned())
        .collect();

    let mut combat_unit_ids: Vec<usize> = state
        .units
        .iter()
        .filter(|u| {
            u.alive
                && u.owner == owner
                && !cargo_ids.contains(&u.id)
                && !matches!(
                    u.kind,
                    UnitKind::ColonyPod | UnitKind::Former | UnitKind::ProbeTeam
                )
                && !state.unit_has_ability(u.id, crate::Ability::Probe)
        })
        .map(|u| u.id)
        .collect();

    // 2. Assign objectives to combat units
    // For now, let's keep it simple: any combat unit not in a group will try to find one or start one

    // a. Check if we need to defend any bases
    for base in state.bases_for(owner) {
        if state.base_local_military_pressure(base.id) >= 2 {
            let mut group_units = Vec::new();
            combat_unit_ids.retain(|&id| {
                let unit = &state.units[id];
                if state.distance(unit.x, unit.y, base.x, base.y) <= 6 {
                    group_units.push(id);
                    false
                } else {
                    true
                }
            });
            if !group_units.is_empty() {
                battle_groups.push(AiBattleGroup {
                    objective: AiObjective::DefendBase(base.id),
                    unit_ids: group_units,
                });
            }
        }
    }

    // Keep one credible defender on each owned base so high-aggression factions
    // do not walk every garrison into attack groups and leave their capital open.
    let reserved_defender_ids = reserve_base_defender_ids(state, owner, &combat_unit_ids);
    combat_unit_ids.retain(|id| !reserved_defender_ids.contains(id));

    // b. Check if we can launch an attack
    let signals = tactical_signals_for_owner(state, owner);
    if signals.attack_bias >= 5 && combat_unit_ids.len() >= 3 {
        let rival = rival_owner(state, owner);
        if let Some(target_base) = state.bases.iter().find(|b| b.owner == rival) {
            let mut group_units = Vec::new();
            combat_unit_ids.retain(|&id| {
                group_units.push(id);
                false
            });
            battle_groups.push(AiBattleGroup {
                objective: AiObjective::AttackBase(target_base.id),
                unit_ids: group_units,
            });
        }
    }

    // d. Naval Invasion Logic
    let transport_ids: Vec<usize> = combat_unit_ids
        .iter()
        .filter(|&&id| state.unit_has_ability(id, Ability::Transport))
        .cloned()
        .collect();

    let mut escortable_transports = Vec::new();

    for transport_id in transport_ids.clone() {
        combat_unit_ids.retain(|&id| id != transport_id);

        let transport = &state.units[transport_id];
        // If transport is empty, try to fill it
        if transport.cargo_unit_ids.is_empty() {
            let mut boarders = Vec::new();
            combat_unit_ids.retain(|&id| {
                if boarders.len() < 2 {
                    boarders.push(id);
                    false
                } else {
                    true
                }
            });
            if !boarders.is_empty() {
                battle_groups.push(AiBattleGroup {
                    objective: AiObjective::BoardTransport(transport_id),
                    unit_ids: boarders,
                });
            }
        }

        // If transport has cargo, find a naval invasion target
        if !transport.cargo_unit_ids.is_empty() {
            let rival = rival_owner(state, owner);
            if let Some(target_base) = state.bases.iter().find(|b| b.owner == rival) {
                battle_groups.push(AiBattleGroup {
                    objective: AiObjective::NavalInvasion(target_base.id),
                    unit_ids: vec![transport_id],
                });
                escortable_transports.push(transport_id);
            }
        } else {
            // Empty transport: go to assembly
            if let Some(hq) = state.bases_for(owner).first() {
                battle_groups.push(AiBattleGroup {
                    objective: AiObjective::AssembleNaval(hq.x, hq.y),
                    unit_ids: vec![transport_id],
                });
            }
        }
    }

    // Assign escorts to transports on mission
    let sea_combat_ids: Vec<usize> = combat_unit_ids
        .iter()
        .filter(|&&id| state.unit_is_sea_unit(id))
        .cloned()
        .collect();

    for sea_id in sea_combat_ids {
        if let Some(&target_transport_id) = escortable_transports.first() {
            combat_unit_ids.retain(|&id| id != sea_id);
            battle_groups.push(AiBattleGroup {
                objective: AiObjective::SupportColony(target_transport_id), // Reuse SupportColony for generic escort
                unit_ids: vec![sea_id],
            });
        }
    }

    // e. Escort vulnerable units (Formers, Colony Pods)
    let vulnerable_ids: Vec<usize> = state
        .units
        .iter()
        .filter(|u| {
            u.alive && u.owner == owner && matches!(u.kind, UnitKind::ColonyPod | UnitKind::Former)
        })
        .map(|u| u.id)
        .collect();

    for vuln_id in vulnerable_ids {
        if combat_unit_ids.is_empty() {
            break;
        }
        let mut group_units = Vec::new();
        // Assign 1 escort per vulnerable unit if available
        if let Some(&escort_id) = combat_unit_ids.first() {
            group_units.push(escort_id);
            combat_unit_ids.remove(0);
            battle_groups.push(AiBattleGroup {
                objective: AiObjective::SupportColony(vuln_id),
                unit_ids: group_units,
            });
        }
    }

    // f. Drop Pod 'Shock Group' Logic
    let drop_pod_ids: Vec<usize> = combat_unit_ids
        .iter()
        .filter(|&&id| state.unit_has_ability(id, Ability::DropPod))
        .cloned()
        .collect();

    if !drop_pod_ids.is_empty() {
        let rival = rival_owner(state, owner);
        if let Some(target_base) = state.bases.iter().find(|b| b.owner == rival) {
            combat_unit_ids.retain(|&id| !drop_pod_ids.contains(&id));
            battle_groups.push(AiBattleGroup {
                objective: AiObjective::Assemble(target_base.x, target_base.y), // Direct teleport target
                unit_ids: drop_pod_ids,
            });
        }
    }

    // g. Planet Buster 'Vaporization Group' Logic
    let pb_ids: Vec<usize> = combat_unit_ids
        .iter()
        .filter(|&&id| matches!(state.unit_weapon(id), Some(Weapon::PlanetBuster(_))))
        .cloned()
        .collect();

    for pb_id in pb_ids {
        let Some(pb_unit) = state.unit(pb_id) else {
            continue;
        };
        if let Some((tx, ty)) = choose_ai_planet_buster_target(state, pb_unit) {
            combat_unit_ids.retain(|&id| id != pb_id);
            battle_groups.push(AiBattleGroup {
                objective: AiObjective::LaunchPlanetBuster(tx, ty),
                unit_ids: vec![pb_id],
            });
        }
    }

    // h. Remaining units Assemble at a border base
    if !combat_unit_ids.is_empty() {
        let rival = rival_owner(state, owner);
        let owned_bases = state.bases_for(owner);
        let frontier_base = owned_bases.iter().min_by_key(|b| {
            state
                .bases
                .iter()
                .filter(|rb| rb.owner == rival)
                .map(|rb| state.distance(b.x, b.y, rb.x, rb.y))
                .min()
                .unwrap_or(99)
        });

        if let Some(target) = frontier_base {
            battle_groups.push(AiBattleGroup {
                objective: AiObjective::Assemble(target.x, target.y),
                unit_ids: combat_unit_ids,
            });
        }
    }

    // 3. Process Battle Groups
    for group in &battle_groups {
        let (tx, ty) = match group.objective {
            AiObjective::Assemble(ax, ay) | AiObjective::AssembleNaval(ax, ay) => (ax, ay),
            AiObjective::AttackBase(base_id) | AiObjective::NavalInvasion(base_id) => {
                let base = &state.bases[base_id];
                (base.x, base.y)
            }
            AiObjective::DefendBase(base_id) => {
                let base = &state.bases[base_id];
                (base.x, base.y)
            }
            AiObjective::SupportColony(unit_id) | AiObjective::BoardTransport(unit_id) => {
                let unit = &state.units[unit_id];
                (unit.x, unit.y)
            }
            AiObjective::LaunchPlanetBuster(tx, ty) => (tx, ty),
        };

        let is_attacking = matches!(group.objective, AiObjective::AttackBase(_));
        let group_size = group.unit_ids.len();

        for &unit_id in &group.unit_ids {
            let unit = state.unit(unit_id).cloned().unwrap();
            if unit.moves_left > 0 {
                // Boarding logic
                if let AiObjective::BoardTransport(transport_id) = group.objective {
                    if unit.x == tx && unit.y == ty {
                        let _ = state.apply_action(GameAction::LoadUnit {
                            unit_id,
                            transport_id,
                        });
                        continue;
                    }
                }

                // Planet Buster logic
                if let AiObjective::LaunchPlanetBuster(tx, ty) = group.objective {
                    if state.distance(unit.x, unit.y, tx, ty) <= 1 {
                        let _ = state.apply_action(GameAction::MoveUnit {
                            unit_id,
                            target_x: tx,
                            target_y: ty,
                        });
                        continue;
                    }
                }

                // Unloading logic
                if let AiObjective::NavalInvasion(_) = group.objective {
                    if !unit.cargo_unit_ids.is_empty()
                        && state.distance(unit.x, unit.y, tx, ty) <= 1
                    {
                        let cargo_ids = unit.cargo_unit_ids.clone();
                        for cid in cargo_ids {
                            let _ = state.apply_action(GameAction::UnloadUnit {
                                unit_id: cid,
                                transport_id: unit_id,
                            });
                        }
                        continue;
                    }
                }

                // Air Patrol logic
                if state.unit_is_aircraft(unit.id)
                    && state.tile(unit.x, unit.y).and_then(|t| t.base).is_some()
                    && unit.moves_left == 1
                {
                    let _ = state.apply_action(GameAction::SetUnitActivity {
                        unit_id: unit.id,
                        activity: crate::model::UnitActivity::Patrol,
                    });
                    continue;
                }

                // Check if unit should retreat even if in a group
                if should_ai_unit_retreat(state, &unit) {
                    if try_ai_retreat(state, &unit) {
                        continue;
                    }
                }

                // If attacking, only advance if we have a sufficient force (at least 3 units)
                // Or if we are already very close to the target
                let dist = state.distance(unit.x, unit.y, tx, ty);
                if is_attacking && group_size < 3 && dist <= 4 && dist > 1 {
                    // Stage nearby instead of charging in solo
                    continue;
                }

                // If defending and already at base, stay put
                if matches!(group.objective, AiObjective::DefendBase(_))
                    && unit.x == tx
                    && unit.y == ty
                {
                    continue;
                }

                let _ = try_ai_move_toward(state, unit.id, unit.x, unit.y, tx, ty);
            }
        }
    }

    // 4. Process non-combat/remaining units individually
    let remaining_units: Vec<usize> = state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == owner)
        .map(|u| u.id)
        .collect();

    let grouped_ids: std::collections::HashSet<usize> = battle_groups
        .iter()
        .flat_map(|g| g.unit_ids.iter().cloned())
        .collect();

    for unit_id in remaining_units {
        if grouped_ids.contains(&unit_id) {
            continue;
        }

        let Some(unit) = state.unit(unit_id).cloned() else {
            continue;
        };

        if unit.moves_left <= 0 {
            continue;
        }

        if should_ai_unit_retreat(state, &unit) {
            if try_ai_retreat(state, &unit) {
                continue;
            }
        }

        if reserved_defender_ids.contains(&unit.id) && is_unit_on_friendly_base(state, &unit) {
            continue;
        }

        if unit.kind == UnitKind::Former && try_ai_terraform(state, unit_id) {
            continue;
        }

        if unit.kind == UnitKind::EscortSpeeder && try_ai_patrol_convoys(state, &unit) {
            continue;
        }

        if (unit.kind == UnitKind::ProbeTeam
            || state.unit_has_ability(unit.id, crate::Ability::Probe))
            && try_ai_probe_action(state, &unit)
        {
            continue;
        }

        if unit.kind == UnitKind::ColonyPod {
            let base_count = state.bases_for(owner).len() as i32;
            let limit = state.base_expansion_limit(owner);
            let expansion_target = desired_ai_expansion_target(state, owner) as i32;

            // Only found base if under both the content-defined target and the bureaucracy limit
            // Or if we are very early and need a foothold
            if base_count < expansion_target && (base_count < limit || base_count < 2) {
                if is_ai_colony_site_acceptable(state, owner, unit.x, unit.y)
                    && state
                        .apply_action(GameAction::FoundBase { unit_id })
                        .is_ok()
                {
                    continue;
                }
                if let Some((tx, ty)) = choose_ai_colony_target(state, &unit) {
                    if try_ai_move_toward(state, unit.id, unit.x, unit.y, tx, ty) {
                        continue;
                    }
                }
            }
        }

        if let Some((tx, ty)) = choose_ai_target_for_unit(state, &unit) {
            let _ = try_ai_move_toward(state, unit.id, unit.x, unit.y, tx, ty);
        }
    }
}

fn reserve_base_defender_ids(
    state: &GameState,
    owner: usize,
    candidate_unit_ids: &[usize],
) -> std::collections::HashSet<usize> {
    let candidate_set: std::collections::HashSet<usize> =
        candidate_unit_ids.iter().copied().collect();
    let mut reserved = std::collections::HashSet::new();

    for base in state.bases_for(owner) {
        let Some(best_defender) = state
            .units
            .iter()
            .filter(|unit| {
                unit.alive
                    && unit.owner == owner
                    && unit.x == base.x
                    && unit.y == base.y
                    && candidate_set.contains(&unit.id)
            })
            .max_by_key(|unit| {
                (
                    !state.unit_is_aircraft(unit.id),
                    !state.unit_has_ability(unit.id, Ability::Transport),
                    state.unit_defense_strength(unit.id),
                    unit.hp,
                    unit.experience,
                )
            })
        else {
            continue;
        };

        reserved.insert(best_defender.id);
    }

    reserved
}

fn live_colony_pod_count(state: &GameState, owner: usize) -> usize {
    state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner == owner && unit.kind == UnitKind::ColonyPod)
        .count()
}

fn live_former_count(state: &GameState, owner: usize) -> usize {
    state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner == owner && unit.kind == UnitKind::Former)
        .count()
}

fn desired_ai_expansion_target(state: &GameState, owner: usize) -> usize {
    let configured_target = content::ai_expansion_base_target(owner);
    let map_scaled_target = ((state.width * state.height) / 25).clamp(5, 20);
    let bureaucracy_limit = state.base_expansion_limit(owner).max(2) as usize;

    configured_target
        .max(map_scaled_target)
        .min(bureaucracy_limit)
}

fn desired_ai_base_spacing(state: &GameState) -> i32 {
    ((state.width.min(state.height) as i32) / 6).clamp(2, 4)
}

fn is_ai_colony_site_acceptable(state: &GameState, owner: usize, x: usize, y: usize) -> bool {
    let Some(tile) = state.tile(x, y) else {
        return false;
    };
    if !tile.terrain.is_land() || tile.base.is_some() {
        return false;
    }

    let nearest_friendly_base = state
        .bases_for(owner)
        .iter()
        .map(|base| manhattan(x, y, base.x, base.y))
        .min();

    let minimum_spacing = if state.bases_for(owner).len() < 4 {
        2
    } else {
        desired_ai_base_spacing(state)
    };

    nearest_friendly_base
        .map(|distance| distance as i32 >= minimum_spacing)
        .unwrap_or(true)
}

fn choose_ai_planet_buster_target(state: &GameState, unit: &crate::Unit) -> Option<(usize, usize)> {
    let rival = rival_owner(state, unit.owner);
    let mut best_target = None;
    let mut best_score = 0;

    for base in state.bases.iter().filter(|b| b.owner == rival) {
        let dist = state.distance(unit.x, unit.y, base.x, base.y);
        if dist > 20 {
            continue;
        }

        // Score based on population and facilities
        let score = (base.population as i32 * 10) + (base.facilities.len() as i32 * 5);
        if score > best_score {
            best_score = score;
            best_target = Some((base.x, base.y));
        }
    }

    best_target
}

fn choose_ai_colony_target(state: &GameState, unit: &crate::Unit) -> Option<(usize, usize)> {
    let owner = unit.owner;
    let friendly_bases = state.bases_for(owner);
    let minimum_spacing = if friendly_bases.len() < 4 {
        2
    } else {
        desired_ai_base_spacing(state)
    };
    let mut best_spaced: Option<(usize, usize, i32)> = None;
    let mut best_relaxed: Option<(usize, usize, i32)> = None;

    for y in 0..state.height {
        for x in 0..state.width {
            let Some(tile) = state.tile(x, y) else {
                continue;
            };

            if !tile.terrain.is_land() || tile.base.is_some() {
                continue;
            }

            if let Some(other_unit_id) = tile.unit {
                if other_unit_id != unit.id {
                    continue;
                }
            }

            let nearest_friendly_base = friendly_bases
                .iter()
                .map(|base| manhattan(x, y, base.x, base.y))
                .min()
                .unwrap_or(0) as i32;
            let nearest_enemy_base = state
                .bases
                .iter()
                .filter(|base| base.owner != unit.owner)
                .map(|base| manhattan(x, y, base.x, base.y))
                .min()
                .unwrap_or(99) as i32;
            let site_yields = state.base_yields(x, y);
            let distance = manhattan(unit.x, unit.y, x, y) as i32;

            let spacing_score = match nearest_friendly_base {
                0 | 1 => -30,
                2 => -18,
                3 => 0,
                4..=7 => 10,
                _ => 4,
            };
            let threat_penalty = if nearest_enemy_base <= 2 {
                8
            } else if nearest_enemy_base <= 4 {
                3
            } else {
                0
            };
            let yield_score =
                site_yields.nutrients * 4 + site_yields.minerals * 3 + site_yields.energy * 2;
            let score = yield_score + spacing_score - threat_penalty - distance;

            if best_relaxed
                .map(|(_, _, best_score)| score > best_score)
                .unwrap_or(true)
            {
                best_relaxed = Some((x, y, score));
            }

            if nearest_friendly_base >= minimum_spacing
                && best_spaced
                    .map(|(_, _, best_score)| score > best_score)
                    .unwrap_or(true)
            {
                best_spaced = Some((x, y, score));
            }
        }
    }

    best_spaced.or(best_relaxed).map(|(x, y, _)| (x, y))
}

fn try_ai_move_toward(
    state: &mut GameState,
    unit_id: usize,
    from_x: usize,
    from_y: usize,
    target_x: usize,
    target_y: usize,
) -> bool {
    let current_distance = manhattan(from_x, from_y, target_x, target_y);
    let step_x = step_toward(from_x, target_x);
    let step_y = step_toward(from_y, target_y);

    let nx = (from_x as isize + step_x).clamp(0, state.width.saturating_sub(1) as isize) as usize;
    let ny = (from_y as isize + step_y).clamp(0, state.height.saturating_sub(1) as isize) as usize;

    if nx == from_x && ny == from_y {
        return false;
    }

    if state
        .apply_action(GameAction::MoveUnit {
            unit_id,
            target_x: nx,
            target_y: ny,
        })
        .is_ok()
    {
        return true;
    }

    let mut fallback_steps = Vec::new();
    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let alt_x = from_x as isize + dx;
            let alt_y = from_y as isize + dy;
            if alt_x < 0
                || alt_y < 0
                || alt_x >= state.width as isize
                || alt_y >= state.height as isize
            {
                continue;
            }

            let alt_x = alt_x as usize;
            let alt_y = alt_y as usize;
            if alt_x == nx && alt_y == ny {
                continue;
            }

            let distance = manhattan(alt_x, alt_y, target_x, target_y);
            if distance > current_distance {
                continue;
            }

            let alignment_penalty = (step_x != 0 && step_toward(from_x, alt_x) != step_x) as usize
                + (step_y != 0 && step_toward(from_y, alt_y) != step_y) as usize;
            fallback_steps.push((distance, alignment_penalty, alt_x, alt_y));
        }
    }

    fallback_steps.sort_unstable();
    for (_, _, alt_x, alt_y) in fallback_steps {
        if state
            .apply_action(GameAction::MoveUnit {
                unit_id,
                target_x: alt_x,
                target_y: alt_y,
            })
            .is_ok()
        {
            return true;
        }
    }

    false
}

fn should_ai_unit_retreat(state: &GameState, unit: &crate::Unit) -> bool {
    if unit.kind == UnitKind::ColonyPod || unit.kind == UnitKind::Former {
        return false;
    }

    let max_hp = content::unit_base_hp(unit.kind.clone());
    if unit.hp >= (max_hp * 70) / 100 {
        return false;
    }

    let pressure = military_pressure_near_base(state, unit.x, unit.y, unit.owner);
    pressure > 0 || !is_unit_on_friendly_base(state, unit)
}

fn try_ai_retreat(state: &mut GameState, unit: &crate::Unit) -> bool {
    if is_unit_on_friendly_base(state, unit) {
        return true;
    }

    if let Some((tx, ty)) = nearest_friendly_base(state, unit.owner, unit.x, unit.y) {
        if let Some((nx, ny)) = safest_retreat_step(state, unit, tx, ty) {
            let _ = state.apply_action(GameAction::MoveUnit {
                unit_id: unit.id,
                target_x: nx,
                target_y: ny,
            });
        }
        return true;
    } else {
        // If no friendly base exists, move away from the nearest enemy
        let rival = rival_owner(state, unit.owner);
        let nearest_enemy = state
            .units
            .iter()
            .filter(|u| u.alive && u.owner == rival)
            .map(|u| (u.x, u.y, state.distance(unit.x, unit.y, u.x, u.y)))
            .min_by_key(|&(_, _, d)| d);

        if let Some((ex, ey, _)) = nearest_enemy {
            // Find a step that increases distance from (ex, ey)
            let candidates = [
                (unit.x.saturating_add(1), unit.y),
                (unit.x.saturating_sub(1), unit.y),
                (unit.x, unit.y.saturating_add(1)),
                (unit.x, unit.y.saturating_sub(1)),
            ];

            let best_step = candidates
                .into_iter()
                .filter(|&(cx, cy)| cx < state.width && cy < state.height)
                .max_by_key(|&(cx, cy)| state.distance(cx, cy, ex, ey));

            if let Some((nx, ny)) = best_step {
                let _ = state.apply_action(GameAction::MoveUnit {
                    unit_id: unit.id,
                    target_x: nx,
                    target_y: ny,
                });
            }
        }
        return true;
    }
}

fn try_ai_probe_action(state: &mut GameState, unit: &crate::Unit) -> bool {
    let Some((tx, ty)) = choose_ai_probe_target(state, unit) else {
        return false;
    };

    if !crate::GameState::is_adjacent(unit.x, unit.y, tx, ty) && !(unit.x == tx && unit.y == ty) {
        return false;
    }

    let Some(target_tile) = state.tile(tx, ty).cloned() else {
        return false;
    };

    // Determine the best action based on the target
    let action = if let Some(base_id) = target_tile.base {
        let base = &state.bases[base_id];

        // Priority 1: Steal Tech
        let target_techs = &state.factions[base.owner].known_techs;
        let our_techs = &state.factions[unit.owner].known_techs;
        let stealable: Vec<_> = target_techs
            .iter()
            .filter(|t| !our_techs.contains(t))
            .cloned()
            .collect();

        if !stealable.is_empty() {
            crate::model::ProbeAction::StealTech
        } else if !base.facilities.is_empty() {
            crate::model::ProbeAction::SabotageFacility
        } else {
            return false;
        }
    } else if let Some(other_unit_id) = target_tile.unit {
        if state.units[other_unit_id].owner != unit.owner {
            if state.factions[unit.owner].energy >= 50 {
                crate::model::ProbeAction::SubvertUnit
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    };

    state
        .apply_action(GameAction::PerformProbeAction {
            unit_id: unit.id,
            target_x: tx,
            target_y: ty,
            action,
        })
        .is_ok()
}

fn try_ai_terraform(state: &mut GameState, unit_id: usize) -> bool {
    let Some(unit) = state.unit(unit_id).cloned() else {
        return false;
    };

    let terraform_bias = content::ai_terraform_bias(unit.owner);
    if terraform_bias == 0 {
        return false;
    }

    let Some(tile) = state.tile(unit.x, unit.y) else {
        return false;
    };

    // If already has an improvement, don't build another one on top for now
    if tile.improvement.is_some() || !tile.terrain.is_land() {
        return false;
    }

    let faction = match state.faction(unit.owner) {
        Some(f) => f,
        None => return false,
    };

    let known_tech_ids: std::collections::HashSet<String> = faction
        .known_techs
        .iter()
        .map(|t| t.content_id().to_string())
        .collect();

    // Helper to check if an improvement is unlocked
    let is_unlocked = |improvement: crate::Improvement| {
        match improvement {
            crate::Improvement::Farm
            | crate::Improvement::Mine
            | crate::Improvement::Solar
            | crate::Improvement::Road => true,
            crate::Improvement::Forest => known_tech_ids.contains("centauri_ecology"),
            crate::Improvement::ThermalBorehole => known_tech_ids.contains("industrial_base"),
            crate::Improvement::Condenser => {
                // In this implementation, Condenser and Echelon Mirror might be default or tied to higher techs
                // Check if they are in any tech's enables
                true // Default to true if not explicitly blocked in tech tree
            }
            crate::Improvement::EchelonMirror => true,
        }
    };

    let econ = faction.social_engineering.economics;
    let energy_pressure = faction.energy < 20;

    let improvement = if tile.terrain == Terrain::Rocky
        && is_unlocked(crate::Improvement::ThermalBorehole)
        && terraform_bias >= 7
    {
        if state.count_adjacent_improvements(unit.x, unit.y, crate::Improvement::ThermalBorehole)
            == 0
        {
            crate::Improvement::ThermalBorehole
        } else {
            crate::Improvement::Mine
        }
    } else if energy_pressure && is_unlocked(crate::Improvement::Solar) {
        crate::Improvement::Solar
    } else if tile.moisture >= 80
        && terraform_bias >= 8
        && is_unlocked(crate::Improvement::Condenser)
    {
        crate::Improvement::Condenser
    } else if tile.moisture >= 70 && terraform_bias >= 6 && is_unlocked(crate::Improvement::Forest)
    {
        crate::Improvement::Forest
    } else if tile.moisture <= 30
        && terraform_bias >= 7
        && is_unlocked(crate::Improvement::EchelonMirror)
    {
        // Build mirror if adjacent to solar or mirror
        if state.count_adjacent_improvements(unit.x, unit.y, crate::Improvement::Solar) > 0
            || state.count_adjacent_improvements(unit.x, unit.y, crate::Improvement::EchelonMirror)
                > 0
        {
            crate::Improvement::EchelonMirror
        } else {
            crate::Improvement::Solar
        }
    } else if tile.moisture <= 40 {
        crate::Improvement::Solar
    } else if tile.terrain == Terrain::Rocky
        || tile.elevation >= 70
        || econ == crate::model::Economics::Planned
    {
        crate::Improvement::Mine
    } else if tile.moisture >= 50 || econ == crate::model::Economics::Green {
        crate::Improvement::Farm
    } else {
        crate::Improvement::Mine
    };

    state
        .apply_action(GameAction::BuildImprovement {
            unit_id,
            improvement,
        })
        .is_ok()
}

fn try_ai_patrol_convoys(state: &mut GameState, unit: &crate::Unit) -> bool {
    let target_base_id = state
        .convoy_pressure_base_ids(unit.owner)
        .into_iter()
        .next()
        .or_else(|| {
            state
                .bases_for(unit.owner)
                .into_iter()
                .find(|base| state.base_potential_trade_links(base.id) >= 1)
                .map(|base| base.id)
        });

    let Some(base_id) = target_base_id else {
        return false;
    };
    let Some(base) = state.base(base_id).cloned() else {
        return false;
    };

    let nx = (unit.x as isize + step_toward(unit.x, base.x))
        .clamp(0, state.width.saturating_sub(1) as isize) as usize;
    let ny = (unit.y as isize + step_toward(unit.y, base.y))
        .clamp(0, state.height.saturating_sub(1) as isize) as usize;

    if nx == unit.x && ny == unit.y {
        return true;
    }

    state
        .apply_action(GameAction::MoveUnit {
            unit_id: unit.id,
            target_x: nx,
            target_y: ny,
        })
        .is_ok()
}

fn run_native_life_turn(state: &mut GameState) {
    let native_owner = state.native_owner();
    let native_units: Vec<usize> = state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == native_owner)
        .map(|u| u.id)
        .collect();

    for unit_id in native_units {
        let Some(unit) = state.unit(unit_id).cloned() else {
            continue;
        };

        let target = find_nearest_non_native_target(state, unit.x, unit.y);

        if let Some((tx, ty)) = target {
            let step_x = step_toward(unit.x, tx);
            let step_y = step_toward(unit.y, ty);

            let nx = (unit.x as isize + step_x).clamp(0, state.width.saturating_sub(1) as isize)
                as usize;
            let ny = (unit.y as isize + step_y).clamp(0, state.height.saturating_sub(1) as isize)
                as usize;

            if nx != unit.x || ny != unit.y {
                let _ = state.apply_action(crate::GameAction::MoveUnit {
                    unit_id,
                    target_x: nx,
                    target_y: ny,
                });
            }
        }
    }
}

fn find_nearest_non_native_target(state: &GameState, x: usize, y: usize) -> Option<(usize, usize)> {
    let native_owner = state.native_owner();
    let mut best_target = None;
    let mut min_dist = i32::MAX;

    for base in &state.bases {
        if base.owner != native_owner {
            let d = state.distance(x, y, base.x, base.y);
            if d < min_dist {
                min_dist = d;
                best_target = Some((base.x, base.y));
            }
        }
    }

    for unit in &state.units {
        if unit.alive && unit.owner != native_owner {
            let d = state.distance(x, y, unit.x, unit.y);
            if d < min_dist {
                min_dist = d;
                best_target = Some((unit.x, unit.y));
            }
        }
    }

    best_target
}

fn spawn_native_life(state: &mut GameState) {
    if state.turn % crate::content::native_spawn_turn_interval() != 0 {
        return;
    }

    let native_owner = state.native_owner();
    let mut total_toxicity = 0;
    for faction in &state.factions {
        total_toxicity += faction.planet_toxicity;
    }

    // High global toxicity increases the chance and count of native spawns
    let spawn_cap = (1 + total_toxicity / 500).clamp(1, 5) as usize;
    let mut spawned_count = 0;

    for y in 0..state.height {
        for x in 0..state.width {
            if spawned_count >= spawn_cap {
                return;
            }

            let idx = state.tile_index(x, y);
            let tile = &state.tiles[idx];
            if tile.unit.is_some() {
                continue;
            }

            let is_ocean = tile.terrain == Terrain::Ocean;
            let is_fungus = tile.terrain == Terrain::Fungus;

            if !is_fungus && !is_ocean {
                continue;
            }

            let roll = state.sample_noise(
                x as i32,
                y as i32,
                state.turn as u32 + content::ai_native_spawn_noise_salt(),
            ) % 100;

            // Global toxicity lowers the threshold (increases spawn rate)
            let threshold = (crate::content::native_spawn_roll_threshold() as i32
                - total_toxicity / 100)
                .clamp(50, 99) as u32;

            if roll > threshold {
                if is_fungus {
                    state.spawn_unit(native_owner, UnitKind::MindWorm, x, y);
                    state.push_log("Native life is stirring in the fungus.".to_string());
                    spawned_count += 1;
                } else if is_ocean && roll > 95 {
                    // Oceanic spawns are rarer and need higher global toxicity/luck
                    state.spawn_unit(native_owner, UnitKind::IsleOfTheDeep, x, y);
                    state.push_log("A massive shape rises from the depths.".to_string());
                    spawned_count += 1;
                }
            }
        }
    }
}

fn choose_ai_target_for_unit(state: &GameState, unit: &crate::Unit) -> Option<(usize, usize)> {
    if unit.kind == UnitKind::RaiderSpeeder {
        return choose_ai_raider_target_for_owner(state, unit.owner, unit.x, unit.y);
    }
    if unit.kind == UnitKind::Former {
        return choose_ai_former_target(state, unit);
    }
    if unit.kind == UnitKind::ProbeTeam || state.unit_has_ability(unit.id, crate::Ability::Probe) {
        return choose_ai_probe_target(state, unit);
    }
    choose_ai_target_for_owner(state, unit.owner, unit.x, unit.y)
}

fn choose_ai_probe_target(state: &GameState, unit: &crate::Unit) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, i32)> = None;

    // Look for enemy bases or units
    for y in 0..state.height {
        for x in 0..state.width {
            let Some(tile) = state.tile(x, y) else {
                continue;
            };

            let mut is_target = false;
            if let Some(base_id) = tile.base {
                if state.bases[base_id].owner != unit.owner {
                    is_target = true;
                }
            } else if let Some(other_unit_id) = tile.unit {
                if state.units[other_unit_id].owner != unit.owner {
                    is_target = true;
                }
            }

            if is_target {
                let dist = manhattan(unit.x, unit.y, x, y) as i32;
                let score = 100 - dist;
                if best.map(|b| score > b.2).unwrap_or(true) {
                    best = Some((x, y, score));
                }
            }
        }
    }

    best.map(|(tx, ty, _)| (tx, ty))
}

fn choose_ai_former_target(state: &GameState, unit: &crate::Unit) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, i32)> = None;

    // Look for tiles across a larger range (e.g., 10 tiles)
    let range = 10;

    for y in 0..state.height {
        for x in 0..state.width {
            let Some(tile) = state.tile(x, y) else {
                continue;
            };

            // Only consider land tiles without improvements
            if !tile.terrain.is_land() || tile.improvement.is_some() {
                continue;
            }

            // Prefer tiles near AI bases
            let dist_to_base = state
                .bases_for(unit.owner)
                .iter()
                .map(|b| manhattan(x, y, b.x, b.y))
                .min()
                .unwrap_or(99);

            if dist_to_base > 5 {
                continue;
            }

            // Score based on potential
            let mut score = 0;
            if tile.moisture >= 70 {
                score += 5;
            }
            if tile.terrain == Terrain::Rocky {
                score += 5;
            }
            if tile.elevation >= 70 {
                score += 3;
            }

            // Subtract distance from unit to minimize travel
            let dist_from_unit = manhattan(unit.x, unit.y, x, y) as i32;
            if dist_from_unit > range as i32 {
                continue;
            }
            score -= dist_from_unit;
            // Heavily prioritize tiles closer to base centers as they are more likely to be worked
            score -= dist_to_base as i32 * 2;

            // Deterministic tie-breaker to prevent bouncing
            if best
                .map(|(bx, by, b_score)| {
                    score > b_score
                        || (score == b_score && (y * state.width + x) < (by * state.width + bx))
                })
                .unwrap_or(true)
            {
                best = Some((x, y, score));
            }
        }
    }

    if let Some((tx, ty, _)) = best {
        Some((tx, ty))
    } else {
        // Fallback: move toward nearest friendly base if idle
        state
            .bases_for(unit.owner)
            .iter()
            .map(|b| (b.x, b.y, manhattan(unit.x, unit.y, b.x, b.y)))
            .min_by_key(|&(_, _, d)| d)
            .map(|(x, y, _)| (x, y))
    }
}

fn choose_ai_target_for_owner(
    state: &GameState,
    owner: usize,
    x: usize,
    y: usize,
) -> Option<(usize, usize)> {
    let signals = tactical_signals_for_owner(state, owner);
    best_scored_target_for_owner(state, owner, x, y, signals)
        .or_else(|| exploratory_target(state, x, y, signals))
}

#[cfg(test)]
fn choose_ai_raider_target(state: &GameState, x: usize, y: usize) -> Option<(usize, usize)> {
    choose_ai_raider_target_for_owner(state, state.ai_owner(), x, y)
}

fn choose_ai_raider_target_for_owner(
    state: &GameState,
    owner: usize,
    x: usize,
    y: usize,
) -> Option<(usize, usize)> {
    let signals = tactical_signals_for_owner(state, owner);
    let mut best: Option<(usize, usize, i32)> = None;
    let rival_owner = rival_owner(state, owner);

    for unit in state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == rival_owner)
    {
        let mut score = score_player_unit_target(x, y, unit.x, unit.y, signals);
        if matches!(unit.kind, UnitKind::Former | UnitKind::ColonyPod) {
            score -= 4;
        } else if unit.hp < content::unit_base_hp(unit.kind.clone()) {
            score -= 2;
        }
        if best.map(|b| score < b.2).unwrap_or(true) {
            best = Some((unit.x, unit.y, score));
        }
    }

    for base in state.bases.iter().filter(|b| b.owner == rival_owner) {
        let score = score_raider_base_target(state, x, y, base.id, signals);
        if best.map(|b| score < b.2).unwrap_or(true) {
            best = Some((base.x, base.y, score));
        }
    }

    best.map(|(tx, ty, _)| (tx, ty))
        .or_else(|| exploratory_target(state, x, y, signals))
}

fn nearest_friendly_base(
    state: &GameState,
    owner: usize,
    x: usize,
    y: usize,
) -> Option<(usize, usize)> {
    state
        .bases
        .iter()
        .filter(|base| base.owner == owner)
        .min_by_key(|base| manhattan(x, y, base.x, base.y))
        .map(|base| (base.x, base.y))
}

fn safest_retreat_step(
    state: &GameState,
    unit: &crate::Unit,
    target_x: usize,
    target_y: usize,
) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, i32, usize)> = None;
    let preferred_x = step_toward(unit.x, target_x);
    let preferred_y = step_toward(unit.y, target_y);
    let ideal_x =
        (unit.x as isize + preferred_x).clamp(0, state.width.saturating_sub(1) as isize) as usize;
    let ideal_y =
        (unit.y as isize + preferred_y).clamp(0, state.height.saturating_sub(1) as isize) as usize;

    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = unit.x as isize + dx;
            let ny = unit.y as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= state.width || ny >= state.height {
                continue;
            }

            let Some(tile) = state.tile(nx, ny) else {
                continue;
            };
            if !tile.terrain.is_land() {
                continue;
            }

            if let Some(other_unit_id) = tile.unit {
                if other_unit_id != unit.id {
                    continue;
                }
            }

            let threat = retreat_threat_score(state, unit.owner, nx, ny);
            let distance = manhattan(nx, ny, target_x, target_y);
            let ideal_penalty = if nx == ideal_x && ny == ideal_y { 0 } else { 1 };
            let candidate = (nx, ny, threat + ideal_penalty, distance);

            if best
                .map(|(_, _, best_threat, best_distance)| {
                    candidate.2 < best_threat
                        || candidate.2 == best_threat && candidate.3 < best_distance
                })
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
    }

    best.map(|(nx, ny, _, _)| (nx, ny))
}

fn retreat_threat_score(state: &GameState, owner: usize, x: usize, y: usize) -> i32 {
    let rival_owner = rival_owner(state, owner);
    let mut threat = 0;

    for unit in state
        .units
        .iter()
        .filter(|unit| unit.alive && unit.owner == rival_owner)
    {
        let d = state.distance(x, y, unit.x, unit.y);
        if d <= 1 {
            threat += 4;
        } else if d <= 2 {
            threat += 2;
        } else if d <= 3 {
            threat += 1;
        }
    }

    for base in state.bases.iter().filter(|base| base.owner == rival_owner) {
        let distance = manhattan(x, y, base.x, base.y);
        if distance <= 2 {
            threat += 1;
        }
    }

    if let Some(base_id) = state.tile(x, y).and_then(|tile| tile.base) {
        if let Some(base) = state.base(base_id) {
            if base.owner == owner {
                threat -= 2;
            }
        }
    }

    threat
}

fn is_unit_on_friendly_base(state: &GameState, unit: &crate::Unit) -> bool {
    state
        .tile(unit.x, unit.y)
        .and_then(|tile| tile.base)
        .and_then(|base_id| state.base(base_id))
        .map(|base| base.owner == unit.owner)
        .unwrap_or(false)
}

#[cfg(test)]
fn tactical_signals(state: &GameState) -> AiTacticalSignals {
    tactical_signals_for_owner(state, state.ai_owner())
}

fn tactical_signals_for_owner(state: &GameState, owner: usize) -> AiTacticalSignals {
    let mut attack_bias = content::ai_attack_bias(owner) as i32;
    let mut exploration_bias = content::ai_exploration_bias(owner) as i32;

    if let Some(faction) = state.faction(owner) {
        use crate::model::{Politics, Values};
        match faction.social_engineering.politics {
            Politics::Fundamentalist => attack_bias += 2,
            Politics::Democratic => exploration_bias += 1,
            _ => {}
        }
        match faction.social_engineering.values {
            Values::Power => attack_bias += 3,
            Values::Knowledge => exploration_bias += 2,
            _ => {}
        }
    }

    AiTacticalSignals {
        attack_bias,
        exploration_bias,
    }
}

#[cfg(test)]
fn best_scored_target(
    state: &GameState,
    x: usize,
    y: usize,
    signals: AiTacticalSignals,
) -> Option<(usize, usize)> {
    best_scored_target_for_owner(state, state.ai_owner(), x, y, signals)
}

fn best_scored_target_for_owner(
    state: &GameState,
    owner: usize,
    x: usize,
    y: usize,
    signals: AiTacticalSignals,
) -> Option<(usize, usize)> {
    let rival_owner = rival_owner(state, owner);
    let mut best: Option<(usize, usize, i32)> = None;

    for unit in state
        .units
        .iter()
        .filter(|u| u.alive && u.owner == rival_owner)
    {
        let score = score_player_unit_target(x, y, unit.x, unit.y, signals);
        if best.map(|b| score < b.2).unwrap_or(true) {
            best = Some((unit.x, unit.y, score));
        }
    }

    for base in state.bases.iter().filter(|b| b.owner == rival_owner) {
        let score = score_player_base_target(x, y, base.x, base.y, signals);
        if best.map(|b| score < b.2).unwrap_or(true) {
            best = Some((base.x, base.y, score));
        }
    }

    for ey in 0..state.height {
        for ex in 0..state.width {
            if state.tile_explored_by_owner(ex, ey, owner) {
                continue;
            }

            let score = score_unexplored_tile_target(x, y, ex, ey, signals);
            if best.map(|b| score < b.2).unwrap_or(true) {
                best = Some((ex, ey, score));
            }
        }
    }

    best.map(|b| (b.0, b.1))
}

fn rival_owner(state: &GameState, owner: usize) -> usize {
    if owner == state.player_owner() {
        state.ai_owner()
    } else {
        state.player_owner()
    }
}

fn exploratory_target(
    state: &GameState,
    x: usize,
    y: usize,
    signals: AiTacticalSignals,
) -> Option<(usize, usize)> {
    let exploration_bias = signals.exploration_bias as isize;
    if exploration_bias == 0 {
        return None;
    }

    // Use noise to provide varied exploratory directions
    let noise = state.sample_noise(x as i32, y as i32, state.turn as u32);
    let dx = (noise % 11) as isize - 5;
    let dy = ((noise / 11) % 11) as isize - 5;

    let target_x = (x as isize + dx + exploration_bias)
        .clamp(0, state.width.saturating_sub(1) as isize) as usize;
    let target_y = (y as isize + dy + exploration_bias / 2)
        .clamp(0, state.height.saturating_sub(1) as isize) as usize;

    if target_x == x && target_y == y {
        // Fallback: move toward opposite corner of the map to cross the frontier
        let corner_x = if x < state.width / 2 {
            state.width - 1
        } else {
            0
        };
        let corner_y = if y < state.height / 2 {
            state.height - 1
        } else {
            0
        };
        Some((corner_x, corner_y))
    } else {
        Some((target_x, target_y))
    }
}

fn score_player_unit_target(
    from_x: usize,
    from_y: usize,
    target_x: usize,
    target_y: usize,
    signals: AiTacticalSignals,
) -> i32 {
    manhattan(from_x, from_y, target_x, target_y) as i32 - signals.attack_bias - 3
}

fn score_player_base_target(
    from_x: usize,
    from_y: usize,
    target_x: usize,
    target_y: usize,
    signals: AiTacticalSignals,
) -> i32 {
    manhattan(from_x, from_y, target_x, target_y) as i32 - signals.attack_bias
}

fn score_raider_base_target(
    state: &GameState,
    from_x: usize,
    from_y: usize,
    target_base_id: usize,
    signals: AiTacticalSignals,
) -> i32 {
    let Some(base) = state.base(target_base_id) else {
        return i32::MAX / 4;
    };
    let mut score = manhattan(from_x, from_y, base.x, base.y) as i32 - signals.attack_bias - 4;
    let route_count = state.convoy_route_details_for_base(base.id).len() as i32;
    let disrupted_routes = state
        .convoy_route_details_for_base(base.id)
        .into_iter()
        .filter(|(_, _, disrupted)| *disrupted)
        .count() as i32;
    let convoy_security = state.base_convoy_security(base.id);
    if route_count > 0 {
        score -= route_count * 2;
    }
    if disrupted_routes > 0 {
        score -= disrupted_routes * 2;
    }
    score + convoy_security
}

fn score_unexplored_tile_target(
    from_x: usize,
    from_y: usize,
    target_x: usize,
    target_y: usize,
    signals: AiTacticalSignals,
) -> i32 {
    manhattan(from_x, from_y, target_x, target_y) as i32 - signals.exploration_bias
}

#[cfg(test)]
mod tests {
    use super::{
        best_scored_target, choose_ai_colony_target, choose_ai_production_for_base,
        choose_ai_raider_target, desired_ai_base_spacing, desired_ai_expansion_target,
        economy_signals_for_base, exploratory_target, is_ai_colony_site_acceptable,
        maybe_assign_ai_convoy_route, run_ai_economy_for_owner, run_ai_tactics_for_owner,
        score_player_base_target, score_player_unit_target, score_raider_base_target,
        score_unexplored_tile_target, tactical_signals, try_ai_move_toward, update_ai_research,
        update_ai_social_engineering, update_ai_unit_designs, AiTacticalSignals,
    };
    use crate::{
        Base, GameState, GovernorMode, ProductionItem, Tech, Terrain, Unit, UnitActivity, UnitKind,
    };

    #[test]
    fn tactical_target_prefers_nearby_player_unit() {
        let mut game = GameState::new_game(16, 16, 9);
        let ai_owner = game.ai_owner();
        let player_owner = game.player_owner();

        let ai_id = game.units.len();
        game.tiles[5 * game.width + 5].unit = Some(ai_id);
        game.units.push(Unit {
            id: ai_id,
            owner: ai_owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 5,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let player_id = game.units.len();
        game.tiles[6 * game.width + 6].unit = Some(player_id);
        game.units.push(Unit {
            id: player_id,
            owner: player_owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 6,
            y: 6,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let target =
            best_scored_target(&game, 5, 5, tactical_signals(&game)).expect("target should exist");
        assert_eq!(target, (6, 6));
    }

    #[test]
    fn expansion_target_scales_above_minimum_on_medium_map() {
        let game = GameState::new_game(20, 20, 12345);
        // Gaia (Player) has +2 Efficiency -> limit 11, target max(2, 16).min(11) = 11
        assert_eq!(desired_ai_expansion_target(&game, game.player_owner()), 11);
        // Sparta (AI) has 0 Efficiency -> limit 7, target max(3, 16).min(7) = 7
        assert_eq!(desired_ai_expansion_target(&game, game.ai_owner()), 7);
    }

    #[test]
    fn colony_target_prefers_spaced_site_when_map_has_room() {
        let mut game = GameState::new_game(20, 20, 7);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Anchor".to_string(),
            x: 3,
            y: 3,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[3 * game.width + 3].base = Some(0);

        game.units.push(Unit {
            id: 0,
            owner,
            kind: UnitKind::ColonyPod,
            design_index: 0,
            x: 4,
            y: 3,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });
        game.tiles[3 * game.width + 4].unit = Some(100);

        let target = choose_ai_colony_target(&game, &game.units[0]).expect("site should exist");
        let distance_from_anchor = target.0.abs_diff(3) + target.1.abs_diff(3);

        assert!(
            distance_from_anchor as i32 >= desired_ai_base_spacing(&game),
            "target {target:?} should not crowd the existing base"
        );
    }

    #[test]
    fn colony_site_check_rejects_adjacent_founding_tile() {
        let mut game = GameState::new_game(20, 20, 7);
        let owner = game.ai_owner();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.base = None;
            tile.terrain = Terrain::Flat;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Anchor".to_string(),
            x: 3,
            y: 3,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[3 * game.width + 3].base = Some(0);

        assert!(!is_ai_colony_site_acceptable(&game, owner, 4, 3));
        assert!(is_ai_colony_site_acceptable(&game, owner, 7, 3));
    }

    #[test]
    fn colony_pod_moves_off_bad_adjacent_tile_before_founding() {
        let mut game = GameState::new_game(20, 20, 7);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Anchor".to_string(),
            x: 3,
            y: 3,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[3 * game.width + 3].base = Some(0);

        game.units.push(Unit {
            id: 0,
            owner,
            kind: UnitKind::ColonyPod,
            design_index: 0,
            x: 4,
            y: 3,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });
        game.tiles[3 * game.width + 4].unit = Some(0);

        run_ai_tactics_for_owner(&mut game, owner);

        let moved_or_founded = game.bases_for(owner).len() > 1
            || game
                .unit(0)
                .map(|unit| (unit.x, unit.y) != (4, 3))
                .unwrap_or(false);
        assert!(moved_or_founded);
    }

    #[test]
    fn colony_pod_detours_when_direct_settlement_step_is_blocked() {
        let mut game = GameState::new_game(12, 12, 7);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Ocean;
        }

        for (x, y) in [
            (4usize, 4usize),
            (5usize, 4usize),
            (5usize, 3usize),
            (6usize, 3usize),
            (5usize, 2usize),
        ] {
            game.tiles[y * game.width + x].terrain = Terrain::Flat;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Anchor".to_string(),
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
        game.tiles[4 * game.width + 4].base = Some(0);

        game.units.push(Unit {
            id: 0,
            owner,
            kind: UnitKind::ColonyPod,
            design_index: 0,
            x: 5,
            y: 4,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });
        game.tiles[4 * game.width + 5].unit = Some(0);

        game.units.push(Unit {
            id: 1,
            owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 5,
            y: 3,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });
        game.tiles[3 * game.width + 5].unit = Some(1);

        assert!(try_ai_move_toward(&mut game, 0, 5, 4, 5, 2));
        let unit = game.unit(0).expect("colony pod should still exist");
        assert_ne!((unit.x, unit.y), (5, 4));
    }

    #[test]
    fn tactical_scoring_prioritizes_unit_over_base_with_same_distance() {
        let signals = AiTacticalSignals {
            attack_bias: 2,
            exploration_bias: 0,
        };
        let unit_score = score_player_unit_target(5, 5, 7, 5, signals);
        let base_score = score_player_base_target(5, 5, 7, 5, signals);
        assert!(unit_score < base_score);
    }

    #[test]
    fn tactical_target_prefers_closer_base_over_farther_unit() {
        let mut game = GameState::new_game(16, 16, 9);
        let player_owner = game.player_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.explored_by_owner.insert(player_owner);
        }

        game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Nearby Base".to_string(),
            x: 6,
            y: 5,
            population: 1,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[5 * game.width + 6].base = Some(0);

        let player_id = game.units.len();
        game.tiles[10 * game.width + 10].unit = Some(player_id);
        game.units.push(Unit {
            id: player_id,
            owner: player_owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 10,
            y: 10,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let target =
            best_scored_target(&game, 5, 5, tactical_signals(&game)).expect("target should exist");
        assert_eq!(target, (6, 5));
    }

    #[test]
    fn raider_target_prefers_player_base_over_same_distance_combat_unit() {
        let mut game = GameState::new_game(16, 16, 9);
        let player_owner = game.player_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.explored_by_owner.insert(player_owner);
        }

        game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Raid Target".to_string(),
            x: 7,
            y: 5,
            population: 1,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[5 * game.width + 7].base = Some(0);

        let player_id = game.units.len();
        game.tiles[5 * game.width + 3].unit = Some(player_id);
        game.units.push(Unit {
            id: player_id,
            owner: player_owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 3,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let target = choose_ai_raider_target(&game, 5, 5).expect("raider target should exist");
        assert_eq!(target, (7, 5));
    }

    #[test]
    fn raider_target_prefers_exposed_colony_pod() {
        let mut game = GameState::new_game(16, 16, 9);
        let player_owner = game.player_owner();
        game.units.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.explored_by_owner.insert(player_owner);
        }

        let scout_id = game.units.len();
        game.tiles[5 * game.width + 6].unit = Some(scout_id);
        game.units.push(Unit {
            id: scout_id,
            owner: player_owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 6,
            y: 5,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let colony_id = game.units.len();
        game.tiles[4 * game.width + 6].unit = Some(colony_id);
        game.units.push(Unit {
            id: colony_id,
            owner: player_owner,
            kind: UnitKind::ColonyPod,
            design_index: 0,
            x: 6,
            y: 4,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let target = choose_ai_raider_target(&game, 5, 5).expect("raider target should exist");
        assert_eq!(target, (6, 4));
    }

    #[test]
    fn raider_base_scoring_prefers_active_convoy_hub() {
        let mut game = GameState::new_game(16, 16, 9);
        let player_owner = game.player_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
        }

        game.bases.push(Base {
            id: 0,
            owner: player_owner,
            name: "Hub".to_string(),
            x: 7,
            y: 5,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: vec![crate::Facility::TradeExchange],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[5 * game.width + 7].base = Some(0);
        game.bases.push(Base {
            id: 1,
            owner: player_owner,
            name: "Spoke".to_string(),
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
        game.add_convoy_route_typed(0, 1, crate::ConvoyRouteKind::Trade)
            .expect("trade route should exist");

        let signals = AiTacticalSignals {
            attack_bias: 2,
            exploration_bias: 0,
        };
        let convoy_score = score_raider_base_target(&game, 5, 5, 0, signals);
        let plain_score = score_raider_base_target(&game, 5, 5, 1, signals);
        assert!(convoy_score < plain_score);
    }

    #[test]
    fn exploratory_target_uses_bias_when_no_known_targets_exist() {
        let game = GameState::new_game(16, 16, 9);
        let signals = AiTacticalSignals {
            attack_bias: 0,
            exploration_bias: 4,
        };

        let target = exploratory_target(&game, 1, 1, signals).expect("target should exist");
        // With DX/DY noise, it could move toward 0, but it should definitely be a valid coordinate
        assert!(target.0 < 16);
        assert!(target.1 < 16);
    }

    #[test]
    fn unexplored_tile_scoring_improves_with_exploration_bias() {
        let low_bias = AiTacticalSignals {
            attack_bias: 0,
            exploration_bias: 1,
        };
        let high_bias = AiTacticalSignals {
            attack_bias: 0,
            exploration_bias: 5,
        };
        assert!(
            score_unexplored_tile_target(1, 1, 4, 4, high_bias)
                < score_unexplored_tile_target(1, 1, 4, 4, low_bias)
        );
    }

    #[test]
    fn economy_signals_report_expansion_and_pressure() {
        let mut game = GameState::new_game(16, 16, 9);
        let ai_owner = game.ai_owner();
        let player_owner = game.player_owner();
        let base_x = 10usize;
        let base_y = 10usize;

        for y in base_y.saturating_sub(1)..=(base_y + 1).min(game.height - 1) {
            for x in base_x.saturating_sub(1)..=(base_x + 1).min(game.width - 1) {
                let idx = y * game.width + x;
                game.tiles[idx].terrain = Terrain::Flat;
                game.tiles[idx].moisture = 70;
            }
        }
        game.tiles[base_y * game.width + base_x].base = Some(0);
        game.bases.push(Base {
            id: 0,
            owner: ai_owner,
            name: "Signal Test".to_string(),
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

        let player_id = game.units.len();
        let px = base_x.saturating_sub(2);
        let py = base_y;
        game.tiles[py * game.width + px].unit = Some(player_id);
        game.units.push(Unit {
            id: player_id,
            owner: player_owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: px,
            y: py,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });

        let signals = economy_signals_for_base(
            &game,
            ai_owner,
            0,
            base_x,
            base_y,
            game.base_yields(base_x, base_y),
        );
        assert!(signals.expansion_pressure);
        assert!(signals.military_pressure > 0);
    }

    #[test]
    fn unrested_base_prioritizes_recreation_commons_over_generic_infrastructure() {
        let mut game = GameState::new_game(16, 16, 9);
        let owner = game.ai_owner();
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
            owner,
            name: "Unrest Test".to_string(),
            x: 6,
            y: 6,
            population: 5,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        // Add dummy bases to bypass single-base forced expansion
        for i in 1..3 {
            game.bases.push(Base {
                id: i,
                owner,
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

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 0;
        if !faction.known_techs.contains(&Tech::SocialPsych) {
            faction.known_techs.push(Tech::SocialPsych);
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::RecreationCommons);
    }

    #[test]
    fn severe_unrest_upgrades_to_hologram_theatre_when_unlocked() {
        let mut game = GameState::new_game(16, 16, 9);
        let owner = game.ai_owner();
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
            owner,
            name: "Morale Test".to_string(),
            x: 6,
            y: 6,
            population: 8,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: vec![crate::Facility::RecreationCommons],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        // Add dummy bases to bypass single-base forced expansion
        for i in 1..3 {
            game.bases.push(Base {
                id: i,
                owner,
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

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 50;
        if !faction.known_techs.contains(&Tech::SocialPsych) {
            faction.known_techs.push(Tech::SocialPsych);
        }
        if !faction.known_techs.contains(&Tech::InformationNetworks) {
            faction.known_techs.push(Tech::InformationNetworks);
        }
        if !faction.known_techs.contains(&Tech::PlanetaryNetworks) {
            faction.known_techs.push(Tech::PlanetaryNetworks);
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::HologramTheatre);
    }

    #[test]
    fn support_pressure_prioritizes_command_center_before_new_colony_pod() {
        let mut game = GameState::new_game(16, 16, 9);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 60;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Support Test".to_string(),
            x: 6,
            y: 6,
            population: 5,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        for (id, x, y) in [
            (1usize, 10usize, 10usize),
            (2usize, 12usize, 10usize),
            (3usize, 10usize, 12usize),
        ] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Dummy {id}"),
                x,
                y,
                population: 1,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::Former,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }

        for (unit_id, kind, x, y) in [
            (100usize, UnitKind::Former, 6usize, 6usize),
            (101usize, UnitKind::ScoutPatrol, 5usize, 6usize),
            (102usize, UnitKind::ScoutPatrol, 7usize, 6usize),
            (103usize, UnitKind::ScoutPatrol, 10usize, 10usize),
            (104usize, UnitKind::ScoutPatrol, 12usize, 10usize),
            (105usize, UnitKind::ScoutPatrol, 10usize, 12usize),
            (106usize, UnitKind::ScoutPatrol, 11usize, 10usize),
            (107usize, UnitKind::ScoutPatrol, 12usize, 11usize),
            (108usize, UnitKind::ScoutPatrol, 10usize, 11usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::CommandCenter);
    }

    #[test]
    fn severe_support_pressure_upgrades_to_transit_hub_after_command_center() {
        let mut game = GameState::new_game(16, 16, 9);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 60;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Transit Support Test".to_string(),
            x: 6,
            y: 6,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: vec![crate::Facility::CommandCenter],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }
        if !faction.known_techs.contains(&Tech::DoctrineMobility) {
            faction.known_techs.push(Tech::DoctrineMobility);
        }

        for (unit_id, kind, x, y) in [
            (100usize, UnitKind::Former, 6usize, 6usize),
            (101usize, UnitKind::ScoutPatrol, 5usize, 6usize),
            (102usize, UnitKind::ScoutPatrol, 7usize, 6usize),
            (103usize, UnitKind::ScoutPatrol, 6usize, 5usize),
            (104usize, UnitKind::ScoutPatrol, 6usize, 7usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::TransitHub);
    }

    #[test]
    fn support_pressure_social_engineering_prefers_police() {
        let mut game = GameState::new_game(16, 16, 9);
        let owner = game.ai_owner();
        game.turn = 10;
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Policy Pressure".to_string(),
            x: 6,
            y: 6,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        if !faction.known_techs.contains(&Tech::InformationNetworks) {
            faction.known_techs.push(Tech::InformationNetworks);
        }

        for (unit_id, kind, x, y) in [
            (100usize, UnitKind::Former, 6usize, 6usize),
            (101usize, UnitKind::ScoutPatrol, 5usize, 6usize),
            (102usize, UnitKind::ScoutPatrol, 7usize, 6usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        update_ai_social_engineering(&mut game, owner);

        let politics = game
            .faction(owner)
            .expect("AI faction must exist")
            .social_engineering
            .politics;
        assert_eq!(politics, crate::model::Politics::Police);
    }

    #[test]
    fn underexpanded_faction_uses_colony_pod_to_escape_mild_support_pressure() {
        let mut game = GameState::new_game(20, 20, 11);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y) in [(0usize, 6usize, 6usize), (1usize, 12usize, 12usize)] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::Former,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }

        for (unit_id, kind, x, y) in [
            (100usize, UnitKind::Former, 6usize, 6usize),
            (101usize, UnitKind::ScoutPatrol, 5usize, 6usize),
            (102usize, UnitKind::ScoutPatrol, 12usize, 12usize),
            (103usize, UnitKind::ScoutPatrol, 11usize, 12usize),
            (104usize, UnitKind::ScoutPatrol, 12usize, 11usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        assert_eq!(game.faction_support_summary(owner).supported_units, 1);

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::ColonyPod);
    }

    #[test]
    fn two_base_faction_uses_colony_pod_to_escape_moderate_support_pressure() {
        let mut game = GameState::new_game(20, 20, 27);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y) in [(0usize, 6usize, 6usize), (1usize, 12usize, 12usize)] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::Former,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }

        for (unit_id, kind, x, y) in [
            (100usize, UnitKind::Former, 6usize, 6usize),
            (101usize, UnitKind::ScoutPatrol, 5usize, 6usize),
            (102usize, UnitKind::ScoutPatrol, 7usize, 6usize),
            (103usize, UnitKind::ScoutPatrol, 12usize, 12usize),
            (104usize, UnitKind::ScoutPatrol, 11usize, 12usize),
            (105usize, UnitKind::ScoutPatrol, 12usize, 11usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        assert_eq!(game.faction_support_summary(owner).supported_units, 2);

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::ColonyPod);
    }

    #[test]
    fn two_base_faction_can_queue_second_colony_pod_when_first_is_already_active() {
        let mut game = GameState::new_game(20, 20, 29);
        let owner = game.ai_owner();
        game.turn = 40;
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y) in [(0usize, 6usize, 6usize), (1usize, 12usize, 12usize)] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population: 4,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::Former,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 80;
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }

        for (unit_id, kind, x, y) in [
            (100usize, UnitKind::Former, 6usize, 6usize),
            (101usize, UnitKind::ScoutPatrol, 5usize, 6usize),
            (102usize, UnitKind::ScoutPatrol, 7usize, 6usize),
            (103usize, UnitKind::ScoutPatrol, 12usize, 12usize),
            (104usize, UnitKind::ScoutPatrol, 11usize, 12usize),
            (105usize, UnitKind::ColonyPod, 8usize, 8usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::ColonyPod);
    }

    #[test]
    fn underexpanded_autoplay_ai_defers_secrets_of_planet_research() {
        let mut game = GameState::new_game(20, 20, 17);
        let owner = game.player_owner();
        game.turn = 20;

        let known_techs: Vec<Tech> = Tech::all()
            .into_iter()
            .filter(|tech| {
                !matches!(
                    tech,
                    Tech::SecretsOfPlanet | Tech::OrbitalMechanics | Tech::SingularityPhysics
                )
            })
            .collect();

        let faction = game.faction_mut(owner).expect("player faction must exist");
        faction.known_techs = known_techs;
        faction.current_research = Tech::IndustrialBase;

        assert!(game.is_research_available(owner, Tech::SecretsOfPlanet));

        update_ai_research(&mut game, owner);

        let current_research = game
            .faction(owner)
            .expect("player faction must exist")
            .current_research;
        assert_ne!(current_research, Tech::SecretsOfPlanet);
    }

    #[test]
    fn support_pressure_avoids_scout_fallback_when_relief_infrastructure_is_available() {
        let mut game = GameState::new_game(16, 16, 19);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Rocky;
            tile.moisture = 10;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Support Relief".to_string(),
            x: 6,
            y: 6,
            population: 3,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        if !faction.known_techs.contains(&Tech::Biogenetics) {
            faction.known_techs.push(Tech::Biogenetics);
        }

        for (unit_id, x, y) in [
            (100usize, 6usize, 6usize),
            (101usize, 5usize, 6usize),
            (102usize, 7usize, 6usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind: UnitKind::ScoutPatrol,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert!(
            matches!(
                choice,
                ProductionItem::RecyclingTanks
                    | ProductionItem::Greenhouse
                    | ProductionItem::TradeExchange
                    | ProductionItem::NetworkNode
                    | ProductionItem::Former
                    | ProductionItem::StockpileEnergy
            ),
            "support pressure should choose relief infrastructure instead of scout fallback, got {choice:?}"
        );
    }

    #[test]
    fn command_center_support_pressure_prefers_military_supply_route() {
        let mut game = GameState::new_game(16, 16, 41);
        let owner = game.ai_owner();
        game.units.clear();
        game.bases.clear();
        game.convoy_routes.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 60;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Supply Hub".to_string(),
            x: 6,
            y: 6,
            population: 4,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: vec![crate::Facility::CommandCenter],
            governor_mode: GovernorMode::Off,
        });
        game.tiles[6 * game.width + 6].base = Some(0);

        game.bases.push(Base {
            id: 1,
            owner,
            name: "Frontier".to_string(),
            x: 10,
            y: 10,
            population: 3,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::Former,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[10 * game.width + 10].base = Some(1);

        for (unit_id, x, y) in [
            (100usize, 6usize, 6usize),
            (101usize, 5usize, 6usize),
            (102usize, 7usize, 6usize),
            (103usize, 10usize, 10usize),
            (104usize, 9usize, 10usize),
            (105usize, 10usize, 9usize),
            (106usize, 11usize, 10usize),
        ] {
            game.tiles[y * game.width + x].unit = Some(unit_id);
            game.units.push(Unit {
                id: unit_id,
                owner,
                kind: UnitKind::ScoutPatrol,
                design_index: 0,
                x,
                y,
                moves_left: 1,
                hp: 10,
                experience: 0,
                alive: true,
                cargo_unit_ids: Vec::new(),
                activity: UnitActivity::None,
            });
        }

        assert!(game.faction_support_summary(owner).supported_units > 0);

        maybe_assign_ai_convoy_route(&mut game, 0, owner);

        assert_eq!(game.convoy_routes.len(), 1);
        assert_eq!(
            game.convoy_routes[0].kind,
            crate::ConvoyRouteKind::MilitarySupply
        );
    }

    #[test]
    fn sole_base_defender_stays_home_despite_high_attack_bias() {
        let mut game = GameState::new_game(16, 16, 31);
        let owner = game.ai_owner();
        let rival = game.player_owner();
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
        }

        game.bases.push(Base {
            id: 0,
            owner,
            name: "Sparta Command".to_string(),
            x: 10,
            y: 10,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[10 * game.width + 10].base = Some(0);

        game.bases.push(Base {
            id: 1,
            owner: rival,
            name: "Landing Point".to_string(),
            x: 2,
            y: 2,
            population: 2,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
            production_queue: Vec::new(),
            facilities: Vec::new(),
            governor_mode: GovernorMode::Off,
        });
        game.tiles[2 * game.width + 2].base = Some(1);

        game.units.push(Unit {
            id: 100,
            owner,
            kind: UnitKind::ScoutPatrol,
            design_index: 0,
            x: 10,
            y: 10,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });
        game.tiles[10 * game.width + 10].unit = Some(100);

        run_ai_tactics_for_owner(&mut game, owner);

        let unit = game.unit(100).expect("garrison should still exist");
        assert_eq!((unit.x, unit.y), (10, 10));
    }

    #[test]
    fn two_base_autoplay_ai_pushes_third_colony_when_stalled_and_safe() {
        let mut game = GameState::new_game(20, 20, 23);
        let owner = game.ai_owner();
        game.turn = 40;
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y) in [(0usize, 6usize, 6usize), (1usize, 12usize, 12usize)] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population: 3,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::Former,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 80;
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }
        if !faction.known_techs.contains(&Tech::CentauriEcology) {
            faction.known_techs.push(Tech::CentauriEcology);
        }
        if !faction.known_techs.contains(&Tech::SocialPsych) {
            faction.known_techs.push(Tech::SocialPsych);
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::ColonyPod);
    }

    #[test]
    fn low_energy_two_base_ai_still_pushes_third_colony_when_safe_late() {
        let mut game = GameState::new_game(20, 20, 33);
        let owner = game.ai_owner();
        game.turn = 60;
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y) in [(0usize, 6usize, 6usize), (1usize, 12usize, 12usize)] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population: 2,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::Former,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 8;
        if !faction.known_techs.contains(&Tech::IndustrialBase) {
            faction.known_techs.push(Tech::IndustrialBase);
        }

        let choice = choose_ai_production_for_base(&game, 0, owner);

        assert_eq!(choice, ProductionItem::ColonyPod);
    }

    #[test]
    fn stalled_two_base_ai_interrupts_half_built_infrastructure_for_colony_pod() {
        let mut game = GameState::new_game(20, 20, 24);
        let owner = game.ai_owner();
        game.turn = 45;
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y, population, production, minerals_stock) in [
            (0usize, 6usize, 6usize, 2i32, ProductionItem::Former, 0i32),
            (
                1usize,
                12usize,
                12usize,
                6i32,
                ProductionItem::RecreationCommons,
                18i32,
            ),
        ] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population,
                nutrients_stock: 0,
                minerals_stock,
                production,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 40;
        if !faction.known_techs.contains(&Tech::CentauriEcology) {
            faction.known_techs.push(Tech::CentauriEcology);
        }
        if !faction.known_techs.contains(&Tech::SocialPsych) {
            faction.known_techs.push(Tech::SocialPsych);
        }

        run_ai_economy_for_owner(&mut game, owner);

        let base = game.base(1).expect("second base should still exist");
        assert_eq!(base.production, ProductionItem::ColonyPod);
    }

    #[test]
    fn native_psi_pressure_does_not_block_third_colony_push() {
        let mut game = GameState::new_game(20, 20, 25);
        let owner = game.ai_owner();
        let native_owner = game.native_owner();
        game.turn = 45;
        game.units.clear();
        game.bases.clear();
        for tile in &mut game.tiles {
            tile.unit = None;
            tile.base = None;
            tile.terrain = Terrain::Flat;
            tile.moisture = 70;
        }

        for (id, x, y) in [(0usize, 6usize, 6usize), (1usize, 12usize, 12usize)] {
            game.bases.push(Base {
                id,
                owner,
                name: format!("Base {id}"),
                x,
                y,
                population: 4,
                nutrients_stock: 0,
                minerals_stock: 0,
                production: ProductionItem::RecyclingTanks,
                production_queue: Vec::new(),
                facilities: Vec::new(),
                governor_mode: GovernorMode::Off,
            });
            game.tiles[y * game.width + x].base = Some(id);
        }

        game.units.push(Unit {
            id: 100,
            owner: native_owner,
            kind: UnitKind::MindWorm,
            design_index: 0,
            x: 13,
            y: 10,
            moves_left: 1,
            hp: 10,
            experience: 0,
            alive: true,
            cargo_unit_ids: Vec::new(),
            activity: UnitActivity::None,
        });
        game.tiles[10 * game.width + 13].unit = Some(100);

        let faction = game.faction_mut(owner).expect("AI faction must exist");
        faction.energy = 45;
        if !faction.known_techs.contains(&Tech::CentauriEcology) {
            faction.known_techs.push(Tech::CentauriEcology);
        }
        if !faction.known_techs.contains(&Tech::SocialPsych) {
            faction.known_techs.push(Tech::SocialPsych);
        }

        let choice = choose_ai_production_for_base(&game, 1, owner);

        assert_eq!(choice, ProductionItem::ColonyPod);
    }

    #[test]
    fn ai_creates_custom_designs_upon_tech_discovery() {
        let mut game = GameState::new_game(16, 16, 7);
        let owner = game.ai_owner();

        // 1. Initial state: only basic techs
        if let Some(faction) = game.faction_mut(owner) {
            faction.known_techs = vec![Tech::IndustrialBase];
        }

        update_ai_unit_designs(&mut game, owner);

        let initial_designs = game.faction(owner).unwrap().unit_designs.len();

        // 2. Discover Field Modulation (Resonance Laser/Armor)
        game.turn = 15; // Trigger the interval
        if let Some(faction) = game.faction_mut(owner) {
            faction.known_techs.push(Tech::FieldModulation);
        }

        update_ai_unit_designs(&mut game, owner);

        let new_designs = &game.faction(owner).unwrap().unit_designs;
        assert!(new_designs.len() > initial_designs);

        // Check if any design has resonance laser (power 4)
        assert!(new_designs.iter().any(|d| d.attack_strength() == 4));
        assert!(new_designs.iter().any(|d| d.defense_strength() == 2));
    }
}

fn step_toward(current: usize, target: usize) -> isize {
    if target > current {
        1
    } else if target < current {
        -1
    } else {
        0
    }
}

fn manhattan(ax: usize, ay: usize, bx: usize, by: usize) -> usize {
    ax.abs_diff(bx) + ay.abs_diff(by)
}
