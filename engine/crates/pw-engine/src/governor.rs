//! Mandate-limited Governor orchestration. This module never mutates `WorldState`.

use serde::{Deserialize, Serialize};

use crate::{
    ai::{ActionIntent, AiLayer, AiStatus, ChoiceRequest, DecisionPort, IntentValidator, PortError, ACTION_INTENT_SCHEMA_VERSION},
    bots::BotT0,
    ids::{CityId, CivId, TileIndex, UnitId},
    world::{AcceptedCommand, CommandOrigin, CommandPayload, GroundingRef, IntentEvidence, WorldState},
};

pub const MINIMUM_BENEFIT: u8 = 10;
pub const RESERVE_PERCENTAGES: [u8; 4] = [0, 10, 25, 40];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope { CitiesEconomy, ExplorationDefense, Technology, Diplomacy, CrisisResponse }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeMode { ActWithinLimits, ProposeWait }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReserveLevel { Zero, Low, Medium, High }

impl ReserveLevel { pub const fn percent(self) -> u8 { RESERVE_PERCENTAGES[self as usize] } }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance { Conciliatory, Reciprocal, Deterrent }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedLine { NoStartWar, NoBreakTreaty, NoCedeCity, NoSpendReserveDiplomacy, NoRelocatePopulation }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Direction { pub security: u8, pub sustenance: u8, pub development: u8, pub relations: u8 }
impl Direction {
    pub const fn total(self) -> u16 { self.security as u16 + self.sustenance as u16 + self.development as u16 + self.relations as u16 }
    pub const fn valid(self) -> bool { self.total() == 100 }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reserves { pub treasury: ReserveLevel, pub strategic_stock: ReserveLevel, pub unit_loss: ReserveLevel }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mandate {
    pub version: u32,
    pub direction: Direction,
    pub red_lines: Vec<RedLine>,
    pub reserves: Reserves,
    pub stance: Stance,
    pub scopes: [ScopeMode; 5],
    pub alerts: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MandatePreset { Balanced, Recover, GrowCautiously }

impl Mandate {
    pub fn preset(preset: MandatePreset, version: u32) -> Self {
        let direction = match preset {
            MandatePreset::Balanced => Direction { security: 25, sustenance: 25, development: 25, relations: 25 },
            MandatePreset::Recover => Direction { security: 30, sustenance: 40, development: 10, relations: 20 },
            MandatePreset::GrowCautiously => Direction { security: 20, sustenance: 30, development: 35, relations: 15 },
        };
        Self { version, direction, red_lines: vec![RedLine::NoStartWar, RedLine::NoBreakTreaty, RedLine::NoCedeCity], reserves: Reserves { treasury: ReserveLevel::Medium, strategic_stock: ReserveLevel::Medium, unit_loss: ReserveLevel::Low }, stance: Stance::Reciprocal, scopes: [ScopeMode::ActWithinLimits; 5], alerts: Vec::new() }
    }
    pub fn balanced(version: u32) -> Self { Self::preset(MandatePreset::Balanced, version) }
    pub fn valid(&self) -> bool { self.direction.valid() && self.red_lines.windows(2).all(|pair| pair[0] < pair[1]) }
    pub fn scope_mode(&self, scope: Scope) -> ScopeMode { self.scopes[scope as usize] }
    /// This is also used by `IntentValidator`, so direct Governor commands cannot bypass a red line.
    pub fn allows_payload(&self, payload: &CommandPayload) -> bool {
        let starts_war = matches!(payload, CommandPayload::DeclareAttack { .. } | CommandPayload::DeclareWar { .. });
        let breaks_treaty = matches!(payload, CommandPayload::BreakTreaty { .. });
        !((starts_war && self.red_lines.contains(&RedLine::NoStartWar)) || (breaks_treaty && self.red_lines.contains(&RedLine::NoBreakTreaty)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateAction {
    pub id: String,
    pub scope: Scope,
    pub payload: CommandPayload,
    pub facts: Vec<GroundingRef>,
    pub benefits: Direction,
    pub opportunity_cost: u8,
    pub exposed_risk: u8,
    pub irreversible: bool,
}

impl CandidateAction {
    pub fn utility(&self) -> i32 {
        3 * i32::from(self.benefits.security) + 3 * i32::from(self.benefits.sustenance)
            + 2 * i32::from(self.benefits.development) + 2 * i32::from(self.benefits.relations)
            - 2 * i32::from(self.opportunity_cost) - i32::from(self.exposed_risk)
    }
    fn benefit_max(&self) -> u8 { self.benefits.security.max(self.benefits.sustenance).max(self.benefits.development).max(self.benefits.relations) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportItem { pub action_id: Option<String>, pub rule: String, pub facts: [GroundingRef; 2] }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GovernorReport { pub did: Vec<ReportItem>, pub did_not: Vec<ReportItem>, pub needs_you: Vec<ReportItem> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernorDecision { pub commands: Vec<AcceptedCommand>, pub report: GovernorReport }

/// An exclusive capability (see `docs/sdd/20-matriz-de-conflitos.md`): at most one accepted
/// command may claim each slot per turn.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Slot { Focus(CityId), Queue(CityId), Research, Investment, Unit(UnitId), Destination(TileIndex), Found(CityId), Practice, Plan, Diplomacy }

fn slots_for(payload: &CommandPayload) -> Vec<Slot> {
    match payload {
        CommandPayload::SetCityFocus { city_id, .. } => vec![Slot::Focus(*city_id)],
        CommandPayload::QueueUnit { city_id, .. } | CommandPayload::RemoveQueuedUnit { city_id, .. } | CommandPayload::MoveQueuedUnit { city_id, .. } => vec![Slot::Queue(*city_id)],
        CommandPayload::SetResearch { .. } => vec![Slot::Research],
        CommandPayload::SetResearchInvestment { .. } => vec![Slot::Investment],
        CommandPayload::MoveUnit { unit_id, target } => vec![Slot::Unit(*unit_id), Slot::Destination(*target)],
        CommandPayload::Explore { unit_id } | CommandPayload::SetUnitOrder { unit_id, .. } | CommandPayload::SkipUnit { unit_id } => vec![Slot::Unit(*unit_id)],
        CommandPayload::FoundCity { city_id, .. } => vec![Slot::Found(*city_id)],
        CommandPayload::DeclareAttack { attacker, .. } => vec![Slot::Unit(*attacker)],
        CommandPayload::ActivatePractice { .. } | CommandPayload::DeactivatePractice { .. } => vec![Slot::Practice],
        CommandPayload::EndTurn | CommandPayload::KeepPlan => vec![Slot::Plan],
        // One diplomatic action per civilization and turn, independent of every other slot.
        CommandPayload::ProposeDiplomacy { .. } | CommandPayload::BreakTreaty { .. } | CommandPayload::DeclareWar { .. } => vec![Slot::Diplomacy],
        // Entropy and event responses are not Governor candidates; they never claim a slot.
        CommandPayload::ApplyEvent { .. } | CommandPayload::RespondToEvent { .. } => Vec::new(),
    }
}

pub struct Governor<'a> { pub mandate: &'a Mandate, pub decision_port: Option<&'a dyn DecisionPort> }

impl<'a> Governor<'a> {
    /// Returns a conflict-free command set: at most one command per slot, best utility first
    /// (ties by id). Command ids and sequences are assigned consecutively from the given bases.
    pub fn decide(&self, state: &WorldState, civilization: CivId, home: TileIndex, absent: bool, command_id: u64, accepted_sequence: u64) -> GovernorDecision {
        let candidates = self.candidates(state, civilization, home);
        let mut report = GovernorReport::default();
        let mut allowed = Vec::new();
        for candidate in candidates {
            match self.filter(&candidate, absent) {
                Filter::Allow => allowed.push(candidate),
                Filter::Await(rule) => report.needs_you.push(item(Some(candidate.id), rule, &candidate.facts)),
                Filter::Deny(rule) => report.did_not.push(item(Some(candidate.id), rule, &candidate.facts)),
            }
        }
        allowed.sort_unstable_by(|left, right| right.utility().cmp(&left.utility()).then_with(|| left.id.cmp(&right.id)));
        // A decision port may promote one candidate; the rest keep utility order.
        if let Some(index) = self.choose(&allowed, command_id).and_then(|chosen| allowed.iter().position(|candidate| candidate.id == chosen.id)) {
            let chosen = allowed.remove(index);
            allowed.insert(0, chosen);
        }
        let mut taken: Vec<Slot> = Vec::new();
        let mut commands = Vec::new();
        for selected in allowed {
            let slots = slots_for(&selected.payload);
            if slots.iter().any(|slot| taken.contains(slot)) {
                report.did_not.push(item(Some(selected.id), "slot_conflict".into(), &selected.facts));
                continue;
            }
            let offset = commands.len() as u64;
            let id = command_id.saturating_add(offset);
            let intent = ActionIntent { request_id: id, actor_id: civilization, turn: state.turn, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(), kind: selected.payload.kind(), parameters: selected.payload.clone(), grounding: selected.facts.clone() };
            let evidence = IntentEvidence { request_id: id, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(), layer: if self.decision_port.is_some() { AiLayer::T1 } else { AiLayer::T0 }, status: AiStatus::Ok, fixture_id: None, fallback_from: None };
            match IntentValidator::accept(state, intent, CommandOrigin::Governor, Some(self.mandate), id, accepted_sequence.saturating_add(offset), evidence) {
                Ok(command) => { taken.extend(slots); report.did.push(item(Some(selected.id.clone()), "mandate_permitted".into(), &selected.facts)); commands.push(command); }
                Err(_) => report.did_not.push(item(Some(selected.id.clone()), "engine_rejected".into(), &selected.facts)),
            }
        }
        GovernorDecision { commands, report }
    }

    fn candidates(&self, state: &WorldState, civilization: CivId, home: TileIndex) -> Vec<CandidateAction> {
        BotT0.decide(state, civilization, home).into_iter()
            .filter(|proposal| match &proposal.payload {
                CommandPayload::SetCityFocus { city_id, focus } => state.cities.get(city_id).is_some_and(|city| city.focus != *focus),
                _ => true,
            })
            .enumerate().map(|(index, proposal)| candidate_from_proposal(index, proposal.payload, proposal.grounding, self.mandate.direction)).collect()
    }
    fn filter(&self, candidate: &CandidateAction, absent: bool) -> Filter {
        if !self.mandate.valid() { return Filter::Deny("invalid_mandate".into()); }
        if !self.mandate.allows_payload(&candidate.payload) { return Filter::Deny("red_line".into()); }
        if candidate.irreversible && absent { return Filter::Deny("absent_irreversible".into()); }
        if !absent && self.mandate.scope_mode(candidate.scope) == ScopeMode::ProposeWait { return Filter::Await("scope_propose_wait".into()); }
        if candidate.benefit_max() <= MINIMUM_BENEFIT { return Filter::Deny("minimum_benefit".into()); }
        Filter::Allow
    }
    fn choose<'b>(&self, allowed: &'b [CandidateAction], request_id: u64) -> Option<&'b CandidateAction> {
        let port = self.decision_port?;
        let choices = allowed.iter().map(|candidate| candidate.id.clone()).collect();
        match port.choice(&ChoiceRequest { request_id, choices }) {
            Ok(response) => allowed.iter().find(|candidate| candidate.id == response.choice_id),
            Err(PortError::Timeout | PortError::Unavailable | PortError::InvalidOutput) => None,
        }
    }
}

enum Filter { Allow, Deny(String), Await(String) }

fn candidate_from_proposal(index: usize, payload: CommandPayload, mut facts: Vec<GroundingRef>, direction: Direction) -> CandidateAction {
    // The T0 bot uses a prospective technology as an internal planning hint. An
    // intent may ground only facts that already exist, so retain its civ/turn facts.
    facts.retain(|fact| !matches!(fact, GroundingRef::Research { .. }));
    let scope = scope_for(&payload);
    let benefit = match &payload {
        CommandPayload::FoundCity { .. } => Direction { security: 0, sustenance: direction.sustenance, development: direction.development, relations: 0 },
        CommandPayload::SetCityFocus { focus: crate::world::CityFocus::Supply, .. } => Direction { security: 0, sustenance: 100, development: 0, relations: 0 },
        CommandPayload::SetCityFocus { .. } => Direction { security: 0, sustenance: 0, development: direction.development, relations: 0 },
        CommandPayload::QueueUnit { .. } | CommandPayload::RemoveQueuedUnit { .. } | CommandPayload::MoveQueuedUnit { .. } => Direction { security: 0, sustenance: 0, development: 100, relations: 0 },
        CommandPayload::SetResearch { .. } => Direction { security: 0, sustenance: 0, development: 100, relations: 0 },
        CommandPayload::SetResearchInvestment { .. } | CommandPayload::ActivatePractice { .. } | CommandPayload::DeactivatePractice { .. } => Direction { security: 0, sustenance: 0, development: direction.development, relations: 0 },
        CommandPayload::DeclareAttack { .. } => Direction { security: direction.security, sustenance: 0, development: 0, relations: 0 },
        CommandPayload::MoveUnit { .. } | CommandPayload::Explore { .. } | CommandPayload::SetUnitOrder { .. } | CommandPayload::SkipUnit { .. } => Direction { security: direction.security, sustenance: 0, development: direction.development, relations: 0 },
        CommandPayload::EndTurn | CommandPayload::KeepPlan | CommandPayload::ApplyEvent { .. } | CommandPayload::RespondToEvent { .. } => Direction { security: 0, sustenance: 0, development: 0, relations: 0 },
        CommandPayload::ProposeDiplomacy { .. } | CommandPayload::BreakTreaty { .. } => Direction { security: 0, sustenance: 0, development: 0, relations: direction.relations },
        CommandPayload::DeclareWar { .. } => Direction { security: direction.security, sustenance: 0, development: 0, relations: direction.relations },
    };
    CandidateAction { id: format!("{:?}-{index}", payload.kind()), scope, payload: payload.clone(), facts, benefits: benefit, opportunity_cost: u8::from(matches!(payload, CommandPayload::Explore { .. })), exposed_risk: 0, irreversible: matches!(payload, CommandPayload::DeclareAttack { .. } | CommandPayload::DeclareWar { .. } | CommandPayload::BreakTreaty { .. }) }
}

fn scope_for(payload: &CommandPayload) -> Scope { match payload { CommandPayload::ProposeDiplomacy { .. } | CommandPayload::BreakTreaty { .. } | CommandPayload::DeclareWar { .. } => Scope::Diplomacy, CommandPayload::SetResearch { .. } | CommandPayload::SetResearchInvestment { .. } | CommandPayload::ActivatePractice { .. } | CommandPayload::DeactivatePractice { .. } => Scope::Technology, CommandPayload::MoveUnit { .. } | CommandPayload::Explore { .. } | CommandPayload::SetUnitOrder { .. } | CommandPayload::SkipUnit { .. } | CommandPayload::DeclareAttack { .. } => Scope::ExplorationDefense, _ => Scope::CitiesEconomy } }
fn item(action_id: Option<String>, rule: String, facts: &[GroundingRef]) -> ReportItem { let first = facts.first().cloned().unwrap_or(GroundingRef::Turn { turn: crate::ids::TurnNumber::ZERO }); let second = facts.get(1).cloned().unwrap_or_else(|| first.clone()); ReportItem { action_id, rule, facts: [first, second] } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ids::TurnNumber, world::{CivilizationState, RulesetRef, TileState, WorldId}};
    use std::collections::BTreeMap;

    fn state() -> WorldState { WorldState { world_id: WorldId(1), turn: TurnNumber::ZERO, seed: 1, ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 1 }, schema_version: 1, map_width: 1, tiles: vec![TileState::default()], civilizations: BTreeMap::from([(CivId(0), CivilizationState::default())]), cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility: BTreeMap::from([(CivId(0), BTreeMap::from([(TileIndex(0), crate::world::Visibility::Visible)]))]), diplomacy: Default::default(), entropy: Default::default() } }

    #[test] fn presets_are_valid() { for preset in [MandatePreset::Balanced, MandatePreset::Recover, MandatePreset::GrowCautiously] { assert!(Mandate::preset(preset, 1).valid()); } }
    #[test] fn red_line_blocks_governor_command_in_engine() { let state = state(); let mandate = Mandate::balanced(1); let command = AcceptedCommand { command_id: 1, world_id: state.world_id, turn: state.turn, accepted_sequence: 1, actor_id: CivId(0), origin: CommandOrigin::Governor, kind: crate::world::CommandKind::DeclareAttack, payload: CommandPayload::DeclareAttack { attacker: crate::ids::UnitId(1), target: crate::ids::UnitId(2) }, grounding: Vec::new(), intent_evidence: None, mandate: Some(mandate) }; let versions = crate::world::SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 }; assert!(matches!(crate::world::step(&state, &[command], state.seed, &versions).events.first(), Some(crate::world::DomainEvent::CommandRejected { reason: crate::world::RejectionReason::MandateViolation, .. }))); }
    #[test] fn absent_governor_never_declares_war() { let mut mandate = Mandate::balanced(1); mandate.red_lines.clear(); let candidate = CandidateAction { id: "war".into(), scope: Scope::ExplorationDefense, payload: CommandPayload::DeclareAttack { attacker: crate::ids::UnitId(1), target: crate::ids::UnitId(2) }, facts: vec![GroundingRef::Civilization { civilization: CivId(0) }, GroundingRef::Turn { turn: TurnNumber::ZERO }], benefits: mandate.direction, opportunity_cost: 0, exposed_risk: 0, irreversible: true }; assert!(matches!(Governor { mandate: &mandate, decision_port: None }.filter(&candidate, true), Filter::Deny(rule) if rule == "absent_irreversible")); }
    #[test] fn governor_is_deterministic_and_reports_two_facts() { let state = state(); let mandate = Mandate::balanced(1); let governor = Governor { mandate: &mandate, decision_port: None }; let first = governor.decide(&state, CivId(0), TileIndex(0), false, 1, 1); assert_eq!(first.commands.len(), 1); let second = governor.decide(&state, CivId(0), TileIndex(0), false, 1, 1); assert_eq!(first, second); assert_eq!(first.report.did[0].facts.len(), 2); }
}
