//! Deterministic bot simulation and replay support for the command-line harness.

use std::{collections::BTreeMap, fs, path::Path};
use serde::{Deserialize, Serialize};
use pw_engine::{entropy::{bundled_catalog, respond_commands, EntropyDirector}, diplomacy::{audit_records, DiplomaticAudit}, governor::{Governor, Mandate}, personality::BotPersonality, hash::StateHash, ids::{CivId, TileIndex, TurnNumber}, mapgen::{generate, Catalog, WorldParams}, memory::{CanonicalKind, CanonicalSet}, rng::Rng, world::{replay, step, CivilizationState, CommandLog, CommandOrigin, DomainEvent, SimulationVersions, TileState, TileYields, Visibility, WorldId, WorldSnapshot, WorldState, TERRAIN_COAST, TERRAIN_DESERT, TERRAIN_FOREST, TERRAIN_JUNGLE, TERRAIN_OCEAN, TERRAIN_PLAINS, TERRAIN_STEPPE, TERRAIN_SWAMP}};

pub const SNAPSHOT_INTERVAL: u32 = 100;
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct RunConfig { pub seed: u64, pub civilizations: u32, pub turns: u32 }
impl Default for RunConfig { fn default() -> Self { Self { seed: 1, civilizations: 8, turns: 1_000 } } }
#[derive(Clone, Debug, Serialize, Deserialize)] pub struct StoredRun { pub snapshot: WorldSnapshot, pub log: CommandLog, pub home_tiles: BTreeMap<CivId, TileIndex> }
#[derive(Clone, Debug)] pub struct SimulationRun { pub stored: StoredRun, pub final_state: WorldState, pub final_events: Vec<DomainEvent>, pub audits: Vec<DiplomaticAudit>, pub terms: PressureTerms, pub personalities: BTreeMap<CivId, BotPersonality>, pressure_sum: u64, pressure_samples: u64 }
/// Population-weighted sums of the `P_c` terms over every resolved turn (harness metric only, GDD 12):
/// `D`, `G`, `W`, `E`, `(100 - S)/10` and `-(C - 50)/5`, the administrative load subtracted from `S`,
/// plus pair-turns spent in tension and war.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] pub struct PressureTerms { pub deprivation: i64, pub group_tension: i64, pub war_threat: i64, pub exposure: i64, pub stability: i64, pub cohesion: i64, pub admin_load: i64, pub population: i64, pub tension_pair_turns: u64, pub war_pair_turns: u64 }
impl PressureTerms {
    fn sample(&mut self, state: &WorldState) {
        for (id, city) in &state.cities {
            let Some(civ) = state.civilizations.get(&city.owner) else { continue; };
            let population = i64::from(city.population);
            self.admin_load += population * i64::from(pw_engine::world::administrative_load(state, *id));
            self.deprivation += population * i64::from(city.deprivation);
            self.group_tension += population * i64::from(city.group_tension);
            self.war_threat += population * i64::from(city.war_threat);
            self.exposure += population * i64::from(city.environmental_exposure);
            self.stability += population * ((100 - i64::from(city.stability)) / 10);
            self.cohesion += population * -((i64::from(civ.cohesion) - 50) / 5);
            self.population += population;
        }
        for row in state.diplomacy.relations.values() {
            for record in row.values() {
                match record.state { pw_engine::diplomacy::RelationState::Tension => self.tension_pair_turns += 1, pw_engine::diplomacy::RelationState::War => self.war_pair_turns += 1, _ => {} }
            }
        }
    }
    /// Population-weighted mean of one term sum, in tenths.
    pub fn tenths(&self, sum: i64) -> i64 { if self.population == 0 { 0 } else { sum * 10 / self.population } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct Metrics { pub cities: u32, pub population: u32, pub average_pressure: u8, pub crises: u32, pub collapses: u32, pub events: u32 }

impl SimulationRun {
    pub fn final_hash(&self) -> StateHash { self.final_state.state_hash() }
    pub fn metrics(&self) -> Metrics {
        let cities = self.final_state.cities.len() as u32;
        let population = self.final_state.cities.values().map(|city| city.population).sum();
        let average_pressure = if self.pressure_samples == 0 { 0 } else { (self.pressure_sum / self.pressure_samples).min(100) as u8 };
        Metrics { cities, population, average_pressure, crises: self.final_state.cities.values().filter(|city| city.crisis_turns >= 2).count() as u32, collapses: self.final_state.civilizations.values().filter(|civ| civ.frozen).count() as u32, events: self.stored.log.commands.iter().filter(|command| command.origin == CommandOrigin::Entropy).count() as u32 }
    }
}

pub fn run_simulation(config: RunConfig) -> Result<SimulationRun, String> {
    if config.civilizations == 0 || config.turns == 0 { return Err("civilizations and turns must be greater than zero".into()); }
    let (mut state, versions, homes) = initial_world(config.seed, config.civilizations)?;
    let snapshot = WorldSnapshot::new(state.clone(), versions);
    // Every harness civilization is a bot: its personality, derived from the seed, sets its Governor's
    // Mandate and war policy (B2). Bots have no player to wait for, so they decide as present.
    let personalities = BotPersonality::assign(config.seed, homes.keys().copied());
    let mandates: BTreeMap<CivId, Mandate> = personalities.iter().map(|(civ, personality)| (*civ, personality.mandate(1))).collect();
    let mut log = CommandLog::default(); let catalog = bundled_catalog(); let director = EntropyDirector { catalog: &catalog, decision_port: None }; let mut command_id = 1_u64; let mut final_events = Vec::new(); let mut audits = Vec::new(); let mut pressure_sum = 0_u64; let mut pressure_samples = 0_u64; let mut terms = PressureTerms::default();
    for _ in 0..config.turns {
        let mut commands = Vec::new();
        for civilization in rotating_order(config.seed, state.turn, &homes) {
            let governor = Governor { mandate: &mandates[&civilization], decision_port: None };
            let decision = governor.decide_with(&state, civilization, homes[&civilization], false, command_id, commands.len() as u64 + 1, personalities[&civilization].war_policy());
            command_id = command_id.checked_add(decision.commands.len().max(1) as u64).ok_or_else(|| "command id exhausted".to_string())?;
            commands.extend(decision.commands);
        }
        // The Governor answers pending Entropy events for every civilization; then the director opens new ones.
        for civilization in homes.keys() {
            let responses = respond_commands(&state, *civilization, &mandates[civilization], command_id, commands.len() as u64 + 1);
            command_id = command_id.saturating_add(responses.len() as u64);
            commands.extend(responses);
        }
        let proposed = director.propose(&state, command_id, commands.len() as u64 + 1);
        command_id = command_id.saturating_add(proposed.len() as u64);
        commands.extend(proposed);
        for command in commands.iter().cloned() { log.append(command); }
        let turn = state.turn; let result = step(&state, &commands, state.seed, &snapshot.versions);
        pressure_sum = pressure_sum.checked_add(result.state.civilizations.values().map(|civ| u64::from(civ.crisis_pressure)).sum()).ok_or_else(|| "pressure total exhausted".to_string())?;
        pressure_samples = pressure_samples.checked_add(result.state.civilizations.len() as u64).ok_or_else(|| "pressure sample count exhausted".to_string())?;
        terms.sample(&result.state);
        audits.extend(audit_records(&commands, &result.events));
        log.record_turn(turn, result.state_hash); final_events = result.events; state = result.state;
    }
    Ok(SimulationRun { stored: StoredRun { snapshot, log, home_tiles: homes }, final_state: state, final_events, audits, terms, personalities, pressure_sum, pressure_samples })
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
    write_json(snapshots.join("final.json"), &WorldSnapshot::new(run.final_state.clone(), run.stored.snapshot.versions.clone()))?;
    write_chronicles(&directory.join("chronicle"), run)
}
fn write_chronicles(directory: &Path, run: &SimulationRun) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let mut state = run.stored.snapshot.state.clone();
    let mut event_history = Vec::new();
    for turn in run.stored.log.turn_hashes.keys() {
        let commands: Vec<_> = run.stored.log.commands.iter().filter(|command| command.turn == *turn).cloned().collect();
        let result = step(&state, &commands, state.seed, &run.stored.snapshot.versions);
        event_history.push((*turn, result.events));
        state = result.state;
    }
    for civilization in state.civilizations.keys() {
        let mandate = run.personalities.get(civilization).map_or_else(|| BotPersonality::Cautious.mandate(1), |personality| personality.mandate(1));
        let chronicle = CanonicalSet::rebuild(&state, *civilization, &mandate, &event_history).document(CanonicalKind::Chronicle).content.clone();
        fs::write(directory.join(format!("civ-{}.md", civilization.0)), chronicle).map_err(|error| error.to_string())?;
    }
    Ok(())
}
pub fn load_log(path: impl AsRef<Path>) -> Result<StoredRun, String> { serde_json::from_str(&fs::read_to_string(path).map_err(|error| error.to_string())?).map_err(|error| error.to_string()) }
fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<(), String> { fs::write(path, serde_json::to_string_pretty(value).map_err(|error| error.to_string())?).map_err(|error| error.to_string()) }

pub fn initial_world(seed: u64, civilizations: u32) -> Result<(WorldState, SimulationVersions, BTreeMap<CivId, TileIndex>), String> {
    let catalog = Catalog::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data/catalogs")).map_err(|error| format!("catalog load failed: {error:?}"))?;
    let generated = generate(WorldParams::for_civilizations(civilizations), seed, &catalog).map_err(|error| format!("map generation failed: {error:?}"))?;
    let mut civilization_states = BTreeMap::new(); let mut homes = BTreeMap::new();
    for start in &generated.starting_points { let id = CivId(start.civilization); let mut civilization = CivilizationState::default(); civilization.researched_technologies.insert("tech.foraging".into()); civilization_states.insert(id, civilization); homes.insert(id, start.tile); }
    let tiles = generated.tiles.iter().map(|tile| TileState { terrain: terrain(&tile.biome_id), river: false, yields: yields(&tile.biome_id), ..Default::default() }).collect();
    let visibility = homes.iter().map(|(civilization, home)| (*civilization, BTreeMap::from([(*home, Visibility::Visible)]))).collect();
    let state = WorldState { world_id: WorldId(seed), turn: TurnNumber::ZERO, seed, ruleset: generated.ruleset_ref.clone(), schema_version: 1, map_width: generated.grid.width, tiles, civilizations: civilization_states, cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility, diplomacy: Default::default(), entropy: Default::default() };
    Ok((state, SimulationVersions { ruleset: generated.ruleset_ref, resolver_version: 1 }, homes))
}
pub fn rotating_order(seed: u64, turn: TurnNumber, homes: &BTreeMap<CivId, TileIndex>) -> Vec<CivId> { let mut order: Vec<_> = homes.keys().copied().collect(); let mut rng = Rng::derive(seed, "bot-order"); for index in (1..order.len()).rev() { order.swap(index, rng.below(index as u32 + 1) as usize); } if !order.is_empty() { let offset = turn.0 as usize % order.len(); order.rotate_left(offset); } order }
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

