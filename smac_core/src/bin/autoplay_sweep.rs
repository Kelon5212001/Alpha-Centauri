use smac_core::content_api::{facility_maintenance, production_name};
use smac_core::{
    offense_readiness_for_owner, CommandCenterTurnTrace, Facility, GameOver, GameState,
    ProductionItem, Tech,
};
use std::collections::HashMap;
use std::env;

struct Config {
    turns: usize,
    width: usize,
    height: usize,
    start_seed: u32,
    count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            turns: 100,
            width: 20,
            height: 20,
            start_seed: 1,
            count: 10,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct OwnerMetrics {
    bases: usize,
    units: usize,
    combat_units: usize,
    energy: i32,
    known_techs: usize,
    food_security: i32,
    frontier_bases: usize,
    unrested_bases: usize,
    max_unrest: i32,
    supported_units: i32,
    unit_upkeep: i32,
    command_centers: usize,
    transit_hubs: usize,
    industrial_base_known: bool,
    command_center_available: bool,
    peak_supported_units: i32,
    peak_unit_upkeep: i32,
    peak_colony_pods: usize,
    peak_formers: usize,
    peak_probes: usize,
    peak_support_combat_units: usize,
    peak_formers_on_base: usize,
    peak_formers_in_field: usize,
    peak_field_formers_with_nearby_work: usize,
    peak_field_formers_saturated: usize,
    peak_saturated_formers_near_base: usize,
    peak_saturated_formers_far_from_base: usize,
    current_active_former_builds: usize,
    peak_active_former_builds: usize,
    facility_upkeep: i32,
    convoy_upkeep: i32,
    total_upkeep: i32,
    current_max_base_facilities: usize,
    current_max_base_facility_upkeep: i32,
    current_max_base_optional_upkeep: i32,
    peak_base_facilities: usize,
    peak_base_facility_upkeep: i32,
    peak_base_optional_upkeep: i32,
    peak_base_stress_turn: usize,
    command_center_gap_bases: usize,
    command_center_active_bases: usize,
    command_center_queued_bases: usize,
    command_center_blocker_count: usize,
    command_center_blocker: Option<ProductionItem>,
    command_center_avg_progress_pct: i32,
    command_center_max_progress_pct: i32,
    command_center_low_mineral_bases: usize,
    command_center_base_turns: usize,
    command_center_loss_turns: usize,
    command_center_positive_yield_loss_turns: usize,
    command_center_avg_start_stock: i32,
    command_center_avg_end_stock: i32,
    command_center_avg_yield_minerals: i32,
    command_center_avg_mineral_margin: i32,
    command_center_trace_turns: usize,
    command_center_avg_post_production_stock: i32,
    command_center_avg_post_interdiction_stock: i32,
    command_center_avg_exact_upkeep_drain: i32,
    command_center_drained_trace_turns: usize,
    command_center_avg_upkeep_order_index: i32,
    command_center_completed_turns: usize,
    command_center_switched_turns: usize,
    command_center_lost_base_turns: usize,
    command_center_retained_active_turns: usize,
    command_center_loss_with_intercepted_freight: usize,
    command_center_loss_with_collapsing_freight: usize,
    command_center_loss_with_support_drain: usize,
    command_center_loss_with_production_reset: usize,
    command_center_loss_with_bankruptcy: usize,
    command_center_loss_with_emergency_support: usize,
    command_center_loss_with_support_famine: usize,
}

#[derive(Clone, Copy, Default)]
struct OwnerPeakBaseStress {
    facilities: usize,
    facility_upkeep: i32,
    optional_upkeep: i32,
    turn: usize,
}

#[derive(Clone, Copy, Default)]
struct OwnerPeakSupport {
    supported_units: i32,
    unit_upkeep: i32,
    colony_pods: usize,
    formers: usize,
    probes: usize,
    combat_units: usize,
    formers_on_base: usize,
    formers_in_field: usize,
    field_formers_with_nearby_work: usize,
    field_formers_saturated: usize,
    saturated_formers_near_base: usize,
    saturated_formers_far_from_base: usize,
    active_former_builds: usize,
    turn: usize,
}

#[derive(Clone, Copy, Default)]
struct OwnerCommandCenterTurnFlow {
    base_turns: usize,
    loss_turns: usize,
    positive_yield_loss_turns: usize,
    total_start_stock: i32,
    total_end_stock: i32,
    total_yield_minerals: i32,
    total_mineral_margin: i32,
    trace_turns: usize,
    total_post_production_stock: i32,
    total_post_interdiction_stock: i32,
    total_exact_upkeep_drain: i32,
    drained_trace_turns: usize,
    total_upkeep_order_index: i32,
    completed_turns: usize,
    switched_turns: usize,
    lost_base_turns: usize,
    retained_active_turns: usize,
    loss_with_intercepted_freight: usize,
    loss_with_collapsing_freight: usize,
    loss_with_support_drain: usize,
    loss_with_production_reset: usize,
    loss_with_bankruptcy: usize,
    loss_with_emergency_support: usize,
    loss_with_support_famine: usize,
}

#[derive(Clone)]
struct ActiveCommandCenterBaseStart {
    base_id: usize,
    base_name: String,
    start_stock: i32,
    yield_minerals: i32,
    mineral_margin: i32,
    intercepted_freight_routes: usize,
    collapsing_freight_routes: usize,
    estimated_support_drain: i32,
}

#[derive(Clone, Copy, Default)]
struct OwnerTurnEconomySignals {
    bankruptcy: bool,
    emergency_support_payment: bool,
    support_famine: bool,
}

struct RunSummary {
    seed: u32,
    completed_turns: usize,
    outcome: Option<GameOver>,
    routes: usize,
    projects: usize,
    nearest_base_gap: i32,
    raids: usize,
    combats: usize,
    captures: usize,
    war_declarations: usize,
    bankruptcies: usize,
    facility_bankruptcies: usize,
    unit_bankruptcies: usize,
    scrap_trade_exchange: usize,
    scrap_freight_depot: usize,
    scrap_network_node: usize,
    scrap_transit_hub: usize,
    scrap_hologram_theatre: usize,
    scrap_research_hospital: usize,
    scrap_other_facility: usize,
    famines: usize,
    starvation_famines: usize,
    support_famines: usize,
    emergency_support_payments: usize,
    emergency_support_energy: i32,
    player_ready_turns: usize,
    player_target_turns: usize,
    ai_ready_turns: usize,
    ai_target_turns: usize,
    player: OwnerMetrics,
    ai: OwnerMetrics,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let mut terminal_runs = 0usize;
    let mut total_raids = 0usize;
    let mut total_combats = 0usize;
    let mut total_captures = 0usize;
    let mut total_wars = 0usize;
    let mut total_bankruptcies = 0usize;
    let mut total_facility_bankruptcies = 0usize;
    let mut total_unit_bankruptcies = 0usize;
    let mut total_scrap_trade_exchange = 0usize;
    let mut total_scrap_freight_depot = 0usize;
    let mut total_scrap_network_node = 0usize;
    let mut total_scrap_transit_hub = 0usize;
    let mut total_scrap_hologram_theatre = 0usize;
    let mut total_scrap_research_hospital = 0usize;
    let mut total_scrap_other_facility = 0usize;
    let mut total_famines = 0usize;
    let mut total_starvation_famines = 0usize;
    let mut total_support_famines = 0usize;
    let mut total_emergency_support_payments = 0usize;
    let mut total_emergency_support_energy = 0i32;
    let mut player_zero_unit_runs = 0usize;
    let mut ai_zero_unit_runs = 0usize;
    let mut player_low_expansion_runs = 0usize;
    let mut ai_low_expansion_runs = 0usize;
    let mut total_player_ready_turns = 0usize;
    let mut total_player_target_turns = 0usize;
    let mut total_ai_ready_turns = 0usize;
    let mut total_ai_target_turns = 0usize;

