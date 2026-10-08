//! Diplomacy: pair state machine, directional Ledger and integer acceptance (GDD 07, SDD 07).
//!
//! Everything here is pure integer state owned by `WorldState::diplomacy`. The only entry points
//! that mutate it are validated commands applied by `world::step` and the per-turn
//! `resolve_diplomacy` phase. AI layers only produce intents; nothing in this module reads
//! a clock, a model or the network.
//!
//! Settlement of trade and aid goods belongs to the Economy slice, which does not exist yet:
//! diplomacy records the obligation (Ledger entries, deltas, deadlines) and nothing else.
//! Counter-proposals and era aggregation of old entries are not implemented yet; an acceptance
//! between the refuse and accept thresholds is reported as `Undecided` without effect.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    hash::{fnv1a, StateHasher},
    ids::{CivId, TurnNumber},
    world::{AcceptedCommand, CommandOrigin, CommandPayload, DomainEvent, GroundingRef, RejectionReason, WorldState},
};

/// Version of the rule tables below; recorded in every audit record.
pub const RULES_VERSION: &str = "diplomacy-rules-1";
/// Era length in turns. Ledger decay only happens when an era closes.
pub const ERA_LENGTH_TURNS: u32 = 100;
/// Acceptance below this value is a refusal.
pub const REFUSE_BELOW: i32 = 40;
/// Neutral trust baseline.
pub const NEUTRAL_CF: i32 = 50;
/// Trust given to both sides by a first contact.
pub const FIRST_CONTACT_CF: i8 = 8;
/// A refused proposal cannot be repeated for this many turns.
pub const REFUSAL_BLOCK_TURNS: u32 = 2;
/// Upper bound of the cost/risk term K.
pub const MAX_COST: i32 = 30;
/// Each point of betrayal severity raises the cost of the offender's proposals.
pub const MARK_COST_PER_SEVERITY: i32 = 2;
/// Severity removed from a betrayal mark by one accepted reparation.
pub const REPARATION_STEP: u8 = 2;
/// A border or incident entry needs at least this intensity to justify a war.
pub const CLAIM_MIN_INTENSITY: u8 = 3;
const AGGRESSION_INTENSITY: u8 = 4;
const WAR_INTENSITY: u8 = 4;
/// Two civilizations share a border when one city of each is at most this far (hexes): their radius-2
/// work areas touch or overlap. Same distance as the border of `W` (SDD 15 §4.2). B2, 2026-10-07.
pub const BORDER_DISPUTE_DISTANCE: u32 = 5;
/// A shared border left in peace without a treaty for this many consecutive turns becomes a dispute.
pub const BORDER_DISPUTE_TURNS: u32 = 8;
/// Intensity of a `border.disputed` claim: exactly enough to justify a war (`CLAIM_MIN_INTENSITY`).
pub const BORDER_DISPUTE_INTENSITY: u8 = 3;
/// A dispute claim expires after this many turns if nobody acts on it.
pub const BORDER_DISPUTE_CLAIM_TURNS: u32 = 30;
/// War weariness: the utility of a truce grows by this much per two turns at war, up to `MAX_TRUCE_UTILITY`.
pub const TRUCE_UTILITY_PER_TWO_WAR_TURNS: i32 = 3;
pub const MAX_TRUCE_UTILITY: i32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationState { Unknown, Contact, Peace, Tension, Pact, Alliance, War, Truce }

/// War objectives from the catalog (GDD 07). They define the end condition, not a victory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarObjective { ProtectRoute, ContainIncursion, RecoverTerritory, EnforceReparation }

