use smac_core::{GameOver, GameState};
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
}

struct RunSummary {
    seed: u32,
    completed_turns: usize,
    outcome: Option<GameOver>,
    routes: usize,
    projects: usize,
    nearest_base_gap: i32,
    combats: usize,
    captures: usize,
    war_declarations: usize,
    bankruptcies: usize,
    famines: usize,
    starvation_famines: usize,
    support_famines: usize,
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
    let mut total_combats = 0usize;
    let mut total_captures = 0usize;
    let mut total_wars = 0usize;
    let mut total_bankruptcies = 0usize;
    let mut total_famines = 0usize;
    let mut total_starvation_famines = 0usize;
    let mut total_support_famines = 0usize;
    let mut player_zero_unit_runs = 0usize;
    let mut ai_zero_unit_runs = 0usize;
    let mut player_low_expansion_runs = 0usize;
    let mut ai_low_expansion_runs = 0usize;

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
        total_combats += summary.combats;
        total_captures += summary.captures;
        total_wars += summary.war_declarations;
        total_bankruptcies += summary.bankruptcies;
        total_famines += summary.famines;
        total_starvation_famines += summary.starvation_famines;
        total_support_famines += summary.support_famines;
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

        println!(
            "seed {:>3} | turns {:>3} | outcome {:<12} | routes {:>2} projects {:>2} gap {:>2} combats {:>3} caps {:>2} wars {:>2} | p bases {:>2} units {:>2}/{:>2} tech {:>2} energy {:>4} food {:>4} frontier {:>2} unrest {:>2}/{:<2} supp {:>2}/{:<2} | ai bases {:>2} units {:>2}/{:>2} tech {:>2} energy {:>4} food {:>4} frontier {:>2} unrest {:>2}/{:<2} supp {:>2}/{:<2} | bankruptcy {:>2} famine {:>2} starve {:>2} support {:>2}",
            summary.seed,
            summary.completed_turns,
            summary
                .outcome
                .map(game_over_label)
                .unwrap_or("none"),
            summary.routes,
            summary.projects,
            summary.nearest_base_gap,
            summary.combats,
            summary.captures,
            summary.war_declarations,
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
            summary.bankruptcies,
            summary.famines,
            summary.starvation_famines,
            summary.support_famines
        );
    }

    println!(
        "aggregate | terminal {} / {} | combats {} | captures {} | wars {} | bankruptcies {} | famines {} | starvation {} | support {} | player low-expansion {} | ai low-expansion {} | player zero-unit {} | ai zero-unit {}",
        terminal_runs,
        config.count,
        total_combats,
        total_captures,
        total_wars,
        total_bankruptcies,
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
    let mut combats = 0usize;
    let mut captures = 0usize;
    let mut war_declarations = 0usize;
    let mut bankruptcies = 0usize;
    let mut famines = 0usize;
    let mut starvation_famines = 0usize;
    let mut support_famines = 0usize;

    while completed_turns < config.turns && game.game_over.is_none() {
        let log_start = game.log.len();
        game.run_autoplay_mission_year();
        completed_turns += 1;

        for entry in &game.log[log_start.min(game.log.len())..] {
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
        combats,
        captures,
        war_declarations,
        bankruptcies,
        famines,
        starvation_famines,
        support_famines,
        player: owner_metrics(&game, game.player_owner()),
        ai: owner_metrics(&game, game.ai_owner()),
    }
}

fn owner_metrics(game: &GameState, owner: usize) -> OwnerMetrics {
    let bases = game.bases_for(owner);
    let unrest_values: Vec<i32> = bases.iter().map(|base| game.base_unrest(base.id)).collect();
    let faction = game.faction(owner);
    let support = game.faction_support_summary(owner);
    let live_units = game.live_units_for(owner);

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
    }
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