    println!(
        "Autoplay sweep: {} seeds from {} on {}x{} for {} turns.",
        config.count, config.start_seed, config.width, config.height, config.turns
    );

    for offset in 0..config.count {
        let seed = config.start_seed + offset as u32;
        let summary = run_seed(seed, &config);
        if summary.outcome.is_some() {
            terminal_runs += 1;
        }
        total_raids += summary.raids;
        total_combats += summary.combats;
        total_captures += summary.captures;
        total_wars += summary.war_declarations;
        total_bankruptcies += summary.bankruptcies;
        total_facility_bankruptcies += summary.facility_bankruptcies;
        total_unit_bankruptcies += summary.unit_bankruptcies;
        total_scrap_trade_exchange += summary.scrap_trade_exchange;
        total_scrap_freight_depot += summary.scrap_freight_depot;
        total_scrap_network_node += summary.scrap_network_node;
        total_scrap_transit_hub += summary.scrap_transit_hub;
        total_scrap_hologram_theatre += summary.scrap_hologram_theatre;
        total_scrap_research_hospital += summary.scrap_research_hospital;
        total_scrap_other_facility += summary.scrap_other_facility;
        total_famines += summary.famines;
        total_starvation_famines += summary.starvation_famines;
        total_support_famines += summary.support_famines;
        total_emergency_support_payments += summary.emergency_support_payments;
        total_emergency_support_energy += summary.emergency_support_energy;
        if summary.player.units == 0 {
            player_zero_unit_runs += 1;
        }
        if summary.ai.units == 0 {
            ai_zero_unit_runs += 1;
        }
        if summary.player.bases < 3 {
            player_low_expansion_runs += 1;
        }
        if summary.ai.bases < 3 {
            ai_low_expansion_runs += 1;
        }
        total_player_ready_turns += summary.player_ready_turns;
        total_player_target_turns += summary.player_target_turns;
        total_ai_ready_turns += summary.ai_ready_turns;
        total_ai_target_turns += summary.ai_target_turns;

        println!(
            "seed {:>3} | turns {:>3} | outcome {:<12} | routes {:>2} projects {:>2} gap {:>2} raids {:>2} combats {:>3} caps {:>2} wars {:>2} | p off {:>3}/{:>3} bases {:>2} units {:>2}/{:>2} tech {:>2} energy {:>4} food {:>4} frontier {:>2} unrest {:>2}/{:<2} supp {:>2}/{:<2} cc {:>2} th {:>2} ib {} ca {} pk {:>2}/{:<2} mix {:>2}/{:>2}/{:>2}/{:>2} fld {:>2}/{:>2} wrk {:>2}/{:>2} sat {:>2}/{:>2} fmb {:>2}/{:>2} upk {:>2}+{:>2}+{:>2} base {:>2}f/{:>2}m/{:>2}o pk {:>2}f/{:>2}m/{:>2}o@{:>3} ccgap {:>2}/{:>2}/{:<2} ccprog {:>2}/{:>2} lm {:>2} ccflow {:>2} loss {:>2}/{:>2} {:>2}/{:>2}/{:>2}/{:>2} fate {:>2}/{:>2}/{:>2}/{:>2} ccupk {:>2}/{:>2}/{:>2}/{:>2}/{:>2}/{:>2} src {:>2}/{:>2}/{:>2}/{:>2} own {:>2}/{:>2}/{:>2} blk {:<16} | ai off {:>3}/{:>3} bases {:>2} units {:>2}/{:>2} tech {:>2} energy {:>4} food {:>4} frontier {:>2} unrest {:>2}/{:<2} supp {:>2}/{:<2} cc {:>2} th {:>2} ib {} ca {} pk {:>2}/{:<2} mix {:>2}/{:>2}/{:>2}/{:>2} fld {:>2}/{:>2} wrk {:>2}/{:>2} sat {:>2}/{:>2} fmb {:>2}/{:>2} upk {:>2}+{:>2}+{:>2} base {:>2}f/{:>2}m/{:>2}o pk {:>2}f/{:>2}m/{:>2}o@{:>3} ccgap {:>2}/{:>2}/{:<2} ccprog {:>2}/{:>2} lm {:>2} ccflow {:>2} loss {:>2}/{:>2} {:>2}/{:>2}/{:>2}/{:>2} fate {:>2}/{:>2}/{:>2}/{:>2} ccupk {:>2}/{:>2}/{:>2}/{:>2}/{:>2}/{:>2} src {:>2}/{:>2}/{:>2}/{:>2} own {:>2}/{:>2}/{:>2} blk {:<16} | bank {:>2} fac {:>2} unit {:>2} scr {:>2}/{:>2}/{:>2}/{:>2}/{:>2}/{:>2}/{:>2} em {:>2}/{:>3} famine {:>2} starve {:>2} support {:>2}",
            summary.seed,
            summary.completed_turns,
            summary
                .outcome
                .map(game_over_label)
                .unwrap_or("none"),
            summary.routes,
            summary.projects,
            summary.nearest_base_gap,
            summary.raids,
            summary.combats,
            summary.captures,
            summary.war_declarations,
            summary.player_ready_turns,
            summary.player_target_turns,
            summary.player.bases,
            summary.player.units,
            summary.player.combat_units,
            summary.player.known_techs,
            summary.player.energy,
            summary.player.food_security,
            summary.player.frontier_bases,
            summary.player.unrested_bases,
            summary.player.max_unrest,
            summary.player.supported_units,
            summary.player.unit_upkeep,
            summary.player.command_centers,
            summary.player.transit_hubs,
            yn(summary.player.industrial_base_known),
            yn(summary.player.command_center_available),
            summary.player.peak_supported_units,
            summary.player.peak_unit_upkeep,
            summary.player.peak_colony_pods,
            summary.player.peak_formers,
            summary.player.peak_probes,
            summary.player.peak_support_combat_units,
            summary.player.peak_formers_on_base,
            summary.player.peak_formers_in_field,
            summary.player.peak_field_formers_with_nearby_work,
            summary.player.peak_field_formers_saturated,
            summary.player.peak_saturated_formers_near_base,
            summary.player.peak_saturated_formers_far_from_base,
            summary.player.current_active_former_builds,
            summary.player.peak_active_former_builds,
            summary.player.facility_upkeep,
            summary.player.convoy_upkeep,
            summary.player.total_upkeep,
            summary.player.current_max_base_facilities,
            summary.player.current_max_base_facility_upkeep,
            summary.player.current_max_base_optional_upkeep,
            summary.player.peak_base_facilities,
            summary.player.peak_base_facility_upkeep,
            summary.player.peak_base_optional_upkeep,
            summary.player.peak_base_stress_turn,
            summary.player.command_center_gap_bases,
            summary.player.command_center_active_bases,
            summary.player.command_center_queued_bases,
            summary.player.command_center_avg_progress_pct,
            summary.player.command_center_max_progress_pct,
            summary.player.command_center_low_mineral_bases,
            summary.player.command_center_base_turns,
            summary.player.command_center_loss_turns,
            summary.player.command_center_positive_yield_loss_turns,
            summary.player.command_center_avg_start_stock,
            summary.player.command_center_avg_end_stock,
            summary.player.command_center_avg_yield_minerals,
            summary.player.command_center_avg_mineral_margin,
            summary.player.command_center_completed_turns,
            summary.player.command_center_switched_turns,
            summary.player.command_center_lost_base_turns,
            summary.player.command_center_retained_active_turns,
            summary.player.command_center_trace_turns,
            summary.player.command_center_avg_post_production_stock,
            summary.player.command_center_avg_post_interdiction_stock,
            summary.player.command_center_avg_exact_upkeep_drain,
            summary.player.command_center_drained_trace_turns,
            summary.player.command_center_avg_upkeep_order_index,
            summary.player.command_center_loss_with_intercepted_freight,
            summary.player.command_center_loss_with_collapsing_freight,
            summary.player.command_center_loss_with_support_drain,
            summary.player.command_center_loss_with_production_reset,
            summary.player.command_center_loss_with_bankruptcy,
            summary.player.command_center_loss_with_emergency_support,
            summary.player.command_center_loss_with_support_famine,
            blocker_label(
                summary.player.command_center_blocker,
                summary.player.command_center_blocker_count,
            ),
            summary.ai_ready_turns,
            summary.ai_target_turns,
            summary.ai.bases,
            summary.ai.units,
            summary.ai.combat_units,
            summary.ai.known_techs,
            summary.ai.energy,
            summary.ai.food_security,
            summary.ai.frontier_bases,
            summary.ai.unrested_bases,
            summary.ai.max_unrest,
            summary.ai.supported_units,
            summary.ai.unit_upkeep,
            summary.ai.command_centers,
            summary.ai.transit_hubs,
            yn(summary.ai.industrial_base_known),
            yn(summary.ai.command_center_available),
            summary.ai.peak_supported_units,
            summary.ai.peak_unit_upkeep,
            summary.ai.peak_colony_pods,
            summary.ai.peak_formers,
            summary.ai.peak_probes,
            summary.ai.peak_support_combat_units,
            summary.ai.peak_formers_on_base,
            summary.ai.peak_formers_in_field,
            summary.ai.peak_field_formers_with_nearby_work,
            summary.ai.peak_field_formers_saturated,
            summary.ai.peak_saturated_formers_near_base,
            summary.ai.peak_saturated_formers_far_from_base,
            summary.ai.current_active_former_builds,
            summary.ai.peak_active_former_builds,
            summary.ai.facility_upkeep,
            summary.ai.convoy_upkeep,
            summary.ai.total_upkeep,
            summary.ai.current_max_base_facilities,
            summary.ai.current_max_base_facility_upkeep,
            summary.ai.current_max_base_optional_upkeep,
            summary.ai.peak_base_facilities,
            summary.ai.peak_base_facility_upkeep,
            summary.ai.peak_base_optional_upkeep,
            summary.ai.peak_base_stress_turn,
            summary.ai.command_center_gap_bases,
            summary.ai.command_center_active_bases,
            summary.ai.command_center_queued_bases,
            summary.ai.command_center_avg_progress_pct,
            summary.ai.command_center_max_progress_pct,
            summary.ai.command_center_low_mineral_bases,
            summary.ai.command_center_base_turns,
            summary.ai.command_center_loss_turns,
            summary.ai.command_center_positive_yield_loss_turns,
            summary.ai.command_center_avg_start_stock,
            summary.ai.command_center_avg_end_stock,
            summary.ai.command_center_avg_yield_minerals,
            summary.ai.command_center_avg_mineral_margin,
            summary.ai.command_center_completed_turns,
            summary.ai.command_center_switched_turns,
            summary.ai.command_center_lost_base_turns,
            summary.ai.command_center_retained_active_turns,
            summary.ai.command_center_trace_turns,
            summary.ai.command_center_avg_post_production_stock,
            summary.ai.command_center_avg_post_interdiction_stock,
            summary.ai.command_center_avg_exact_upkeep_drain,
            summary.ai.command_center_drained_trace_turns,
            summary.ai.command_center_avg_upkeep_order_index,
            summary.ai.command_center_loss_with_intercepted_freight,
            summary.ai.command_center_loss_with_collapsing_freight,
            summary.ai.command_center_loss_with_support_drain,
            summary.ai.command_center_loss_with_production_reset,
            summary.ai.command_center_loss_with_bankruptcy,
            summary.ai.command_center_loss_with_emergency_support,
            summary.ai.command_center_loss_with_support_famine,
            blocker_label(
                summary.ai.command_center_blocker,
                summary.ai.command_center_blocker_count,
            ),
            summary.bankruptcies,
            summary.facility_bankruptcies,
            summary.unit_bankruptcies,
            summary.scrap_trade_exchange,
            summary.scrap_freight_depot,
            summary.scrap_network_node,
            summary.scrap_transit_hub,
            summary.scrap_hologram_theatre,
            summary.scrap_research_hospital,
            summary.scrap_other_facility,
            summary.emergency_support_payments,
            summary.emergency_support_energy,
            summary.famines,
            summary.starvation_famines,
            summary.support_famines
        );
    }

