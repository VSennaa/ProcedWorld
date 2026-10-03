//! Deterministic bot simulation and replay support for the command-line harness.

use std::{collections::BTreeMap, fs, path::Path};
use serde::{Deserialize, Serialize};
use pw_engine::{bots::BotT0, hash::StateHash, ids::{CivId, TileIndex, TurnNumber}, mapgen::{generate, Catalog, WorldParams}, rng::Rng, world::{replay, step, CivilizationState, CommandLog, DomainEvent, SimulationVersions, TileState, TileYields, WorldId, WorldSnapshot, WorldState, TERRAIN_COAST, TERRAIN_DESERT, TERRAIN_FOREST, TERRAIN_JUNGLE, TERRAIN_OCEAN, TERRAIN_PLAINS, TERRAIN_STEPPE, TERRAIN_SWAMP}};

pub const SNAPSHOT_INTERVAL: u32 = 100;
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct RunConfig { pub seed: u64, pub civilizations: u32, pub turns: u32 }
impl Default for RunConfig { fn default() -> Self { Self { seed: 1, civilizations: 8, turns: 1_000 } } }
#[derive(Clone, Debug, Serialize, Deserialize)] pub struct StoredRun { pub snapshot: WorldSnapshot, pub log: CommandLog, pub home_tiles: BTreeMap<CivId, TileIndex> }
#[derive(Clone, Debug)] pub struct SimulationRun { pub stored: StoredRun, pub final_state: WorldState, pub final_events: Vec<DomainEvent> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Metrics { pub cities: u32, pub population: u32, pub average_pressure: u8, pub crises: u32, pub collapses: u32 }

impl SimulationRun {
    pub fn final_hash(&self) -> StateHash { self.final_state.state_hash() }
    pub fn metrics(&self) -> Metrics {
        let cities = self.final_state.cities.len() as u32;
        let population = self.final_state.cities.values().map(|city| city.population).sum();
        let pressure: u32 = self.final_state.civilizations.values().map(|civ| u32::from(civ.crisis_pressure)).sum();
        Metrics { cities, population, average_pressure: if self.final_state.civilizations.is_empty() { 0 } else { (pressure / self.final_state.civilizations.len() as u32) as u8 }, crises: self.final_state.cities.values().filter(|city| city.crisis_turns >= 2).count() as u32, collapses: self.final_state.civilizations.values().filter(|civ| civ.frozen).count() as u32 }
    }
}

pub fn run_simulation(config: RunConfig) -> Result<SimulationRun, String> {
    if config.civilizations == 0 || config.turns == 0 { return Err("civilizations and turns must be greater than zero".into()); }
    let (mut state, versions, homes) = initial_world(config.seed, config.civilizations)?;
    let snapshot = WorldSnapshot::new(state.clone(), versions);
    let mut log = CommandLog::default(); let bot = BotT0; let mut command_id = 1_u64; let mut final_events = Vec::new();
    for _ in 0..config.turns {
        let mut commands = Vec::new();
        for civilization in rotating_order(config.seed, state.turn, &homes) {
            for proposal in bot.decide(&state, civilization, homes[&civilization]) {
                commands.push(proposal.accept(&state, civilization, command_id, commands.len() as u64 + 1));
                command_id = command_id.checked_add(1).ok_or_else(|| "command id exhausted".to_string())?;
            }
        }
        for command in commands.iter().cloned() { log.append(command); }
        let turn = state.turn; let result = step(&state, &commands, state.seed, &snapshot.versions);
        log.record_turn(turn, result.state_hash); final_events = result.events; state = result.state;
    }
    Ok(SimulationRun { stored: StoredRun { snapshot, log, home_tiles: homes }, final_state: state, final_events })
}

pub fn replay_log(stored: &StoredRun) -> Result<StateHash, String> { replay(&stored.snapshot, &stored.log).map(|result| result.state.state_hash()).map_err(|error| format!("replay failed: {error:?}")) }
pub fn write_run(directory: &Path, run: &SimulationRun) -> Result<(), String> {
    let snapshots = directory.join("snapshots");
    fs::create_dir_all(&snapshots).map_err(|error| error.to_string())?;
    write_json(directory.join("log.json"), &run.stored)?;
    write_json(snapshots.join("initial.json"), &run.stored.snapshot)?;
    let mut state = run.stored.snapshot.state.clone();
    for (turn, expected) in &run.stored.log.turn_hashes {
        let commands: Vec<_> = run.stored.log.commands.iter().filter(|command| command.turn == *turn).cloned().collect();
        let result = step(&state, &commands, state.seed, &run.stored.snapshot.versions);
        if result.state_hash != *expected { return Err(format!("cannot snapshot divergent turn {}", turn.0)); }
        state = result.state;
        if state.turn.0 % SNAPSHOT_INTERVAL == 0 { write_json(snapshots.join(format!("turn-{}.json", state.turn.0)), &WorldSnapshot::new(state.clone(), run.stored.snapshot.versions.clone()))?; }
    }
    write_json(snapshots.join("final.json"), &WorldSnapshot::new(run.final_state.clone(), run.stored.snapshot.versions.clone()))
}
pub fn load_log(path: impl AsRef<Path>) -> Result<StoredRun, String> { serde_json::from_str(&fs::read_to_string(path).map_err(|error| error.to_string())?).map_err(|error| error.to_string()) }
fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<(), String> { fs::write(path, serde_json::to_string_pretty(value).map_err(|error| error.to_string())?).map_err(|error| error.to_string()) }

