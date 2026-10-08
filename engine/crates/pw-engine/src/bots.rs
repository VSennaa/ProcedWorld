//! Deterministic T0 utility bot. It proposes commands; the engine still owns validation.

use crate::{
    hex::Grid,
    ids::{CivId, CityId, TileIndex},
    improvements::{civilization_upkeep, improvement_value, improvements, territory_owner, ImprovementDefinition},
    world::{can_explore, grid_for, movement_cost, next_technology, queued_food_cost, unit_movement, unit_role, unit_route, AcceptedCommand, CityFocus, CommandOrigin, CommandPayload, GroundingRef, UnitOrder, WorldState, SETTLER_CITY_ID_BASE, TERRAIN_OCEAN},
};
use std::collections::{BTreeSet, VecDeque};

// Keep the two-hex work areas disjoint when selecting a new settlement.
const SETTLEMENT_SPACING: u32 = 5;
/// A city trains a worker only once it has this many people (balance proposal, 2026-10-03).
const WORKER_MIN_POPULATION: u32 = 3;
/// Improvements are started only while the treasury holds this reserve plus twice the upkeep
/// the civilization would owe afterwards (balance proposal, 2026-10-03).
const IMPROVEMENT_TREASURY_RESERVE: u32 = 4;
/// ... and only while last turn's wealth income exceeds essential city upkeep plus all improvement
/// upkeep by this margin, so improvements never eat the reserve new cities live on (2026-10-03).
const IMPROVEMENT_INCOME_MARGIN: u32 = 1;

/// A deterministic, utility-based civilization bot. It has no external model or I/O.
#[derive(Clone, Debug, Default)]
pub struct BotT0;

impl BotT0 {
    /// Produces a small ordered proposal set for one civilization. `home_tile` is
    /// supplied by world setup and makes the first settlement fair and reproducible.
    pub fn decide(&self, state: &WorldState, civilization: CivId, home_tile: TileIndex) -> Vec<BotProposal> {
        self.decide_with(state, civilization, home_tile, crate::diplomacy::WarPolicy::default())
    }