    println!(
        "aggregate | terminal {} / {} | raids {} | combats {} | captures {} | wars {} | p off {}/{} | ai off {}/{} | bankruptcies {} fac {} unit {} scr {}/{}/{}/{}/{}/{}/{} em {}/{} | famines {} | starvation {} | support {} | player low-expansion {} | ai low-expansion {} | player zero-unit {} | ai zero-unit {}",
        terminal_runs,
        config.count,
        total_raids,
        total_combats,
        total_captures,
        total_wars,
        total_player_ready_turns,
        total_player_target_turns,
        total_ai_ready_turns,
        total_ai_target_turns,
        total_bankruptcies,
        total_facility_bankruptcies,
        total_unit_bankruptcies,
        total_scrap_trade_exchange,
        total_scrap_freight_depot,
        total_scrap_network_node,
        total_scrap_transit_hub,
        total_scrap_hologram_theatre,
        total_scrap_research_hospital,
        total_scrap_other_facility,
        total_emergency_support_payments,
        total_emergency_support_energy,
        total_famines,
        total_starvation_famines,
        total_support_famines,
        player_low_expansion_runs,
        ai_low_expansion_runs,
        player_zero_unit_runs,
        ai_zero_unit_runs
    );

    Ok(())
}

fn run_seed(seed: u32, config: &Config) -> RunSummary {
    let mut game = GameState::new_game(config.width, config.height, seed);
    let mut completed_turns = 0usize;
    let mut raids = 0usize;
    let mut combats = 0usize;
    let mut captures = 0usize;
    let mut war_declarations = 0usize;
    let mut bankruptcies = 0usize;
    let mut facility_bankruptcies = 0usize;
    let mut unit_bankruptcies = 0usize;
    let mut scrap_trade_exchange = 0usize;
    let mut scrap_freight_depot = 0usize;
    let mut scrap_network_node = 0usize;
    let mut scrap_transit_hub = 0usize;
    let mut scrap_hologram_theatre = 0usize;
    let mut scrap_research_hospital = 0usize;
    let mut scrap_other_facility = 0usize;
    let mut famines = 0usize;
    let mut starvation_famines = 0usize;
    let mut support_famines = 0usize;
    let mut emergency_support_payments = 0usize;
    let mut emergency_support_energy = 0i32;
    let mut player_ready_turns = 0usize;
    let mut player_target_turns = 0usize;
    let mut ai_ready_turns = 0usize;
    let mut ai_target_turns = 0usize;
    let mut player_peak_base_stress = owner_peak_base_stress(&game, game.player_owner(), 0);
    let mut ai_peak_base_stress = owner_peak_base_stress(&game, game.ai_owner(), 0);
    let mut player_peak_support = owner_peak_support(&game, game.player_owner(), 0);
    let mut ai_peak_support = owner_peak_support(&game, game.ai_owner(), 0);
    let mut player_command_center_turn_flow = OwnerCommandCenterTurnFlow::default();
    let mut ai_command_center_turn_flow = OwnerCommandCenterTurnFlow::default();

    while completed_turns < config.turns && game.game_over.is_none() {
        let player_readiness = offense_readiness_for_owner(&game, game.player_owner());
        let ai_readiness = offense_readiness_for_owner(&game, game.ai_owner());
        let player_cc_starts = active_command_center_base_starts(&game, game.player_owner());
        let ai_cc_starts = active_command_center_base_starts(&game, game.ai_owner());
        if player_readiness.has_offensive_target {
            player_target_turns += 1;
        }
        if player_readiness.can_form_attack_group {
            player_ready_turns += 1;
        }
        if ai_readiness.has_offensive_target {
            ai_target_turns += 1;
        }
        if ai_readiness.can_form_attack_group {
            ai_ready_turns += 1;
        }

        game.run_autoplay_mission_year();
        completed_turns += 1;
        let player_cc_traces = game.command_center_turn_traces_for_owner(game.player_owner());
        let ai_cc_traces = game.command_center_turn_traces_for_owner(game.ai_owner());
        let player_economy_signals = owner_turn_economy_signals(&game, game.player_owner());
        let ai_economy_signals = owner_turn_economy_signals(&game, game.ai_owner());
        player_command_center_turn_flow.observe_turn(
            &game,
            &player_cc_starts,
            &player_cc_traces,
            player_economy_signals,
        );
        ai_command_center_turn_flow.observe_turn(
            &game,
            &ai_cc_starts,
            &ai_cc_traces,
            ai_economy_signals,
        );
        player_peak_base_stress = player_peak_base_stress.max(owner_peak_base_stress(
            &game,
            game.player_owner(),
            completed_turns,
        ));
        ai_peak_base_stress =
            ai_peak_base_stress.max(owner_peak_base_stress(&game, game.ai_owner(), completed_turns));
        player_peak_support =
            player_peak_support.max(owner_peak_support(&game, game.player_owner(), completed_turns));
        ai_peak_support = ai_peak_support.max(owner_peak_support(&game, game.ai_owner(), completed_turns));

        for entry in game.log.iter().filter(|entry| entry.turn == game.turn) {
            if entry.message.contains("TACTICS:") {
                raids += 1;
            }
            if entry.message.contains("COMBAT:") || entry.message.contains("BOMBARDMENT:") {
                combats += 1;
            }
            if entry.message.contains("captured") {
                captures += 1;
            }
            if entry.message.contains("DIPLOMACY:")
                && entry.message.contains("signed a War")
            {
                war_declarations += 1;
            }
            if entry.message.contains("BANKRUPTCY:") {
                bankruptcies += 1;
                if entry.message.contains("scrapped") {
                    facility_bankruptcies += 1;
                    if entry.message.contains("TradeExchange") {
                        scrap_trade_exchange += 1;
                    } else if entry.message.contains("FreightDepot") {
                        scrap_freight_depot += 1;
                    } else if entry.message.contains("NetworkNode") {
                        scrap_network_node += 1;
                    } else if entry.message.contains("TransitHub") {
                        scrap_transit_hub += 1;
                    } else if entry.message.contains("HologramTheatre") {
                        scrap_hologram_theatre += 1;
                    } else if entry.message.contains("ResearchHospital") {
                        scrap_research_hospital += 1;
                    } else {
                        scrap_other_facility += 1;
                    }
                } else if entry.message.contains("unit disbanded") {
                    unit_bankruptcies += 1;
                }
            }
            if entry.message.contains("spent ")
                && entry.message.contains(" energy reserves to cover mineral support")
            {
                emergency_support_payments += 1;
                if let Some(amount) = entry
                    .message
                    .split(" spent ")
                    .nth(1)
                    .and_then(|tail| tail.split(" energy").next())
                    .and_then(|digits| digits.parse::<i32>().ok())
                {
                    emergency_support_energy += amount;
                }
            }
            if entry.message.contains("FAMINE:") {
                famines += 1;
                if entry
                    .message
                    .contains("population reduced due to starvation")
                {
                    starvation_famines += 1;
                } else if entry.message.contains("lack of support") {
                    support_famines += 1;
                }
            }
        }
    }

    RunSummary {
        seed,
        completed_turns,
        outcome: game.game_over,
        routes: game.convoy_routes.len(),
        projects: game.built_secret_projects.len(),
        nearest_base_gap: nearest_base_gap(&game, game.player_owner(), game.ai_owner()),
        raids,
        combats,
        captures,
        war_declarations,
        bankruptcies,
        facility_bankruptcies,
        unit_bankruptcies,
        scrap_trade_exchange,
        scrap_freight_depot,
        scrap_network_node,
        scrap_transit_hub,
        scrap_hologram_theatre,
        scrap_research_hospital,
        scrap_other_facility,
        famines,
        starvation_famines,
        support_famines,
        emergency_support_payments,
        emergency_support_energy,
        player_ready_turns,
        player_target_turns,
        ai_ready_turns,
        ai_target_turns,
        player: owner_metrics(
            &game,
            game.player_owner(),
            player_peak_base_stress,
            player_peak_support,
            player_command_center_turn_flow,
        ),
        ai: owner_metrics(
            &game,
            game.ai_owner(),
            ai_peak_base_stress,
            ai_peak_support,
            ai_command_center_turn_flow,
        ),
    }
}

fn owner_metrics(
    game: &GameState,
    owner: usize,
    peak_base_stress: OwnerPeakBaseStress,
    peak_support: OwnerPeakSupport,
    command_center_turn_flow: OwnerCommandCenterTurnFlow,
) -> OwnerMetrics {
    let bases = game.bases_for(owner);
    let unrest_values: Vec<i32> = bases.iter().map(|base| game.base_unrest(base.id)).collect();
    let faction = game.faction(owner);
    let support = game.faction_support_summary(owner);
    let live_units = game.live_units_for(owner);
    let (facility_upkeep, convoy_upkeep, _, total_upkeep) = game.faction_upkeep_breakdown(owner);
    let current_max_base_facilities = bases.iter().map(|base| base.facilities.len()).max().unwrap_or(0);
    let current_max_base_facility_upkeep = bases
        .iter()
        .map(|base| base_facility_upkeep(base))
        .max()
        .unwrap_or_default();
    let current_max_base_optional_upkeep = bases
        .iter()
        .map(|base| base_optional_facility_upkeep(base))
        .max()
        .unwrap_or_default();
    let command_center_gap = owner_command_center_gap(game, owner, &bases);
    let command_center_builds = owner_command_center_build_metrics(game, owner, &bases);

    OwnerMetrics {
        bases: bases.len(),
        units: live_units.len(),
        combat_units: live_units
            .iter()
            .filter(|unit| {
                !matches!(
                    unit.kind,
                    smac_core::UnitKind::ColonyPod
                        | smac_core::UnitKind::Former
                        | smac_core::UnitKind::ProbeTeam
                )
            })
            .count(),
        energy: faction.map(|f| f.energy).unwrap_or_default(),
        known_techs: faction.map(|f| f.known_techs.len()).unwrap_or_default(),
        food_security: faction.map(|f| f.food_security).unwrap_or_default(),
        frontier_bases: bases
            .iter()
            .filter(|base| game.base_local_military_pressure(base.id) >= 1)
            .count(),
        unrested_bases: unrest_values.iter().filter(|value| **value > 0).count(),
        max_unrest: unrest_values.into_iter().max().unwrap_or_default(),
        supported_units: support.supported_units,
        unit_upkeep: support.unit_upkeep,
        command_centers: bases
            .iter()
            .filter(|base| base.facilities.contains(&Facility::CommandCenter))
            .count(),
        transit_hubs: bases
            .iter()
            .filter(|base| base.facilities.contains(&Facility::TransitHub))
            .count(),
        industrial_base_known: faction
            .map(|f| f.known_techs.contains(&Tech::IndustrialBase))
            .unwrap_or(false),
        command_center_available: game.is_production_available(owner, ProductionItem::CommandCenter),
        peak_supported_units: peak_support.supported_units,
        peak_unit_upkeep: peak_support.unit_upkeep,
        peak_colony_pods: peak_support.colony_pods,
        peak_formers: peak_support.formers,
        peak_probes: peak_support.probes,
        peak_support_combat_units: peak_support.combat_units,
        peak_formers_on_base: peak_support.formers_on_base,
        peak_formers_in_field: peak_support.formers_in_field,
        peak_field_formers_with_nearby_work: peak_support.field_formers_with_nearby_work,
        peak_field_formers_saturated: peak_support.field_formers_saturated,
        peak_saturated_formers_near_base: peak_support.saturated_formers_near_base,
        peak_saturated_formers_far_from_base: peak_support.saturated_formers_far_from_base,
        current_active_former_builds: bases
            .iter()
            .filter(|base| base.production == ProductionItem::Former)
            .count(),
        peak_active_former_builds: peak_support.active_former_builds,
        facility_upkeep,
        convoy_upkeep,
        total_upkeep,
        current_max_base_facilities,
        current_max_base_facility_upkeep,
        current_max_base_optional_upkeep,
        peak_base_facilities: peak_base_stress.facilities,
        peak_base_facility_upkeep: peak_base_stress.facility_upkeep,
        peak_base_optional_upkeep: peak_base_stress.optional_upkeep,
        peak_base_stress_turn: peak_base_stress.turn,
        command_center_gap_bases: command_center_gap.gap_bases,
        command_center_active_bases: command_center_gap.active_bases,
        command_center_queued_bases: command_center_gap.queued_bases,
        command_center_blocker_count: command_center_gap.blocker_count,
        command_center_blocker: command_center_gap.blocker,
        command_center_avg_progress_pct: command_center_builds.avg_progress_pct,
        command_center_max_progress_pct: command_center_builds.max_progress_pct,
        command_center_low_mineral_bases: command_center_builds.low_mineral_bases,
        command_center_base_turns: command_center_turn_flow.base_turns,
        command_center_loss_turns: command_center_turn_flow.loss_turns,
        command_center_positive_yield_loss_turns: command_center_turn_flow.positive_yield_loss_turns,
        command_center_avg_start_stock: command_center_turn_flow.avg_start_stock(),
        command_center_avg_end_stock: command_center_turn_flow.avg_end_stock(),
        command_center_avg_yield_minerals: command_center_turn_flow.avg_yield_minerals(),
        command_center_avg_mineral_margin: command_center_turn_flow.avg_mineral_margin(),
        command_center_trace_turns: command_center_turn_flow.trace_turns,
        command_center_avg_post_production_stock: command_center_turn_flow
            .avg_post_production_stock(),
        command_center_avg_post_interdiction_stock: command_center_turn_flow
            .avg_post_interdiction_stock(),
        command_center_avg_exact_upkeep_drain: command_center_turn_flow
            .avg_exact_upkeep_drain(),
        command_center_drained_trace_turns: command_center_turn_flow.drained_trace_turns,
        command_center_avg_upkeep_order_index: command_center_turn_flow
            .avg_upkeep_order_index(),
        command_center_completed_turns: command_center_turn_flow.completed_turns,
        command_center_switched_turns: command_center_turn_flow.switched_turns,
        command_center_lost_base_turns: command_center_turn_flow.lost_base_turns,
        command_center_retained_active_turns: command_center_turn_flow.retained_active_turns,
        command_center_loss_with_intercepted_freight: command_center_turn_flow
            .loss_with_intercepted_freight,
        command_center_loss_with_collapsing_freight: command_center_turn_flow
            .loss_with_collapsing_freight,
        command_center_loss_with_support_drain: command_center_turn_flow.loss_with_support_drain,
        command_center_loss_with_production_reset: command_center_turn_flow
            .loss_with_production_reset,
        command_center_loss_with_bankruptcy: command_center_turn_flow.loss_with_bankruptcy,
        command_center_loss_with_emergency_support: command_center_turn_flow
            .loss_with_emergency_support,
        command_center_loss_with_support_famine: command_center_turn_flow
            .loss_with_support_famine,
    }
}

fn active_command_center_base_starts(game: &GameState, owner: usize) -> Vec<ActiveCommandCenterBaseStart> {
    let support_drain = owner_support_drain_estimates(game, owner);
    game.bases_for(owner)
        .into_iter()
        .filter(|base| base.production == ProductionItem::CommandCenter)
        .map(|base| {
            let yields = game
                .operational_base_yields(base.id)
                .unwrap_or_else(|| game.base_yields(base.x, base.y));
            let route_statuses = game.convoy_route_status_for_base(base.id);
            ActiveCommandCenterBaseStart {
                base_id: base.id,
                base_name: base.name.clone(),
                start_stock: base.minerals_stock,
                yield_minerals: yields.minerals,
                mineral_margin: game.base_mineral_margin(base.id).unwrap_or_default(),
                intercepted_freight_routes: route_statuses
                    .iter()
                    .filter(|(_, kind, _, intercepted, _)| {
                        *kind == smac_core::ConvoyRouteKind::Freight && *intercepted
                    })
                    .count(),
                collapsing_freight_routes: route_statuses
                    .iter()
                    .filter(|(_, kind, _, intercepted, integrity)| {
                        *kind == smac_core::ConvoyRouteKind::Freight && *intercepted && *integrity <= 1
                    })
                    .count(),
                estimated_support_drain: support_drain.get(&base.id).copied().unwrap_or_default(),
            }
        })
        .collect()
}

fn owner_support_drain_estimates(game: &GameState, owner: usize) -> HashMap<usize, i32> {
    let mut remaining = game.faction_support_summary(owner).unit_upkeep.max(0);
    let mut drains = HashMap::new();
    for base in game.bases_for(owner) {
        if remaining <= 0 {
            break;
        }
        let drain = base.minerals_stock.min(remaining).max(0);
        if drain > 0 {
            drains.insert(base.id, drain);
            remaining -= drain;
        }
    }
    drains
}

fn owner_turn_economy_signals(game: &GameState, owner: usize) -> OwnerTurnEconomySignals {
    let faction_name = game.faction_name(owner).to_string();
    let mut signals = OwnerTurnEconomySignals::default();
    for entry in game.log.iter().filter(|entry| entry.turn == game.turn) {
        if entry.message.contains("BANKRUPTCY:") && entry.message.contains(&faction_name) {
            signals.bankruptcy = true;
        }
        if entry.message.contains("spent ")
            && entry.message.contains(" energy reserves to cover mineral support")
            && entry.message.contains(&faction_name)
        {
            signals.emergency_support_payment = true;
        }
        if entry.message.contains("FAMINE:")
            && entry.message.contains("lack of support")
            && entry.message.contains(&faction_name)
        {
            signals.support_famine = true;
        }
    }
    signals
}

#[derive(Clone, Copy, Default)]
struct CommandCenterGapSummary {
    gap_bases: usize,
    active_bases: usize,
    queued_bases: usize,
    blocker_count: usize,
    blocker: Option<ProductionItem>,
}

#[derive(Clone, Copy, Default)]
struct CommandCenterBuildMetrics {
    avg_progress_pct: i32,
    max_progress_pct: i32,
    low_mineral_bases: usize,
}

fn owner_command_center_gap(
    game: &GameState,
    owner: usize,
    bases: &[&smac_core::Base],
) -> CommandCenterGapSummary {
    let support = game.faction_support_summary(owner);
    if support.supported_units <= 0
        || !game
            .faction(owner)
            .map(|f| f.known_techs.contains(&Tech::IndustrialBase))
            .unwrap_or(false)
        || !game.is_production_available(owner, ProductionItem::CommandCenter)
    {
        return CommandCenterGapSummary::default();
    }

    let mut summary = CommandCenterGapSummary::default();
    let mut blocker_counts: Vec<(ProductionItem, usize)> = Vec::new();

    for base in bases {
        if base.facilities.contains(&Facility::CommandCenter) {
            continue;
        }
        summary.gap_bases += 1;

        if base.production == ProductionItem::CommandCenter {
            summary.active_bases += 1;
            continue;
        }
        if base
            .production_queue
            .contains(&ProductionItem::CommandCenter)
        {
            summary.queued_bases += 1;
            continue;
        }

        summary.blocker_count += 1;
        if let Some((_, count)) = blocker_counts
            .iter_mut()
            .find(|(item, _)| *item == base.production)
        {
            *count += 1;
        } else {
            blocker_counts.push((base.production, 1));
        }
    }

    summary.blocker = blocker_counts
        .into_iter()
        .max_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| production_name(left.0).cmp(production_name(right.0)))
        })
        .map(|(item, count)| {
            summary.blocker_count = count;
            item
        });

    summary
}

