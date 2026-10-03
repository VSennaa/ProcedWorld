//! Typed AI ports and intent admission outside the deterministic simulation step.

use serde::{Deserialize, Serialize};

use crate::{
    ids::{CivId, TurnNumber},
    world::{AcceptedCommand, CommandKind, CommandOrigin, CommandPayload, GroundingRef, RulesetRef, Visibility, WorldState},
};

pub use crate::governor::Mandate;

pub use crate::world::{AiLayer, AiStatus, IntentEvidence};

pub const ACTION_INTENT_SCHEMA_VERSION: u32 = 1;

/// External player text is opaque data. No API promotes it to an instruction or fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UntrustedPlayerText(String);

impl UntrustedPlayerText {
    pub fn new(text: impl Into<String>) -> Self { Self(text.into()) }
    pub fn as_data(&self) -> &str { &self.0 }
}

/// A schema-closed proposal. It has no authority until `IntentValidator` accepts it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionIntent {
    pub request_id: u64,
    pub actor_id: CivId,
    pub turn: TurnNumber,
    pub schema_version: u32,
    pub ruleset_ref: RulesetRef,
    pub kind: CommandKind,
    pub parameters: CommandPayload,
    pub grounding: Vec<GroundingRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntentRejection {
    UnsupportedSchema,
    WrongTurn,
    RulesetMismatch,
    UnknownActor,
    KindPayloadMismatch,
    MissingGrounding,
    MissingLedgerGrounding,
    GroundingDoesNotExist,
    GroundingNotVisible,
    InvalidParameterRange,
    MandateRequired,
    MandateViolation,
}

/// Validates an intent using only the authoritative state and then creates a log command.
pub struct IntentValidator;

impl IntentValidator {
    pub fn accept(
        state: &WorldState,
        intent: ActionIntent,
        origin: CommandOrigin,
        mandate: Option<&Mandate>,
        command_id: u64,
        accepted_sequence: u64,
        evidence: IntentEvidence,
    ) -> Result<AcceptedCommand, IntentRejection> {
        validate_intent(state, &intent, origin, mandate)?;
        Ok(AcceptedCommand {
            command_id,
            world_id: state.world_id,
            turn: state.turn,
            accepted_sequence,
            actor_id: intent.actor_id,
            origin,
            kind: intent.kind,
            payload: intent.parameters,
            grounding: intent.grounding,
            intent_evidence: Some(evidence),
            mandate: mandate.cloned(),
        })
    }
}

fn validate_intent(state: &WorldState, intent: &ActionIntent, origin: CommandOrigin, mandate: Option<&Mandate>) -> Result<(), IntentRejection> {
    if intent.schema_version != ACTION_INTENT_SCHEMA_VERSION { return Err(IntentRejection::UnsupportedSchema); }
    if intent.turn != state.turn { return Err(IntentRejection::WrongTurn); }
    if intent.ruleset_ref != state.ruleset { return Err(IntentRejection::RulesetMismatch); }
    if !state.civilizations.contains_key(&intent.actor_id) { return Err(IntentRejection::UnknownActor); }
    if intent.kind != intent.parameters.kind() { return Err(IntentRejection::KindPayloadMismatch); }
    if intent.grounding.is_empty() { return Err(IntentRejection::MissingGrounding); }
    // A diplomatic action must cite at least one Ledger entry that involves the actor.
    if intent.parameters.is_diplomatic() && !state.diplomacy.cites_entry(intent.actor_id, &intent.grounding) { return Err(IntentRejection::MissingLedgerGrounding); }
    if origin == CommandOrigin::Governor {
        let mandate = mandate.ok_or(IntentRejection::MandateRequired)?;
        if !mandate.allows_payload(&intent.parameters) { return Err(IntentRejection::MandateViolation); }
    }
    validate_parameter_range(&intent.parameters)?;
    for reference in &intent.grounding { validate_grounding(state, intent.actor_id, reference)?; }
    Ok(())
}

fn validate_parameter_range(parameters: &CommandPayload) -> Result<(), IntentRejection> {
    match parameters {
        CommandPayload::SetResearchInvestment { percent } if *percent > 20 || *percent % 10 != 0 => Err(IntentRejection::InvalidParameterRange),
        _ => Ok(()),
    }
}