    #[test] fn pressure_metric_covers_each_resolved_turn() {
        let run = run_simulation(RunConfig { seed: 84, civilizations: 8, turns: 2 }).unwrap();
        assert_eq!(run.pressure_samples, 16);
        assert_eq!(run.metrics().average_pressure, (run.pressure_sum / run.pressure_samples) as u8);
    }

    #[test] fn initial_world_makes_each_starting_tile_visible_to_its_civilization() {
        let (state, _, homes) = initial_world(84, 8).unwrap();
        for (civilization, home) in homes {
            assert_eq!(state.visibility[&civilization][&home], Visibility::Visible);
        }
    }

    #[test] fn one_thousand_bot_turns_complete_without_a_panic() {
        assert_eq!(run_simulation(RunConfig { seed: 84, civilizations: 8, turns: 1_000 }).unwrap().stored.log.turn_hashes.len(), 1_000);
    }

    #[test] fn every_bot_diplomatic_action_is_audited_with_ledger_grounding() {
        let run = run_simulation(RunConfig { seed: 20_261_001, civilizations: 8, turns: 300 }).unwrap();
        let diplomacy = &run.final_state.diplomacy;
        assert!(!run.audits.is_empty(), "bots never acted diplomatically; contacts: {}", diplomacy.relations.values().map(|row| row.len()).sum::<usize>());
        for audit in &run.audits {
            assert!(audit.is_grounded(diplomacy), "ungrounded diplomatic action: {audit:?}");
            assert!(audit.outcome.is_some() || audit.rejection.is_some(), "action without a recorded decision: {audit:?}");
            assert_eq!(audit.rules_version, pw_engine::diplomacy::RULES_VERSION);
        }
        assert!(run.audits.iter().any(|audit| audit.outcome == Some(pw_engine::diplomacy::DiplomaticOutcome::Accepted)));
        // Replay reproduces the diplomatic state without any AI call.
        assert_eq!(replay_log(&run.stored).unwrap(), run.final_hash());
    }

    #[test] fn health_simulation_expands_grows_and_avoids_systemic_collapse() {
        let run = run_simulation(RunConfig { seed: 20_261_001, civilizations: 8, turns: 300 }).unwrap();
        let metrics = run.metrics();
        assert!(metrics.cities >= 16, "expected at least 16 cities, got {}", metrics.cities);
        assert!(metrics.population >= 40, "expected at least 40 population, got {}", metrics.population);
        assert!(metrics.collapses <= 2, "expected at most two collapses, got {}", metrics.collapses);
        assert!(metrics.events >= 10, "expected the Entropy director to open events, got {}", metrics.events);
        // Bot personalities and border disputes make the world contentious (B2): tension and war must happen.
        assert!(run.terms.tension_pair_turns > 0, "expected border tensions between bots");
        assert!(run.terms.war_pair_turns > 0, "expected at least one war between bots");
        // Upper bound only. The floor of 10 is a balance goal reached on this seed but not on seeds 42 and 7
        // (B2: 10, 8 and 9): reported as a warning, not a red test (docs/process/agentes-e-cotas.md §4).
        // See engine/DIAGNOSTICO-P7.md, "P12b", "B1" and "B2".
        if metrics.average_pressure < 10 {
            eprintln!("warning: average pressure {} below the balance goal of 10 (events {}, crises {})", metrics.average_pressure, metrics.events, metrics.crises);
        }
        assert!(metrics.average_pressure <= 80, "expected average pressure at most 80, got {} (events {}, crises {})", metrics.average_pressure, metrics.events, metrics.crises);
    }

    fn entropy_commands(run: &SimulationRun) -> Vec<pw_engine::world::AcceptedCommand> {
        run.stored.log.commands.iter().filter(|command| command.origin == CommandOrigin::Entropy).cloned().collect()
    }

    #[test] fn same_seed_gives_the_same_entropy_events() {
        let config = RunConfig { seed: 20_261_001, civilizations: 8, turns: 120 };
        let first = run_simulation(config).unwrap();
        let second = run_simulation(config).unwrap();
        assert!(!entropy_commands(&first).is_empty());
        assert_eq!(entropy_commands(&first), entropy_commands(&second));
        assert_eq!(first.final_hash(), second.final_hash());
        assert_eq!(replay_log(&first.stored).unwrap(), first.final_hash());
    }

    #[test] fn harness_writes_a_chronicle_for_each_civilization() {
        let run = run_simulation(RunConfig { seed: 84, civilizations: 2, turns: 2 }).unwrap();
        let directory = std::env::temp_dir().join("pw-harness-chronicle-test");
        let _ = fs::remove_dir_all(&directory);
        write_run(&directory, &run).unwrap();
        for civilization in run.final_state.civilizations.keys() {
            let chronicle = fs::read_to_string(directory.join("chronicle").join(format!("civ-{}.md", civilization.0))).unwrap();
            assert!(chronicle.contains("# Chronicle"));
        }
        let _ = fs::remove_dir_all(&directory);
    }

    #[test] fn different_seeds_give_different_entropy_events() {
        let first = run_simulation(RunConfig { seed: 11, civilizations: 8, turns: 120 }).unwrap();
        let second = run_simulation(RunConfig { seed: 12, civilizations: 8, turns: 120 }).unwrap();
        assert_ne!(entropy_commands(&first), entropy_commands(&second));
    }

    #[test] fn entropy_respects_cooldowns_budgets_and_the_engine_accepts_every_event() {
        use pw_engine::{entropy::{BASE_WORLD_BUDGET, BUDGET_PER_LIVING_CIVILIZATION, ERA_TURNS, RESERVE_MAX}, world::CommandPayload};
        let run = run_simulation(RunConfig { seed: 20_261_001, civilizations: 8, turns: 150 }).unwrap();
        let events = entropy_commands(&run);
        assert!(events.len() >= 10);
        let mut last_turn: BTreeMap<String, u32> = BTreeMap::new();
        let mut world_spent: BTreeMap<u32, u32> = BTreeMap::new();
        let mut civ_spent: BTreeMap<(u32, u32), u32> = BTreeMap::new();
        for command in &events {
            let CommandPayload::ApplyEvent { template_id, cost, cooldown, .. } = &command.payload else { panic!("entropy emits only events"); };
            if let Some(previous) = last_turn.insert(template_id.clone(), command.turn.0) { assert!(command.turn.0 >= previous + cooldown, "{template_id} repeated after {} turns, cooldown {cooldown}", command.turn.0 - previous); }
            let era = command.turn.0 / ERA_TURNS;
            *world_spent.entry(era).or_default() += u32::from(*cost);
            *civ_spent.entry((era, command.actor_id.0)).or_default() += u32::from(*cost);
        }
        assert!(world_spent.values().all(|spent| *spent <= BASE_WORLD_BUDGET + BUDGET_PER_LIVING_CIVILIZATION * 8), "world budget exceeded: {world_spent:?}");
        assert!(civ_spent.values().all(|spent| *spent <= RESERVE_MAX), "reserve exceeded: {civ_spent:?}");
        let mut state = run.stored.snapshot.state.clone();
        for turn in run.stored.log.turn_hashes.keys() {
            let commands: Vec<_> = run.stored.log.commands.iter().filter(|command| command.turn == *turn).cloned().collect();
            let result = step(&state, &commands, state.seed, &run.stored.snapshot.versions);
            for command in commands.iter().filter(|command| command.origin == CommandOrigin::Entropy) {
                assert!(!result.events.iter().any(|event| matches!(event, DomainEvent::CommandRejected { command_id, .. } if *command_id == command.command_id)), "engine rejected entropy command {command:?}");
            }
            state = result.state;
        }
    }

    #[test] fn no_entropy_event_eliminates_or_freezes_its_target() {
        let run = run_simulation(RunConfig { seed: 20_261_001, civilizations: 8, turns: 150 }).unwrap();
        let mut state = run.stored.snapshot.state.clone();
        for turn in run.stored.log.turn_hashes.keys() {
            let commands: Vec<_> = run.stored.log.commands.iter().filter(|command| command.turn == *turn).cloned().collect();
            let result = step(&state, &commands, state.seed, &run.stored.snapshot.versions);
            for command in commands.iter().filter(|command| command.origin == CommandOrigin::Entropy) {
                let actor = command.actor_id;
                let before = state.cities.values().filter(|city| city.owner == actor).count();
                let after = result.state.cities.values().filter(|city| city.owner == actor).count();
                assert!(after >= before, "an event removed a city of civilization {}", actor.0);
                if !state.civilizations[&actor].frozen { assert!(!result.state.civilizations[&actor].frozen, "an event froze civilization {} at turn {}", actor.0, turn.0); }
            }
            state = result.state;
        }
    }
}