fn owner_command_center_build_metrics(
    game: &GameState,
    owner: usize,
    bases: &[&smac_core::Base],
) -> CommandCenterBuildMetrics {
    let mut active_count = 0i32;
    let mut total_progress_pct = 0i32;
    let mut max_progress_pct = 0i32;
    let mut low_mineral_bases = 0usize;

    for base in bases {
        if base.production != ProductionItem::CommandCenter {
            continue;
        }
        active_count += 1;
        let cost = game.production_cost(owner, ProductionItem::CommandCenter).max(1);
        let progress_pct = ((base.minerals_stock * 100) / cost).clamp(0, 999);
        total_progress_pct += progress_pct;
        max_progress_pct = max_progress_pct.max(progress_pct);

        let yields = game
            .operational_base_yields(base.id)
            .unwrap_or_else(|| game.base_yields(base.x, base.y));
        if yields.minerals <= 1 || game.base_mineral_margin(base.id).unwrap_or_default() <= 0 {
            low_mineral_bases += 1;
        }
    }

    CommandCenterBuildMetrics {
        avg_progress_pct: if active_count > 0 {
            total_progress_pct / active_count
        } else {
            0
        },
        max_progress_pct,
        low_mineral_bases,
    }
}

impl OwnerCommandCenterTurnFlow {
    fn observe_turn(
        &mut self,
        game: &GameState,
        starts: &[ActiveCommandCenterBaseStart],
        traces: &[CommandCenterTurnTrace],
        economy_signals: OwnerTurnEconomySignals,
    ) {
        let trace_map: HashMap<usize, &CommandCenterTurnTrace> =
            traces.iter().map(|trace| (trace.base_id, trace)).collect();
        for start in starts {
            let Some(base) = game.base(start.base_id) else {
                self.lost_base_turns += 1;
                continue;
            };
            if base.facilities.contains(&Facility::CommandCenter) {
                self.completed_turns += 1;
            } else if base.production == ProductionItem::CommandCenter {
                self.retained_active_turns += 1;
            } else {
                self.switched_turns += 1;
            }
            let end_stock = base.minerals_stock;
            self.base_turns += 1;
            self.total_start_stock += start.start_stock;
            self.total_end_stock += end_stock;
            self.total_yield_minerals += start.yield_minerals;
            self.total_mineral_margin += start.mineral_margin;
            if let Some(trace) = trace_map.get(&start.base_id) {
                self.trace_turns += 1;
                self.total_post_production_stock += trace.post_production_stock;
                self.total_post_interdiction_stock += trace.post_interdiction_stock;
                self.total_exact_upkeep_drain += trace.upkeep_drain;
                if let Some(order_index) = trace.upkeep_order_index {
                    self.drained_trace_turns += 1;
                    self.total_upkeep_order_index += order_index as i32;
                }
            }
            if end_stock < start.start_stock {
                self.loss_turns += 1;
                if start.yield_minerals > 0 {
                    self.positive_yield_loss_turns += 1;
                }
                if start.intercepted_freight_routes > 0 {
                    self.loss_with_intercepted_freight += 1;
                }
                if start.collapsing_freight_routes > 0 {
                    self.loss_with_collapsing_freight += 1;
                }
                if trace_map
                    .get(&start.base_id)
                    .map(|trace| trace.upkeep_drain > 0)
                    .unwrap_or(start.estimated_support_drain > 0)
                {
                    self.loss_with_support_drain += 1;
                }
                if base_had_production_reset(game, &start.base_name) {
                    self.loss_with_production_reset += 1;
                }
                if economy_signals.bankruptcy {
                    self.loss_with_bankruptcy += 1;
                }
                if economy_signals.emergency_support_payment {
                    self.loss_with_emergency_support += 1;
                }
                if economy_signals.support_famine {
                    self.loss_with_support_famine += 1;
                }
            }
        }
    }