impl WarObjective {
    pub const ALL: [WarObjective; 4] = [Self::ProtectRoute, Self::ContainIncursion, Self::RecoverTerritory, Self::EnforceReparation];
    pub const fn id(self) -> &'static str {
        match self {
            Self::ProtectRoute => "objective.protect_route",
            Self::ContainIncursion => "objective.contain_incursion",
            Self::RecoverTerritory => "objective.recover_territory",
            Self::EnforceReparation => "objective.enforce_reparation",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> { Self::ALL.into_iter().find(|objective| objective.id() == id) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind { Trade, Passage, Aid, Pact, Alliance, Reparation, Truce }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerCategory { Promise, Debt, Offense, Aid, Trade, Border, Incident, Treaty }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryStatus { Active, Fulfilled, Expired, Breached, Superseded }

/// Immutable Ledger entry. `holder -> subject` is the direction: the deltas apply to the
/// holder's view of the subject. A status change is a new entry with `supersedes` set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: u64,
    pub turn: TurnNumber,
    pub holder: CivId,
    pub subject: CivId,
    pub category: LedgerCategory,
    pub status: EntryStatus,
    pub intensity: u8,
    pub cf_delta: i8,
    pub resentment_delta: i8,
    pub debt_delta: i8,
    pub due: Option<TurnNumber>,
    pub cause_command: Option<u64>,
    pub supersedes: Option<u64>,
}

/// Betrayal mark: it has no ordinary decay and only shrinks through an accepted reparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitorMark { pub offender: CivId, pub affected: CivId, pub cause_entry: u64, pub era: u32, pub severity: u8 }

/// Projection of the Ledger for one direction: trust `cf` 0..100, resentment `r` 0..100, debt `dv` -100..100.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance { pub cf: u8, pub r: u8, pub dv: i8 }

impl Default for Balance { fn default() -> Self { Self { cf: NEUTRAL_CF as u8, r: 0, dv: 0 } } }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationRecord { pub state: RelationState, pub since: TurnNumber, pub until: Option<TurnNumber>, pub objective: Option<WarObjective> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refusal { pub issuer: CivId, pub recipient: CivId, pub kind: ProposalKind, pub turn: TurnNumber }

/// The components of `A = 40 + 0.30(Cf-50) - 0.25R + 0.20Dv + U - K`, in integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceComponents { pub cf: i32, pub r: i32, pub dv: i32, pub utility: i32, pub cost: i32, pub acceptance: u8, pub threshold: u8 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaticOutcome { Accepted, Refused, Undecided, Executed }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum DiplomaticAction { Propose { kind: ProposalKind }, BreakTreaty, DeclareWar { objective: String } }

impl DiplomaticAction {
    pub fn from_payload(payload: &CommandPayload) -> Option<Self> {
        match payload {
            CommandPayload::ProposeDiplomacy { kind, .. } => Some(Self::Propose { kind: *kind }),
            CommandPayload::BreakTreaty { .. } => Some(Self::BreakTreaty),
            CommandPayload::DeclareWar { objective, .. } => Some(Self::DeclareWar { objective: objective.clone() }),
            _ => None,
        }
    }
}

/// What a diplomatic command did. It is carried by `DomainEvent::DiplomacyResolved`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomaticResolution {
    pub outcome: DiplomaticOutcome,
    pub components: Option<AcceptanceComponents>,
    pub from: RelationState,
    pub to: RelationState,
    pub entries: Vec<u64>,
    pub marks: Vec<u64>,
}

struct Terms { utility: i32, cost: i32, threshold: i32, duration: u32, intensity: u8 }

/// Initial catalog of proposal terms: concrete utility `U`, base cost/risk `K`, own acceptance
/// threshold, duration in turns and Ledger intensity. All values are initial and subject to balance.
const fn terms(kind: ProposalKind) -> Terms {
    match kind {
        ProposalKind::Trade => Terms { utility: 20, cost: 0, threshold: 60, duration: 20, intensity: 2 },
        ProposalKind::Aid => Terms { utility: 16, cost: 2, threshold: 60, duration: 30, intensity: 3 },
        ProposalKind::Passage => Terms { utility: 16, cost: 2, threshold: 60, duration: 20, intensity: 1 },
        ProposalKind::Pact => Terms { utility: 16, cost: 4, threshold: 60, duration: 50, intensity: 3 },
        ProposalKind::Alliance => Terms { utility: 20, cost: 4, threshold: 65, duration: 0, intensity: 5 },
        ProposalKind::Reparation => Terms { utility: 14, cost: 2, threshold: 60, duration: 0, intensity: 2 },
        // The truce utility is war weariness, computed in `evaluate` from the turns at war.
        ProposalKind::Truce => Terms { utility: 0, cost: 4, threshold: 50, duration: 10, intensity: 2 },
    }
}

fn target_state(kind: ProposalKind, current: RelationState) -> Option<RelationState> {
    use ProposalKind as K;
    use RelationState as S;
    match (kind, current) {
        (K::Trade | K::Aid, S::Contact) => Some(S::Peace),
        (K::Trade | K::Aid, S::Peace | S::Tension | S::Pact | S::Alliance) => Some(current),
        (K::Passage | K::Pact, S::Peace | S::Tension) => Some(S::Pact),
        (K::Alliance, S::Pact) => Some(S::Alliance),
        (K::Reparation, S::Peace | S::Tension | S::Pact | S::Alliance | S::Truce) => Some(current),
        (K::Truce, S::War) => Some(S::Truce),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct EntryDraft {
    holder: CivId, subject: CivId, category: LedgerCategory, status: EntryStatus, intensity: u8,
    cf: i8, r: i8, dv: i8, due: Option<TurnNumber>, cause: Option<u64>, supersedes: Option<u64>,
}

impl EntryDraft {
    fn new(holder: CivId, subject: CivId, category: LedgerCategory, intensity: u8) -> Self {
        Self { holder, subject, category, status: EntryStatus::Active, intensity, cf: 0, r: 0, dv: 0, due: None, cause: None, supersedes: None }
    }
    fn deltas(mut self, cf: i8, r: i8, dv: i8) -> Self { self.cf = cf; self.r = r; self.dv = dv; self }
    fn due(mut self, due: Option<TurnNumber>) -> Self { self.due = due; self }
    fn cause(mut self, command: u64) -> Self { self.cause = Some(command); self }
    fn status(mut self, status: EntryStatus) -> Self { self.status = status; self }
    fn replacing(mut self, old: u64) -> Self { self.supersedes = Some(old); self }
}

fn bump(balance: &mut Balance, cf: i8, r: i8, dv: i8) {
    balance.cf = (i16::from(balance.cf) + i16::from(cf)).clamp(0, 100) as u8;
    balance.r = (i16::from(balance.r) + i16::from(r)).clamp(0, 100) as u8;
    balance.dv = (i16::from(balance.dv) + i16::from(dv)).clamp(-100, 100) as i8;
}

fn pair(a: CivId, b: CivId) -> (CivId, CivId) { if a < b { (a, b) } else { (b, a) } }

fn push_option(bytes: &mut Vec<u8>, value: Option<u64>) {
    bytes.push(u8::from(value.is_some()));
    bytes.extend_from_slice(&value.unwrap_or(0).to_le_bytes());
}

/// Diplomatic state of a world: one record per pair, directional balances and the immutable log.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyState {
    /// Keyed by the lower, then the higher civilization id of the pair.
    pub relations: BTreeMap<CivId, BTreeMap<CivId, RelationRecord>>,
    /// Keyed by holder, then subject.
    pub balances: BTreeMap<CivId, BTreeMap<CivId, Balance>>,
    /// Append-only; an entry id is its index.
    pub entries: Vec<LedgerEntry>,
    /// Ids of entries that are `Active` and not superseded (a projection of `entries`).
    pub active: BTreeSet<u64>,
    /// Latest entry id per holder and subject (a projection of `entries`).
    pub latest: BTreeMap<CivId, BTreeMap<CivId, u64>>,
    pub marks: Vec<TraitorMark>,
    pub refusals: Vec<Refusal>,
    pub era: u32,
    /// Rolling digest of every entry ever recorded, so the state hash covers the full history in O(1).
    pub digest: u64,
    /// Consecutive turns each bordering pair (lower id, then higher) spent in peace without a treaty.
    #[serde(default)]
    pub disputes: BTreeMap<CivId, BTreeMap<CivId, u32>>,
}

impl DiplomacyState {
    pub fn state(&self, a: CivId, b: CivId) -> RelationState {
        let (low, high) = pair(a, b);
        self.relations.get(&low).and_then(|row| row.get(&high)).map_or(RelationState::Unknown, |record| record.state)
    }

    pub fn relation(&self, a: CivId, b: CivId) -> RelationRecord {
        let (low, high) = pair(a, b);
        self.relations.get(&low).and_then(|row| row.get(&high)).cloned()
            .unwrap_or(RelationRecord { state: RelationState::Unknown, since: TurnNumber::ZERO, until: None, objective: None })
    }

    pub(crate) fn set_relation(&mut self, a: CivId, b: CivId, record: RelationRecord) {
        let (low, high) = pair(a, b);
        self.relations.entry(low).or_default().insert(high, record);
    }

    pub fn balance(&self, holder: CivId, subject: CivId) -> Balance {
        self.balances.get(&holder).and_then(|row| row.get(&subject)).copied().unwrap_or_default()
    }

    pub fn entry(&self, id: u64) -> Option<&LedgerEntry> { self.entries.get(id as usize) }

    pub fn latest_entry(&self, holder: CivId, subject: CivId) -> Option<u64> {
        self.latest.get(&holder).and_then(|row| row.get(&subject)).copied()
    }

    /// True when a grounding list cites at least one Ledger entry that exists and involves `actor`.
    pub fn cites_entry(&self, actor: CivId, grounding: &[GroundingRef]) -> bool {
        grounding.iter().any(|fact| matches!(fact, GroundingRef::Ledger { entry } if self.involves(*entry, actor)))
    }

    pub fn involves(&self, entry: u64, actor: CivId) -> bool {
        self.entry(entry).is_some_and(|entry| entry.holder == actor || entry.subject == actor)
    }

    pub fn has_active(&self, a: CivId, b: CivId, category: LedgerCategory) -> bool {
        self.active.iter().any(|id| {
            let entry = &self.entries[*id as usize];
            entry.category == category && pair(entry.holder, entry.subject) == pair(a, b)
        })
    }

    fn owes_reparation(&self, offender: CivId, affected: CivId) -> bool {
        self.marks.iter().any(|mark| mark.offender == offender && mark.affected == affected)
    }

    fn push(&mut self, turn: TurnNumber, draft: EntryDraft) -> u64 {
        let id = self.entries.len() as u64;
        bump(self.balances.entry(draft.holder).or_default().entry(draft.subject).or_default(), draft.cf, draft.r, draft.dv);
        if let Some(old) = draft.supersedes { self.active.remove(&old); }
        if draft.status == EntryStatus::Active { self.active.insert(id); }
        self.latest.entry(draft.holder).or_default().insert(draft.subject, id);
        let entry = LedgerEntry {
            id, turn, holder: draft.holder, subject: draft.subject, category: draft.category, status: draft.status,
            intensity: draft.intensity, cf_delta: draft.cf, resentment_delta: draft.r, debt_delta: draft.dv,
            due: draft.due, cause_command: draft.cause, supersedes: draft.supersedes,
        };
        let mut bytes = self.digest.to_le_bytes().to_vec();
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&turn.0.to_le_bytes());
        bytes.extend_from_slice(&entry.holder.0.to_le_bytes());
        bytes.extend_from_slice(&entry.subject.0.to_le_bytes());
        bytes.extend_from_slice(&[entry.category as u8, entry.status as u8, entry.intensity, entry.cf_delta as u8, entry.resentment_delta as u8, entry.debt_delta as u8]);
        push_option(&mut bytes, entry.due.map(|due| u64::from(due.0)));
        push_option(&mut bytes, entry.cause_command);
        push_option(&mut bytes, entry.supersedes);
        self.digest = fnv1a(&bytes);
        self.entries.push(entry);
        id
    }

    fn push_pair(&mut self, turn: TurnNumber, a: CivId, b: CivId, category: LedgerCategory, intensity: u8, deltas: (i8, i8, i8), due: Option<TurnNumber>, command: u64) -> Vec<u64> {
        [(a, b), (b, a)].into_iter()
            .map(|(holder, subject)| self.push(turn, EntryDraft::new(holder, subject, category, intensity).deltas(deltas.0, deltas.1, deltas.2).due(due).cause(command)))
            .collect()
    }

    /// Closes every active entry of a category between two civilizations with a superseding entry.
    fn close_between(&mut self, turn: TurnNumber, a: CivId, b: CivId, category: LedgerCategory, status: EntryStatus) -> Vec<u64> {
        let ids: Vec<u64> = self.active.iter().copied().filter(|id| {
            let entry = &self.entries[*id as usize];
            entry.category == category && pair(entry.holder, entry.subject) == pair(a, b)
        }).collect();
        let mut created = Vec::new();
        for id in ids {
            let old = self.entries[id as usize].clone();
            let mut draft = EntryDraft::new(old.holder, old.subject, old.category, old.intensity).status(status).replacing(id);
            draft.cause = old.cause_command;
            created.push(self.push(turn, draft));
        }
        created
    }

    /// An Entropy event with a counterparty leaves an incident in the Ledger (holder = affected civilization).
    pub(crate) fn record_incident(&mut self, turn: TurnNumber, holder: CivId, subject: CivId, duration: u32) -> u64 {
        let intensity = duration.clamp(1, 10) as u8;
        self.push(turn, EntryDraft::new(holder, subject, LedgerCategory::Incident, intensity).deltas(0, 2, 0))
    }

    /// First contact: `unknown -> contact`, with one border fact per direction.
    pub(crate) fn establish_contact(&mut self, turn: TurnNumber, a: CivId, b: CivId) -> bool {
        if a == b || self.state(a, b) != RelationState::Unknown { return false; }
        self.set_relation(a, b, RelationRecord { state: RelationState::Contact, since: turn, until: None, objective: None });
        for (holder, subject) in [(a, b), (b, a)] {
            self.push(turn, EntryDraft::new(holder, subject, LedgerCategory::Border, 1).deltas(FIRST_CONTACT_CF, 0, 0));
        }
        true
    }

    /// `A = 40 + 0.30(Cf-50) - 0.25R + 0.20Dv + U - K`, clamped to 0..100, in integers
    /// (hundredths, floor division). `Cf`, `R` and `Dv` are the recipient's view of the issuer.
    /// For a truce `U` is war weariness: `min(20, 3 × turns at war / 2)` (B2, 2026-10-07).
    pub fn evaluate(&self, issuer: CivId, recipient: CivId, kind: ProposalKind, now: TurnNumber) -> AcceptanceComponents {
        let view = self.balance(recipient, issuer);
        let mut table = terms(kind);
        if kind == ProposalKind::Truce {
            let record = self.relation(issuer, recipient);
            let turns = if record.state == RelationState::War { now.0.saturating_sub(record.since.0) } else { 0 };
            table.utility = (TRUCE_UTILITY_PER_TWO_WAR_TURNS * turns.min(100) as i32 / 2).min(MAX_TRUCE_UTILITY);
        }
        let severity: i32 = if kind == ProposalKind::Reparation { 0 } else {
            self.marks.iter().filter(|mark| mark.offender == issuer && mark.affected == recipient).map(|mark| i32::from(mark.severity)).sum()
        };
        let cost = (table.cost + MARK_COST_PER_SEVERITY * severity).clamp(0, MAX_COST);
        let (cf, r, dv) = (i32::from(view.cf), i32::from(view.r), i32::from(view.dv));
        let hundredths = 4000 + 30 * (cf - NEUTRAL_CF) - 25 * r + 20 * dv + 100 * (table.utility - cost);
        AcceptanceComponents { cf, r, dv, utility: table.utility, cost, acceptance: hundredths.div_euclid(100).clamp(0, 100) as u8, threshold: table.threshold as u8 }
    }

    /// State and rate-limit preconditions of a proposal. Returns the state it would lead to.
    pub fn check_proposal(&self, issuer: CivId, recipient: CivId, kind: ProposalKind, now: TurnNumber) -> Result<RelationState, RejectionReason> {
        if issuer == recipient { return Err(RejectionReason::SelfDiplomacy); }
        if self.refusals.iter().any(|refusal| refusal.issuer == issuer && refusal.recipient == recipient && refusal.kind == kind && now.0 <= refusal.turn.0.saturating_add(REFUSAL_BLOCK_TURNS)) {
            return Err(RejectionReason::RepeatedProposalBlocked);
        }
        let target = target_state(kind, self.state(issuer, recipient)).ok_or(RejectionReason::InvalidDiplomaticTransition)?;
        if kind == ProposalKind::Reparation && !self.owes_reparation(issuer, recipient) { return Err(RejectionReason::InvalidDiplomaticTransition); }
        Ok(target)
    }

    /// Evaluates and, when accepted, applies a proposal. The counterpart is automatic, so bots and
    /// governors use the same formula (GDD 07).
    pub(crate) fn propose(&mut self, turn: TurnNumber, command: u64, issuer: CivId, recipient: CivId, kind: ProposalKind) -> Result<DiplomaticResolution, RejectionReason> {
        let target = self.check_proposal(issuer, recipient, kind, turn)?;
        let record = self.relation(issuer, recipient);
        let from = record.state;
        let components = self.evaluate(issuer, recipient, kind, turn);
        let acceptance = i32::from(components.acceptance);
        if acceptance < REFUSE_BELOW {
            self.refusals.retain(|refusal| turn.0 <= refusal.turn.0.saturating_add(REFUSAL_BLOCK_TURNS));
            self.refusals.push(Refusal { issuer, recipient, kind, turn });
            return Ok(DiplomaticResolution { outcome: DiplomaticOutcome::Refused, components: Some(components), from, to: from, entries: Vec::new(), marks: Vec::new() });
        }
        if acceptance < i32::from(components.threshold) {
            return Ok(DiplomaticResolution { outcome: DiplomaticOutcome::Undecided, components: Some(components), from, to: from, entries: Vec::new(), marks: Vec::new() });
        }
        let table = terms(kind);
        let w = table.intensity as i8;
        let due = (table.duration > 0).then(|| TurnNumber(turn.0.saturating_add(table.duration)));
        let mut to = target;
        let mut marks = Vec::new();
        let entries = match kind {
            ProposalKind::Trade => self.push_pair(turn, issuer, recipient, LedgerCategory::Trade, table.intensity, (w, 0, 0), due, command),
            ProposalKind::Aid => vec![self.push(turn, EntryDraft::new(recipient, issuer, LedgerCategory::Aid, table.intensity).deltas(2 * w, 0, w).due(due).cause(command))],
            ProposalKind::Passage | ProposalKind::Pact => self.push_pair(turn, issuer, recipient, LedgerCategory::Treaty, table.intensity, ((w + 1) / 2, 0, 0), due, command),
            ProposalKind::Alliance => {
                let mut created = self.close_between(turn, issuer, recipient, LedgerCategory::Treaty, EntryStatus::Superseded);
                created.extend(self.push_pair(turn, issuer, recipient, LedgerCategory::Treaty, table.intensity, ((w + 1) / 2, 0, 0), None, command));
                created
            }
            ProposalKind::Truce => self.push_pair(turn, issuer, recipient, LedgerCategory::Treaty, table.intensity, (1, -2, 0), due, command),
            ProposalKind::Reparation => {
                if let Some(mark) = self.marks.iter_mut().find(|mark| mark.offender == issuer && mark.affected == recipient) {
                    mark.severity = mark.severity.saturating_sub(REPARATION_STEP);
                    marks.push(mark.cause_entry);
                }
                self.marks.retain(|mark| mark.severity > 0);
                if from == RelationState::Tension && !self.owes_reparation(issuer, recipient) { to = RelationState::Peace; }
                vec![self.push(turn, EntryDraft::new(recipient, issuer, LedgerCategory::Debt, table.intensity).status(EntryStatus::Fulfilled).deltas(w, -3, 0).cause(command))]
            }
        };
        let until = if to == from { record.until } else if matches!(to, RelationState::Pact | RelationState::Truce) { due } else { None };
        let objective = if matches!(to, RelationState::War | RelationState::Truce) { record.objective } else { None };
        self.set_relation(issuer, recipient, RelationRecord { state: to, since: if to == from { record.since } else { turn }, until, objective });
        Ok(DiplomaticResolution { outcome: DiplomaticOutcome::Accepted, components: Some(components), from, to, entries, marks })
    }

    /// Breaks a binding commitment: records the breach, creates a betrayal mark and lowers the state.
    fn do_break(&mut self, turn: TurnNumber, command: u64, offender: CivId, other: CivId) -> Result<(RelationState, Vec<u64>, u64), RejectionReason> {
        let (to, severity) = match self.state(offender, other) {
            RelationState::Alliance => (RelationState::Tension, 6u8),
            RelationState::Pact => (RelationState::Tension, 4),
            RelationState::Truce => (RelationState::War, 5),
            _ => return Err(RejectionReason::InvalidDiplomaticTransition),
        };
        let mut entries = self.close_between(turn, offender, other, LedgerCategory::Treaty, EntryStatus::Breached);
        let breach = self.push(turn, EntryDraft::new(other, offender, LedgerCategory::Promise, severity / 2).status(EntryStatus::Breached).deltas(-(severity as i8), severity as i8, 0).cause(command));
        entries.push(breach);
        self.marks.push(TraitorMark { offender, affected: other, cause_entry: breach, era: self.era, severity });
        let record = self.relation(offender, other);
        self.set_relation(offender, other, RelationRecord { state: to, since: turn, until: None, objective: if to == RelationState::War { record.objective } else { None } });
        Ok((to, entries, breach))
    }

    pub(crate) fn break_treaty(&mut self, turn: TurnNumber, command: u64, offender: CivId, other: CivId) -> Result<DiplomaticResolution, RejectionReason> {
        if offender == other { return Err(RejectionReason::SelfDiplomacy); }
        let from = self.state(offender, other);
        let (to, entries, breach) = self.do_break(turn, command, offender, other)?;
        Ok(DiplomaticResolution { outcome: DiplomaticOutcome::Executed, components: None, from, to, entries, marks: vec![breach] })
    }

    /// A war needs a catalog objective and a registered claim or aggression of the actor against the target.
    pub(crate) fn declare_war(&mut self, turn: TurnNumber, command: u64, actor: CivId, target: CivId, objective: &str, cause: u64) -> Result<DiplomaticResolution, RejectionReason> {
        if actor == target { return Err(RejectionReason::SelfDiplomacy); }
        let from = self.state(actor, target);
        if matches!(from, RelationState::Unknown | RelationState::War) { return Err(RejectionReason::InvalidDiplomaticTransition); }
        let objective = WarObjective::from_id(objective).ok_or(RejectionReason::UnknownWarObjective)?;
        let claim = self.entry(cause).filter(|entry| {
            self.active.contains(&entry.id) && entry.holder == actor && entry.subject == target
                && matches!(entry.category, LedgerCategory::Offense | LedgerCategory::Border | LedgerCategory::Incident)
                && entry.intensity >= CLAIM_MIN_INTENSITY
        }).cloned().ok_or(RejectionReason::MissingCasusBelli)?;
        let mut entries = Vec::new();
        let mut marks = Vec::new();
        if matches!(from, RelationState::Pact | RelationState::Alliance | RelationState::Truce) {
            let (_, broken, breach) = self.do_break(turn, command, actor, target)?;
            entries.extend(broken);
            marks.push(breach);
        }
        // The claim is consumed so one fact cannot justify an endless chain of wars.
        entries.push(self.push(turn, EntryDraft::new(claim.holder, claim.subject, claim.category, claim.intensity).status(EntryStatus::Superseded).replacing(claim.id).cause(command)));
        let w = WAR_INTENSITY as i8;
        entries.push(self.push(turn, EntryDraft::new(target, actor, LedgerCategory::Incident, WAR_INTENSITY).deltas(-2 * w, 2 * w, 0).cause(command)));
        self.set_relation(actor, target, RelationRecord { state: RelationState::War, since: turn, until: None, objective: Some(objective) });
        Ok(DiplomaticResolution { outcome: DiplomaticOutcome::Executed, components: None, from, to: RelationState::War, entries, marks })
    }

    /// An attack outside a war is a registered aggression: it is the victim's casus belli.
    pub(crate) fn record_aggression(&mut self, turn: TurnNumber, command: u64, attacker: CivId, victim: CivId) {
        if attacker == victim || self.state(attacker, victim) == RelationState::War { return; }
        let w = AGGRESSION_INTENSITY as i8;
        self.push(turn, EntryDraft::new(victim, attacker, LedgerCategory::Offense, AGGRESSION_INTENSITY).deltas(-w, w, 0).cause(command));
        if self.state(attacker, victim) == RelationState::Peace {
            self.set_relation(attacker, victim, RelationRecord { state: RelationState::Tension, since: turn, until: None, objective: None });
        }
    }

    /// Border disputes (GDD 07; decided on 2026-10-07 by delegation of the user, B2). `borders` holds the
    /// bordering pairs (lower id first). A pair in `peace` keeps a counter of consecutive turns; any
    /// other state (contact, tension, pact, alliance, war, truce) resets it, so a treaty settles the
    /// border. At `BORDER_DISPUTE_TURNS` both sides record a `border.disputed` claim (category `Border`,
    /// intensity 3, `Cf -2`, `R +3`, expires in 30 turns) and the pair moves to `tension`.
    pub(crate) fn advance_disputes(&mut self, turn: TurnNumber, borders: &BTreeSet<(CivId, CivId)>) {
        let mut next: BTreeMap<CivId, BTreeMap<CivId, u32>> = BTreeMap::new();
        for &(low, high) in borders {
            if low == high || self.state(low, high) != RelationState::Peace { continue; }
            let turns = self.disputes.get(&low).and_then(|row| row.get(&high)).copied().unwrap_or(0).saturating_add(1);
            if turns < BORDER_DISPUTE_TURNS {
                next.entry(low).or_default().insert(high, turns);
                continue;
            }
            let due = Some(TurnNumber(turn.0.saturating_add(BORDER_DISPUTE_CLAIM_TURNS)));
            for (holder, subject) in [(low, high), (high, low)] {
                self.push(turn, EntryDraft::new(holder, subject, LedgerCategory::Border, BORDER_DISPUTE_INTENSITY).deltas(-2, 3, 0).due(due));
            }
            self.set_relation(low, high, RelationRecord { state: RelationState::Tension, since: turn, until: None, objective: None });
        }
        self.disputes = next;
    }

    /// Number of pairs in which `civ` is in `relation` (wars and tensions are the `active_conflicts` of
    /// the cohesion rule, GDD 12).
    pub fn pairs_in(&self, civ: CivId, relation: RelationState) -> u32 {
        self.relations.iter().flat_map(|(low, row)| row.iter().map(move |(high, record)| (*low, *high, record.state)))
            .filter(|(low, high, state)| *state == relation && (*low == civ || *high == civ)).count() as u32
    }

    pub fn active_wars(&self, civ: CivId) -> u32 { self.pairs_in(civ, RelationState::War) }

    /// Commitments of `civ` kept and broken for the cohesion rule (GDD 12): treaty entries held by `civ`
    /// fulfilled at their deadline during the previous turn's upkeep (`turn - 1`), and promises `civ`
    /// broke this turn (breaches are only recorded by commands). Each entry is counted exactly once.
    pub fn commitments(&self, civ: CivId, turn: TurnNumber) -> (u32, u32) {
        let previous = turn.0.checked_sub(1);
        let mut kept = 0;
        let mut broken = 0;
        for entry in self.entries.iter().rev() {
            if entry.turn.0 < previous.unwrap_or(turn.0) { break; }
            if entry.category == LedgerCategory::Treaty && entry.status == EntryStatus::Fulfilled && entry.holder == civ && Some(entry.turn.0) == previous { kept += 1; }
            if entry.category == LedgerCategory::Promise && entry.status == EntryStatus::Breached && entry.subject == civ && entry.turn == turn { broken += 1; }
        }
        (kept, broken)
    }

    /// Per-turn upkeep: due entries close, pacts and truces lapse to peace, eras decay the balances.
    pub(crate) fn advance(&mut self, turn: TurnNumber) {
        let due: Vec<u64> = self.active.iter().copied().filter(|id| self.entries[*id as usize].due.is_some_and(|due| due.0 <= turn.0)).collect();
        for id in due {
            let old = self.entries[id as usize].clone();
            let cf = i8::from(matches!(old.category, LedgerCategory::Trade | LedgerCategory::Treaty));
            let status = if matches!(old.category, LedgerCategory::Trade | LedgerCategory::Aid | LedgerCategory::Treaty | LedgerCategory::Promise) { EntryStatus::Fulfilled } else { EntryStatus::Expired };
            let mut draft = EntryDraft::new(old.holder, old.subject, old.category, old.intensity).status(status).deltas(cf, 0, 0).replacing(id);
            draft.cause = old.cause_command;
            self.push(turn, draft);
        }
        let lapsed: Vec<(CivId, CivId)> = self.relations.iter().flat_map(|(low, row)| row.iter().filter_map(move |(high, record)| {
            (matches!(record.state, RelationState::Pact | RelationState::Truce) && record.until.is_some_and(|until| until.0 <= turn.0)).then_some((*low, *high))
        })).collect();
        for (low, high) in lapsed {
            self.set_relation(low, high, RelationRecord { state: RelationState::Peace, since: turn, until: None, objective: None });
        }
        if (turn.0.saturating_add(1)) % ERA_LENGTH_TURNS == 0 { self.close_era(); }
    }

    /// Era decay: trust moves one point towards neutral, resentment drops one, debt moves one towards zero.
    /// Betrayal marks are deliberately untouched.
    pub fn close_era(&mut self) {
        for row in self.balances.values_mut() {
            for balance in row.values_mut() {
                balance.cf = if i32::from(balance.cf) > NEUTRAL_CF { balance.cf - 1 } else if i32::from(balance.cf) < NEUTRAL_CF { balance.cf + 1 } else { balance.cf };
                balance.r = balance.r.saturating_sub(1);
                balance.dv -= balance.dv.signum();
            }
        }
        self.era = self.era.saturating_add(1);
    }

    /// An active claim of `actor` against `target` that could justify a war, with the matching objective.
    pub fn war_basis(&self, actor: CivId, target: CivId) -> Option<(u64, WarObjective)> {
        self.active.iter().rev().find_map(|id| {
            let entry = &self.entries[*id as usize];
            if entry.holder != actor || entry.subject != target || entry.intensity < CLAIM_MIN_INTENSITY { return None; }
            match entry.category {
                LedgerCategory::Offense => Some((*id, WarObjective::EnforceReparation)),
                LedgerCategory::Border => Some((*id, WarObjective::RecoverTerritory)),
                LedgerCategory::Incident => Some((*id, WarObjective::ContainIncursion)),
                _ => None,
            }
        })
    }

    pub(crate) fn hash_into(&self, hasher: &mut StateHasher) {
        hasher.write_u32(self.era);
        hasher.write_u64(self.digest);
        hasher.write_u64(self.entries.len() as u64);
        for (low, row) in &self.relations {
            for (high, record) in row {
                hasher.write_u32(low.0);
                hasher.write_u32(high.0);
                hasher.write_u8(record.state as u8);
                hasher.write_u32(record.since.0);
                hasher.write_bool(record.until.is_some());
                hasher.write_u32(record.until.map_or(0, |until| until.0));
                hasher.write_u8(record.objective.map_or(255, |objective| objective as u8));
            }
        }
        for (holder, row) in &self.balances {
            for (subject, balance) in row {
                hasher.write_u32(holder.0);
                hasher.write_u32(subject.0);
                hasher.write_u8(balance.cf);
                hasher.write_u8(balance.r);
                hasher.write_i64(i64::from(balance.dv));
            }
        }
        hasher.write_u64(self.active.len() as u64);
        for id in &self.active { hasher.write_u64(*id); }
        hasher.write_u64(self.marks.len() as u64);
        for mark in &self.marks {
            hasher.write_u32(mark.offender.0);
            hasher.write_u32(mark.affected.0);
            hasher.write_u64(mark.cause_entry);
            hasher.write_u32(mark.era);
            hasher.write_u8(mark.severity);
        }
        hasher.write_u64(self.refusals.len() as u64);
        for refusal in &self.refusals {
            hasher.write_u32(refusal.issuer.0);
            hasher.write_u32(refusal.recipient.0);
            hasher.write_u8(refusal.kind as u8);
            hasher.write_u32(refusal.turn.0);
        }
        for (low, row) in &self.disputes {
            for (high, turns) in row {
                hasher.write_u32(low.0);
                hasher.write_u32(high.0);
                hasher.write_u32(*turns);
            }
        }
    }
}

fn preferred_kinds(relation: RelationState) -> &'static [ProposalKind] {
    use ProposalKind as K;
    match relation {
        RelationState::Contact => &[K::Trade, K::Aid],
        RelationState::Peace => &[K::Pact, K::Trade, K::Aid],
        RelationState::Tension => &[K::Reparation, K::Trade, K::Pact],
        RelationState::Pact => &[K::Alliance, K::Trade, K::Aid],
        RelationState::Alliance => &[K::Trade, K::Aid],
        RelationState::War => &[K::Truce],
        RelationState::Unknown | RelationState::Truce => &[],
    }
}

/// How readily a T0 bot turns a registered claim into a war (bot personalities, B2 2026-10-07).
/// It never bypasses the Mandate: the engine still rejects a war that violates a red line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarAppetite {
    /// War only when no peaceful proposal to that civilization would be accepted (the default T0 order).
    LastResort,
    /// War first when the claim is a border dispute (`recover_territory`), last resort otherwise.
    BorderFirst,
    /// War first on any claim.
    Eager,
}

