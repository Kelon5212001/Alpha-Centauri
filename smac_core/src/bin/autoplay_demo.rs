use smac_core::content_api::tech_name;
use smac_core::{offense_readiness_for_owner, EventCategory, GameOver, GameState, GameStateSnapshot};
use std::env;
use std::fs;
use std::path::PathBuf;

struct Config {
    turns: usize,
    width: usize,
    height: usize,
    seed: u32,
    summary_every: usize,
    save_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            turns: 100,
            width: 20,
            height: 20,
            seed: 7,
            summary_every: 10,
            save_path: None,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let config = parse_args()?;
    let mut game = GameState::new_game(config.width, config.height, config.seed);

    println!(
        "Autoplay demo starting: {} turns on {}x{} map with seed {}.",
        config.turns, config.width, config.height, config.seed
    );
    print_turn_summary(&game, 0);

    let mut completed_turns = 0usize;
    while completed_turns < config.turns && game.game_over.is_none() {
        game.run_autoplay_mission_year();
        completed_turns += 1;

        if completed_turns == 1
            || completed_turns % config.summary_every == 0
            || game.game_over.is_some()
        {
            print_turn_summary(&game, completed_turns);
        }
    }

    if let Some(path) = &config.save_path {
        save_snapshot(&game, path)?;
    }

    match game.game_over {
        Some(outcome) => {
            println!(
                "Demo finished early after {completed_turns} turns: {}.",
                game_over_label(outcome)
            );
        }
        None => {
            println!(
                "Demo completed {} turns without a terminal outcome.",
                completed_turns
            );
        }
    }

    Ok(())
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--turns" => config.turns = parse_value(args.next(), "--turns")?,
            "--width" => config.width = parse_value(args.next(), "--width")?,
            "--height" => config.height = parse_value(args.next(), "--height")?,
            "--seed" => config.seed = parse_value(args.next(), "--seed")?,
            "--summary-every" => {
                config.summary_every = parse_value(args.next(), "--summary-every")?
            }
            "--save-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--save-path requires a file path".to_string())?;
                config.save_path = Some(PathBuf::from(value));
            }
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
    if config.summary_every == 0 {
        return Err("--summary-every must be greater than 0".to_string());
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
        "Usage: cargo run -p smac_core --bin autoplay_demo -- [--turns N] [--width N] [--height N] [--seed N] [--summary-every N] [--save-path PATH]"
    );
}

fn print_turn_summary(game: &GameState, completed_turns: usize) {
    println!(
        "Turn {:>3} | Mission Year {} | routes {} | projects {}",
        completed_turns,
        2100 + game.turn,
        game.convoy_routes.len(),
        game.built_secret_projects.len()
    );
    println!("  {}", owner_summary(game, game.player_owner()));
    println!("  {}", owner_summary(game, game.ai_owner()));

    let new_entries: Vec<_> = game
        .log
        .iter()
        .filter(|entry| entry.turn == game.turn)
        .collect();
    if !new_entries.is_empty() {
        let general = new_entries
            .iter()
            .filter(|entry| entry.category == EventCategory::General)
            .count();
        let crisis = new_entries
            .iter()
            .filter(|entry| entry.category == EventCategory::Crisis)
            .count();
        let diplomacy = new_entries
            .iter()
            .filter(|entry| entry.category == EventCategory::Diplomacy)
            .count();
        let economics = new_entries
            .iter()
            .filter(|entry| entry.category == EventCategory::Economics)
            .count();
        let projects = new_entries
            .iter()
            .filter(|entry| entry.category == EventCategory::SecretProject)
            .count();
        let tactics = new_entries
            .iter()
            .filter(|entry| entry.message.contains("TACTICS:"))
            .count();
        let combats = new_entries
            .iter()
            .filter(|entry| {
                entry.message.contains("COMBAT:") || entry.message.contains("BOMBARDMENT:")
            })
            .count();
        println!(
            "  New events: {} total (general {}, crisis {}, diplomacy {}, economics {}, projects {}, tactics {}, combats {})",
            new_entries.len(),
            general,
            crisis,
            diplomacy,
            economics,
            projects,
            tactics,
            combats
        );

        let highlights: Vec<&str> = new_entries
            .iter()
            .filter(|entry| {
                entry.category != EventCategory::General
                    || entry.message.contains("founded")
                    || entry.message.contains("captured")
                    || entry.message.contains("COMBAT:")
                    || entry.message.contains("BOMBARDMENT:")
                    || entry.message.contains("TACTICS:")
                    || entry.message.contains("VICTORY")
                    || entry.message.contains("DEFEAT")
            })
            .map(|entry| entry.message.as_str())
            .take(3)
            .collect();
        if !highlights.is_empty() {
            println!("  Highlights: {}", highlights.join(" | "));
        }
    }
}

fn owner_summary(game: &GameState, owner: usize) -> String {
    let bases = game.bases_for(owner).len();
    let units = game.live_units_for(owner).len();
    let Some(faction) = game.faction(owner) else {
        return format!("owner {owner}: unavailable");
    };
    let research = game
        .research_progress(owner)
        .map(|(tech, progress, cost)| format!("{} {}/{}", tech_name(tech), progress, cost))
        .unwrap_or_else(|| "none".to_string());
    let offense = offense_readiness_for_owner(game, owner);

    format!(
        "{} -> bases {}, units {}, energy {}, known techs {}, research {}, offense atk {} min {} ready {} target {} free {}/{} mobile {} def {} res {}",
        faction.name,
        bases,
        units,
        faction.energy,
        faction.known_techs.len(),
        research,
        offense.attack_bias,
        offense.minimum_attack_group_size,
        offense.can_form_attack_group as u8,
        offense.has_offensive_target as u8,
        offense.available_attackers,
        offense.total_combat_units,
        offense.mobile_combat_units,
        offense.units_committed_to_defense,
        offense.reserved_defenders
    )
}

fn save_snapshot(game: &GameState, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create save directory {}: {err}", parent.display()))?;
        }
    }

    let mut snapshot = GameStateSnapshot::from(game);
    snapshot.save_name = Some(format!("Autoplay Demo Turn {}", game.turn));
    snapshot
        .save_to_path(path)
        .map_err(|err| format!("save snapshot {}: {err}", path.display()))?;
    println!("Saved final snapshot to {}.", path.display());
    Ok(())
}

fn game_over_label(game_over: GameOver) -> &'static str {
    match game_over {
        GameOver::PlayerWonConquest => "player conquest victory",
        GameOver::PlayerWonEconomic => "player economic victory",
        GameOver::PlayerWonTranscendence => "player transcendence victory",
        GameOver::PlayerWonSpaceTranscendence => "player space victory",
        GameOver::PlayerWonBlackHoleHarvesting => "player singularity victory",
        GameOver::AiWonConquest => "ai conquest victory",
        GameOver::AiWonEconomic => "ai economic victory",
        GameOver::AiWonTranscendence => "ai transcendence victory",
        GameOver::AiWonSpaceTranscendence => "ai space victory",
        GameOver::AiWonBlackHoleHarvesting => "ai singularity victory",
        GameOver::PlayerLost => "player defeat",
        GameOver::DiplomaticVictory => "diplomatic victory",
        GameOver::CouncilGovernorElected => "governor elected",
        GameOver::PlanetUnited => "planet united",
    }
}