    fn avg_start_stock(self) -> i32 {
        average_i32(self.total_start_stock, self.base_turns)
    }

    fn avg_end_stock(self) -> i32 {
        average_i32(self.total_end_stock, self.base_turns)
    }

    fn avg_yield_minerals(self) -> i32 {
        average_i32(self.total_yield_minerals, self.base_turns)
    }

    fn avg_mineral_margin(self) -> i32 {
        average_i32(self.total_mineral_margin, self.base_turns)
    }

    fn avg_post_production_stock(self) -> i32 {
        average_i32(self.total_post_production_stock, self.trace_turns)
    }

    fn avg_post_interdiction_stock(self) -> i32 {
        average_i32(self.total_post_interdiction_stock, self.trace_turns)
    }

    fn avg_exact_upkeep_drain(self) -> i32 {
        average_i32(self.total_exact_upkeep_drain, self.trace_turns)
    }

    fn avg_upkeep_order_index(self) -> i32 {
        average_i32(self.total_upkeep_order_index, self.drained_trace_turns)
    }
}

fn base_had_production_reset(game: &GameState, base_name: &str) -> bool {
    game.log.iter().filter(|entry| entry.turn == game.turn).any(|entry| {
        entry.message.contains(base_name)
            && (entry.message.contains("switched production to")
                || entry.message.contains("promoted "))
    })
}

