use std::{env, path::PathBuf, process::ExitCode, time::Instant};

use pw_harness::{load_log, replay_log, run_simulation, write_run, RunConfig};

fn main() -> ExitCode {
    match execute(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => { eprintln!("{message}"); ExitCode::from(2) }
    }
}

fn execute(arguments: Vec<String>) -> Result<(), String> {
    let Some(command) = arguments.first().map(String::as_str) else { return Err(usage()); };
    match command {
        "run" => {
            let config = parse_run(&arguments[1..])?;
            let started = Instant::now();
            let run = run_simulation(config)?;
            for (turn, hash) in &run.stored.log.turn_hashes {
                let resolved_turn = turn.0 + 1;
                if resolved_turn % 100 == 0 { println!("turn {} hash {}", resolved_turn, hash.0); }
            }
            let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;
            let metrics = run.metrics();
            println!("FINAL {}", run.final_hash().0);
            println!("cities {} population {} average_pressure {} crises {} collapses {} ms/turn {:.3}", metrics.cities, metrics.population, metrics.average_pressure, metrics.crises, metrics.collapses, elapsed_ms / f64::from(config.turns));
            for (civilization, state) in &run.final_state.civilizations {
                let cities = run.final_state.cities.values().filter(|city| city.owner == *civilization).count();
                let population: u32 = run.final_state.cities.values().filter(|city| city.owner == *civilization).map(|city| city.population).sum();
                println!("civ {} cities {} population {} cohesion {} collapsed {}", civilization.0, cities, population, state.cohesion, state.frozen);
            }
            write_run(&PathBuf::from("out").join(config.seed.to_string()), &run)?;
            Ok(())
        }
        "replay" => {
            let Some(path) = arguments.get(1) else { return Err(usage()); };
            println!("FINAL {}", replay_log(&load_log(PathBuf::from(path).join("log.json"))?)?.0);
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn parse_run(arguments: &[String]) -> Result<RunConfig, String> {
    let mut config = RunConfig::default();
    let mut index = 0;
    while index < arguments.len() {
        let value = arguments.get(index + 1).ok_or_else(usage)?;
        match arguments[index].as_str() {
            "--seed" => config.seed = value.parse().map_err(|_| "--seed must be a u64".to_string())?,
            "--civs" => config.civilizations = value.parse().map_err(|_| "--civs must be a u32".to_string())?,
            "--turns" => config.turns = value.parse().map_err(|_| "--turns must be a u32".to_string())?,
            _ => return Err(usage()),
        }
        index += 2;
    }
    if config.civilizations == 0 || config.turns == 0 { return Err("--civs and --turns must be greater than zero".into()); }
    Ok(config)
}

fn usage() -> String { "usage: pw-harness run --seed N --civs 8 --turns 1000 | pw-harness replay out/<seed>".into() }
