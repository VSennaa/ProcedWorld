//! Deterministic T0 utility bot. It proposes commands; the engine still owns validation.

use crate::{
    hex::Grid,
    ids::{CivId, CityId, TileIndex},
    world::{AcceptedCommand, CityFocus, CommandKind, CommandOrigin, CommandPayload, GroundingRef, WorldState},
};

/// A deterministic, utility-based civilization bot. It has no external model or I/O.
#[derive(Clone, Debug, Default)]
pub struct BotT0;

impl BotT0 {
    /// Produces a small ordered proposal set for one civilization. `home_tile` is
    /// supplied by world setup and makes the first settlement fair and reproducible.
    pub fn decide(&self, state: &WorldState, civilization: CivId, home_tile: TileIndex) -> Vec<BotProposal> {
        let mut proposals = Vec::new();
        let cities: Vec<(CityId, _)> = state.cities.iter().filter(|(_, city)| city.owner == civilization).map(|(id, city)| (*id, city)).collect();
        let base = vec![GroundingRef::Civilization { civilization }, GroundingRef::Turn { turn: state.turn }];
        if cities.is_empty() {
            let mut grounding = base;
            grounding.push(GroundingRef::Tile { tile: home_tile });
            proposals.push(BotProposal { payload: CommandPayload::FoundCity { city_id: CityId(civilization.0), target: home_tile }, grounding });
            return proposals;
        }
        if let Some((settler_id, settler)) = state.units.iter().find(|(_, unit)| unit.owner == civilization && unit.unit_type == "unit.settler") {
            let mut grounding = base.clone();
            grounding.extend([GroundingRef::Unit { unit: *settler_id }, GroundingRef::Tile { tile: settler.tile }]);
            proposals.push(BotProposal { payload: CommandPayload::FoundCity { city_id: next_city_id(state), target: settler.tile }, grounding });
        }
        for (city_id, city) in cities {
            let focus = if city.food_stock < city.population { CityFocus::Supply } else if city.unit_queue.is_empty() { CityFocus::Build } else { CityFocus::Diversify };
            let mut grounding = base.clone();
            grounding.push(GroundingRef::City { city: city_id });
            proposals.push(BotProposal { payload: CommandPayload::SetCityFocus { city_id, focus }, grounding: grounding.clone() });
            if city.unit_queue.is_empty() && !state.units.values().any(|unit| unit.owner == civilization && unit.unit_type == "unit.scout") {
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
        let grid = Grid::new(state.map_width, state.tiles.len() as u32 / state.map_width).ok();
        for (unit_id, unit) in state.units.iter().filter(|(_, unit)| unit.owner == civilization) {
            let mut grounding = base.clone();
            grounding.push(GroundingRef::Unit { unit: *unit_id });
            if let Some(target) = grid.and_then(|grid| next_exploration_tile(state, grid, unit.tile)) {
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

fn next_city_id(state: &WorldState) -> CityId { CityId(state.cities.keys().next_back().and_then(|id| id.0.checked_add(1)).unwrap_or(0)) }

fn next_exploration_tile(state: &WorldState, grid: Grid, from: TileIndex) -> Option<TileIndex> {
    let cell = grid.cell(from).ok()?;
    grid.neighbors(cell).ok()?.into_iter().filter_map(|cell| grid.tile_index(cell).ok()).filter(|tile| !state.units.values().any(|unit| unit.tile == *tile)).min_by_key(|tile| (state.visibility.values().all(|known| !known.contains_key(tile)), std::cmp::Reverse(tile.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::{ids::TurnNumber, world::{CivilizationState, RulesetRef, TileState, WorldId}};

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