fn owner_peak_base_stress(game: &GameState, owner: usize, turn: usize) -> OwnerPeakBaseStress {
    game.bases_for(owner)
        .into_iter()
        .map(|base| OwnerPeakBaseStress {
            facilities: base.facilities.len(),
            facility_upkeep: base_facility_upkeep(base),
            optional_upkeep: base_optional_facility_upkeep(base),
            turn,
        })
        .max()
        .unwrap_or(OwnerPeakBaseStress {
            turn,
            ..OwnerPeakBaseStress::default()
        })
}

fn owner_peak_support(game: &GameState, owner: usize, turn: usize) -> OwnerPeakSupport {
    let support = game.faction_support_summary(owner);
    let active_former_builds = game
        .bases_for(owner)
        .into_iter()
        .filter(|base| base.production == ProductionItem::Former)
        .count();
    let mut colony_pods = 0usize;
    let mut formers = 0usize;
    let mut probes = 0usize;
    let mut combat_units = 0usize;
    let mut formers_on_base = 0usize;
    let mut formers_in_field = 0usize;
    let mut field_formers_with_nearby_work = 0usize;
    let mut field_formers_saturated = 0usize;
    let mut saturated_formers_near_base = 0usize;
    let mut saturated_formers_far_from_base = 0usize;
    for unit in game.live_units_for(owner) {
        match unit.kind {
            smac_core::UnitKind::ColonyPod | smac_core::UnitKind::SeaColonyPod => {
                colony_pods += 1;
            }
            smac_core::UnitKind::Former => {
                formers += 1;
                let on_base = game
                    .tile(unit.x, unit.y)
                    .map(|tile| tile.base.is_some())
                    .unwrap_or(false);
                if on_base {
                    formers_on_base += 1;
                } else {
                    formers_in_field += 1;
                    if former_has_nearby_work(game, unit.x, unit.y) {
                        field_formers_with_nearby_work += 1;
                    } else {
                        field_formers_saturated += 1;
                        if nearest_owned_base_distance(game, owner, unit.x, unit.y) <= 3 {
                            saturated_formers_near_base += 1;
                        } else {
                            saturated_formers_far_from_base += 1;
                        }
                    }
                }
            }
            smac_core::UnitKind::ProbeTeam => {
                probes += 1;
            }
            _ => {
                combat_units += 1;
            }
        }
    }
    OwnerPeakSupport {
        supported_units: support.supported_units,
        unit_upkeep: support.unit_upkeep,
        colony_pods,
        formers,
        probes,
        combat_units,
        formers_on_base,
        formers_in_field,
        field_formers_with_nearby_work,
        field_formers_saturated,
        saturated_formers_near_base,
        saturated_formers_far_from_base,
        active_former_builds,
        turn,
    }
}