    /// `decide` with an explicit diplomatic war policy (bot personalities, B2).
    pub fn decide_with(&self, state: &WorldState, civilization: CivId, home_tile: TileIndex, war_policy: crate::diplomacy::WarPolicy) -> Vec<BotProposal> {
        let mut proposals = Vec::new();
        let cities: Vec<(CityId, _)> = state.cities.iter().filter(|(_, city)| city.owner == civilization).map(|(id, city)| (*id, city)).collect();
        if state.civilizations.get(&civilization).is_some_and(|civ| civ.frozen) { return proposals; }
        let grid = (state.tiles.len() as u32).checked_div(state.map_width)
            .and_then(|height| Grid::new(state.map_width, height).ok());
        let base = vec![GroundingRef::Civilization { civilization }, GroundingRef::Turn { turn: state.turn }];
        if cities.is_empty() {
            let mut grounding = base;
            grounding.push(GroundingRef::Tile { tile: home_tile });
            proposals.push(BotProposal { payload: CommandPayload::FoundCity { city_id: CityId(civilization.0), target: home_tile }, grounding });
            return proposals;
        }
        let mut expansion_pending = state.units.values().any(|unit| unit.owner == civilization && unit.unit_type == "unit.settler")
            || cities.iter().any(|(_, city)| city.unit_queue.iter().any(|id| id == "unit.settler"));
        // One worker per city: count trained and queued workers.
        let mut workers = state.units.values().filter(|unit| unit.owner == civilization && unit_role(&unit.unit_type) == "worker").count()
            + cities.iter().flat_map(|(_, city)| &city.unit_queue).filter(|id| unit_role(id) == "worker").count();
        let city_count = cities.len();
        for (city_id, city) in cities {
            let focus = if city.food_stock < city.population.saturating_add(queued_food_cost(city)) { CityFocus::Supply } else if city.unit_queue.is_empty() { CityFocus::Build } else { CityFocus::Diversify };
            let mut grounding = base.clone();
            grounding.push(GroundingRef::City { city: city_id });
            proposals.push(BotProposal { payload: CommandPayload::SetCityFocus { city_id, focus }, grounding: grounding.clone() });
            if city.unit_queue.is_empty()
                && city.population >= 2
                && state.civilizations[&civilization].researched_technologies.contains("tech.storage")
                && !expansion_pending
                && grid.and_then(|grid| settlement_step(state, grid, city.tile)).is_some() {
                proposals.push(BotProposal { payload: CommandPayload::QueueUnit { city_id, unit_type: "unit.settler".into() }, grounding });
                expansion_pending = true;
            } else if city.unit_queue.is_empty() && !state.units.values().any(|unit| unit.owner == civilization && unit.unit_type == "unit.scout") {
                proposals.push(BotProposal { payload: CommandPayload::QueueUnit { city_id, unit_type: "unit.scout".into() }, grounding });
            } else if city.unit_queue.is_empty() && city.population >= WORKER_MIN_POPULATION && workers < city_count {
                proposals.push(BotProposal { payload: CommandPayload::QueueUnit { city_id, unit_type: "unit.worker".into() }, grounding });
                workers += 1;
            }
        }
        let civ = &state.civilizations[&civilization];
        if civ.research.is_none() {
            // The next technology in catalog order that is still unknown and whose prerequisites are met.
            if let Some(technology) = next_technology(&civ.researched_technologies) {
                let mut grounding = base.clone();
                grounding.push(GroundingRef::Research { technology: technology.clone() });
                proposals.push(BotProposal { payload: CommandPayload::SetResearch { research: technology }, grounding });
                proposals.push(BotProposal { payload: CommandPayload::SetResearchInvestment { percent: 20 }, grounding: base.clone() });
            }
        }
        // Governors select one command per civilization and turn. Keep proposing
        // the selected investment until it is accepted after research begins.
        if civ.research.is_some() && civ.research_investment != 20 {
            proposals.push(BotProposal { payload: CommandPayload::SetResearchInvestment { percent: 20 }, grounding: base.clone() });
        }
        for (unit_id, unit) in state.units.iter().filter(|(_, unit)| unit.owner == civilization) {
            let mut grounding = base.clone();
            grounding.push(GroundingRef::Unit { unit: *unit_id });
            if unit.unit_type == "unit.settler" {
                if let Some(target) = grid.and_then(|grid| settlement_step(state, grid, unit.tile)) {
                    grounding.push(GroundingRef::Tile { tile: target });
                    let payload = if target == unit.tile {
                        CommandPayload::FoundCity { city_id: city_id_for_settler(*unit_id), target }
                    } else { CommandPayload::MoveUnit { unit_id: *unit_id, target } };
                    proposals.push(BotProposal { payload, grounding });
                }
                continue;
            }
            // Persistent orders replace per-turn micro-moves: only idle units need a new order.
            if unit.order != UnitOrder::Idle { continue; }
            if unit_role(&unit.unit_type) == "worker" {
                // A worker with nothing useful to build stays idle and is re-evaluated next turn.
                if let Some((order, tile)) = worker_order(state, civilization, *unit_id) {
                    grounding.push(GroundingRef::Tile { tile });
                    proposals.push(BotProposal { payload: CommandPayload::SetUnitOrder { unit_id: *unit_id, order }, grounding });
                }
                continue;
            }
            let order = if unit_role(&unit.unit_type) == "defense" || !can_explore(state, *unit_id) { UnitOrder::Fortify } else { UnitOrder::Explore };
            proposals.push(BotProposal { payload: CommandPayload::SetUnitOrder { unit_id: *unit_id, order }, grounding });
        }
        // At most one diplomatic action per turn, always grounded in Ledger entries (diplomacy slot).
        if let Some((payload, grounding)) = crate::diplomacy::t0_proposal_with(state, civilization, war_policy) {
            proposals.push(BotProposal { payload, grounding });
        }
        proposals
    }
}

