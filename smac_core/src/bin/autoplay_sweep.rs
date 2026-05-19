use smac_core::content_api::facility_maintenance;
use smac_core::{offense_readiness_for_owner, Facility, GameOver, GameState, ProductionItem, Tech};
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
    turn: usize,
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
            "seed {:>3} | turns {:>3} | outcome {:<12} | routes {:>2} projects {:>2} gap {:>2} raids {:>2} combats {:>3} caps {:>2} wars {:>2} | p off {:>3}/{:>3} bases {:>2} units {:>2}/{:>2} tech {:>2} energy {:>4} food {:>4} frontier {:>2} unrest {:>2}/{:<2} supp {:>2}/{:<2} cc {:>2} th {:>2} ib {} ca {} pk {:>2}/{:<2} upk {:>2}+{:>2}+{:>2} base {:>2}f/{:>2}m/{:>2}o pk {:>2}f/{:>2}m/{:>2}o@{:>3} | ai off {:>3}/{:>3} bases {:>2} units {:>2}/{:>2} tech {:>2} energy {:>4} food {:>4} frontier {:>2} unrest {:>2}/{:<2} supp {:>2}/{:<2} cc {:>2} th {:>2} ib {} ca {} pk {:>2}/{:<2} upk {:>2}+{:>2}+{:>2} base {:>2}f/{:>2}m/{:>2}o pk {:>2}f/{:>2}m/{:>2}o@{:>3} | bank {:>2} fac {:>2} unit {:>2} em {:>2}/{:>3} famine {:>2} starve {:>2} support {:>2}",
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
            summary.bankruptcies,
            summary.facility_bankruptcies,
            summary.unit_bankruptcies,
            summary.emergency_support_payments,
            summary.emergency_support_energy,
            summary.famines,
            summary.starvation_famines,
            summary.support_famines
        );
    }

    println!(
        "aggregate | terminal {} / {} | raids {} | combats {} | captures {} | wars {} | p off {}/{} | ai off {}/{} | bankruptcies {} fac {} unit {} em {}/{} | famines {} | starvation {} | support {} | player low-expansion {} | ai low-expansion {} | player zero-unit {} | ai zero-unit {}",
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

    while completed_turns < config.turns && game.game_over.is_none() {
        let player_readiness = offense_readiness_for_owner(&game, game.player_owner());
        let ai_readiness = offense_readiness_for_owner(&game, game.ai_owner());
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
        famines,
        starvation_famines,
        support_famines,
        emergency_support_payments,
        emergency_support_energy,
        player_ready_turns,
        player_target_turns,
        ai_ready_turns,
        ai_target_turns,
        player: owner_metrics(&game, game.player_owner(), player_peak_base_stress, player_peak_support),
        ai: owner_metrics(&game, game.ai_owner(), ai_peak_base_stress, ai_peak_support),
    }
}

fn owner_metrics(
    game: &GameState,
    owner: usize,
    peak_base_stress: OwnerPeakBaseStress,
    peak_support: OwnerPeakSupport,
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
    }
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
    OwnerPeakSupport {
        supported_units: support.supported_units,
        unit_upkeep: support.unit_upkeep,
        turn,
    }
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
        (self.unit_upkeep, self.supported_units, self.turn)
            .cmp(&(other.unit_upkeep, other.supported_units, other.turn))
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