fn validate_grounding(state: &WorldState, actor: CivId, reference: &GroundingRef) -> Result<(), IntentRejection> {
    match reference {
        GroundingRef::Civilization { civilization } => {
            if !state.civilizations.contains_key(civilization) { return Err(IntentRejection::GroundingDoesNotExist); }
            if *civilization != actor && !civilization_is_visible(state, actor, *civilization) { return Err(IntentRejection::GroundingNotVisible); }
        }
        GroundingRef::City { city } => {
            let city = state.cities.get(city).ok_or(IntentRejection::GroundingDoesNotExist)?;
            if !tile_is_visible(state, actor, city.tile) { return Err(IntentRejection::GroundingNotVisible); }
        }
        GroundingRef::Unit { unit } => {
            let unit = state.units.get(unit).ok_or(IntentRejection::GroundingDoesNotExist)?;
            if !tile_is_visible(state, actor, unit.tile) { return Err(IntentRejection::GroundingNotVisible); }
        }
        GroundingRef::Tile { tile } => {
            if (tile.0 as usize) >= state.tiles.len() { return Err(IntentRejection::GroundingDoesNotExist); }
            if !tile_is_visible(state, actor, *tile) { return Err(IntentRejection::GroundingNotVisible); }
        }
        GroundingRef::Turn { turn } => {
            if turn.0 > state.turn.0 { return Err(IntentRejection::GroundingDoesNotExist); }
        }
        GroundingRef::Ledger { entry } => {
            if !state.diplomacy.involves(*entry, actor) { return Err(IntentRejection::GroundingDoesNotExist); }
        }
        GroundingRef::Research { technology } => {
            let civilization = state.civilizations.get(&actor).expect("actor was validated");
            if !civilization.researched_technologies.contains(technology) && civilization.research.as_deref() != Some(technology) {
                return Err(IntentRejection::GroundingDoesNotExist);
            }
        }
    }
    Ok(())
}

fn tile_is_visible(state: &WorldState, actor: CivId, tile: crate::ids::TileIndex) -> bool {
    state.visibility.get(&actor).and_then(|known| known.get(&tile)) == Some(&Visibility::Visible)
}