/// An unsequenced intention with its required auditable facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BotProposal { pub payload: CommandPayload, pub grounding: Vec<GroundingRef> }

impl BotProposal {
    pub fn accept(self, state: &WorldState, actor_id: CivId, command_id: u64, accepted_sequence: u64) -> AcceptedCommand {
        let kind = self.payload.kind();
        AcceptedCommand { command_id, world_id: state.world_id, turn: state.turn, accepted_sequence, actor_id, origin: CommandOrigin::Bot, kind, payload: self.payload, grounding: self.grounding, intent_evidence: None, mandate: None }
    }
}

/// The most useful legal improvement near an own city for an idle worker: `Build` when the worker
/// already stands on the chosen tile, `MoveTo` otherwise. Candidates are tiles of the civilization's
/// territory inside its cities' work radius, without improvement, city or another unit, and not
/// already targeted by another own worker. Order: value (`improvement_value`), worked tiles first,
/// distance, tile id, catalog order. Nothing is proposed while the treasury cannot carry the upkeep.
fn worker_order(state: &WorldState, civilization: CivId, unit_id: crate::ids::UnitId) -> Option<(UnitOrder, TileIndex)> {
    let unit = state.units.get(&unit_id)?;
    let grid = grid_for(state).ok()?;
    let from = grid.cell(unit.tile).ok()?;
    let civ = state.civilizations.get(&civilization)?;
    let upkeep = civilization_upkeep(state, civilization);
    let own_cities = || state.cities.values().filter(|city| city.owner == civilization);
    let income: u32 = own_cities().map(|city| u32::from(city.last_yields.wealth)).sum();
    let essential: u32 = own_cities().map(|city| 1 + (city.population + 3) / 4).sum();
    let affordable = |extra: u32| civ.treasury_wealth >= IMPROVEMENT_TREASURY_RESERVE + 2 * (upkeep + extra)
        && income >= essential + upkeep + extra + IMPROVEMENT_INCOME_MARGIN;
    let cheapest = improvements().iter().map(ImprovementDefinition::upkeep).min()?;
    if !affordable(cheapest) { return None; }
    let movement = unit_movement(&unit.unit_type);
    let claimed: BTreeSet<TileIndex> = state.units.iter()
        .filter(|(id, other)| **id != unit_id && other.owner == civilization)
        .filter_map(|(_, other)| match &other.order {
            UnitOrder::MoveTo { target } if unit_role(&other.unit_type) == "worker" => Some(*target),
            UnitOrder::Build { .. } => Some(other.tile),
            _ => None,
        })
        .collect();
    let mut candidates = BTreeSet::new();
    for city in state.cities.values().filter(|city| city.owner == civilization) {
        let Ok(area) = grid.area(grid.cell(city.tile).ok()?, 2) else { continue };
        candidates.extend(area.into_iter().filter_map(|cell| grid.tile_index(cell).ok()));
    }
    let worked: BTreeSet<TileIndex> = state.cities.values().filter(|city| city.owner == civilization).flat_map(|city| city.workplaces.iter().copied()).collect();
    let mut best: Option<((std::cmp::Reverse<u32>, bool, u32, TileIndex, usize), &ImprovementDefinition, TileIndex)> = None;
    for tile in candidates {
        let data = &state.tiles[tile.0 as usize];
        if data.improvement.is_some() || claimed.contains(&tile)
            || state.cities.values().any(|city| city.tile == tile)
            || state.units.iter().any(|(id, other)| *id != unit_id && other.tile == tile) { continue; }
        let distance = grid.distance(from, grid.cell(tile).ok()?).ok()?;
        let mut local: Option<(_, &ImprovementDefinition)> = None;
        for (index, definition) in improvements().iter().enumerate() {
            if !civ.researched_technologies.contains(&definition.requires_technology)
                || !crate::improvements::allowed_on_terrain(definition, data.terrain)
                || !affordable(definition.upkeep()) { continue; }
            let value = improvement_value(data, definition);
            if value == 0 { continue; }
            let key = (std::cmp::Reverse(value), !worked.contains(&tile), distance, tile, index);
            if local.as_ref().map_or(true, |(current, _)| key < *current) { local = Some((key, definition)); }
        }
        // The costlier territory and route checks run only for a tile that would win.
        let Some((key, definition)) = local else { continue };
        if best.as_ref().is_some_and(|(current, _, _)| *current <= key) { continue; }
        if territory_owner(state, tile) != Some(civilization)
            || (tile != unit.tile && unit_route(state, unit.tile, tile, movement).is_none()) { continue; }
        best = Some((key, definition, tile));
    }
    let (_, definition, tile) = best?;
    if tile == unit.tile {
        crate::improvements::check_build(state, unit_id, &definition.id).ok()?;
        Some((UnitOrder::Build { improvement: definition.id.clone() }, tile))
    } else {
        Some((UnitOrder::MoveTo { target: tile }, tile))
    }
}