fn former_has_nearby_work(game: &GameState, x: usize, y: usize) -> bool {
    for dy in -1isize..=1 {
        for dx in -1isize..=1 {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            let Some(tile) = game.tile(nx, ny) else {
                continue;
            };
            if !tile.terrain.is_land() || tile.improvement.is_some() {
                continue;
            }
            if !game.tile_potential_improvements(nx, ny).is_empty() {
                return true;
            }
        }
    }
    false
}

fn nearest_owned_base_distance(game: &GameState, owner: usize, x: usize, y: usize) -> usize {
    game.bases_for(owner)
        .into_iter()
        .map(|base| base.x.abs_diff(x) + base.y.abs_diff(y))
        .min()
        .unwrap_or(usize::MAX)
}

impl OwnerPeakBaseStress {
    fn max(self, other: Self) -> Self {
        if other > self {
            other
        } else {
            self
        }
    }
}

impl PartialEq for OwnerPeakBaseStress {
    fn eq(&self, other: &Self) -> bool {
        self.facilities == other.facilities
            && self.facility_upkeep == other.facility_upkeep
            && self.optional_upkeep == other.optional_upkeep
            && self.turn == other.turn
    }
}

impl Eq for OwnerPeakBaseStress {}

impl PartialOrd for OwnerPeakBaseStress {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OwnerPeakBaseStress {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.facility_upkeep, self.optional_upkeep, self.facilities, self.turn)
            .cmp(&(other.facility_upkeep, other.optional_upkeep, other.facilities, other.turn))
    }
}