/// War policy of a T0 bot: appetite plus prudence limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarPolicy {
    pub appetite: WarAppetite,
    /// No declaration while the actor's cohesion is below this value.
    pub min_cohesion: u8,
    /// No declaration while the actor is already at war with someone.
    pub single_front: bool,
    /// Whether the bot proposes pacts and alliances (binding treaties settle borders). It still accepts
    /// them by the engine's formula when another civilization proposes.
    pub seeks_treaties: bool,
}

impl Default for WarPolicy {
    /// The behaviour of the T0 bot before personalities: last resort, no prudence limits.
    fn default() -> Self { Self { appetite: WarAppetite::LastResort, min_cohesion: 0, single_front: false, seeks_treaties: true } }
}

fn ledger_grounding(state: &WorldState, actor: CivId, other: CivId) -> Option<Vec<GroundingRef>> {
    let diplomacy = &state.diplomacy;
    let anchor = diplomacy.latest_entry(actor, other)?;
    let mut grounding = vec![GroundingRef::Civilization { civilization: actor }, GroundingRef::Turn { turn: state.turn }, GroundingRef::Ledger { entry: anchor }];
    if let Some(reverse) = diplomacy.latest_entry(other, actor) { grounding.push(GroundingRef::Ledger { entry: reverse }); }
    Some(grounding)
}