fn initial_world(seed: u64, civilizations: u32) -> Result<(WorldState, SimulationVersions, BTreeMap<CivId, TileIndex>), String> {
    let catalog = Catalog::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data/catalogs")).map_err(|error| format!("catalog load failed: {error:?}"))?;
    let generated = generate(WorldParams::for_civilizations(civilizations), seed, &catalog).map_err(|error| format!("map generation failed: {error:?}"))?;
    let mut civilization_states = BTreeMap::new(); let mut homes = BTreeMap::new();
    for start in &generated.starting_points { let id = CivId(start.civilization); let mut civilization = CivilizationState::default(); civilization.researched_technologies.insert("tech.foraging".into()); civilization_states.insert(id, civilization); homes.insert(id, start.tile); }
    let tiles = generated.tiles.iter().map(|tile| TileState { terrain: terrain(&tile.biome_id), river: false, yields: yields(&tile.biome_id) }).collect();
    let state = WorldState { world_id: WorldId(seed), turn: TurnNumber::ZERO, seed, ruleset: generated.ruleset_ref.clone(), schema_version: 1, map_width: generated.grid.width, tiles, civilizations: civilization_states, cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility: BTreeMap::new(), ledger: Vec::new() };
    Ok((state, SimulationVersions { ruleset: generated.ruleset_ref, resolver_version: 1 }, homes))
}
fn rotating_order(seed: u64, turn: TurnNumber, homes: &BTreeMap<CivId, TileIndex>) -> Vec<CivId> { let mut order: Vec<_> = homes.keys().copied().collect(); let mut rng = Rng::derive(seed, "bot-order"); for index in (1..order.len()).rev() { order.swap(index, rng.below(index as u32 + 1) as usize); } if !order.is_empty() { let offset = turn.0 as usize % order.len(); order.rotate_left(offset); } order }
fn terrain(biome: &str) -> u8 { match biome { "forest" => TERRAIN_FOREST, "jungle" => TERRAIN_JUNGLE, "swamp" => TERRAIN_SWAMP, "desert" => TERRAIN_DESERT, "steppe" | "savanna" | "tundra" => TERRAIN_STEPPE, "coast" => TERRAIN_COAST, "ocean" => TERRAIN_OCEAN, _ => TERRAIN_PLAINS } }
fn yields(biome: &str) -> TileYields { match biome { "forest" => TileYields { food: 2, production: 2, wealth: 0, knowledge: 1, culture: 0 }, "jungle" | "swamp" => TileYields { food: 3, production: 1, wealth: 0, knowledge: 0, culture: 0 }, "desert" => TileYields { food: 0, production: 1, wealth: 1, knowledge: 0, culture: 0 }, "steppe" | "savanna" | "tundra" => TileYields { food: 1, production: 1, wealth: 1, knowledge: 0, culture: 0 }, "coast" => TileYields { food: 2, production: 0, wealth: 1, knowledge: 0, culture: 0 }, "ocean" => TileYields { food: 1, production: 0, wealth: 0, knowledge: 0, culture: 0 }, _ => TileYields { food: 2, production: 1, wealth: 1, knowledge: 0, culture: 0 } } }

#[cfg(test)] mod tests {
    use super::*;

    #[test] fn two_runs_with_same_seed_have_the_same_final_hash() {
        let config = RunConfig { seed: 84, civilizations: 8, turns: 20 };
        assert_eq!(run_simulation(config).unwrap().final_hash(), run_simulation(config).unwrap().final_hash());
    }

    #[test] fn two_hundred_bot_turns_replay_identically() {
        let run = run_simulation(RunConfig { seed: 84, civilizations: 8, turns: 200 }).unwrap();
        assert_eq!(replay_log(&run.stored).unwrap(), run.final_hash());
    }

    #[test] fn one_thousand_bot_turns_complete_without_a_panic() {
        assert_eq!(run_simulation(RunConfig { seed: 84, civilizations: 8, turns: 1_000 }).unwrap().stored.log.turn_hashes.len(), 1_000);
    }
}