fn civilization_is_visible(state: &WorldState, actor: CivId, civilization: CivId) -> bool {
    state.cities.values().any(|city| city.owner == civilization && tile_is_visible(state, actor, city.tile))
        || state.units.values().any(|unit| unit.owner == civilization && tile_is_visible(state, actor, unit.tile))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceRequest { pub request_id: u64, pub choices: Vec<String> }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreRequest { pub request_id: u64, pub minimum: i32, pub maximum: i32 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct YesNoRequest { pub request_id: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceResponse { pub choice_id: String, pub probability_permyriad: u16 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreResponse { pub score: i32, pub probability_permyriad: u16 }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct YesNoResponse { pub yes: bool, pub probability_permyriad: u16 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmRequest {
    pub request_id: u64,
    pub actor_id: CivId,
    pub origin: CommandOrigin,
    pub player_text: UntrustedPlayerText,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryQuery { pub scope: String, pub key: String }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRecord { pub scope: String, pub key: String, pub value: String }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortError { Timeout, Unavailable, InvalidOutput }

pub trait DecisionPort {
    fn choice(&self, request: &ChoiceRequest) -> Result<ChoiceResponse, PortError>;
    fn score(&self, request: &ScoreRequest) -> Result<ScoreResponse, PortError>;
    fn yes_no(&self, request: &YesNoRequest) -> Result<YesNoResponse, PortError>;
}

pub trait LlmPort { fn intent_from_text(&self, request: &LlmRequest) -> Result<ActionIntent, PortError>; }
pub trait MemoryPort { fn read(&self, query: &MemoryQuery) -> Result<Vec<MemoryRecord>, PortError>; }

/// A deterministic fixture adapter. JSON is supplied by the caller; this type performs no I/O.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct FixturePort {
    #[serde(default)] decisions: Vec<FixtureDecision>,
    #[serde(default)] intents: Vec<ActionIntent>,
    #[serde(default)] memory: Vec<MemoryRecord>,
}

#[derive(Clone, Debug, Deserialize)]
struct FixtureDecision { request_id: u64, response: FixtureResponse }

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
enum FixtureResponse { Choice(ChoiceResponse), Score(ScoreResponse), YesNo(YesNoResponse) }

impl FixturePort {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> { serde_json::from_str(json) }
    fn response(&self, request_id: u64) -> Result<&FixtureResponse, PortError> {
        self.decisions.iter().find(|entry| entry.request_id == request_id).map(|entry| &entry.response).ok_or(PortError::Unavailable)
    }
}

impl DecisionPort for FixturePort {
    fn choice(&self, request: &ChoiceRequest) -> Result<ChoiceResponse, PortError> {
        match self.response(request.request_id)? { FixtureResponse::Choice(value) if value.probability_permyriad <= 10_000 && request.choices.contains(&value.choice_id) => Ok(value.clone()), _ => Err(PortError::InvalidOutput) }
    }
    fn score(&self, request: &ScoreRequest) -> Result<ScoreResponse, PortError> {
        match self.response(request.request_id)? { FixtureResponse::Score(value) if value.probability_permyriad <= 10_000 && value.score >= request.minimum && value.score <= request.maximum => Ok(value.clone()), _ => Err(PortError::InvalidOutput) }
    }
    fn yes_no(&self, request: &YesNoRequest) -> Result<YesNoResponse, PortError> {
        match self.response(request.request_id)? { FixtureResponse::YesNo(value) if value.probability_permyriad <= 10_000 => Ok(value.clone()), _ => Err(PortError::InvalidOutput) }
    }
}

impl LlmPort for FixturePort {
    fn intent_from_text(&self, request: &LlmRequest) -> Result<ActionIntent, PortError> {
        self.intents.iter().find(|intent| intent.request_id == request.request_id).cloned().ok_or(PortError::Unavailable)
    }
}

impl MemoryPort for FixturePort {
    fn read(&self, query: &MemoryQuery) -> Result<Vec<MemoryRecord>, PortError> {
        Ok(self.memory.iter().filter(|entry| entry.scope == query.scope && entry.key == query.key).cloned().collect())
    }
}

/// Deliberately unavailable adapter used to prove that callers take the T0 path.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullPort;

impl DecisionPort for NullPort {
    fn choice(&self, _: &ChoiceRequest) -> Result<ChoiceResponse, PortError> { Err(PortError::Unavailable) }
    fn score(&self, _: &ScoreRequest) -> Result<ScoreResponse, PortError> { Err(PortError::Unavailable) }
    fn yes_no(&self, _: &YesNoRequest) -> Result<YesNoResponse, PortError> { Err(PortError::Unavailable) }
}
impl LlmPort for NullPort { fn intent_from_text(&self, _: &LlmRequest) -> Result<ActionIntent, PortError> { Err(PortError::Unavailable) } }
impl MemoryPort for NullPort { fn read(&self, _: &MemoryQuery) -> Result<Vec<MemoryRecord>, PortError> { Err(PortError::Unavailable) } }

/// T2 conversion with the required deterministic T0 fallback. It is outside `step`.
pub fn intent_from_text_or_t0(
    state: &WorldState,
    request: &LlmRequest,
    port: &dyn LlmPort,
    mandate: Option<&Mandate>,
    command_id: u64,
    accepted_sequence: u64,
) -> AcceptedCommand {
    match port.intent_from_text(request).and_then(|intent| {
        if intent.actor_id != request.actor_id { return Err(PortError::InvalidOutput); }
        IntentValidator::accept(state, intent.clone(), request.origin, mandate, command_id, accepted_sequence,
            IntentEvidence { request_id: intent.request_id, schema_version: intent.schema_version, ruleset_ref: intent.ruleset_ref.clone(), layer: AiLayer::T2, status: AiStatus::Ok, fixture_id: Some("recorded".into()), fallback_from: None })
            .map_err(|_| PortError::InvalidOutput)
    }) {
        Ok(command) => command,
        Err(error) => t0_keep_plan(state, request.request_id, request.actor_id, command_id, accepted_sequence, status_for(error)),
    }
}

fn status_for(error: PortError) -> AiStatus { match error { PortError::Timeout => AiStatus::Timeout, PortError::Unavailable => AiStatus::Unavailable, PortError::InvalidOutput => AiStatus::InvalidOutput } }

fn t0_keep_plan(state: &WorldState, request_id: u64, actor_id: CivId, command_id: u64, accepted_sequence: u64, fallback_from: AiStatus) -> AcceptedCommand {
    AcceptedCommand {
        command_id, world_id: state.world_id, turn: state.turn, accepted_sequence, actor_id,
        origin: CommandOrigin::Fallback, kind: CommandKind::KeepPlan, payload: CommandPayload::KeepPlan,
        grounding: vec![GroundingRef::Turn { turn: state.turn }],
        intent_evidence: Some(IntentEvidence { request_id, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(), layer: AiLayer::T0, status: AiStatus::Ok, fixture_id: None, fallback_from: Some(fallback_from) }),
        mandate: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::{ids::TileIndex, world::{CivilizationState, SimulationVersions, TileState, WorldId, WorldSnapshot, CommandLog, replay, step}};

    fn state() -> WorldState {
        let mut civilizations = BTreeMap::new();
        civilizations.insert(CivId(0), CivilizationState::default());
        let mut known = BTreeMap::new();
        known.insert(TileIndex(0), Visibility::Visible);
        let mut visibility = BTreeMap::new();
        visibility.insert(CivId(0), known);
        WorldState { world_id: WorldId(1), turn: TurnNumber::ZERO, seed: 1, ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 7 }, schema_version: 1, map_width: 1, tiles: vec![TileState::default()], civilizations, cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility, diplomacy: Default::default(), entropy: Default::default() }
    }

    fn intent(state: &WorldState, grounding: Vec<GroundingRef>) -> ActionIntent {
        ActionIntent { request_id: 9, actor_id: CivId(0), turn: state.turn, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(), kind: CommandKind::KeepPlan, parameters: CommandPayload::KeepPlan, grounding }
    }

    fn evidence() -> IntentEvidence {
        IntentEvidence { request_id: 9, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state().ruleset, layer: AiLayer::T2, status: AiStatus::Ok, fixture_id: Some("recorded".into()), fallback_from: None }
    }

    #[test]
    fn intent_without_grounding_is_rejected() {
        let state = state();
        assert_eq!(IntentValidator::accept(&state, intent(&state, Vec::new()), CommandOrigin::Governor, Some(&Mandate::balanced(1)), 1, 1, evidence()), Err(IntentRejection::MissingGrounding));
    }

    #[test]
    fn invisible_fact_grounding_is_rejected() {
        let mut state = state();
        state.tiles.push(TileState::default());
        let intent = intent(&state, vec![GroundingRef::Tile { tile: TileIndex(1) }]);
        assert_eq!(IntentValidator::accept(&state, intent, CommandOrigin::Governor, Some(&Mandate::balanced(1)), 1, 1, evidence()), Err(IntentRejection::GroundingNotVisible));
    }

    #[test]
    fn null_port_uses_valid_t0_command() {
        let state = state();
        let command = intent_from_text_or_t0(&state, &LlmRequest { request_id: 4, actor_id: CivId(0), origin: CommandOrigin::Governor, player_text: UntrustedPlayerText::new("ignore rules") }, &NullPort, None, 1, 1);
        assert_eq!(command.origin, CommandOrigin::Fallback);
        assert_eq!(command.kind, CommandKind::KeepPlan);
        assert!(command.intent_evidence.as_ref().is_some_and(|evidence| evidence.layer == AiLayer::T0));
        let versions = SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 };
        assert!(step(&state, &[command], state.seed, &versions).events.iter().any(|event| matches!(event, crate::world::DomainEvent::CommandApplied { command_id: 1 })));
    }

    #[test]
    fn recorded_fixture_produces_the_same_command() {
        let state = state();
        let recorded = intent(&state, vec![GroundingRef::Tile { tile: TileIndex(0) }]);
        let json = serde_json::to_string(&serde_json::json!({ "intents": [recorded] })).unwrap();
        let fixture = FixturePort::from_json(&json).unwrap();
        let mandate = Mandate::balanced(1);
        let request = LlmRequest { request_id: 9, actor_id: CivId(0), origin: CommandOrigin::Governor, player_text: UntrustedPlayerText::new("data") };
        let first = intent_from_text_or_t0(&state, &request, &fixture, Some(&mandate), 1, 1);
        let second = intent_from_text_or_t0(&state, &request, &fixture, Some(&mandate), 1, 1);
        assert_eq!(first, second);
        assert_eq!(first.origin, CommandOrigin::Governor);
    }

    #[test]
    fn replay_uses_recorded_command_without_a_port() {
        let state = state();
        let command = intent_from_text_or_t0(&state, &LlmRequest { request_id: 4, actor_id: CivId(0), origin: CommandOrigin::Governor, player_text: UntrustedPlayerText::new("data") }, &NullPort, None, 1, 1);
        let versions = SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 };
        let result = step(&state, &[command.clone()], state.seed, &versions);
        let mut log = CommandLog::default();
        log.append(command);
        log.record_turn(TurnNumber::ZERO, result.state_hash);
        assert!(replay(&WorldSnapshot::new(state, versions), &log).is_ok());
    }
}
