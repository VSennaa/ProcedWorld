//! Deterministic T0 utility bot. It proposes commands; the engine still owns validation.

use crate::{
    hex::Grid,
    ids::{CivId, CityId, TileIndex},
    world::{movement_cost, queued_food_cost, unit_movement, AcceptedCommand, CityFocus, CommandKind, CommandOrigin, CommandPayload, GroundingRef, WorldState, SETTLER_CITY_ID_BASE, TERRAIN_OCEAN},
};
use std::collections::{BTreeSet, VecDeque};

// Keep the two-hex work areas disjoint when selecting a new settlement.
const SETTLEMENT_SPACING: u32 = 5;

/// A deterministic, utility-based civilization bot. It has no external model or I/O.
#[derive(Clone, Debug, Default)]
pub struct BotT0;

impl BotT0 {
    /// Produces a small ordered proposal set for one civilization. `home_tile` is
    /// supplied by world setup and makes the first settlement fair and reproducible.
    pub fn decide(&self, state: &WorldState, civilization: CivId, home_tile: TileIndex) -> Vec<BotProposal> {
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
            }
        }
        let civ = &state.civilizations[&civilization];
        if civ.research.is_none() {
            let technology = if civ.researched_technologies.contains("tech.foraging") { "tech.storage" } else { "tech.foraging" };
            let mut grounding = base.clone();
            grounding.push(GroundingRef::Research { technology: technology.into() });
            proposals.push(BotProposal { payload: CommandPayload::SetResearch { research: technology.into() }, grounding });
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
            if let Some(target) = grid.and_then(|grid| next_exploration_tile(state, grid, civilization, unit.tile, unit_movement(&unit.unit_type))) {
                grounding.push(GroundingRef::Tile { tile: target });
                proposals.push(BotProposal { payload: CommandPayload::MoveUnit { unit_id: *unit_id, target }, grounding: grounding.clone() });
            }
            proposals.push(BotProposal { payload: CommandPayload::Explore { unit_id: *unit_id }, grounding });
        }
        proposals
    }
}

/// An unsequenced intention with its required auditable facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BotProposal { pub payload: CommandPayload, pub grounding: Vec<GroundingRef> }

impl BotProposal {
    pub fn accept(self, state: &WorldState, actor_id: CivId, command_id: u64, accepted_sequence: u64) -> AcceptedCommand {
        let kind = command_kind(&self.payload);
        AcceptedCommand { command_id, world_id: state.world_id, turn: state.turn, accepted_sequence, actor_id, origin: CommandOrigin::Bot, kind, payload: self.payload, grounding: self.grounding }
    }
}

fn command_kind(payload: &CommandPayload) -> CommandKind {
    match payload {
        CommandPayload::EndTurn => CommandKind::EndTurn, CommandPayload::MoveUnit { .. } => CommandKind::MoveUnit, CommandPayload::FoundCity { .. } => CommandKind::FoundCity, CommandPayload::SetCityFocus { .. } => CommandKind::SetCityFocus, CommandPayload::SetResearch { .. } => CommandKind::SetResearch, CommandPayload::SetResearchInvestment { .. } => CommandKind::SetResearchInvestment, CommandPayload::ActivatePractice { .. } => CommandKind::ActivatePractice, CommandPayload::DeactivatePractice { .. } => CommandKind::DeactivatePractice, CommandPayload::QueueUnit { .. } => CommandKind::QueueUnit, CommandPayload::DeclareAttack { .. } => CommandKind::DeclareAttack, CommandPayload::Explore { .. } => CommandKind::Explore, CommandPayload::KeepPlan => CommandKind::KeepPlan,
    }
}

fn city_id_for_settler(settler_id: crate::ids::UnitId) -> CityId {
    // Production reserves ids represented by existing settlements as well.
    CityId(SETTLER_CITY_ID_BASE.saturating_add(settler_id.0))
}

fn next_exploration_tile(state: &WorldState, grid: Grid, owner: CivId, from: TileIndex, movement: u8) -> Option<TileIndex> {
    let cell = grid.cell(from).ok()?;
    grid.neighbors(cell).ok()?.into_iter().filter_map(|cell| grid.tile_index(cell).ok())
        .filter(|tile| !state.units.values().any(|unit| unit.tile == *tile))
        .filter(|tile| movement_cost(state, from, *tile).is_ok_and(|cost| cost <= u32::from(movement)))
        .min_by_key(|tile| (state.visibility.get(&owner).is_some_and(|known| known.contains_key(tile)), tile.0))
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
            } }; 200],
            civilizations: BTreeMap::from([(CivId(0), civ)]), cities: BTreeMap::new(),
            units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(),
            visibility: BTreeMap::new(), ledger: Vec::new(),
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
            hit_points: 100, movement_left: 0, explored: false,
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
            hit_points: 100, movement_left: 0, explored: false,
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
    fn first_decision_founds_a_city_with_grounding() {
        let mut civilizations = BTreeMap::new(); civilizations.insert(CivId(0), CivilizationState::default());
        let state = WorldState { world_id: WorldId(1), turn: TurnNumber::ZERO, seed: 1, ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 1 }, schema_version: 1, map_width: 2, tiles: vec![TileState::default(); 4], civilizations, cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility: BTreeMap::new(), ledger: Vec::new() };
        let proposal = BotT0.decide(&state, CivId(0), TileIndex(3)).pop().unwrap();
        assert!(matches!(proposal.payload, CommandPayload::FoundCity { target: TileIndex(3), .. }));
        assert!(!proposal.grounding.is_empty());
        assert_eq!(proposal.accept(&state, CivId(0), 1, 1).origin, CommandOrigin::Bot);
    }
}