fn city_id_for_settler(settler_id: crate::ids::UnitId) -> CityId {
    // Production reserves ids represented by existing settlements as well.
    CityId(SETTLER_CITY_ID_BASE.saturating_add(settler_id.0))
}

fn settlement_site(state: &WorldState, grid: Grid, tile: TileIndex) -> bool {
    if state.tiles[tile.0 as usize].terrain == TERRAIN_OCEAN { return false; }
    let Ok(cell) = grid.cell(tile) else { return false; };
    if state.cities.values().any(|city| grid.cell(city.tile).ok()
        .and_then(|center| grid.distance(cell, center).ok()).map_or(true, |distance| distance < SETTLEMENT_SPACING)) { return false; }
    let Ok(area) = grid.area(cell, 2) else { return false; };
    let yields: Vec<_> = area.into_iter().filter_map(|cell| grid.tile_index(cell).ok())
        .map(|tile| state.tiles[tile.0 as usize].yields).collect();
    // A food surplus enables growth and future settlers; upkeep and production
    // must also be available locally before the bot commits to this site.
    yields.iter().any(|output| output.food >= 2)
        && yields.iter().map(|output| u32::from(output.wealth)).sum::<u32>() >= 2
        && yields.iter().any(|output| output.production > 0)
}

fn settlement_step(state: &WorldState, grid: Grid, from: TileIndex) -> Option<TileIndex> {
    let movement = u32::from(unit_movement("unit.settler"));
    let mut visited = BTreeSet::from([from]);
    let mut frontier = VecDeque::from([(from, from)]);
    // Breadth-first traversal finds the nearest reachable site, with stable
    // tile-id ties. Recompute each turn so occupied routes can be avoided.
    while let Some((tile, first_step)) = frontier.pop_front() {
        if settlement_site(state, grid, tile) { return Some(first_step); }
        let mut neighbors: Vec<_> = grid.neighbors(grid.cell(tile).ok()?).ok()?.into_iter()
            .filter_map(|cell| grid.tile_index(cell).ok()).collect();
        neighbors.sort_unstable();
        for neighbor in neighbors {
            if visited.contains(&neighbor) || state.units.values().any(|unit| unit.tile == neighbor)
                || !movement_cost(state, tile, neighbor).is_ok_and(|cost| cost <= movement) { continue; }
            visited.insert(neighbor);
            frontier.push_back((neighbor, if tile == from { neighbor } else { first_step }));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::{ids::TurnNumber, world::{CivilizationState, RulesetRef, TileState, WorldId}};

    fn expansion_world() -> WorldState {
        let mut civ = CivilizationState::default();
        civ.researched_technologies.extend(["tech.foraging", "tech.storage"].map(str::to_owned));
        WorldState {
            world_id: WorldId(1), turn: TurnNumber::ZERO, seed: 1,
            ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 1 },
            schema_version: 1, map_width: 20,
            tiles: vec![TileState { terrain: 0, river: false, yields: crate::world::TileYields {
                food: 2, production: 1, wealth: 1, knowledge: 0, culture: 0,
            }, ..Default::default() }; 200],
            civilizations: BTreeMap::from([(CivId(0), civ)]), cities: BTreeMap::new(),
            units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(),
            visibility: BTreeMap::new(), diplomacy: Default::default(), entropy: Default::default(),
        }
    }

    #[test]
    fn bot_repeats_expansion_with_paid_settlers_beyond_two_cities() {
        let mut state = expansion_world();
        let versions = crate::world::SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 };
        let mut founded = 0;
        for turn in 0..150 {
            let commands: Vec<_> = BotT0.decide(&state, CivId(0), TileIndex(0)).into_iter()
                .enumerate().map(|(index, proposal)| proposal.accept(&state, CivId(0), turn * 100 + index as u64, index as u64)).collect();
            let result = crate::world::step(&state, &commands, state.seed, &versions);
            for event in &result.events {
                if let crate::world::DomainEvent::CommandRejected { command_id, .. } = event {
                    assert!(!commands.iter().any(|command| command.command_id == *command_id
                        && matches!(command.payload, CommandPayload::FoundCity { .. })), "founding rejected: {event:?}");
                }
            }
            let queued = result.state.cities.values().flat_map(|city| &city.unit_queue).filter(|id| *id == "unit.settler").count();
            let produced = result.state.units.values().filter(|unit| unit.unit_type == "unit.settler").count();
            assert!(queued + produced <= 1);
            if result.state.cities.len() > state.cities.len() {
                if !state.cities.is_empty() {
                    assert!(state.units.values().any(|unit| unit.unit_type == "unit.settler"));
                }
                founded += 1;
            }
            state = result.state;
        }
        assert!(founded >= 3, "expected repeated expansion, got {founded}");
        assert!(!state.civilizations[&CivId(0)].frozen);
    }

    #[test]
    fn settler_routes_around_unaffordable_terrain_and_occupied_tiles() {
        let mut state = expansion_world();
        let unit_id = crate::ids::UnitId(1);
        state.units.insert(unit_id, crate::world::UnitState {
            owner: CivId(0), tile: TileIndex(0), unit_type: "unit.settler".into(),
            hit_points: 100, movement_left: 0, explored: false, order: crate::world::UnitOrder::Idle, skipped_turn: None,
        });
        state.tiles[0].terrain = TERRAIN_OCEAN;
        state.tiles[1].terrain = TERRAIN_OCEAN;
        let grid = Grid::new(20, 10).unwrap();
        let target = settlement_step(&state, grid, TileIndex(0)).unwrap();
        assert_ne!(target, TileIndex(0));
        assert_ne!(target, TileIndex(1));
        assert!(movement_cost(&state, TileIndex(0), target).unwrap() <= 2);
        // Occupying the selected destination must force another route.
        state.units.insert(crate::ids::UnitId(2), crate::world::UnitState {
            owner: CivId(0), tile: target, unit_type: "unit.scout".into(),
            hit_points: 100, movement_left: 0, explored: false, order: crate::world::UnitOrder::Idle, skipped_turn: None,
        });
        assert_ne!(settlement_step(&state, grid, TileIndex(0)), Some(target));
    }

    #[test]
    fn queued_settler_requests_food_beyond_daily_consumption() {
        let initial = expansion_world();
        let versions = crate::world::SimulationVersions { ruleset: initial.ruleset.clone(), resolver_version: 1 };
        let founding = BotT0.decide(&initial, CivId(0), TileIndex(0)).remove(0).accept(&initial, CivId(0), 1, 1);
        let mut state = crate::world::step(&initial, &[founding], initial.seed, &versions).state;
        let city = state.cities.get_mut(&CityId(0)).unwrap();
        city.food_stock = city.population;
        city.unit_queue.push("unit.settler".into());
        assert!(BotT0.decide(&state, CivId(0), TileIndex(0)).iter().any(|proposal| matches!(
            proposal.payload, CommandPayload::SetCityFocus { focus: CityFocus::Supply, .. }
        )));
    }

    #[test]
    fn active_research_requests_the_selected_investment() {
        // A civilization without cities only proposes founding one, so found the capital first.
        let initial = expansion_world();
        let versions = crate::world::SimulationVersions { ruleset: initial.ruleset.clone(), resolver_version: 1 };
        let founding = BotT0.decide(&initial, CivId(0), TileIndex(0)).remove(0).accept(&initial, CivId(0), 1, 1);
        let mut state = crate::world::step(&initial, &[founding], initial.seed, &versions).state;
        let civ = state.civilizations.get_mut(&CivId(0)).unwrap();
        civ.research = Some("tech.storage".into());
        // Governors only re-propose the investment while it differs from the selected one.
        civ.research_investment = 0;
        assert!(BotT0.decide(&state, CivId(0), TileIndex(0)).iter().any(|proposal| matches!(
            proposal.payload, CommandPayload::SetResearchInvestment { percent: 20 }
        )));
    }

    #[test]
    fn research_picks_the_next_valid_technology_and_never_repeats_one() {
        let initial = expansion_world();
        let versions = crate::world::SimulationVersions { ruleset: initial.ruleset.clone(), resolver_version: 1 };
        let founding = BotT0.decide(&initial, CivId(0), TileIndex(0)).remove(0).accept(&initial, CivId(0), 1, 1);
        let mut state = crate::world::step(&initial, &[founding], initial.seed, &versions).state;
        let research = |state: &WorldState| BotT0.decide(state, CivId(0), TileIndex(0)).into_iter().find_map(|proposal| match proposal.payload {
            CommandPayload::SetResearch { research } => Some(research),
            _ => None,
        });
        // Foraging and storage are already known; the catalog continues with irrigation.
        assert_eq!(research(&state).as_deref(), Some("tech.irrigation"));
        let mut known = std::collections::BTreeSet::new();
        for _ in 0..40 {
            let Some(next) = crate::world::next_technology(&known) else { break; };
            assert!(known.insert(next), "a technology was picked twice");
        }
        assert!(crate::world::next_technology(&known).is_none());
        assert_eq!(known.len(), 17);
        let civ = state.civilizations.get_mut(&CivId(0)).unwrap();
        civ.researched_technologies.extend(known);
        assert_eq!(research(&state), None);
    }

    #[test]
    fn first_decision_founds_a_city_with_grounding() {
        let mut civilizations = BTreeMap::new(); civilizations.insert(CivId(0), CivilizationState::default());
        let state = WorldState { world_id: WorldId(1), turn: TurnNumber::ZERO, seed: 1, ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 1 }, schema_version: 1, map_width: 2, tiles: vec![TileState::default(); 4], civilizations, cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility: BTreeMap::new(), diplomacy: Default::default(), entropy: Default::default() };
        let proposal = BotT0.decide(&state, CivId(0), TileIndex(3)).pop().unwrap();
        assert!(matches!(proposal.payload, CommandPayload::FoundCity { target: TileIndex(3), .. }));
        assert!(!proposal.grounding.is_empty());
        assert_eq!(proposal.accept(&state, CivId(0), 1, 1).origin, CommandOrigin::Bot);
    }

    #[test]
    fn idle_units_get_a_persistent_order_and_ordered_units_are_left_alone() {
        let initial = expansion_world();
        let versions = crate::world::SimulationVersions { ruleset: initial.ruleset.clone(), resolver_version: 1 };
        let founding = BotT0.decide(&initial, CivId(0), TileIndex(0)).remove(0).accept(&initial, CivId(0), 1, 1);
        let mut state = crate::world::step(&initial, &[founding], initial.seed, &versions).state;
        state.units.insert(crate::ids::UnitId(7), crate::world::UnitState {
            owner: CivId(0), tile: TileIndex(0), unit_type: "unit.scout".into(),
            hit_points: 100, movement_left: 3, explored: false, order: UnitOrder::Idle, skipped_turn: None,
        });
        let orders = |state: &WorldState| BotT0.decide(state, CivId(0), TileIndex(0)).into_iter().filter_map(|proposal| match proposal.payload {
            CommandPayload::SetUnitOrder { unit_id, order } => Some((unit_id, order)),
            _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(orders(&state), vec![(crate::ids::UnitId(7), UnitOrder::Explore)]);
        state.units.get_mut(&crate::ids::UnitId(7)).unwrap().order = UnitOrder::Fortify;
        assert!(orders(&state).is_empty());
    }

    #[test]
    fn idle_worker_improves_a_tile_near_the_city_and_cities_train_workers() {
        let mut initial = expansion_world();
        // Income must cover essential upkeep plus the new improvement's upkeep and the margin.
        for tile in &mut initial.tiles { tile.yields.wealth = 4; }
        let versions = crate::world::SimulationVersions { ruleset: initial.ruleset.clone(), resolver_version: 1 };
        let founding = BotT0.decide(&initial, CivId(0), TileIndex(0)).remove(0).accept(&initial, CivId(0), 1, 1);
        let mut state = crate::world::step(&initial, &[founding], initial.seed, &versions).state;
        let worker = crate::ids::UnitId(5);
        state.units.insert(worker, crate::world::UnitState {
            owner: CivId(0), tile: TileIndex(22), unit_type: "unit.worker".into(),
            hit_points: 100, movement_left: 2, explored: false, order: UnitOrder::Idle, skipped_turn: None,
        });
        let first = BotT0.decide(&state, CivId(0), TileIndex(0));
        let order = first.iter().find_map(|proposal| match &proposal.payload {
            CommandPayload::SetUnitOrder { unit_id, order } if *unit_id == worker => Some(order.clone()),
            _ => None,
        });
        assert!(matches!(order, Some(UnitOrder::Build { .. } | UnitOrder::MoveTo { .. })), "{order:?}");
        let mut built = 0;
        for turn in 0..30_u64 {
            let commands: Vec<_> = BotT0.decide(&state, CivId(0), TileIndex(0)).into_iter().enumerate()
                .map(|(index, proposal)| proposal.accept(&state, CivId(0), 1_000 + turn * 100 + index as u64, index as u64)).collect();
            let result = crate::world::step(&state, &commands, state.seed, &versions);
            for event in &result.events {
                if let crate::world::DomainEvent::CommandRejected { command_id, reason } = event {
                    assert!(!commands.iter().any(|command| command.command_id == *command_id
                        && matches!(command.payload, CommandPayload::SetUnitOrder { unit_id, .. } if unit_id == worker)), "worker order rejected: {reason:?}");
                }
            }
            state = result.state;
            built = state.tiles.iter().filter(|tile| tile.improvement.is_some()).count();
        }
        assert!(built >= 1, "expected the worker to finish an improvement, got {built}");
        // No worker is queued below the population threshold; a grown city queues one.
        let city = state.cities.get_mut(&CityId(0)).unwrap();
        city.unit_queue.clear();
        city.population = WORKER_MIN_POPULATION;
        state.units.remove(&worker);
        state.units.insert(crate::ids::UnitId(9), crate::world::UnitState {
            owner: CivId(0), tile: TileIndex(199), unit_type: "unit.scout".into(),
            hit_points: 100, movement_left: 3, explored: false, order: UnitOrder::Fortify, skipped_turn: None,
        });
        state.civilizations.get_mut(&CivId(0)).unwrap().researched_technologies.remove("tech.storage");
        assert!(BotT0.decide(&state, CivId(0), TileIndex(0)).iter().any(|proposal| matches!(
            &proposal.payload, CommandPayload::QueueUnit { unit_type, .. } if unit_type == "unit.worker")));
    }
}