impl OwnerPeakSupport {
    fn max(self, other: Self) -> Self {
        if other > self {
            other
        } else {
            self
        }
    }
}

impl PartialEq for OwnerPeakSupport {
    fn eq(&self, other: &Self) -> bool {
        self.supported_units == other.supported_units
            && self.unit_upkeep == other.unit_upkeep
            && self.colony_pods == other.colony_pods
            && self.formers == other.formers
            && self.probes == other.probes
            && self.combat_units == other.combat_units
            && self.formers_on_base == other.formers_on_base
            && self.formers_in_field == other.formers_in_field
            && self.field_formers_with_nearby_work == other.field_formers_with_nearby_work
            && self.field_formers_saturated == other.field_formers_saturated
            && self.saturated_formers_near_base == other.saturated_formers_near_base
            && self.saturated_formers_far_from_base == other.saturated_formers_far_from_base
            && self.active_former_builds == other.active_former_builds
            && self.turn == other.turn
    }
}

impl Eq for OwnerPeakSupport {}

impl PartialOrd for OwnerPeakSupport {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OwnerPeakSupport {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.unit_upkeep
            .cmp(&other.unit_upkeep)
            .then_with(|| self.supported_units.cmp(&other.supported_units))
            .then_with(|| self.combat_units.cmp(&other.combat_units))
            .then_with(|| self.field_formers_saturated.cmp(&other.field_formers_saturated))
            .then_with(|| {
                self.field_formers_with_nearby_work
                    .cmp(&other.field_formers_with_nearby_work)
            })
            .then_with(|| {
                self.saturated_formers_far_from_base
                    .cmp(&other.saturated_formers_far_from_base)
            })
            .then_with(|| {
                self.saturated_formers_near_base
                    .cmp(&other.saturated_formers_near_base)
            })
            .then_with(|| self.formers_in_field.cmp(&other.formers_in_field))
            .then_with(|| self.formers_on_base.cmp(&other.formers_on_base))
            .then_with(|| self.active_former_builds.cmp(&other.active_former_builds))
            .then_with(|| self.formers.cmp(&other.formers))
            .then_with(|| self.colony_pods.cmp(&other.colony_pods))
            .then_with(|| self.probes.cmp(&other.probes))
            .then_with(|| self.turn.cmp(&other.turn))
    }
}

fn base_facility_upkeep(base: &smac_core::Base) -> i32 {
    base.facilities
        .iter()
        .copied()
        .map(facility_maintenance)
        .sum()
}

fn base_optional_facility_upkeep(base: &smac_core::Base) -> i32 {
    base.facilities
        .iter()
        .copied()
        .filter(|facility| is_optional_facility(*facility))
        .map(facility_maintenance)
        .sum()
}

fn is_optional_facility(facility: Facility) -> bool {
    matches!(
        facility,
        Facility::NetworkNode
            | Facility::TradeExchange
            | Facility::FreightDepot
            | Facility::PatrolGrid
            | Facility::MilitaryAcademy
            | Facility::SensorArray
            | Facility::TransitHub
            | Facility::PsiBeacon
            | Facility::ForwardDepot
            | Facility::HologramTheatre
            | Facility::BioenhancementCenter
            | Facility::ResearchHospital
    )
}

fn nearest_base_gap(game: &GameState, owner: usize, other: usize) -> i32 {
    let our_bases = game.bases_for(owner);
    let their_bases = game.bases_for(other);
    our_bases
        .iter()
        .flat_map(|base| {
            their_bases
                .iter()
                .map(move |other_base| game.distance(base.x, base.y, other_base.x, other_base.y))
        })
        .min()
        .unwrap_or(99)
}

fn yn(value: bool) -> &'static str {
    if value { "y" } else { "n" }
}

fn blocker_label(item: Option<ProductionItem>, count: usize) -> String {
    match item {
        Some(item) if count > 0 => format!("{}x{count}", production_name(item)),
        _ => "-".to_string(),
    }
}

fn average_i32(total: i32, count: usize) -> i32 {
    if count > 0 {
        total / count as i32
    } else {
        0
    }
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--turns" => config.turns = parse_value(args.next(), "--turns")?,
            "--width" => config.width = parse_value(args.next(), "--width")?,
            "--height" => config.height = parse_value(args.next(), "--height")?,
            "--start-seed" => config.start_seed = parse_value(args.next(), "--start-seed")?,
            "--count" => config.count = parse_value(args.next(), "--count")?,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    if config.turns == 0 {
        return Err("--turns must be greater than 0".to_string());
    }
    if config.width == 0 || config.height == 0 {
        return Err("--width and --height must be greater than 0".to_string());
    }
    if config.count == 0 {
        return Err("--count must be greater than 0".to_string());
    }

    Ok(config)
}

fn parse_value<T: std::str::FromStr>(value: Option<String>, flag: &str) -> Result<T, String> {
    let Some(value) = value else {
        return Err(format!("{flag} requires a value"));
    };
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {flag}: {value}"))
}

fn print_usage() {
    println!(
        "Usage: cargo run -p smac_core --bin autoplay_sweep -- [--turns N] [--width N] [--height N] [--start-seed N] [--count N]"
    );
}

fn game_over_label(game_over: GameOver) -> &'static str {
    match game_over {
        GameOver::PlayerWonConquest => "player-conquest",
        GameOver::PlayerWonEconomic => "player-economic",
        GameOver::PlayerWonTranscendence => "player-transcend",
        GameOver::PlayerWonSpaceTranscendence => "player-space",
        GameOver::PlayerWonBlackHoleHarvesting => "player-singularity",
        GameOver::AiWonConquest => "ai-conquest",
        GameOver::AiWonEconomic => "ai-economic",
        GameOver::AiWonTranscendence => "ai-transcend",
        GameOver::AiWonSpaceTranscendence => "ai-space",
        GameOver::AiWonBlackHoleHarvesting => "ai-singularity",
        GameOver::PlayerLost => "player-lost",
        GameOver::DiplomaticVictory => "diplomatic-victory",
        GameOver::CouncilGovernorElected => "governor-elected",
        GameOver::PlanetUnited => "planet-united",
    }
}