/// T0 diplomacy for one civilization with the default war policy (see `t0_proposal_with`).
pub fn t0_proposal(state: &WorldState, actor: CivId) -> Option<(CommandPayload, Vec<GroundingRef>)> {
    t0_proposal_with(state, actor, WarPolicy::default())
}

/// T0 diplomacy for one civilization: at most one action per turn, the first valid one in
/// civilization and preference order. A proposal is only made when the engine's own formula says
/// the counterpart would accept. Every action cites Ledger entries that involve the actor.
/// Order (B2): (1) a truce the enemy would accept, (2) an eager war, (3) peaceful proposals, each
/// civilization in tension followed by a last-resort war.
pub fn t0_proposal_with(state: &WorldState, actor: CivId, policy: WarPolicy) -> Option<(CommandPayload, Vec<GroundingRef>)> {
    let me = state.civilizations.get(&actor)?;
    if me.frozen { return None; }
    let diplomacy = &state.diplomacy;
    let others = || state.civilizations.iter().filter(move |(other, civ)| **other != actor && !civ.frozen);
    let at_war = others().any(|(other, _)| diplomacy.state(actor, *other) == RelationState::War);
    for (other, _) in others() {
        if diplomacy.state(actor, *other) != RelationState::War || diplomacy.check_proposal(actor, *other, ProposalKind::Truce, state.turn).is_err() { continue; }
        let components = diplomacy.evaluate(actor, *other, ProposalKind::Truce, state.turn);
        if components.acceptance < components.threshold { continue; }
        let Some(grounding) = ledger_grounding(state, actor, *other) else { continue; };
        return Some((CommandPayload::ProposeDiplomacy { recipient: *other, kind: ProposalKind::Truce }, grounding));
    }
    let may_declare = me.cohesion >= policy.min_cohesion && !(policy.single_front && at_war);
    let war = |other: CivId, eager_only: bool| -> Option<(CommandPayload, Vec<GroundingRef>)> {
        if !may_declare || diplomacy.state(actor, other) != RelationState::Tension { return None; }
        let (cause, objective) = diplomacy.war_basis(actor, other)?;
        let eager = match policy.appetite {
            WarAppetite::LastResort => false,
            WarAppetite::BorderFirst => objective == WarObjective::RecoverTerritory,
            WarAppetite::Eager => true,
        };
        if eager_only && !eager { return None; }
        let mut grounding = ledger_grounding(state, actor, other)?;
        grounding.push(GroundingRef::Ledger { entry: cause });
        Some((CommandPayload::DeclareWar { target: other, objective: objective.id().into(), cause }, grounding))
    };
    if policy.appetite != WarAppetite::LastResort {
        if let Some(found) = others().find_map(|(other, _)| war(*other, true)) { return Some(found); }
    }
    for (other, civ) in others() {
        let relation = diplomacy.state(actor, *other);
        let Some(grounding) = ledger_grounding(state, actor, *other) else { continue; };
        for kind in preferred_kinds(relation) {
            let wanted = match kind {
                ProposalKind::Trade => !diplomacy.has_active(actor, *other, LedgerCategory::Trade),
                ProposalKind::Aid => me.treasury_wealth >= 20 && (civ.deprivation > 0 || civ.crisis_pressure >= 30),
                ProposalKind::Pact | ProposalKind::Alliance => policy.seeks_treaties,
                _ => true,
            };
            if !wanted || diplomacy.check_proposal(actor, *other, *kind, state.turn).is_err() { continue; }
            let components = diplomacy.evaluate(actor, *other, *kind, state.turn);
            if components.acceptance >= components.threshold {
                return Some((CommandPayload::ProposeDiplomacy { recipient: *other, kind: *kind }, grounding));
            }
        }
        if let Some(found) = war(*other, false) { return Some(found); }
    }
    None
}

/// Audit record of one diplomatic command of a bot, Governor or fallback (SDD 07).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomaticAudit {
    pub command_id: u64,
    pub turn: TurnNumber,
    pub actor: CivId,
    pub origin: CommandOrigin,
    pub action: Option<DiplomaticAction>,
    pub outcome: Option<DiplomaticOutcome>,
    pub rejection: Option<RejectionReason>,
    pub components: Option<AcceptanceComponents>,
    pub transition: Option<(RelationState, RelationState)>,
    pub produced_entries: Vec<u64>,
    pub grounding: Vec<GroundingRef>,
    /// Ledger entries cited by the action, extracted from `grounding`.
    pub ledger_entries: Vec<u64>,
    pub rules_version: String,
    pub fallback_reason: Option<String>,
}

impl DiplomaticAudit {
    /// An action is grounded when it cites at least one Ledger entry that exists and involves its actor.
    pub fn is_grounded(&self, diplomacy: &DiplomacyState) -> bool {
        !self.ledger_entries.is_empty() && self.ledger_entries.iter().all(|id| diplomacy.involves(*id, self.actor))
    }
}

/// Builds one audit record per diplomatic command issued by a bot, Governor or fallback.
pub fn audit_records(commands: &[AcceptedCommand], events: &[DomainEvent]) -> Vec<DiplomaticAudit> {
    let mut audits = Vec::new();
    for command in commands {
        if !command.payload.is_diplomatic() || !matches!(command.origin, CommandOrigin::Bot | CommandOrigin::Governor | CommandOrigin::Fallback) { continue; }
        let mut audit = DiplomaticAudit {
            command_id: command.command_id, turn: command.turn, actor: command.actor_id, origin: command.origin,
            action: DiplomaticAction::from_payload(&command.payload), outcome: None, rejection: None, components: None,
            transition: None, produced_entries: Vec::new(), grounding: command.grounding.clone(),
            ledger_entries: command.grounding.iter().filter_map(|fact| match fact { GroundingRef::Ledger { entry } => Some(*entry), _ => None }).collect(),
            rules_version: RULES_VERSION.to_string(),
            fallback_reason: command.intent_evidence.as_ref().and_then(|evidence| evidence.fallback_from).map(|status| format!("{status:?}")),
        };
        for event in events {
            match event {
                DomainEvent::DiplomacyResolved { command_id, resolution, .. } if *command_id == command.command_id => {
                    audit.outcome = Some(resolution.outcome);
                    audit.components = resolution.components;
                    audit.transition = Some((resolution.from, resolution.to));
                    audit.produced_entries = resolution.entries.clone();
                }
                DomainEvent::CommandRejected { command_id, reason } if *command_id == command.command_id => audit.rejection = Some(*reason),
                _ => {}
            }
        }
        audits.push(audit);
    }
    audits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        governor::{Governor, Mandate},
        ids::{CityId, TileIndex},
        world::{step, AcceptedCommand, CityFocus, CityGroup, CityState, CivilizationState, CommandKind, GroupFunction, RulesetRef, SimulationVersions, TileState, TileYields, WorldId},
    };

    const A: CivId = CivId(0);
    const B: CivId = CivId(1);

    fn versions(state: &WorldState) -> SimulationVersions { SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 } }

    fn world() -> WorldState {
        WorldState {
            world_id: WorldId(1), turn: TurnNumber::ZERO, seed: 5,
            ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 3 },
            schema_version: 1, map_width: 5, tiles: vec![TileState::default(); 25],
            civilizations: BTreeMap::from([(A, CivilizationState::default()), (B, CivilizationState::default())]),
            cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(),
            visibility: BTreeMap::new(), diplomacy: DiplomacyState::default(), entropy: Default::default(),
        }
    }

    fn city(owner: CivId, tile: u32) -> CityState {
        CityState {
            owner, tile: TileIndex(tile), focus: CityFocus::Supply, population: 2, housing: 4, food_stock: 2, growth_progress: 0,
            consecutive_food_shortages: 0, stability: 50,
            groups: vec![CityGroup { function: GroupFunction::Cultivators, population: 2, satisfaction: 50 }, CityGroup { function: GroupFunction::Crafts, population: 0, satisfaction: 50 }, CityGroup { function: GroupFunction::Merchants, population: 0, satisfaction: 50 }],
            workplaces: Vec::new(), last_yields: TileYields::default(), deprivation: 0, group_tension: 0, war_threat: 0,
            environmental_exposure: 0, crisis_pressure: 0, crisis_turns: 0, essential_maintenance_unpaid: false,
            unit_queue: Vec::new(), unit_production: 0,
        }
    }

    fn player(id: u64, sequence: u64, actor: CivId, payload: CommandPayload) -> AcceptedCommand {
        AcceptedCommand { command_id: id, world_id: WorldId(1), turn: TurnNumber::ZERO, accepted_sequence: sequence, actor_id: actor, origin: CommandOrigin::Player, kind: payload.kind(), payload, grounding: Vec::new(), intent_evidence: None, mandate: None }
    }

    fn run(state: &WorldState, commands: &[AcceptedCommand]) -> crate::world::StepResult { step(state, commands, state.seed, &versions(state)) }

    fn resolution(result: &crate::world::StepResult, id: u64) -> DiplomaticResolution {
        result.events.iter().find_map(|event| match event { DomainEvent::DiplomacyResolved { command_id, resolution, .. } if *command_id == id => Some(resolution.clone()), _ => None }).expect("a resolution event")
    }

    fn rejection(result: &crate::world::StepResult, id: u64) -> Option<RejectionReason> {
        result.events.iter().find_map(|event| match event { DomainEvent::CommandRejected { command_id, reason } if *command_id == id => Some(*reason), _ => None })
    }

    fn with_pact(state: &mut WorldState) {
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        state.diplomacy.set_relation(A, B, RelationRecord { state: RelationState::Pact, since: TurnNumber::ZERO, until: None, objective: None });
        state.diplomacy.push_pair(TurnNumber::ZERO, A, B, LedgerCategory::Treaty, 3, (2, 0, 0), None, 0);
    }

    #[test]
    fn acceptance_formula_is_exact_integer_and_deterministic() {
        let mut diplomacy = DiplomacyState::default();
        diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 70, r: 20, dv: 10 });
        // 40 + 0.30*20 - 0.25*20 + 0.20*10 + 20 - 0 = 63 for trade; floor of 4000+600-500+200+2000 = 6300.
        let first = diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO);
        assert_eq!((first.cf, first.r, first.dv, first.utility, first.cost, first.acceptance, first.threshold), (70, 20, 10, 20, 0, 63, 60));
        assert_eq!(first, diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO));
        // Floor division on a fractional hundredth: 4000 + 30*3 + 2000 = 6090 -> 60.
        diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 53, r: 0, dv: 0 });
        assert_eq!(diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO).acceptance, 60);
        diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 0, r: 100, dv: -100 });
        assert_eq!(diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO).acceptance, 0);
        diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 100, r: 0, dv: 100 });
        assert_eq!(diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO).acceptance, 95);
    }

    #[test]
    fn first_contact_trade_is_accepted_and_opens_peace() {
        let mut state = world();
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        assert_eq!(state.diplomacy.state(A, B), RelationState::Contact);
        let result = run(&state, &[player(1, 1, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Trade })]);
        let resolved = resolution(&result, 1);
        assert_eq!(resolved.outcome, DiplomaticOutcome::Accepted);
        assert_eq!((resolved.from, resolved.to), (RelationState::Contact, RelationState::Peace));
        assert_eq!(result.state.diplomacy.state(B, A), RelationState::Peace);
        assert!(result.state.diplomacy.balance(A, B).cf > 58);
        assert_eq!(run(&state, &[player(1, 1, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Trade })]).state_hash, result.state_hash);
    }

    #[test]
    fn invalid_transitions_are_rejected_without_effect() {
        let state = world();
        // Unknown pair: nothing can be proposed.
        let result = run(&state, &[player(1, 1, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Trade })]);
        assert_eq!(rejection(&result, 1), Some(RejectionReason::InvalidDiplomaticTransition));
        assert!(result.state.diplomacy.entries.is_empty());
        let result = run(&state, &[player(2, 1, A, CommandPayload::ProposeDiplomacy { recipient: A, kind: ProposalKind::Trade })]);
        assert_eq!(rejection(&result, 2), Some(RejectionReason::SelfDiplomacy));
    }

    #[test]
    fn betrayal_marks_do_not_decay() {
        let mut state = world();
        with_pact(&mut state);
        let result = run(&state, &[player(1, 1, A, CommandPayload::BreakTreaty { counterpart: B })]);
        assert_eq!(resolution(&result, 1).to, RelationState::Tension);
        let mut diplomacy = result.state.diplomacy;
        assert_eq!(diplomacy.marks.len(), 1);
        let mark = diplomacy.marks[0].clone();
        assert_eq!((mark.offender, mark.affected, mark.severity), (A, B, 4));
        let baseline = diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO).cost;
        let resentment = diplomacy.balance(B, A).r;
        assert!(resentment >= 4);
        for _ in 0..200 { diplomacy.close_era(); }
        assert_eq!(diplomacy.marks, vec![mark]);
        assert_eq!(diplomacy.balance(B, A).r, 0, "ordinary resentment decays");
        assert_eq!(diplomacy.balance(B, A).cf, 50, "trust returns to the neutral baseline");
        assert_eq!(diplomacy.evaluate(A, B, ProposalKind::Trade, TurnNumber::ZERO).cost, baseline, "the mark keeps raising the offender's cost");
        assert!(baseline >= MARK_COST_PER_SEVERITY * 4);
    }

    #[test]
    fn reparation_reduces_severity_only_through_an_accepted_proposal() {
        let mut state = world();
        with_pact(&mut state);
        let mut state = run(&state, &[player(1, 1, A, CommandPayload::BreakTreaty { counterpart: B })]).state;
        state.turn = TurnNumber::ZERO;
        state.diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 80, r: 0, dv: 0 });
        let result = run(&state, &[player(2, 1, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Reparation })]);
        assert_eq!(resolution(&result, 2).outcome, DiplomaticOutcome::Accepted);
        assert_eq!(result.state.diplomacy.marks[0].severity, 4 - REPARATION_STEP);
        // Without a mark there is nothing to repair.
        let mut clean = world();
        clean.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        assert_eq!(rejection(&run(&clean, &[player(3, 1, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Reparation })]), 3), Some(RejectionReason::InvalidDiplomaticTransition));
    }

    #[test]
    fn war_requires_a_catalog_objective_and_a_registered_claim() {
        let mut state = world();
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        state.diplomacy.set_relation(A, B, RelationRecord { state: RelationState::Peace, since: TurnNumber::ZERO, until: None, objective: None });
        state.diplomacy.record_aggression(TurnNumber::ZERO, 9, B, A);
        let claim = state.diplomacy.latest_entry(A, B).unwrap();
        assert_eq!(state.diplomacy.state(A, B), RelationState::Tension);
        let war = |id: u64, objective: &str, cause: u64| player(id, id, A, CommandPayload::DeclareWar { target: B, objective: objective.into(), cause });
        let result = run(&state, &[war(1, "", claim)]);
        assert_eq!(rejection(&result, 1), Some(RejectionReason::UnknownWarObjective));
        let result = run(&state, &[war(2, "objective.conquer_everything", claim)]);
        assert_eq!(rejection(&result, 2), Some(RejectionReason::UnknownWarObjective));
        assert_eq!(result.state.diplomacy.state(A, B), RelationState::Tension);
        // The first contact fact is not a claim.
        let contact = state.diplomacy.entries.iter().find(|entry| entry.category == LedgerCategory::Border && entry.holder == A).unwrap().id;
        let result = run(&state, &[war(3, "objective.protect_route", contact)]);
        assert_eq!(rejection(&result, 3), Some(RejectionReason::MissingCasusBelli));
        let result = run(&state, &[war(4, "objective.enforce_reparation", 9_999)]);
        assert_eq!(rejection(&result, 4), Some(RejectionReason::MissingCasusBelli));
        let result = run(&state, &[war(5, "objective.enforce_reparation", claim)]);
        assert_eq!(resolution(&result, 5).to, RelationState::War);
        assert_eq!(result.state.diplomacy.relation(B, A).objective, Some(WarObjective::EnforceReparation));
        // The consumed claim cannot justify the same war twice.
        assert!(!result.state.diplomacy.active.contains(&claim));
    }

    #[test]
    fn truce_ends_a_war_and_lapses_to_peace_at_its_deadline() {
        let mut state = world();
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        state.diplomacy.set_relation(A, B, RelationRecord { state: RelationState::War, since: TurnNumber::ZERO, until: None, objective: Some(WarObjective::ProtectRoute) });
        // War weariness: no utility on the day of the declaration, 3 per two turns at war afterwards.
        let truce = |id: u64, turn: TurnNumber| AcceptedCommand { turn, ..player(id, id, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Truce }) };
        assert_eq!(state.diplomacy.evaluate(A, B, ProposalKind::Truce, TurnNumber::ZERO).utility, 0);
        assert_eq!(state.diplomacy.evaluate(A, B, ProposalKind::Truce, TurnNumber(4)).utility, 6);
        assert_eq!(state.diplomacy.evaluate(A, B, ProposalKind::Truce, TurnNumber(40)).utility, MAX_TRUCE_UTILITY);
        assert_ne!(resolution(&run(&state, &[truce(1, TurnNumber::ZERO)]), 1).outcome, DiplomaticOutcome::Accepted);
        state.turn = TurnNumber(10);
        let result = run(&state, &[truce(2, TurnNumber(10))]);
        assert_eq!(resolution(&result, 2).outcome, DiplomaticOutcome::Accepted);
        let mut state = result.state;
        assert_eq!(state.diplomacy.state(A, B), RelationState::Truce);
        for _ in 0..12 { let next = step(&state, &[], state.seed, &versions(&state)); state = next.state; }
        assert_eq!(state.diplomacy.state(A, B), RelationState::Peace);
    }

    #[test]
    fn refused_proposal_is_blocked_for_two_turns() {
        let mut state = world();
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        state.diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 0, r: 100, dv: 0 });
        let trade = |id: u64| player(id, id, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Trade });
        let first = run(&state, &[trade(1)]);
        assert_eq!(resolution(&first, 1).outcome, DiplomaticOutcome::Refused);
        let again = run(&first.state, &[]);
        let blocked = step(&again.state, &[AcceptedCommand { turn: again.state.turn, ..trade(2) }], 5, &versions(&again.state));
        assert_eq!(rejection(&blocked, 2), Some(RejectionReason::RepeatedProposalBlocked));
    }

    #[test]
    fn era_decay_is_monotonic_and_bounded() {
        let mut diplomacy = DiplomacyState::default();
        diplomacy.balances.entry(A).or_default().insert(B, Balance { cf: 100, r: 3, dv: -2 });
        let mut previous = diplomacy.balance(A, B);
        for _ in 0..120 {
            diplomacy.close_era();
            let now = diplomacy.balance(A, B);
            assert!(now.cf <= previous.cf && now.cf >= 50 && now.r <= previous.r && now.dv.abs() <= previous.dv.abs());
            previous = now;
        }
        assert_eq!(previous, Balance { cf: 50, r: 0, dv: 0 });
    }

    #[test]
    fn contact_is_detected_from_visibility_and_is_deterministic() {
        let mut state = world();
        state.cities.insert(CityId(0), city(A, 0));
        state.cities.insert(CityId(1), city(B, 1));
        let first = run(&state, &[]);
        assert_eq!(first.state.diplomacy.state(A, B), RelationState::Contact);
        assert_eq!(first.state.diplomacy.entries.len(), 2);
        assert_eq!(first.state_hash, run(&state, &[]).state_hash);
        assert_ne!(first.state_hash, {
            let mut other = state.clone();
            other.cities.remove(&CityId(1));
            run(&other, &[]).state_hash
        });
    }

    #[test]
    fn governor_diplomacy_cites_ledger_entries_and_is_audited() {
        let mut state = world();
        state.cities.insert(CityId(0), city(A, 0));
        state.cities.insert(CityId(1), city(B, 24));
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        let mandate = Mandate::balanced(1);
        let governor = Governor { mandate: &mandate, decision_port: None };
        let decision = governor.decide(&state, A, TileIndex(0), true, 1, 1);
        let diplomatic: Vec<_> = decision.commands.iter().filter(|command| command.payload.is_diplomatic()).collect();
        assert_eq!(diplomatic.len(), 1, "the diplomacy slot yields exactly one action");
        assert_eq!(diplomatic[0].kind, CommandKind::ProposeDiplomacy);
        let result = run(&state, &decision.commands);
        let audits = audit_records(&decision.commands, &result.events);
        assert_eq!(audits.len(), 1);
        let audit = &audits[0];
        assert_eq!(audit.outcome, Some(DiplomaticOutcome::Accepted));
        assert_eq!(audit.rejection, None);
        assert!(audit.is_grounded(&result.state.diplomacy));
        assert!(audit.ledger_entries.len() >= 2);
        assert!(audit.grounding.iter().any(|fact| matches!(fact, GroundingRef::Ledger { .. })));
        assert_eq!(audit.grounding, diplomatic[0].grounding);
        assert_eq!(audit.rules_version, RULES_VERSION);
        assert_eq!(audit.transition, Some((RelationState::Contact, RelationState::Peace)));
        assert!(audit.components.is_some_and(|components| components.acceptance >= components.threshold));
        assert_eq!(decision.commands, governor.decide(&state, A, TileIndex(0), true, 1, 1).commands);
    }

    #[test]
    fn ungrounded_bot_diplomacy_is_rejected_by_the_engine() {
        let mut state = world();
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        let mut command = player(1, 1, A, CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Trade });
        command.origin = CommandOrigin::Bot;
        command.grounding = vec![GroundingRef::Turn { turn: TurnNumber::ZERO }, GroundingRef::Ledger { entry: 9_999 }];
        let result = run(&state, &[command.clone()]);
        assert_eq!(rejection(&result, 1), Some(RejectionReason::UngroundedDiplomacy));
        let audits = audit_records(&[command], &result.events);
        assert_eq!(audits[0].rejection, Some(RejectionReason::UngroundedDiplomacy));
        assert!(!audits[0].is_grounded(&result.state.diplomacy));
    }

    #[test]
    fn mandate_red_lines_block_war_and_treaty_breaks() {
        let mandate = Mandate::balanced(1);
        assert!(!mandate.allows_payload(&CommandPayload::DeclareWar { target: B, objective: "objective.protect_route".into(), cause: 0 }));
        assert!(!mandate.allows_payload(&CommandPayload::BreakTreaty { counterpart: B }));
        assert!(mandate.allows_payload(&CommandPayload::ProposeDiplomacy { recipient: B, kind: ProposalKind::Trade }));
    }

    #[test]
    fn governor_declares_war_only_with_a_claim_and_when_the_mandate_allows_it() {
        let mut state = world();
        state.cities.insert(CityId(0), city(A, 0));
        state.cities.insert(CityId(1), city(B, 24));
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        state.diplomacy.set_relation(A, B, RelationRecord { state: RelationState::Peace, since: TurnNumber::ZERO, until: None, objective: None });
        state.diplomacy.record_aggression(TurnNumber::ZERO, 9, B, A);
        state.diplomacy.balances.entry(B).or_default().insert(A, Balance { cf: 50, r: 40, dv: 0 });
        let mut mandate = Mandate::balanced(1);
        let blocked = Governor { mandate: &mandate, decision_port: None }.decide(&state, A, TileIndex(0), false, 1, 1);
        assert!(!blocked.commands.iter().any(|command| matches!(command.payload, CommandPayload::DeclareWar { .. })));
        assert!(blocked.report.did_not.iter().any(|item| item.rule == "red_line"));
        mandate.red_lines.clear();
        let allowed = Governor { mandate: &mandate, decision_port: None }.decide(&state, A, TileIndex(0), false, 1, 1);
        let result = run(&state, &allowed.commands);
        let audits = audit_records(&allowed.commands, &result.events);
        let war = audits.iter().find(|audit| matches!(audit.action, Some(DiplomaticAction::DeclareWar { .. }))).expect("a war audit");
        assert_eq!(war.outcome, Some(DiplomaticOutcome::Executed));
        assert_eq!(war.transition, Some((RelationState::Tension, RelationState::War)));
        assert!(war.is_grounded(&result.state.diplomacy));
        // An absent player's Governor never takes an irreversible step.
        let absent = Governor { mandate: &mandate, decision_port: None }.decide(&state, A, TileIndex(0), true, 1, 1);
        assert!(!absent.commands.iter().any(|command| matches!(command.payload, CommandPayload::DeclareWar { .. })));
    }

    fn bordering_peace() -> WorldState {
        let mut state = world();
        state.cities.insert(CityId(0), city(A, 0));
        state.cities.insert(CityId(1), city(B, 2));
        state.diplomacy.establish_contact(TurnNumber::ZERO, A, B);
        state.diplomacy.set_relation(A, B, RelationRecord { state: RelationState::Peace, since: TurnNumber::ZERO, until: None, objective: None });
        state
    }

    fn steps(mut state: WorldState, turns: u32) -> WorldState {
        for _ in 0..turns { state = step(&state, &[], state.seed, &versions(&state)).state; }
        state
    }

    #[test]
    fn an_unsettled_border_becomes_a_dispute_claim_and_tension() {
        let state = steps(bordering_peace(), BORDER_DISPUTE_TURNS - 1);
        assert_eq!(state.diplomacy.state(A, B), RelationState::Peace);
        assert_eq!(state.diplomacy.war_basis(A, B), None);
        let state = steps(state, 1);
        assert_eq!(state.diplomacy.state(A, B), RelationState::Tension);
        for (holder, subject) in [(A, B), (B, A)] {
            let (cause, objective) = state.diplomacy.war_basis(holder, subject).expect("a border claim");
            let claim = state.diplomacy.entry(cause).unwrap();
            assert_eq!((claim.category, claim.intensity, objective), (LedgerCategory::Border, BORDER_DISPUTE_INTENSITY, WarObjective::RecoverTerritory));
        }
        assert!(state.diplomacy.balance(A, B).r >= 3);
        // The claim expires if nobody acts on it; the pair stays in tension until a treaty or reparation.
        let later = steps(state.clone(), BORDER_DISPUTE_CLAIM_TURNS);
        assert_eq!(later.diplomacy.war_basis(A, B), None);
        // Determinism: the same history gives the same hash.
        assert_eq!(steps(bordering_peace(), BORDER_DISPUTE_TURNS).state_hash(), state.state_hash());
    }

    #[test]
    fn a_treaty_or_distance_prevents_border_disputes() {
        let mut pact = bordering_peace();
        pact.diplomacy.set_relation(A, B, RelationRecord { state: RelationState::Pact, since: TurnNumber::ZERO, until: None, objective: None });
        assert_eq!(steps(pact, BORDER_DISPUTE_TURNS + 2).diplomacy.state(A, B), RelationState::Pact);
        let mut far = bordering_peace();
        far.map_width = 20;
        far.tiles = vec![TileState::default(); 40];
        far.cities.get_mut(&CityId(1)).unwrap().tile = TileIndex(10);
        assert_eq!(steps(far, BORDER_DISPUTE_TURNS + 2).diplomacy.state(A, B), RelationState::Peace);
    }

    #[test]
    fn commitments_count_fulfilled_treaties_once_and_breaches_on_their_turn() {
        let mut diplomacy = DiplomacyState::default();
        diplomacy.push_pair(TurnNumber::ZERO, A, B, LedgerCategory::Treaty, 3, (2, 0, 0), Some(TurnNumber(3)), 0);
        diplomacy.push_pair(TurnNumber::ZERO, A, B, LedgerCategory::Trade, 2, (2, 0, 0), Some(TurnNumber(3)), 0);
        diplomacy.advance(TurnNumber(3));
        assert_eq!(diplomacy.commitments(A, TurnNumber(3)), (0, 0), "fulfilled after this turn's society");
        assert_eq!(diplomacy.commitments(A, TurnNumber(4)), (1, 0), "trade is not a binding commitment");
        assert_eq!(diplomacy.commitments(B, TurnNumber(4)), (1, 0));
        assert_eq!(diplomacy.commitments(A, TurnNumber(5)), (0, 0), "counted exactly once");
        let mut state = world();
        with_pact(&mut state);
        let broken = run(&state, &[player(1, 1, A, CommandPayload::BreakTreaty { counterpart: B })]).state.diplomacy;
        assert_eq!(broken.commitments(A, TurnNumber::ZERO), (0, 1));
        assert_eq!(broken.commitments(B, TurnNumber::ZERO), (0, 0));
        assert_eq!(broken.active_wars(A), 0);
    }

    #[test]
    fn war_policy_orders_war_truce_and_peaceful_proposals() {
        let mut state = steps(bordering_peace(), BORDER_DISPUTE_TURNS);
        assert_eq!(state.diplomacy.state(A, B), RelationState::Tension);
        // A trade the counterpart would accept comes first for a last-resort bot.
        assert!(matches!(t0_proposal(&state, A), Some((CommandPayload::ProposeDiplomacy { kind: ProposalKind::Trade, .. }, _))));
        let eager = WarPolicy { appetite: WarAppetite::Eager, min_cohesion: 10, single_front: true, seeks_treaties: false };
        let border_first = WarPolicy { appetite: WarAppetite::BorderFirst, ..eager };
        for policy in [eager, border_first] {
            let Some((CommandPayload::DeclareWar { target, objective, cause }, grounding)) = t0_proposal_with(&state, A, policy) else { panic!("expected a war for {policy:?}") };
            assert_eq!((target, objective.as_str()), (B, "objective.recover_territory"));
            assert!(grounding.contains(&GroundingRef::Ledger { entry: cause }));
        }
        // Prudence: no war below the cohesion floor or on a second front.
        state.civilizations.get_mut(&A).unwrap().cohesion = 9;
        assert!(!matches!(t0_proposal_with(&state, A, eager), Some((CommandPayload::DeclareWar { .. }, _))));
        state.civilizations.get_mut(&A).unwrap().cohesion = 50;
        let c = CivId(2);
        state.civilizations.insert(c, CivilizationState::default());
        state.diplomacy.set_relation(A, c, RelationRecord { state: RelationState::War, since: state.turn, until: None, objective: Some(WarObjective::ContainIncursion) });
        state.diplomacy.push_pair(state.turn, A, c, LedgerCategory::Incident, 4, (0, 0, 0), None, 0);
        assert!(!matches!(t0_proposal_with(&state, A, eager), Some((CommandPayload::DeclareWar { .. }, _))));
        // A wearied war ends: once the enemy would accept a truce it is proposed before anything else.
        state.turn = TurnNumber(state.turn.0 + 12);
        assert!(matches!(t0_proposal_with(&state, A, eager), Some((CommandPayload::ProposeDiplomacy { recipient, kind: ProposalKind::Truce }, _)) if recipient == c));
    }

    #[test]
    fn belligerent_bots_declare_grounded_wars_and_cautious_bots_cannot() {
        use crate::personality::BotPersonality;
        let state = steps(bordering_peace(), BORDER_DISPUTE_TURNS);
        let decide = |personality: BotPersonality| {
            let mandate = personality.mandate(1);
            let decision = Governor { mandate: &mandate, decision_port: None }.decide_with(&state, A, TileIndex(0), false, 1, 1, personality.war_policy());
            (decision.clone(), run(&state, &decision.commands))
        };
        let (decision, result) = decide(BotPersonality::Belligerent);
        let audits = audit_records(&decision.commands, &result.events);
        let war = audits.iter().find(|audit| matches!(audit.action, Some(DiplomaticAction::DeclareWar { .. }))).expect("a war audit");
        assert_eq!(war.outcome, Some(DiplomaticOutcome::Executed));
        assert!(war.is_grounded(&result.state.diplomacy));
        assert_eq!(result.state.diplomacy.state(A, B), RelationState::War);
        let (cautious, result) = decide(BotPersonality::Cautious);
        assert!(!cautious.commands.iter().any(|command| matches!(command.payload, CommandPayload::DeclareWar { .. })));
        assert_ne!(result.state.diplomacy.state(A, B), RelationState::War);
        // The engine still enforces the red line of the Mandate carried by the command.
        let mut forged = decision.commands.iter().find(|command| matches!(command.payload, CommandPayload::DeclareWar { .. })).unwrap().clone();
        forged.mandate = Some(BotPersonality::Cautious.mandate(1));
        assert_eq!(rejection(&run(&state, &[forged.clone()]), forged.command_id), Some(RejectionReason::MandateViolation));
    }
}
