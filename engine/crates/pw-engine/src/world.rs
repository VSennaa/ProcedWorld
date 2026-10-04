//! Deterministic world state, accepted commands, turn resolution, and replay.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::{
    diplomacy::{DiplomacyState, DiplomaticAction, DiplomaticResolution, ProposalKind},
    hex::Grid,
    hash::{StateHash, StateHasher},
    ids::{CivId, CityId, TileIndex, TurnNumber, UnitId},
    rng::Rng,
    dsl::EntityRef,
    entropy::EntropyState,
};

/// Stable identity of a world. It has no ordering semantics beyond its numeric value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct WorldId(pub u64);

/// Immutable reference to the rule package used by a world.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesetRef {
    pub id: String,
    pub version: String,
    pub content_hash: u64,
}

/// Versions required to run a deterministic transition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationVersions {
    pub ruleset: RulesetRef,
    pub resolver_version: u32,
}

/// Minimal mechanical data for one map tile.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileState {
    pub terrain: u8,
    /// A river increases the cost of entering this tile until bridge rules exist.
    pub river: bool,
    pub yields: TileYields,
    /// Finished terrain improvement (catalog id), at most one per tile (GDD 02).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub improvement: Option<String>,
    /// Construction in progress on this tile (docs/sdd/15 section 4.2.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<crate::improvements::TileBuild>,
}

pub const TERRAIN_PLAINS: u8 = 0;
pub const TERRAIN_FOREST: u8 = 1;
pub const TERRAIN_JUNGLE: u8 = 2;
pub const TERRAIN_SWAMP: u8 = 3;
pub const TERRAIN_DESERT: u8 = 4;
pub const TERRAIN_STEPPE: u8 = 5;
pub const TERRAIN_COAST: u8 = 6;
pub const TERRAIN_OCEAN: u8 = 7;

const RESEARCH_PERCENTAGES: [u8; 3] = [0, 10, 20];
const RESEARCH_PER_TURN_CAP: u32 = 10;
const ACTIVE_PRACTICE_LIMIT: usize = 3;
const VISIBILITY_RADIUS: u32 = 2;
/// Initial reserve funds the mandatory upkeep while a new settlement grows a
/// second worker able to collect wealth.
pub const INITIAL_TREASURY_WEALTH: u32 = 12;
/// Starter housing is an initial balance value. No housing construction slice
/// exists yet, so a lower value would make the health simulation impossible.
pub const INITIAL_CITY_HOUSING: u32 = 4;
/// A founding city starts with the one-turn food reserve required for growth.
/// This lets a one-food opening support its second worker instead of trapping
/// the civilization before it can allocate production or wealth.
pub const INITIAL_CITY_FOOD_STOCK: u32 = 1;
pub(crate) const SETTLER_CITY_ID_BASE: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Unknown,
    Remembered,
    Visible,
}

/// Per-worker tile output. Each category is capped at six after catalog effects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileYields {
    pub food: u8,
    pub production: u8,
    pub wealth: u8,
    pub knowledge: u8,
    pub culture: u8,
}

impl TileYields {
    pub(crate) fn capped(self) -> Self {
        Self { food: self.food.min(6), production: self.production.min(6), wealth: self.wealth.min(6), knowledge: self.knowledge.min(6), culture: self.culture.min(6) }
    }
}

/// Minimal civilization state, expanded by the domain tasks that own each system.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivilizationState {
    pub research: Option<String>,
    pub research_investment: u8,
    pub researched_technologies: BTreeSet<String>,
    pub research_progress: BTreeMap<String, u32>,
    pub active_practices: BTreeSet<String>,
    pub treasury_wealth: u32,
    pub cohesion: u8,
    pub legitimacy: u8,
    pub knowledge: u32,
    pub culture: u32,
    pub deprivation: u8,
    pub group_tension: u8,
    pub war_threat: u8,
    pub environmental_exposure: u8,
    pub crisis_pressure: u8,
    pub zero_cohesion_turns: u8,
    pub frozen: bool,
}

impl Default for CivilizationState {
    fn default() -> Self {
        Self { research: None, research_investment: 0, researched_technologies: BTreeSet::new(), research_progress: BTreeMap::new(), active_practices: BTreeSet::new(), treasury_wealth: INITIAL_TREASURY_WEALTH, cohesion: 50, legitimacy: 50, knowledge: 0, culture: 0, deprivation: 0, group_tension: 0, war_threat: 0, environmental_exposure: 0, crisis_pressure: 0, zero_cohesion_turns: 0, frozen: false }
    }
}

/// City production emphasis supported by the initial command surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityFocus {
    Supply,
    Build,
    Diversify,
}

/// The only social functions in a city. Their populations always sum to city population.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupFunction { Cultivators, Crafts, Merchants }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityGroup {
    pub function: GroupFunction,
    pub population: u32,
    pub satisfaction: u8,
}

impl CityGroup {
    fn new(function: GroupFunction, population: u32) -> Self { Self { function, population, satisfaction: 50 } }
}

/// Minimal city state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityState {
    pub owner: CivId,
    pub tile: TileIndex,
    pub focus: CityFocus,
    pub population: u32,
    pub housing: u32,
    pub food_stock: u32,
    pub growth_progress: u32,
    pub consecutive_food_shortages: u8,
    pub stability: u8,
    pub groups: Vec<CityGroup>,
    /// Filled each turn in ascending tile order after automatic allocation.
    pub workplaces: Vec<TileIndex>,
    pub last_yields: TileYields,
    pub deprivation: u8,
    pub group_tension: u8,
    /// Provisional values owned by later warfare/environment slices.
    pub war_threat: u8,
    pub environmental_exposure: u8,
    pub crisis_pressure: u8,
    pub crisis_turns: u8,
    pub essential_maintenance_unpaid: bool,
    pub unit_queue: Vec<String>,
    pub unit_production: u32,
}

/// Persistent order of a unit (docs/sdd/15 section 4.3). `MoveTo` and `Explore` run during
/// movement resolution of every turn; `Fortify` only ends with a new order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum UnitOrder {
    #[default]
    Idle,
    Fortify,
    Explore,
    MoveTo { target: TileIndex },
    /// Prontidao: parked outside the idle queue until a foreign unit comes into sight.
    Sentry,
    /// Worker builds `improvement` (catalog id) on its current tile; returns to `Idle` when done.
    Build { improvement: String },
}

/// Minimal unit state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitState {
    pub owner: CivId,
    pub tile: TileIndex,
    pub unit_type: String,
    pub hit_points: u8,
    pub movement_left: u8,
    pub explored: bool,
    /// Persistent order; a new unit starts `Idle`.
    #[serde(default)]
    pub order: UnitOrder,
    /// Turn number in which the unit was skipped (`SkipUnit`); only that turn is affected.
    #[serde(default)]
    pub skipped_turn: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackReservation {
    pub attacker: UnitId,
    pub target: UnitId,
    pub target_tile: TileIndex,
}

/// Canonical state required by the initial deterministic engine slice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub world_id: WorldId,
    pub turn: TurnNumber,
    pub seed: u64,
    pub ruleset: RulesetRef,
    pub schema_version: u32,
    /// Width of dense, row-major tile storage. It is required for the radius-two work rule.
    pub map_width: u32,
    /// Tiles are stored by ascending `TileIndex`.
    pub tiles: Vec<TileState>,
    pub civilizations: BTreeMap<CivId, CivilizationState>,
    pub cities: BTreeMap<CityId, CityState>,
    pub units: BTreeMap<UnitId, UnitState>,
    pub attacks: BTreeMap<UnitId, AttackReservation>,
    pub control: BTreeMap<TileIndex, CivId>,
    pub visibility: BTreeMap<CivId, BTreeMap<TileIndex, Visibility>>,
    /// Pair states, directional Ledger and betrayal marks (see `diplomacy`).
    #[serde(default)]
    pub diplomacy: DiplomacyState,
    /// Budgets, cooldowns, protections, pending events and tags of the Entropy director.
    #[serde(default)]
    pub entropy: EntropyState,
}

impl WorldState {
    pub fn state_hash(&self) -> StateHash {
        let mut hasher = StateHasher::new();
        hasher.write_u64(self.world_id.0);
        hasher.write_u32(self.turn.0);
        hasher.write_u64(self.seed);
        write_ruleset_ref(&mut hasher, &self.ruleset);
        hasher.write_u32(self.schema_version);
        hasher.write_u32(self.map_width);

        hasher.write_u64(self.tiles.len() as u64);
        for tile in &self.tiles {
            hasher.write_u8(tile.terrain);
            hasher.write_bool(tile.river);
            write_yields(&mut hasher, tile.yields);
            write_option_string(&mut hasher, tile.improvement.as_deref());
            hasher.write_bool(tile.build.is_some());
            if let Some(build) = &tile.build {
                write_string(&mut hasher, &build.improvement);
                hasher.write_u32(build.progress);
            }
        }

        hasher.write_u64(self.civilizations.len() as u64);
        for (id, civilization) in &self.civilizations {
            hasher.write_u32(id.0);
            write_option_string(&mut hasher, civilization.research.as_deref());
            hasher.write_u8(civilization.research_investment);
            write_string_set(&mut hasher, &civilization.researched_technologies);
            hasher.write_u64(civilization.research_progress.len() as u64);
            for (technology, progress) in &civilization.research_progress {
                write_string(&mut hasher, technology);
                hasher.write_u32(*progress);
            }
            write_string_set(&mut hasher, &civilization.active_practices);
            hasher.write_u32(civilization.treasury_wealth);
            hasher.write_u8(civilization.cohesion);
            hasher.write_u8(civilization.legitimacy);
            hasher.write_u32(civilization.knowledge);
            hasher.write_u32(civilization.culture);
            hasher.write_u8(civilization.deprivation);
            hasher.write_u8(civilization.group_tension);
            hasher.write_u8(civilization.war_threat);
            hasher.write_u8(civilization.environmental_exposure);
            hasher.write_u8(civilization.crisis_pressure);
            hasher.write_u8(civilization.zero_cohesion_turns);
            hasher.write_bool(civilization.frozen);
        }

        hasher.write_u64(self.cities.len() as u64);
        for (id, city) in &self.cities {
            hasher.write_u32(id.0);
            hasher.write_u32(city.owner.0);
            hasher.write_u32(city.tile.0);
            hasher.write_u8(city_focus_tag(city.focus));
            hasher.write_u32(city.population);
            hasher.write_u32(city.housing);
            hasher.write_u32(city.food_stock);
            hasher.write_u32(city.growth_progress);
            hasher.write_u8(city.consecutive_food_shortages);
            hasher.write_u8(city.stability);
            hasher.write_u64(city.groups.len() as u64);
            for group in &city.groups {
                hasher.write_u8(group_function_tag(group.function));
                hasher.write_u32(group.population);
                hasher.write_u8(group.satisfaction);
            }
            hasher.write_u64(city.workplaces.len() as u64);
            for workplace in &city.workplaces { hasher.write_u32(workplace.0); }
            write_yields(&mut hasher, city.last_yields);
            hasher.write_u8(city.deprivation);
            hasher.write_u8(city.group_tension);
            hasher.write_u8(city.war_threat);
            hasher.write_u8(city.environmental_exposure);
            hasher.write_u8(city.crisis_pressure);
            hasher.write_u8(city.crisis_turns);
            hasher.write_bool(city.essential_maintenance_unpaid);
            hasher.write_u64(city.unit_queue.len() as u64);
            for unit_type in &city.unit_queue { write_string(&mut hasher, unit_type); }
            hasher.write_u32(city.unit_production);
        }

        hasher.write_u64(self.units.len() as u64);
        for (id, unit) in &self.units {
            hasher.write_u32(id.0);
            hasher.write_u32(unit.owner.0);
            hasher.write_u32(unit.tile.0);
            write_string(&mut hasher, &unit.unit_type);
            hasher.write_u8(unit.hit_points);
            hasher.write_u8(unit.movement_left);
            hasher.write_bool(unit.explored);
            match &unit.order {
                UnitOrder::Idle => hasher.write_u8(0),
                UnitOrder::Fortify => hasher.write_u8(1),
                UnitOrder::Explore => hasher.write_u8(2),
                UnitOrder::MoveTo { target } => { hasher.write_u8(3); hasher.write_u32(target.0); }
                UnitOrder::Sentry => hasher.write_u8(4),
                UnitOrder::Build { improvement } => { hasher.write_u8(5); write_string(&mut hasher, improvement); }
            }
            hasher.write_bool(unit.skipped_turn.is_some());
            hasher.write_u32(unit.skipped_turn.unwrap_or(0));
        }

        hasher.write_u64(self.attacks.len() as u64);
        for (id, attack) in &self.attacks {
            hasher.write_u32(id.0);
            hasher.write_u32(attack.target.0);
            hasher.write_u32(attack.target_tile.0);
        }
        hasher.write_u64(self.control.len() as u64);
        for (tile, owner) in &self.control { hasher.write_u32(tile.0); hasher.write_u32(owner.0); }
        hasher.write_u64(self.visibility.len() as u64);
        for (civ, tiles) in &self.visibility {
            hasher.write_u32(civ.0);
            hasher.write_u64(tiles.len() as u64);
            for (tile, visibility) in tiles { hasher.write_u32(tile.0); hasher.write_u8(visibility_tag(*visibility)); }
        }

        self.diplomacy.hash_into(&mut hasher);
        self.entropy.write_hash(&mut hasher);
        hasher.finish()
    }
}

fn write_yields(hasher: &mut StateHasher, yields: TileYields) {
    hasher.write_u8(yields.food);
    hasher.write_u8(yields.production);
    hasher.write_u8(yields.wealth);
    hasher.write_u8(yields.knowledge);
    hasher.write_u8(yields.culture);
}

fn write_ruleset_ref(hasher: &mut StateHasher, ruleset: &RulesetRef) {
    write_string(hasher, &ruleset.id);
    write_string(hasher, &ruleset.version);
    hasher.write_u64(ruleset.content_hash);
}

fn write_option_string(hasher: &mut StateHasher, value: Option<&str>) {
    hasher.write_bool(value.is_some());
    if let Some(value) = value {
        write_string(hasher, value);
    }
}

fn write_string(hasher: &mut StateHasher, value: &str) {
    hasher.write_u64(value.len() as u64);
    hasher.write_bytes(value.as_bytes());
}

fn write_string_set(hasher: &mut StateHasher, values: &BTreeSet<String>) {
    hasher.write_u64(values.len() as u64);
    for value in values { write_string(hasher, value); }
}

const fn visibility_tag(visibility: Visibility) -> u8 {
    match visibility { Visibility::Unknown => 0, Visibility::Remembered => 1, Visibility::Visible => 2 }
}

const fn city_focus_tag(focus: CityFocus) -> u8 {
    match focus {
        CityFocus::Supply => 0,
        CityFocus::Build => 1,
        CityFocus::Diversify => 2,
    }
}

const fn group_function_tag(function: GroupFunction) -> u8 {
    match function { GroupFunction::Cultivators => 0, GroupFunction::Crafts => 1, GroupFunction::Merchants => 2 }
}

/// Source that produced a command. JSON uses the canonical lower-case names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandOrigin {
    Player,
    Governor,
    Bot,
    Entropy,
    Fallback,
    System,
}

/// The layer that supplied an audit record. It has no role in turn resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiLayer { T0, T1, T2 }

/// Recorded external-call status. These values are audit data, not simulation input.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus { Ok, Timeout, Unavailable, InvalidOutput }

/// Audit-only provenance recorded with an accepted command. Replay never consults it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEvidence {
    pub request_id: u64,
    pub schema_version: u32,
    pub ruleset_ref: RulesetRef,
    pub layer: AiLayer,
    pub status: AiStatus,
    pub fixture_id: Option<String>,
    pub fallback_from: Option<AiStatus>,
}

/// A stable reference to a state fact used to justify an automated decision.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum GroundingRef {
    Civilization { civilization: CivId },
    City { city: CityId },
    Unit { unit: UnitId },
    Tile { tile: TileIndex },
    Turn { turn: TurnNumber },
    Research { technology: String },
    /// A diplomatic Ledger entry (by index) that involves the acting civilization.
    Ledger { entry: u64 },
}

/// Initial set of command kinds. `KeepPlan` is the English name for ManterPlano.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    EndTurn,
    MoveUnit,
    FoundCity,
    SetCityFocus,
    SetResearch,
    SetResearchInvestment,
    ActivatePractice,
    DeactivatePractice,
    QueueUnit,
    RemoveQueuedUnit,
    MoveQueuedUnit,
    DeclareAttack,
    Explore,
    SetUnitOrder,
    SkipUnit,
    KeepPlan,
    ProposeDiplomacy,
    BreakTreaty,
    DeclareWar,
    ApplyEvent,
    RespondToEvent,
}

/// Typed payloads whose variant must agree with `AcceptedCommand::kind`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum CommandPayload {
    EndTurn,
    MoveUnit { unit_id: UnitId, target: TileIndex },
    FoundCity { city_id: CityId, target: TileIndex },
    SetCityFocus { city_id: CityId, focus: CityFocus },
    SetResearch { research: String },
    SetResearchInvestment { percent: u8 },
    ActivatePractice { practice: String },
    DeactivatePractice { practice: String },
    QueueUnit { city_id: CityId, unit_type: String },
    /// Drops the queue item at `index`; removing the head keeps the accumulated `unit_production`.
    RemoveQueuedUnit { city_id: CityId, index: u32 },
    /// Moves the queue item at `from` so it ends at position `to`.
    MoveQueuedUnit { city_id: CityId, from: u32, to: u32 },
    DeclareAttack { attacker: UnitId, target: UnitId },
    Explore { unit_id: UnitId },
    /// Replaces the persistent order of an own unit.
    SetUnitOrder { unit_id: UnitId, order: UnitOrder },
    /// Marks the unit as skipped for the current turn only; no mechanical effect.
    SkipUnit { unit_id: UnitId },
    KeepPlan,
    /// Structured proposal; the engine evaluates the automatic counterpart with `A`.
    ProposeDiplomacy { recipient: CivId, kind: ProposalKind },
    BreakTreaty { counterpart: CivId },
    /// `objective` is a catalog id (`objective.*`); `cause` is the Ledger entry that justifies the war.
    DeclareWar { target: CivId, objective: String, cause: u64 },
    /// An Entropy event: a materialized template (effects and choices already bound to one target).
    ApplyEvent {
        template_id: String,
        catalog_hash: u64,
        category: String,
        cost: u8,
        duration: u8,
        cooldown: u32,
        target: EntityRef,
        related: Option<CivId>,
        ops: Vec<crate::dsl::EffectOp>,
        choices: Vec<crate::dsl::EventChoice>,
    },
    RespondToEvent { event_id: u64, choice_id: String },
}

impl CommandPayload {
    pub(crate) const fn kind(&self) -> CommandKind {
        match self {
            Self::EndTurn => CommandKind::EndTurn,
            Self::MoveUnit { .. } => CommandKind::MoveUnit,
            Self::FoundCity { .. } => CommandKind::FoundCity,
            Self::SetCityFocus { .. } => CommandKind::SetCityFocus,
            Self::SetResearch { .. } => CommandKind::SetResearch,
            Self::SetResearchInvestment { .. } => CommandKind::SetResearchInvestment,
            Self::ActivatePractice { .. } => CommandKind::ActivatePractice,
            Self::DeactivatePractice { .. } => CommandKind::DeactivatePractice,
            Self::QueueUnit { .. } => CommandKind::QueueUnit,
            Self::RemoveQueuedUnit { .. } => CommandKind::RemoveQueuedUnit,
            Self::MoveQueuedUnit { .. } => CommandKind::MoveQueuedUnit,
            Self::DeclareAttack { .. } => CommandKind::DeclareAttack,
            Self::Explore { .. } => CommandKind::Explore,
            Self::SetUnitOrder { .. } => CommandKind::SetUnitOrder,
            Self::SkipUnit { .. } => CommandKind::SkipUnit,
            Self::KeepPlan => CommandKind::KeepPlan,
            Self::ProposeDiplomacy { .. } => CommandKind::ProposeDiplomacy,
            Self::BreakTreaty { .. } => CommandKind::BreakTreaty,
            Self::DeclareWar { .. } => CommandKind::DeclareWar,
            Self::ApplyEvent { .. } => CommandKind::ApplyEvent,
            Self::RespondToEvent { .. } => CommandKind::RespondToEvent,
        }
    }

    pub(crate) const fn is_diplomatic(&self) -> bool {
        matches!(self, Self::ProposeDiplomacy { .. } | Self::BreakTreaty { .. } | Self::DeclareWar { .. })
    }
}

/// A command already accepted and ordered by the authoritative server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptedCommand {
    pub command_id: u64,
    pub world_id: WorldId,
    pub turn: TurnNumber,
    pub accepted_sequence: u64,
    pub actor_id: CivId,
    pub origin: CommandOrigin,
    pub kind: CommandKind,
    pub payload: CommandPayload,
    #[serde(default)]
    pub grounding: Vec<GroundingRef>,
    /// Audit-only AI provenance. Replay applies this command without consulting it.
    #[serde(default)]
    pub intent_evidence: Option<IntentEvidence>,
    /// The immutable mandate snapshot used for a Governor command. The pure
    /// engine repeats the red-line check; callers cannot bypass it by avoiding
    /// Governor orchestration.
    #[serde(default)]
    pub mandate: Option<crate::governor::Mandate>,
}

/// Stable reason codes for command rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionReason {
    WrongWorld,
    WrongTurn,
    DuplicateAcceptedSequence,
    KindPayloadMismatch,
    UnknownCivilization,
    UnknownUnit,
    UnknownCity,
    NotCommandOwner,
    InvalidTile,
    DestinationOccupied,
    CityAlreadyExists,
    CityTileOccupied,
    RulesetMismatch,
    CivilizationFrozen,
    UnknownTechnology,
    MissingTechnologyPrerequisite,
    InvalidResearchInvestment,
    UnknownPractice,
    PracticeTechnologyNotResearched,
    PracticeLimitReached,
    UnknownUnitType,
    UnitTechnologyNotResearched,
    UnitAlreadyReserved,
    InvalidQueueIndex,
    MissingSettler,
    AttackNotHostile,
    AttackOutOfRange,
    UnitIdExhausted,
    TurnOverflow,
    MandateRequired,
    MandateViolation,
    InvalidDiplomaticTransition,
    UnknownWarObjective,
    MissingCasusBelli,
    RepeatedProposalBlocked,
    UngroundedDiplomacy,
    SelfDiplomacy,
    EventForbidden,
    EventInvalid,
    EventOnCooldown,
    EventOverBudget,
    EventProtected,
    EventBusy,
    EventUnsafeTarget,
    EventNoUsefulResponse,
    EventUnknown,
    EventChoiceInvalid,
    EventUnaffordable,
    NotAWorker,
    UnknownImprovement,
    ImprovementTechnologyNotResearched,
    ImprovementTerrainInvalid,
    TileAlreadyImproved,
    TileWorkInProgress,
    TileOutsideTerritory,
    CityTileNotImprovable,
}

#[derive(Deserialize)]
struct TechCatalog { technologies: Vec<TechDefinition> }

#[derive(Deserialize)]
struct TechDefinition {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    branch: String,
    prerequisites: Vec<String>,
    cost: u32,
    practice: PracticeDefinition,
}

#[derive(Deserialize)]
struct PracticeDefinition { id: String, maintenance: u32 }

#[derive(Deserialize)]
struct UnitCatalog { units: Vec<UnitDefinition> }

#[derive(Deserialize)]
struct UnitDefinition {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    role: String,
    cost: BTreeMap<String, u32>,
    movement: u8,
    strength: u8,
    requires_technology: String,
}

fn tech_catalog() -> TechCatalog {
    serde_json::from_str(include_str!("../../../../data/catalogs/tech_tree.json"))
        .expect("the bundled technology catalog is valid")
}

/// The first technology in catalog order that is not researched and whose prerequisites are met.
pub(crate) fn next_technology(researched: &BTreeSet<String>) -> Option<String> {
    tech_catalog().technologies.into_iter()
        .find(|technology| !researched.contains(&technology.id) && technology.prerequisites.iter().all(|required| researched.contains(required)))
        .map(|technology| technology.id)
}

const TECH_CATALOG_JSON: &str = include_str!("../../../../data/catalogs/tech_tree.json");
const UNIT_CATALOG_JSON: &str = include_str!("../../../../data/catalogs/units.json");

/// Technology tree and unit types for clients (docs/sdd/10 section 5.2). `hash` identifies the
/// exact bundled catalog data so a client can cache it.
pub fn client_catalog() -> serde_json::Value {
    let mut bytes = TECH_CATALOG_JSON.as_bytes().to_vec();
    bytes.extend_from_slice(UNIT_CATALOG_JSON.as_bytes());
    let version = |raw: &str| serde_json::from_str::<serde_json::Value>(raw).ok().and_then(|value| value.get("catalog_version").and_then(serde_json::Value::as_u64));
    let technologies: Vec<serde_json::Value> = tech_catalog().technologies.into_iter().map(|technology| serde_json::json!({
        "id": technology.id, "name": technology.name, "branch": technology.branch,
        "cost": technology.cost, "prerequisites": technology.prerequisites,
    })).collect();
    let units: Vec<serde_json::Value> = unit_catalog().units.into_iter().map(|unit| serde_json::json!({
        "id": unit.id, "name": unit.name, "role": unit.role, "movement": unit.movement,
        "strength": unit.strength, "requires_technology": unit.requires_technology,
        "cost": unit.cost,
    })).collect();
    serde_json::json!({
        "hash": format!("{:016x}", crate::hash::fnv1a(&bytes)),
        "versions": { "tech_tree": version(TECH_CATALOG_JSON), "units": version(UNIT_CATALOG_JSON) },
        "technologies": technologies,
        "units": units,
    })
}

fn unit_catalog() -> UnitCatalog {
    serde_json::from_str(include_str!("../../../../data/catalogs/units.json"))
        .expect("the bundled unit catalog is valid")
}

fn technology(id: &str) -> Option<TechDefinition> {
    tech_catalog().technologies.into_iter().find(|technology| technology.id == id)
}

fn practice(id: &str) -> Option<(String, PracticeDefinition)> {
    tech_catalog().technologies.into_iter().find_map(|technology| {
        (technology.practice.id == id).then_some((technology.id, technology.practice))
    })
}

fn unit_definition(id: &str) -> Option<UnitDefinition> {
    unit_catalog().units.into_iter().find(|unit| unit.id == id)
}

/// A fact emitted by this small domain slice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum DomainEvent {
    CommandApplied { command_id: u64 },
    CommandRejected { command_id: u64, reason: RejectionReason },
    PopulationMigrated { from: CityId, to: CityId, group: GroupFunction },
    CollapseTriggered { civilization: CivId },
    DiplomacyResolved { command_id: u64, actor: CivId, other: CivId, action: DiplomaticAction, resolution: DiplomaticResolution },
}

/// Result of a pure turn transition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepResult {
    pub state: WorldState,
    pub events: Vec<DomainEvent>,
    pub state_hash: StateHash,
}

/// Applies a closed turn in canonical command-sequence order.
pub fn step(
    state: &WorldState,
    accepted_commands: &[AcceptedCommand],
    seed: u64,
    versions: &SimulationVersions,
) -> StepResult {
    let mut next = state.clone();
    let mut events = Vec::new();
    reset_unit_movement(&mut next);

    if seed != state.seed || versions.ruleset != state.ruleset {
        for command in ordered_commands(accepted_commands) {
            events.push(DomainEvent::CommandRejected {
                command_id: command.command_id,
                reason: RejectionReason::RulesetMismatch,
            });
        }
        let state_hash = state.state_hash();
        return StepResult { state: next, events, state_hash };
    }

    let ordered = ordered_commands(accepted_commands);
    for (index, command) in ordered.iter().enumerate() {
        if (index > 0 && command.accepted_sequence == ordered[index - 1].accepted_sequence)
            || (index + 1 < ordered.len()
                && command.accepted_sequence == ordered[index + 1].accepted_sequence)
        {
            events.push(DomainEvent::CommandRejected {
                command_id: command.command_id,
                reason: RejectionReason::DuplicateAcceptedSequence,
            });
            continue;
        }

        match apply_command(&mut next, command, &mut events) {
            Ok(()) => events.push(DomainEvent::CommandApplied { command_id: command.command_id }),
            Err(reason) => events.push(DomainEvent::CommandRejected { command_id: command.command_id, reason }),
        }
    }

    resolve_movement(&mut next, seed);
    resolve_conflicts(&mut next, seed);
    crate::improvements::resolve_works(&mut next);
    resolve_visibility(&mut next);
    resolve_sentries(&mut next);
    resolve_economy(&mut next, seed);
    resolve_growth_and_migration(&mut next, seed, &mut events);
    resolve_society(&mut next, seed, &mut events);
    resolve_entropy(&mut next, seed);
    resolve_diplomacy(&mut next, seed);
    resolve_synthesis(&mut next, seed);

    if let Some(turn) = next.turn.checked_next() {
        next.turn = turn;
    } else {
        events.push(DomainEvent::CommandRejected {
            command_id: 0,
            reason: RejectionReason::TurnOverflow,
        });
    }
    let state_hash = next.state_hash();
    StepResult { state: next, events, state_hash }
}

fn ordered_commands(commands: &[AcceptedCommand]) -> Vec<&AcceptedCommand> {
    let mut ordered: Vec<_> = commands.iter().collect();
    ordered.sort_unstable_by_key(|command| (command.accepted_sequence, command.command_id));
    ordered
}

fn apply_command(state: &mut WorldState, command: &AcceptedCommand, events: &mut Vec<DomainEvent>) -> Result<(), RejectionReason> {
    if command.world_id != state.world_id {
        return Err(RejectionReason::WrongWorld);
    }
    if command.turn != state.turn {
        return Err(RejectionReason::WrongTurn);
    }
    if command.kind != command.payload.kind() {
        return Err(RejectionReason::KindPayloadMismatch);
    }
    if !state.civilizations.contains_key(&command.actor_id) {
        return Err(RejectionReason::UnknownCivilization);
    }
    if state.civilizations.get(&command.actor_id).is_some_and(|civilization| civilization.frozen) {
        return Err(RejectionReason::CivilizationFrozen);
    }
    if command.origin == CommandOrigin::Governor {
        let mandate = command.mandate.as_ref().ok_or(RejectionReason::MandateRequired)?;
        if !mandate.valid() || !mandate.allows_payload(&command.payload) {
            return Err(RejectionReason::MandateViolation);
        }
    }

    match &command.payload {
        CommandPayload::EndTurn | CommandPayload::KeepPlan => Ok(()),
        CommandPayload::ApplyEvent { .. } => crate::entropy::apply_event(state, command),
        CommandPayload::RespondToEvent { .. } => crate::entropy::respond_to_event(state, command),
        CommandPayload::MoveUnit { unit_id, target } => {
            ensure_tile(state, *target)?;
            let unit = state.units.get(unit_id).ok_or(RejectionReason::UnknownUnit)?;
            if unit.owner != command.actor_id {
                return Err(RejectionReason::NotCommandOwner);
            }
            if state.attacks.contains_key(unit_id) { return Err(RejectionReason::UnitAlreadyReserved); }
            if state.units.values().any(|other| other.tile == *target) {
                return Err(RejectionReason::DestinationOccupied);
            }
            let movement_cost = movement_cost(state, unit.tile, *target)?;
            if movement_cost > u32::from(unit.movement_left) { return Err(RejectionReason::InvalidTile); }
            let unit = state.units.get_mut(unit_id).expect("unit was checked above");
            unit.tile = *target;
            unit.movement_left -= movement_cost as u8;
            unit.explored = true;
            Ok(())
        }
        CommandPayload::FoundCity { city_id, target } => {
            ensure_tile(state, *target)?;
            if state.cities.contains_key(city_id) {
                return Err(RejectionReason::CityAlreadyExists);
            }
            if state.cities.values().any(|city| city.tile == *target) {
                return Err(RejectionReason::CityTileOccupied);
            }
            let founding_with_settler = state.cities.values().any(|city| city.owner == command.actor_id);
            if founding_with_settler {
                let settler_id = state.units.iter()
                    .find_map(|(unit_id, unit)| (unit.owner == command.actor_id && unit.tile == *target && unit.unit_type == "unit.settler").then_some(*unit_id))
                    .ok_or(RejectionReason::MissingSettler)?;
                state.units.remove(&settler_id);
            }
            state.cities.insert(
                *city_id,
                CityState {
                    owner: command.actor_id, tile: *target, focus: CityFocus::Supply,
                    population: 1, housing: INITIAL_CITY_HOUSING, food_stock: INITIAL_CITY_FOOD_STOCK, growth_progress: 0,
                    consecutive_food_shortages: 0, stability: 50,
                    groups: vec![CityGroup::new(GroupFunction::Cultivators, 1), CityGroup::new(GroupFunction::Crafts, 0), CityGroup::new(GroupFunction::Merchants, 0)],
                    workplaces: Vec::new(), last_yields: TileYields::default(), deprivation: 0,
                    group_tension: 0, war_threat: 0, environmental_exposure: 0,
                    crisis_pressure: 0, crisis_turns: 0, essential_maintenance_unpaid: false,
                    unit_queue: Vec::new(), unit_production: 0,
                },
            );
            Ok(())
        }
        CommandPayload::SetCityFocus { city_id, focus } => {
            let city = state.cities.get_mut(city_id).ok_or(RejectionReason::UnknownCity)?;
            if city.owner != command.actor_id {
                return Err(RejectionReason::NotCommandOwner);
            }
            city.focus = *focus;
            Ok(())
        }
        CommandPayload::SetResearch { research } => {
            let technology = technology(research).ok_or(RejectionReason::UnknownTechnology)?;
            let civ = state.civilizations.get(&command.actor_id).expect("civilization was checked above");
            if !technology.prerequisites.iter().all(|required| civ.researched_technologies.contains(required)) {
                return Err(RejectionReason::MissingTechnologyPrerequisite);
            }
            state.civilizations.get_mut(&command.actor_id).expect("civilization was checked above").research = Some(research.clone());
            Ok(())
        }
        CommandPayload::SetResearchInvestment { percent } => {
            if !RESEARCH_PERCENTAGES.contains(percent) { return Err(RejectionReason::InvalidResearchInvestment); }
            state.civilizations.get_mut(&command.actor_id).expect("civilization was checked above").research_investment = *percent;
            Ok(())
        }
        CommandPayload::ActivatePractice { practice: practice_id } => {
            let (technology_id, _) = practice(practice_id).ok_or(RejectionReason::UnknownPractice)?;
            let civ = state.civilizations.get_mut(&command.actor_id).expect("civilization was checked above");
            if !civ.researched_technologies.contains(&technology_id) { return Err(RejectionReason::PracticeTechnologyNotResearched); }
            if !civ.active_practices.contains(practice_id) && civ.active_practices.len() >= ACTIVE_PRACTICE_LIMIT { return Err(RejectionReason::PracticeLimitReached); }
            civ.active_practices.insert(practice_id.clone());
            Ok(())
        }
        CommandPayload::DeactivatePractice { practice: practice_id } => {
            let (technology_id, _) = practice(practice_id).ok_or(RejectionReason::UnknownPractice)?;
            let civ = state.civilizations.get_mut(&command.actor_id).expect("civilization was checked above");
            if !civ.researched_technologies.contains(&technology_id) { return Err(RejectionReason::PracticeTechnologyNotResearched); }
            civ.active_practices.remove(practice_id);
            Ok(())
        }
        CommandPayload::QueueUnit { city_id, unit_type } => {
            let definition = unit_definition(unit_type).ok_or(RejectionReason::UnknownUnitType)?;
            let civ = state.civilizations.get(&command.actor_id).expect("civilization was checked above");
            if !civ.researched_technologies.contains(&definition.requires_technology) { return Err(RejectionReason::UnitTechnologyNotResearched); }
            let city = state.cities.get_mut(city_id).ok_or(RejectionReason::UnknownCity)?;
            if city.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            city.unit_queue.push(unit_type.clone());
            Ok(())
        }
        CommandPayload::RemoveQueuedUnit { city_id, index } => {
            let city = state.cities.get_mut(city_id).ok_or(RejectionReason::UnknownCity)?;
            if city.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            let index = *index as usize;
            if index >= city.unit_queue.len() { return Err(RejectionReason::InvalidQueueIndex); }
            city.unit_queue.remove(index);
            Ok(())
        }
        CommandPayload::MoveQueuedUnit { city_id, from, to } => {
            let city = state.cities.get_mut(city_id).ok_or(RejectionReason::UnknownCity)?;
            if city.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            let (from, to) = (*from as usize, *to as usize);
            if from >= city.unit_queue.len() || to >= city.unit_queue.len() { return Err(RejectionReason::InvalidQueueIndex); }
            let item = city.unit_queue.remove(from);
            city.unit_queue.insert(to, item);
            Ok(())
        }
        CommandPayload::DeclareAttack { attacker, target } => {
            let attacker_state = state.units.get(attacker).ok_or(RejectionReason::UnknownUnit)?;
            let target_state = state.units.get(target).ok_or(RejectionReason::UnknownUnit)?;
            if attacker_state.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            if attacker_state.owner == target_state.owner { return Err(RejectionReason::AttackNotHostile); }
            if state.attacks.contains_key(attacker) { return Err(RejectionReason::UnitAlreadyReserved); }
            if hex_distance(state, attacker_state.tile, target_state.tile)? != 1 { return Err(RejectionReason::AttackOutOfRange); }
            state.attacks.insert(*attacker, AttackReservation { attacker: *attacker, target: *target, target_tile: target_state.tile });
            let (attacker_owner, target_owner) = (attacker_state.owner, target_state.owner);
            let turn = state.turn;
            state.diplomacy.record_aggression(turn, command.command_id, attacker_owner, target_owner);
            Ok(())
        }
        CommandPayload::Explore { unit_id } => {
            if state.attacks.contains_key(unit_id) { return Err(RejectionReason::UnitAlreadyReserved); }
            let unit = state.units.get_mut(unit_id).ok_or(RejectionReason::UnknownUnit)?;
            if unit.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            unit.explored = true;
            Ok(())
        }
        CommandPayload::SetUnitOrder { unit_id, order } => {
            let unit = state.units.get(unit_id).ok_or(RejectionReason::UnknownUnit)?;
            if unit.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            match order {
                UnitOrder::MoveTo { target } => {
                    ensure_tile(state, *target)?;
                    if state.attacks.contains_key(unit_id) { return Err(RejectionReason::UnitAlreadyReserved); }
                    if state.units.values().any(|other| other.tile == *target) { return Err(RejectionReason::DestinationOccupied); }
                    if unit_route(state, unit.tile, *target, unit_movement(&unit.unit_type)).is_none() { return Err(RejectionReason::InvalidTile); }
                }
                UnitOrder::Explore => {
                    if state.attacks.contains_key(unit_id) { return Err(RejectionReason::UnitAlreadyReserved); }
                }
                UnitOrder::Build { improvement } => {
                    if state.attacks.contains_key(unit_id) { return Err(RejectionReason::UnitAlreadyReserved); }
                    crate::improvements::check_build(state, *unit_id, improvement)?;
                }
                UnitOrder::Idle | UnitOrder::Fortify | UnitOrder::Sentry => {}
            }
            state.units.get_mut(unit_id).expect("unit was checked above").order = order.clone();
            Ok(())
        }
        CommandPayload::SkipUnit { unit_id } => {
            let turn = state.turn.0;
            let unit = state.units.get_mut(unit_id).ok_or(RejectionReason::UnknownUnit)?;
            if unit.owner != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
            unit.skipped_turn = Some(turn);
            Ok(())
        }
        CommandPayload::ProposeDiplomacy { recipient, kind } => {
            ensure_diplomatic_target(state, command, *recipient)?;
            let turn = state.turn;
            let resolution = state.diplomacy.propose(turn, command.command_id, command.actor_id, *recipient, *kind)?;
            events.push(DomainEvent::DiplomacyResolved { command_id: command.command_id, actor: command.actor_id, other: *recipient, action: DiplomaticAction::Propose { kind: *kind }, resolution });
            Ok(())
        }
        CommandPayload::BreakTreaty { counterpart } => {
            ensure_diplomatic_target(state, command, *counterpart)?;
            let turn = state.turn;
            let resolution = state.diplomacy.break_treaty(turn, command.command_id, command.actor_id, *counterpart)?;
            events.push(DomainEvent::DiplomacyResolved { command_id: command.command_id, actor: command.actor_id, other: *counterpart, action: DiplomaticAction::BreakTreaty, resolution });
            Ok(())
        }
        CommandPayload::DeclareWar { target, objective, cause } => {
            ensure_diplomatic_target(state, command, *target)?;
            let turn = state.turn;
            let resolution = state.diplomacy.declare_war(turn, command.command_id, command.actor_id, *target, objective, *cause)?;
            events.push(DomainEvent::DiplomacyResolved { command_id: command.command_id, actor: command.actor_id, other: *target, action: DiplomaticAction::DeclareWar { objective: objective.clone() }, resolution });
            Ok(())
        }
    }
}

/// Automated diplomacy must cite a Ledger entry that involves the actor (CLAUDE.md section 2, rule 5).
fn ensure_diplomatic_target(state: &WorldState, command: &AcceptedCommand, other: CivId) -> Result<(), RejectionReason> {
    if matches!(command.origin, CommandOrigin::Bot | CommandOrigin::Governor | CommandOrigin::Fallback)
        && !state.diplomacy.cites_entry(command.actor_id, &command.grounding) {
        return Err(RejectionReason::UngroundedDiplomacy);
    }
    let civilization = state.civilizations.get(&other).ok_or(RejectionReason::UnknownCivilization)?;
    if civilization.frozen { return Err(RejectionReason::CivilizationFrozen); }
    Ok(())
}

fn ensure_tile(state: &WorldState, tile: TileIndex) -> Result<(), RejectionReason> {
    if (tile.0 as usize) < state.tiles.len() {
        Ok(())
    } else {
        Err(RejectionReason::InvalidTile)
    }
}

pub(crate) fn grid_for(state: &WorldState) -> Result<Grid, RejectionReason> {
    let height = (state.tiles.len() as u32).checked_div(state.map_width).ok_or(RejectionReason::InvalidTile)?;
    Grid::new(state.map_width, height).map_err(|_| RejectionReason::InvalidTile)
}

fn hex_distance(state: &WorldState, from: TileIndex, to: TileIndex) -> Result<u32, RejectionReason> {
    let grid = grid_for(state)?;
    grid.distance(grid.cell(from).map_err(|_| RejectionReason::InvalidTile)?, grid.cell(to).map_err(|_| RejectionReason::InvalidTile)?).map_err(|_| RejectionReason::InvalidTile)
}

pub(crate) fn movement_cost(state: &WorldState, from: TileIndex, to: TileIndex) -> Result<u32, RejectionReason> {
    let grid = grid_for(state)?;
    let path = grid.line(grid.cell(from).map_err(|_| RejectionReason::InvalidTile)?, grid.cell(to).map_err(|_| RejectionReason::InvalidTile)?).map_err(|_| RejectionReason::InvalidTile)?;
    Ok(path.into_iter().skip(1).map(|cell| {
        let tile = &state.tiles[grid.tile_index(cell).expect("grid cell came from line").0 as usize];
        terrain_movement_cost(tile.terrain) + u32::from(tile.river)
    }).sum())
}

const fn terrain_movement_cost(terrain: u8) -> u32 {
    match terrain { TERRAIN_FOREST | TERRAIN_JUNGLE | TERRAIN_SWAMP => 2, TERRAIN_OCEAN => 3, _ => 1 }
}

pub(crate) fn unit_movement(unit_type: &str) -> u8 {
    unit_definition(unit_type).map_or(1, |definition| definition.movement)
}

pub(crate) fn queued_food_cost(city: &CityState) -> u32 {
    city.unit_queue.first().and_then(|id| unit_definition(id))
        .and_then(|definition| definition.cost.get("food").copied()).unwrap_or(0)
}

// These phase seams intentionally have no mechanics yet. Each derives an independent
// stream so later deterministic work cannot couple random consumption across phases.
fn reset_unit_movement(state: &mut WorldState) {
    for unit in state.units.values_mut() {
        unit.movement_left = unit_definition(&unit.unit_type).map_or(1, |definition| definition.movement);
    }
}

/// Role of a unit type in the catalog (`exploration`, `defense`, `settler`, ...).
pub(crate) fn unit_role(unit_type: &str) -> String {
    // Cached: the T0 bot asks for roles of every unit and queue item each turn.
    static ROLES: std::sync::OnceLock<BTreeMap<String, String>> = std::sync::OnceLock::new();
    ROLES.get_or_init(|| unit_catalog().units.into_iter().map(|unit| (unit.id, unit.role)).collect())
        .get(unit_type).cloned().unwrap_or_default()
}

/// Straight-line route (excluding the start) whose every single step is affordable with the full
/// movement allowance. `None` when a step can never be paid, so the target is unreachable.
pub(crate) fn unit_route(state: &WorldState, from: TileIndex, to: TileIndex, movement: u8) -> Option<Vec<TileIndex>> {
    let grid = grid_for(state).ok()?;
    let line = grid.line(grid.cell(from).ok()?, grid.cell(to).ok()?).ok()?;
    let mut tiles = Vec::new();
    let mut previous = from;
    for cell in line.into_iter().skip(1) {
        let tile = grid.tile_index(cell).ok()?;
        if movement_cost(state, previous, tile).ok()? > u32::from(movement) { return None; }
        tiles.push(tile);
        previous = tile;
    }
    Some(tiles)
}

fn tile_taken_by_other(state: &WorldState, id: UnitId, tile: TileIndex) -> bool {
    state.units.iter().any(|(other, unit)| *other != id && unit.tile == tile)
}

/// First step toward the nearest tile unknown to the unit's owner, or `None` when no unknown tile
/// is reachable (breadth-first, ties by tile id; occupied tiles are avoided).
fn explore_step(state: &WorldState, id: UnitId) -> Option<TileIndex> {
    let unit = state.units.get(&id)?;
    let grid = grid_for(state).ok()?;
    let movement = u32::from(unit_movement(&unit.unit_type));
    let known = state.visibility.get(&unit.owner);
    let is_known = |tile: TileIndex| known.and_then(|map| map.get(&tile)).is_some_and(|seen| *seen != Visibility::Unknown);
    let occupied: BTreeSet<TileIndex> = state.units.iter().filter(|(other, _)| **other != id).map(|(_, other)| other.tile).collect();
    let start = unit.tile;
    let mut visited = BTreeSet::from([start]);
    let mut frontier = VecDeque::from([(start, start)]);
    while let Some((tile, first)) = frontier.pop_front() {
        if tile != start && !is_known(tile) { return Some(first); }
        let mut neighbors: Vec<TileIndex> = grid.neighbors(grid.cell(tile).ok()?).ok()?.into_iter()
            .filter_map(|cell| grid.tile_index(cell).ok()).collect();
        neighbors.sort_unstable();
        for neighbor in neighbors {
            if visited.contains(&neighbor) || occupied.contains(&neighbor) { continue; }
            if !movement_cost(state, tile, neighbor).is_ok_and(|cost| cost <= movement) { continue; }
            visited.insert(neighbor);
            frontier.push_back((neighbor, if tile == start { neighbor } else { first }));
        }
    }
    None
}

/// True when an `Explore` order for this unit would find an unknown tile to head to.
pub fn can_explore(state: &WorldState, unit: UnitId) -> bool { explore_step(state, unit).is_some() }

/// Idle units of `civ` in unit-id order: `Idle`, movement left and not skipped this turn.
pub fn idle_units(state: &WorldState, civ: CivId) -> Vec<UnitId> {
    idle_units_with(state, civ, &[])
}

/// Like `idle_units`, but as if the open turn's `pending` commands of `civ` were already applied.
/// The server uses this for the Ready gate, because orders only change state when the turn resolves.
pub fn idle_units_with(state: &WorldState, civ: CivId, pending: &[AcceptedCommand]) -> Vec<UnitId> {
    let effective = effective_unit_orders(state, civ, pending);
    state.units.iter()
        // `movement_left` is not consulted: during the open turn it still holds the previous turn's
        // leftover, because commands only spend movement when the turn resolves.
        .filter(|(_, unit)| unit.owner == civ)
        .filter(|(id, unit)| {
            let (order, skipped) = effective.get(*id).cloned().unwrap_or((unit.order.clone(), unit.skipped_turn));
            order == UnitOrder::Idle && skipped != Some(state.turn.0)
        })
        .map(|(id, _)| *id)
        .collect()
}

/// Order and skipped turn of the `civ` units touched by `pending` commands, applied in sequence.
pub fn effective_unit_orders(state: &WorldState, civ: CivId, pending: &[AcceptedCommand]) -> BTreeMap<UnitId, (UnitOrder, Option<u32>)> {
    let mut effective: BTreeMap<UnitId, (UnitOrder, Option<u32>)> = BTreeMap::new();
    for command in pending.iter().filter(|command| command.actor_id == civ && command.turn == state.turn) {
        match &command.payload {
            CommandPayload::SetUnitOrder { unit_id, order } => {
                let Some(unit) = state.units.get(unit_id).filter(|unit| unit.owner == civ) else { continue };
                effective.entry(*unit_id).or_insert((unit.order.clone(), unit.skipped_turn)).0 = order.clone();
            }
            CommandPayload::SkipUnit { unit_id } => {
                let Some(unit) = state.units.get(unit_id).filter(|unit| unit.owner == civ) else { continue };
                effective.entry(*unit_id).or_insert((unit.order.clone(), unit.skipped_turn)).1 = Some(state.turn.0);
            }
            // A one-turn action already accepted for this unit counts as its order for the turn.
            CommandPayload::MoveUnit { unit_id, .. } | CommandPayload::Explore { unit_id }
            | CommandPayload::DeclareAttack { attacker: unit_id, .. } => {
                let Some(unit) = state.units.get(unit_id).filter(|unit| unit.owner == civ) else { continue };
                effective.entry(*unit_id).or_insert((unit.order.clone(), unit.skipped_turn)).1 = Some(state.turn.0);
            }
            // The settler standing on the founding tile is consumed when the turn resolves, so it
            // must not keep blocking the Ready gate.
            CommandPayload::FoundCity { target, .. } => {
                let settler = state.units.iter()
                    .find_map(|(id, unit)| (unit.owner == civ && unit.tile == *target && unit.unit_type == "unit.settler").then_some(*id));
                let Some(unit_id) = settler else { continue };
                let unit = &state.units[&unit_id];
                effective.entry(unit_id).or_insert((unit.order.clone(), unit.skipped_turn)).1 = Some(state.turn.0);
            }
            _ => {}
        }
    }
    effective
}

/// Runs persistent `MoveTo` and `Explore` orders, in unit-id order, after the turn's commands.
fn resolve_movement(state: &mut WorldState, seed: u64) {
    let _rng = Rng::derive(seed, "movement");
    let ids: Vec<UnitId> = state.units.keys().copied().collect();
    for id in ids { execute_order(state, id); }
}

fn execute_order(state: &mut WorldState, id: UnitId) {
    let Some(unit) = state.units.get(&id) else { return };
    let order = unit.order.clone();
    if matches!(order, UnitOrder::Idle | UnitOrder::Fortify | UnitOrder::Sentry | UnitOrder::Build { .. }) || state.attacks.contains_key(&id) { return; }
    let (from, movement) = (unit.tile, unit_movement(&unit.unit_type));
    let mut left = unit.movement_left;
    let mut at = from;
    let mut finished = false;
    match order {
        UnitOrder::MoveTo { target } => {
            match unit_route(state, from, target, movement) {
                Some(path) if !tile_taken_by_other(state, id, target) => {
                    for tile in path {
                        let cost = movement_cost(state, at, tile).unwrap_or(u32::MAX);
                        if cost > u32::from(left) { break; }
                        if tile_taken_by_other(state, id, tile) { finished = at == from; break; }
                        at = tile;
                        left -= cost as u8;
                    }
                    finished = finished || at == target;
                }
                _ => finished = true,
            }
        }
        UnitOrder::Explore => {
            loop {
                let Some(next) = explore_step(state, id) else { finished = true; break };
                let cost = movement_cost(state, at, next).unwrap_or(u32::MAX);
                if cost > u32::from(left) { break; }
                // Move one tile at a time so the next step is computed from the new position.
                let unit = state.units.get_mut(&id).expect("unit was checked above");
                unit.tile = next;
                at = next;
                left -= cost as u8;
            }
        }
        UnitOrder::Idle | UnitOrder::Fortify | UnitOrder::Sentry | UnitOrder::Build { .. } => return,
    }
    let unit = state.units.get_mut(&id).expect("unit was checked above");
    if at != from { unit.explored = true; }
    unit.tile = at;
    unit.movement_left = left;
    if finished { unit.order = UnitOrder::Idle; }
}

fn resolve_conflicts(state: &mut WorldState, seed: u64) {
    let _rng = Rng::derive(seed, "conflicts");
    let reservations = std::mem::take(&mut state.attacks);
    let mut by_tile: BTreeMap<TileIndex, Vec<AttackReservation>> = BTreeMap::new();
    for reservation in reservations.into_values() {
        if state.units.contains_key(&reservation.attacker) && state.units.contains_key(&reservation.target) {
            by_tile.entry(reservation.target_tile).or_default().push(reservation);
        }
    }
    for (tile, reservations) in by_tile {
        let mut participants: BTreeMap<CivId, Vec<UnitId>> = BTreeMap::new();
        for reservation in reservations {
            for id in [reservation.attacker, reservation.target] {
                if let Some(unit) = state.units.get(&id) {
                    participants.entry(unit.owner).or_default().push(id);
                }
            }
        }
        for units in participants.values_mut() { units.sort_unstable(); units.dedup(); }
        if participants.len() < 2 { continue; }
        let strengths: BTreeMap<CivId, u32> = participants.iter().map(|(civ, units)| {
            (*civ, units.iter().filter_map(|id| state.units.get(id)).map(unit_strength).sum())
        }).collect();
        let total_strength: u32 = strengths.values().sum();
        let mut losses: BTreeMap<UnitId, u8> = BTreeMap::new();
        for (civ, units) in &participants {
            let own = strengths[civ];
            let hostile = total_strength.saturating_sub(own);
            let loss = combat_loss_points(own, hostile);
            let total_hp: u32 = units.iter().filter_map(|id| state.units.get(id)).map(|unit| u32::from(unit.hit_points)).sum();
            distribute_loss(units, total_hp.min(loss), state, &mut losses);
        }
        for (id, loss) in losses { if let Some(unit) = state.units.get_mut(&id) { unit.hit_points = unit.hit_points.saturating_sub(loss); } }
        state.units.retain(|_, unit| unit.hit_points > 0);
        let survivors: BTreeSet<CivId> = participants.values().flatten().filter_map(|id| state.units.get(id).map(|unit| unit.owner)).collect();
        if survivors.len() == 1 {
            let winner = *survivors.iter().next().expect("non-empty set was checked");
            state.control.insert(tile, winner);
        }
    }
}

fn unit_strength(unit: &UnitState) -> u32 { unit_definition(&unit.unit_type).map_or(1, |definition| u32::from(definition.strength)) }

fn combat_loss_points(own: u32, hostile: u32) -> u32 {
    if own.saturating_add(hostile) == 0 { return 0; }
    // Integer half-up rounding of the GDD's 30 * hostile / total formula.
    ((30 * hostile + (own + hostile) / 2) / (own + hostile)).clamp(5, 40)
}

fn distribute_loss(units: &[UnitId], mut remaining: u32, state: &WorldState, losses: &mut BTreeMap<UnitId, u8>) {
    for id in units {
        if remaining == 0 { break; }
        let hp = state.units.get(id).map_or(0, |unit| u32::from(unit.hit_points));
        let loss = hp.min(remaining);
        losses.insert(*id, loss as u8);
        remaining -= loss;
    }
}
fn resolve_economy(state: &mut WorldState, seed: u64) {
    let _rng = Rng::derive(seed, "economy");
    let mut allocations: BTreeMap<CityId, Vec<TileIndex>> = BTreeMap::new();
    let mut claimed = BTreeSet::new();
    for (city_id, city) in &state.cities {
        let workplaces = allocate_workplaces(state, city, &claimed);
        claimed.extend(workplaces.iter().copied());
        allocations.insert(*city_id, workplaces);
    }
    // Income assumes every improvement is active; an improvement left unpaid below loses its bonus.
    let mut wealth_by_civ: BTreeMap<CivId, u32> = BTreeMap::new();
    for (city_id, workplaces) in &allocations {
        let wealth: u32 = workplaces.iter().map(|tile| u32::from(crate::improvements::tile_output(&state.tiles[tile.0 as usize]).wealth)).sum();
        *wealth_by_civ.entry(state.cities[city_id].owner).or_default() += wealth;
    }
    let mut available_wealth: BTreeMap<_, _> = state.civilizations.iter().map(|(id, civ)| {
        (*id, civ.treasury_wealth.saturating_add(wealth_by_civ.get(id).copied().unwrap_or(0)))
    }).collect();
    // Payment priority (GDD 03): city sustenance first, in city-id order; then improvements.
    let mut unpaid_cities = BTreeSet::new();
    for (city_id, city) in &state.cities {
        let maintenance = 1 + (city.population + 3) / 4;
        let available = available_wealth.entry(city.owner).or_default();
        if *available < maintenance { unpaid_cities.insert(*city_id); }
        *available = available.saturating_sub(maintenance);
    }
    let suspended = crate::improvements::pay_upkeep(state, &mut available_wealth);
    let mut city_yields = BTreeMap::new();
    for (city_id, workplaces) in allocations {
        let owner = state.cities[&city_id].owner;
        let mut total = TileYields::default();
        for tile in &workplaces {
            let data = &state.tiles[tile.0 as usize];
            let yields = if suspended.contains(tile) {
                // A suspended improvement produces nothing, including the wealth already counted.
                let lost = crate::improvements::tile_output(data).wealth.saturating_sub(crate::improvements::base_output(data).wealth);
                let available = available_wealth.entry(owner).or_default();
                *available = available.saturating_sub(u32::from(lost));
                crate::improvements::base_output(data)
            } else { crate::improvements::tile_output(data) };
            total.food = total.food.saturating_add(yields.food);
            total.production = total.production.saturating_add(yields.production);
            total.wealth = total.wealth.saturating_add(yields.wealth);
            total.knowledge = total.knowledge.saturating_add(yields.knowledge);
            total.culture = total.culture.saturating_add(yields.culture);
        }
        city_yields.insert(city_id, (workplaces, total));
    }
    for (city_id, city) in &mut state.cities {
        let (workplaces, yields) = city_yields.remove(city_id).expect("all cities were allocated");
        city.workplaces = workplaces;
        city.last_yields = yields;
        city.food_stock = city.food_stock.saturating_add(u32::from(yields.food));
        city.essential_maintenance_unpaid = unpaid_cities.contains(city_id);
        let demand = city.population;
        let consumed = city.food_stock.min(demand);
        city.food_stock -= consumed;
        let missing = demand - consumed;
        city.consecutive_food_shortages = if missing > 0 { city.consecutive_food_shortages.saturating_add(1) } else { 0 };
        city.deprivation = (missing.saturating_mul(5) + if city.essential_maintenance_unpaid { 5 } else { 0 }).min(20) as u8;
        city.food_stock = city.food_stock.min(3u32.saturating_mul(city.population).saturating_add(4));
    }
    for (civ_id, civilization) in &mut state.civilizations {
        // Treasury plus income minus every maintenance actually paid (never negative).
        civilization.treasury_wealth = available_wealth.get(civ_id).copied().unwrap_or(civilization.treasury_wealth);
        let population: u32 = state.cities.values().filter(|city| city.owner == *civ_id).map(|city| city.population).sum();
        civilization.treasury_wealth = civilization.treasury_wealth.min(6u32.saturating_mul(population).saturating_add(12));
        civilization.knowledge = civilization.knowledge.saturating_add(state.cities.values().filter(|city| city.owner == *civ_id).map(|city| u32::from(city.last_yields.knowledge)).sum());
        civilization.culture = civilization.culture.saturating_add(state.cities.values().filter(|city| city.owner == *civ_id).map(|city| u32::from(city.last_yields.culture)).sum());
    }
    resolve_research_and_practices(state);
    resolve_unit_production(state);
}

fn resolve_research_and_practices(state: &mut WorldState) {
    let civ_ids: Vec<CivId> = state.civilizations.keys().copied().collect();
    for civ_id in civ_ids {
        let production: u32 = state.cities.values().filter(|city| city.owner == civ_id).map(|city| u32::from(city.last_yields.production)).sum();
        let maintenance: u32 = state.civilizations[&civ_id].active_practices.iter().filter_map(|id| practice(id).map(|(_, definition)| definition.maintenance)).sum();
        let available = production.saturating_sub(maintenance);
        let (research, investment) = {
            let civ = &state.civilizations[&civ_id];
            (civ.research.clone(), civ.research_investment)
        };
        let Some(research) = research else { continue; };
        // Research uses half of the allocated production. Round the resulting
        // point up so the supported 10/20% investments can progress from a
        // one-production founding city instead of remaining permanently zero.
        let points = ((available.saturating_mul(u32::from(investment)) + 199) / 200)
            .min(RESEARCH_PER_TURN_CAP);
        let Some(definition) = technology(&research) else { continue; };
        let civ = state.civilizations.get_mut(&civ_id).expect("civilization id was collected from state");
        let progress = civ.research_progress.entry(research.clone()).or_default();
        *progress = progress.saturating_add(points);
        if *progress >= definition.cost {
            civ.researched_technologies.insert(research);
            civ.research = None;
        }
    }
}

fn resolve_unit_production(state: &mut WorldState) {
    let city_ids: Vec<CityId> = state.cities.keys().copied().collect();
    for city_id in city_ids {
        let Some(city) = state.cities.get(&city_id) else { continue; };
        let Some(unit_type) = city.unit_queue.first().cloned() else { continue; };
        let Some(definition) = unit_definition(&unit_type) else { continue; };
        let cost = definition.cost.get("production").copied().unwrap_or(0);
        let food_cost = definition.cost.get("food").copied().unwrap_or(0);
        let produced = u32::from(city.last_yields.production);
        let tile = city.tile;
        let owner = city.owner;
        let complete = city.unit_production.saturating_add(produced) >= cost && city.food_stock >= food_cost;
        // Settled unit ids remain reserved by their city, even after consumption.
        let highest_id = state.units.keys().map(|id| id.0)
            .chain(state.cities.keys().filter_map(|id| id.0.checked_sub(SETTLER_CITY_ID_BASE))).max();
        let next_id = match highest_id { Some(id) => id.checked_add(1), None => Some(0) };
        // The city tile first; when occupied, the first free adjacent land tile in index order.
        let spawn_tile = spawn_tile_for(state, tile);
        let can_spawn = next_id.is_some_and(|id| id <= u32::MAX - SETTLER_CITY_ID_BASE) && spawn_tile.is_some();
        {
            let city = state.cities.get_mut(&city_id).expect("city id was collected from state");
            city.unit_production = city.unit_production.saturating_add(produced);
            if !complete || !can_spawn { continue; }
            city.unit_production -= cost;
            city.food_stock -= food_cost;
            city.unit_queue.remove(0);
        }
        let next_id = next_id.expect("spawn id was checked before spending resources");
        let tile = spawn_tile.expect("spawn tile was checked before spending resources");
        state.units.insert(UnitId(next_id), UnitState { owner, tile, unit_type, hit_points: 100, movement_left: definition.movement, explored: false, order: UnitOrder::Idle, skipped_turn: None });
    }
}

/// Where a unit produced by the city on `city_tile` appears: the city tile if free, otherwise the
/// lowest-index free adjacent land tile. `None` when every candidate is taken.
fn spawn_tile_for(state: &WorldState, city_tile: TileIndex) -> Option<TileIndex> {
    let occupied = |tile: TileIndex| state.units.values().any(|unit| unit.tile == tile);
    if !occupied(city_tile) { return Some(city_tile); }
    let grid = grid_for(state).ok()?;
    let mut neighbors: Vec<TileIndex> = grid.neighbors(grid.cell(city_tile).ok()?).ok()?.into_iter()
        .filter_map(|cell| grid.tile_index(cell).ok()).collect();
    neighbors.sort_unstable();
    neighbors.dedup();
    neighbors.into_iter().find(|tile| {
        state.tiles.get(tile.0 as usize).is_some_and(|data| data.terrain != TERRAIN_OCEAN) && !occupied(*tile)
    })
}

fn resolve_visibility(state: &mut WorldState) {
    for known in state.visibility.values_mut() {
        for status in known.values_mut() { if *status == Visibility::Visible { *status = Visibility::Remembered; } }
    }
    let Ok(grid) = grid_for(state) else { return; };
    for unit in state.units.values() {
        let Ok(cell) = grid.cell(unit.tile) else { continue; };
        let known = state.visibility.entry(unit.owner).or_default();
        for cell in grid.area(cell, VISIBILITY_RADIUS).unwrap_or_default() {
            if let Ok(tile) = grid.tile_index(cell) { known.insert(tile, Visibility::Visible); }
        }
    }
    for city in state.cities.values() {
        let Ok(cell) = grid.cell(city.tile) else { continue; };
        let known = state.visibility.entry(city.owner).or_default();
        for cell in grid.area(cell, VISIBILITY_RADIUS).unwrap_or_default() {
            if let Ok(tile) = grid.tile_index(cell) { known.insert(tile, Visibility::Visible); }
        }
    }
}

/// `Sentry` units wake to `Idle` when a unit of another civilization is inside their vision range
/// (the same radius `resolve_visibility` uses). Runs in unit-id order on the post-movement state.
fn resolve_sentries(state: &mut WorldState) {
    let Ok(grid) = grid_for(state) else { return; };
    let woken: Vec<UnitId> = state.units.iter()
        .filter(|(_, unit)| unit.order == UnitOrder::Sentry)
        .filter(|(_, unit)| {
            let Ok(cell) = grid.cell(unit.tile) else { return false; };
            let seen: BTreeSet<TileIndex> = grid.area(cell, VISIBILITY_RADIUS).unwrap_or_default().into_iter()
                .filter_map(|cell| grid.tile_index(cell).ok()).collect();
            state.units.values().any(|other| other.owner != unit.owner && seen.contains(&other.tile))
        })
        .map(|(id, _)| *id)
        .collect();
    for id in woken { if let Some(unit) = state.units.get_mut(&id) { unit.order = UnitOrder::Idle; } }
}

fn allocate_workplaces(state: &WorldState, city: &CityState, claimed: &BTreeSet<TileIndex>) -> Vec<TileIndex> {
    let candidates: Vec<_> = workable_tiles(state, city.tile).into_iter()
        .filter(|tile| !claimed.contains(tile))
        .collect();
    let mut selected = Vec::new();
    let mut food_candidates = candidates.clone();
    food_candidates.sort_unstable_by_key(|tile| (std::cmp::Reverse(crate::improvements::tile_output(&state.tiles[tile.0 as usize]).food), tile.0));
    let mut food = 0u32;
    for tile in food_candidates {
        if food >= city.population || selected.len() >= city.population as usize { break; }
        let yield_food = u32::from(crate::improvements::tile_output(&state.tiles[tile.0 as usize]).food);
        if yield_food == 0 { break; }
        food += yield_food;
        selected.push(tile);
    }
    let mut remaining: Vec<_> = candidates.into_iter().filter(|tile| !selected.contains(tile)).collect();
    // Essential upkeep precedes the discretionary focus (GDD 04).
    let maintenance = 1 + (city.population + 3) / 4;
    let mut wealth: u32 = selected.iter().map(|tile| u32::from(crate::improvements::tile_output(&state.tiles[tile.0 as usize]).wealth)).sum();
    remaining.sort_unstable_by_key(|tile| (std::cmp::Reverse(crate::improvements::tile_output(&state.tiles[tile.0 as usize]).wealth), tile.0));
    for tile in &remaining {
        if wealth >= maintenance || selected.len() >= city.population as usize { break; }
        let output = u32::from(crate::improvements::tile_output(&state.tiles[tile.0 as usize]).wealth);
        if output == 0 { break; }
        wealth += output;
        selected.push(*tile);
    }
    remaining.retain(|tile| !selected.contains(tile));
    remaining.sort_unstable_by_key(|tile| {
        let yields = crate::improvements::tile_output(&state.tiles[tile.0 as usize]);
        let primary = match city.focus { CityFocus::Supply => yields.food, CityFocus::Build => yields.production, CityFocus::Diversify => yields.food.saturating_add(yields.production).saturating_add(yields.wealth).saturating_add(yields.knowledge).saturating_add(yields.culture) };
        (std::cmp::Reverse(primary), std::cmp::Reverse(yields.food), tile.0)
    });
    let open_slots = city.population.saturating_sub(selected.len() as u32) as usize;
    selected.extend(remaining.into_iter().take(open_slots));
    selected.sort_unstable();
    selected
}

fn workable_tiles(state: &WorldState, center: TileIndex) -> Vec<TileIndex> {
    let Ok(grid) = Grid::new(state.map_width, (state.tiles.len() as u32).checked_div(state.map_width).unwrap_or(0)) else { return vec![center]; };
    let Ok(cell) = grid.cell(center) else { return vec![center]; };
    grid.area(cell, 2).unwrap_or_else(|_| vec![cell]).into_iter().filter_map(|cell| grid.tile_index(cell).ok()).collect()
}

fn resolve_growth_and_migration(state: &mut WorldState, seed: u64, events: &mut Vec<DomainEvent>) {
    let _rng = Rng::derive(seed, "growth-and-migration");
    for city in state.cities.values_mut() {
        if city.consecutive_food_shortages >= 2 && city.population > 0 {
            city.population -= 1;
            remove_person(city);
        } else if city.food_stock >= city.population && city.population < city.housing && city.stability >= 50 {
            city.growth_progress += 1;
            if city.growth_progress >= city.population.max(2) {
                city.population += 1;
                city.growth_progress = 0;
                city.groups[0].population += 1;
            }
        } else { city.growth_progress = 0; }
    }
    let ids: Vec<_> = state.cities.keys().copied().collect();
    for from_id in &ids {
        let Some(source) = state.cities.get(from_id) else { continue; };
        if source.population == 0 || (source.deprivation == 0 && source.stability >= 40) { continue; }
        let owner = source.owner;
        let destination = ids.iter().copied().find(|to_id| {
            *to_id != *from_id && state.cities.get(to_id).is_some_and(|target| target.owner == owner && target.population < target.housing && target.food_stock >= target.population && target.stability >= 40)
        });
        if let Some(to_id) = destination {
            let mut destination_city = state.cities.remove(&to_id).expect("destination exists");
            let group = move_person(state.cities.get_mut(from_id).expect("source exists"), &mut destination_city);
            state.cities.insert(to_id, destination_city);
            events.push(DomainEvent::PopulationMigrated { from: *from_id, to: to_id, group });
        }
    }
}

fn remove_person(city: &mut CityState) {
    if let Some(group) = city.groups.iter_mut().rev().find(|group| group.population > 0) { group.population -= 1; }
}

fn move_person(source: &mut CityState, destination: &mut CityState) -> GroupFunction {
    let index = source.groups.iter().position(|group| group.population > 0).unwrap_or(0);
    source.groups[index].population -= 1;
    let function = source.groups[index].function;
    if let Some(group) = destination.groups.iter_mut().find(|group| group.function == function) { group.population += 1; }
    destination.population += 1;
    source.population -= 1;
    function
}

fn resolve_society(state: &mut WorldState, seed: u64, events: &mut Vec<DomainEvent>) {
    let _rng = Rng::derive(seed, "society");
    for city in state.cities.values_mut() {
        for group in &mut city.groups {
            let employed = match group.function { GroupFunction::Cultivators => city.last_yields.food > 0, GroupFunction::Crafts => city.last_yields.production > 0, GroupFunction::Merchants => city.last_yields.wealth > 0 };
            let change = if city.deprivation > 0 { -5 } else if employed { 5 } else { -3 };
            group.satisfaction = (i16::from(group.satisfaction) + change).clamp(0, 100) as u8;
        }
        let weighted_satisfaction: u32 = city.groups.iter().map(|group| group.population * u32::from(group.satisfaction)).sum();
        city.stability = if city.population == 0 { 0 } else { (weighted_satisfaction / city.population).min(100) as u8 };
        let dissatisfied: u32 = city.groups.iter().filter(|group| group.satisfaction < 40).map(|group| group.population).sum();
        city.group_tension = if city.population == 0 { 0 } else { ((20 * dissatisfied + city.population - 1) / city.population).min(20) as u8 };
    }
    for (civ_id, civ) in &mut state.civilizations {
        let cities: Vec<_> = state.cities.values().filter(|city| city.owner == *civ_id).collect();
        let population: u32 = cities.iter().map(|city| city.population).sum();
        let weighted = |f: fn(&CityState) -> u8| -> u8 { if population == 0 { 0 } else { ((cities.iter().map(|city| city.population * u32::from(f(city))).sum::<u32>() + population - 1) / population).min(20) as u8 } };
        civ.deprivation = weighted(|city| city.deprivation);
        civ.group_tension = weighted(|city| city.group_tension);
        civ.war_threat = weighted(|city| city.war_threat);
        civ.environmental_exposure = weighted(|city| city.environmental_exposure);
        let mean_tension = if population == 0 { 0 } else { cities.iter().flat_map(|city| city.groups.iter()).map(|group| group.population * u32::from(100 - group.satisfaction)).sum::<u32>() / population };
        civ.cohesion = (i32::from(civ.cohesion) - (mean_tension / 10) as i32).clamp(0, 100) as u8;
        civ.zero_cohesion_turns = if civ.cohesion == 0 { civ.zero_cohesion_turns.saturating_add(1) } else { 0 };
        drop(cities);
        for city in state.cities.values_mut().filter(|city| city.owner == *civ_id) {
            city.crisis_pressure = (2 * i32::from(city.deprivation) + 2 * i32::from(city.group_tension) + i32::from(city.war_threat) + i32::from(city.environmental_exposure) + (100 - i32::from(city.stability)) / 10 - (i32::from(civ.cohesion) - 50) / 5).clamp(0, 100) as u8;
            city.crisis_turns = if city.crisis_pressure >= 70 { city.crisis_turns.saturating_add(1) } else { 0 };
        }
        civ.crisis_pressure = if population == 0 { 0 } else { (state.cities.values().filter(|city| city.owner == *civ_id).map(|city| city.population * u32::from(city.crisis_pressure)).sum::<u32>() / population).min(100) as u8 };
        if civ.zero_cohesion_turns >= 2 && !civ.frozen { civ.frozen = true; events.push(DomainEvent::CollapseTriggered { civilization: *civ_id }); }
    }
}
fn resolve_entropy(state: &mut WorldState, seed: u64) { crate::entropy::resolve(state, seed); }
fn resolve_diplomacy(state: &mut WorldState, seed: u64) {
    let _rng = Rng::derive(seed, "diplomacy");
    // Contact: a civilization that currently sees a foreign city or unit has met its owner.
    let mut met = BTreeSet::new();
    let sightings = state.cities.values().map(|city| (city.owner, city.tile)).chain(state.units.values().map(|unit| (unit.owner, unit.tile)));
    for (owner, tile) in sightings {
        for (observer, known) in &state.visibility {
            if *observer != owner && known.get(&tile) == Some(&Visibility::Visible)
                && state.civilizations.contains_key(observer) && state.civilizations.contains_key(&owner) {
                met.insert(if *observer < owner { (*observer, owner) } else { (owner, *observer) });
            }
        }
    }
    let turn = state.turn;
    for (low, high) in met { state.diplomacy.establish_contact(turn, low, high); }
    state.diplomacy.advance(turn);
}
fn resolve_synthesis(_state: &mut WorldState, seed: u64) { let _rng = Rng::derive(seed, "synthesis"); }

/// In-memory append-only accepted-command log plus the resulting hash for each resolved turn.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandLog {
    pub commands: Vec<AcceptedCommand>,
    pub turn_hashes: BTreeMap<TurnNumber, StateHash>,
}

impl CommandLog {
    pub fn append(&mut self, command: AcceptedCommand) {
        self.commands.push(command);
    }

    pub fn record_turn(&mut self, turn: TurnNumber, state_hash: StateHash) {
        self.turn_hashes.insert(turn, state_hash);
    }
}

/// Serializable acceleration point for replay. Its hash verifies its mechanical state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub state: WorldState,
    pub versions: SimulationVersions,
    pub state_hash: StateHash,
}

impl WorldSnapshot {
    pub fn new(state: WorldState, versions: SimulationVersions) -> Self {
        let state_hash = state.state_hash();
        Self { state, versions, state_hash }
    }

    pub fn verify(&self) -> bool {
        self.state.ruleset == self.versions.ruleset && self.state.state_hash() == self.state_hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayResult {
    pub state: WorldState,
    pub state_hashes: BTreeMap<TurnNumber, StateHash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayError {
    InvalidSnapshot,
    MissingTurnSeal(TurnNumber),
    HashMismatch { turn: TurnNumber, expected: StateHash, actual: StateHash },
}

/// Replays only already accepted commands and checks the recorded hash after every turn.
pub fn replay(snapshot: &WorldSnapshot, log: &CommandLog) -> Result<ReplayResult, ReplayError> {
    if !snapshot.verify() {
        return Err(ReplayError::InvalidSnapshot);
    }
    let mut state = snapshot.state.clone();
    let mut state_hashes = BTreeMap::new();
    for (turn, expected) in &log.turn_hashes {
        if *turn != state.turn {
            return Err(ReplayError::MissingTurnSeal(state.turn));
        }
        let commands: Vec<_> = log.commands.iter().filter(|command| command.turn == *turn).cloned().collect();
        let result = step(&state, &commands, state.seed, &snapshot.versions);
        if result.state_hash != *expected {
            return Err(ReplayError::HashMismatch { turn: *turn, expected: *expected, actual: result.state_hash });
        }
        state_hashes.insert(*turn, result.state_hash);
        state = result.state;
    }
    Ok(ReplayResult { state, state_hashes })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ruleset() -> RulesetRef {
        RulesetRef { id: "initial".into(), version: "1.0.0".into(), content_hash: 7 }
    }

    fn versions() -> SimulationVersions {
        SimulationVersions { ruleset: ruleset(), resolver_version: 1 }
    }

    fn state() -> WorldState {
        let mut civilizations = BTreeMap::new();
        civilizations.insert(CivId(1), CivilizationState::default());
        civilizations.insert(CivId(2), CivilizationState::default());
        let mut units = BTreeMap::new();
        units.insert(UnitId(1), UnitState { owner: CivId(1), tile: TileIndex(0), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 0, explored: false, order: UnitOrder::Idle, skipped_turn: None });
        units.insert(UnitId(2), UnitState { owner: CivId(2), tile: TileIndex(2), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 0, explored: false, order: UnitOrder::Idle, skipped_turn: None });
        WorldState {
            world_id: WorldId(9),
            turn: TurnNumber::ZERO,
            seed: 99,
            ruleset: ruleset(),
            schema_version: 1,
            map_width: 2,
            tiles: vec![TileState::default(); 4],
            civilizations,
            cities: BTreeMap::new(),
            units,
            attacks: BTreeMap::new(),
            control: BTreeMap::new(),
            visibility: BTreeMap::new(),
            diplomacy: DiplomacyState::default(),
            entropy: EntropyState::default(),
        }
    }

    fn move_command(id: u64, sequence: u64, actor: CivId, unit: UnitId, target: u32) -> AcceptedCommand {
        AcceptedCommand {
            command_id: id,
            world_id: WorldId(9),
            turn: TurnNumber::ZERO,
            accepted_sequence: sequence,
            actor_id: actor,
            origin: CommandOrigin::Player,
            kind: CommandKind::MoveUnit,
            payload: CommandPayload::MoveUnit { unit_id: unit, target: TileIndex(target) },
            grounding: Vec::new(),
            intent_evidence: None,
            mandate: None,
        }
    }

    fn city(owner: CivId, tile: u32, population: u32) -> CityState {
        CityState {
            owner, tile: TileIndex(tile), focus: CityFocus::Supply, population, housing: population + 2,
            food_stock: 0, growth_progress: 0, consecutive_food_shortages: 0, stability: 50,
            groups: vec![CityGroup::new(GroupFunction::Cultivators, population), CityGroup::new(GroupFunction::Crafts, 0), CityGroup::new(GroupFunction::Merchants, 0)],
            workplaces: Vec::new(), last_yields: TileYields::default(), deprivation: 0, group_tension: 0,
            war_threat: 0, environmental_exposure: 0, crisis_pressure: 0, crisis_turns: 0,
            essential_maintenance_unpaid: false,
            unit_queue: Vec::new(), unit_production: 0,
        }
    }

    fn command(id: u64, sequence: u64, actor: CivId, payload: CommandPayload) -> AcceptedCommand {
        AcceptedCommand { command_id: id, world_id: WorldId(9), turn: TurnNumber::ZERO, accepted_sequence: sequence, actor_id: actor, origin: CommandOrigin::Player, kind: payload.kind(), payload, grounding: Vec::new(), intent_evidence: None, mandate: None }
    }

    #[test]
    fn maintenance_spends_shared_income_once_before_computing_deprivation() {
        let mut initial = state();
        initial.units.clear();
        initial.civilizations.get_mut(&CivId(1)).unwrap().treasury_wealth = 0;
        for tile in &mut initial.tiles { tile.yields = TileYields { food: 2, wealth: 1, ..TileYields::default() }; }
        initial.cities.insert(CityId(1), city(CivId(1), 0, 1));
        initial.cities.insert(CityId(2), city(CivId(1), 1, 1));
        resolve_economy(&mut initial, 99);
        assert_eq!(initial.civilizations[&CivId(1)].treasury_wealth, 0);
        assert_eq!(initial.cities[&CityId(1)].deprivation, 0);
        assert_eq!(initial.cities[&CityId(2)].deprivation, 5);
        assert!(!initial.cities[&CityId(1)].essential_maintenance_unpaid);
        assert!(initial.cities[&CityId(2)].essential_maintenance_unpaid);
    }

    #[test]
    fn pressure_uses_all_gdd_terms_and_population_weighting() {
        let mut initial = state();
        let mut deprived = city(CivId(1), 0, 1);
        deprived.deprivation = 5;
        deprived.groups[0].satisfaction = 40;
        deprived.war_threat = 7;
        deprived.environmental_exposure = 3;
        let mut healthy = city(CivId(1), 1, 3);
        healthy.groups[0].satisfaction = 95;
        healthy.last_yields.food = 6;
        initial.cities.insert(CityId(1), deprived);
        initial.cities.insert(CityId(2), healthy);
        resolve_society(&mut initial, 99, &mut Vec::new());
        let civ = &initial.civilizations[&CivId(1)];
        assert_eq!(civ.cohesion, 49);
        assert_eq!(civ.deprivation, 2);
        assert_eq!(civ.group_tension, 5);
        let local = &initial.cities[&CityId(1)];
        assert_eq!(local.stability, 35);
        assert_eq!(local.group_tension, 20);
        // 2*5 + 2*20 + 7 + 3 + 65/10 - (49-50)/5 = 66 (integer division truncates toward zero).
        assert_eq!(local.crisis_pressure, 66);
        assert_eq!(initial.cities[&CityId(2)].crisis_pressure, 0);
        assert_eq!(civ.crisis_pressure, 16);
    }

    #[test]
    fn low_cohesion_contributes_pressure() {
        let mut initial = state();
        initial.civilizations.get_mut(&CivId(1)).unwrap().cohesion = 26;
        let mut healthy = city(CivId(1), 0, 4);
        healthy.groups[0].satisfaction = 100;
        healthy.last_yields.food = 4;
        initial.cities.insert(CityId(1), healthy);
        resolve_society(&mut initial, 99, &mut Vec::new());
        assert_eq!(initial.civilizations[&CivId(1)].cohesion, 26);
        assert_eq!(initial.civilizations[&CivId(1)].crisis_pressure, 4);
    }

    #[test]
    fn blocked_production_preserves_queue_food_and_completed_work() {
        let mut initial = state();
        let mut producer = city(CivId(1), 0, 4);
        producer.unit_queue.push("unit.settler".into());
        producer.food_stock = 12;
        producer.unit_production = 22;
        initial.cities.insert(CityId(1), producer);
        // Every neighbour is water: nowhere to spawn, so production waits complete.
        for tile in 1..4 { initial.tiles[tile].terrain = TERRAIN_OCEAN; }
        resolve_unit_production(&mut initial);
        let blocked = &initial.cities[&CityId(1)];
        assert_eq!(blocked.unit_queue, vec!["unit.settler"]);
        assert_eq!(blocked.unit_production, 22);
        assert_eq!(blocked.food_stock, 12);
        initial.units.clear();
        initial.cities.insert(CityId(SETTLER_CITY_ID_BASE + 9), city(CivId(2), 3, 1));
        resolve_unit_production(&mut initial);
        assert!(initial.cities[&CityId(1)].unit_queue.is_empty());
        assert_eq!(initial.cities[&CityId(1)].food_stock, 2);
        assert_eq!(initial.units[&UnitId(10)].unit_type, "unit.settler");
    }

    fn queue_state(queue: &[&str]) -> WorldState {
        let mut initial = state();
        let mut producer = city(CivId(1), 0, 4);
        producer.unit_queue = queue.iter().map(|unit| (*unit).to_owned()).collect();
        initial.cities.insert(CityId(1), producer);
        initial
    }

    fn rejected(result: &StepResult, reason: RejectionReason) -> bool { result.events.iter().any(|event| matches!(event, DomainEvent::CommandRejected { reason: found, .. } if *found == reason)) }

    fn queue_of(state: &WorldState) -> Vec<&str> { state.cities[&CityId(1)].unit_queue.iter().map(String::as_str).collect() }

    #[test]
    fn queue_remove_and_move_validate_and_reorder() {
        let initial = queue_state(&["unit.scout", "unit.settler", "unit.scout"]);
        let remove = |index: u32, actor: CivId| command(1, 1, actor, CommandPayload::RemoveQueuedUnit { city_id: CityId(1), index });
        let shift = |from: u32, to: u32| command(1, 1, CivId(1), CommandPayload::MoveQueuedUnit { city_id: CityId(1), from, to });
        let result = step(&initial, &[remove(1, CivId(1))], 99, &versions());
        assert_eq!(queue_of(&result.state), vec!["unit.scout", "unit.scout"]);
        let result = step(&initial, &[shift(1, 0)], 99, &versions());
        assert_eq!(queue_of(&result.state), vec!["unit.settler", "unit.scout", "unit.scout"]);
        let result = step(&initial, &[shift(0, 2)], 99, &versions());
        assert_eq!(queue_of(&result.state), vec!["unit.settler", "unit.scout", "unit.scout"]);
        for bad in [remove(3, CivId(1)), shift(3, 0), shift(0, 3)] {
            let result = step(&initial, &[bad], 99, &versions());
            assert_eq!(queue_of(&result.state), queue_of(&initial));
            assert!(rejected(&result, RejectionReason::InvalidQueueIndex), "{:?}", result.events);
        }
        let result = step(&initial, &[remove(0, CivId(2))], 99, &versions());
        assert!(rejected(&result, RejectionReason::NotCommandOwner));
        let unknown = command(1, 1, CivId(1), CommandPayload::RemoveQueuedUnit { city_id: CityId(77), index: 0 });
        let result = step(&initial, &[unknown], 99, &versions());
        assert!(rejected(&result, RejectionReason::UnknownCity));
    }

    #[test]
    fn removing_the_queue_head_keeps_accumulated_production() {
        let mut initial = queue_state(&["unit.settler", "unit.scout"]);
        initial.cities.get_mut(&CityId(1)).unwrap().unit_production = 7;
        let command = command(1, 1, CivId(1), CommandPayload::RemoveQueuedUnit { city_id: CityId(1), index: 0 });
        let result = step(&initial, &[command], 99, &versions());
        assert_eq!(queue_of(&result.state), vec!["unit.scout"]);
        assert_eq!(result.state.cities[&CityId(1)].unit_production, 7);
    }

    #[test]
    fn production_spawns_on_first_free_adjacent_land_tile_when_city_tile_is_occupied() {
        let mut initial = queue_state(&["unit.scout"]);
        initial.cities.get_mut(&CityId(1)).unwrap().unit_production = 100;
        // Unit 1 stands on the city tile; the foreign unit is moved around below.
        initial.units.get_mut(&UnitId(2)).unwrap().tile = TileIndex(0);
        let grid = grid_for(&initial).unwrap();
        let mut around: Vec<TileIndex> = grid.neighbors(grid.cell(TileIndex(0)).unwrap()).unwrap().into_iter().map(|cell| grid.tile_index(cell).unwrap()).collect();
        around.sort_unstable();
        around.dedup();
        assert!(around.len() >= 2, "test map needs two neighbours");
        let (first, second) = (around[0], around[1]);
        // Lowest neighbour is water, so the second one is the first free land tile.
        initial.tiles[first.0 as usize].terrain = TERRAIN_OCEAN;
        let mut resolved = initial.clone();
        resolve_unit_production(&mut resolved);
        let spawned: Vec<_> = resolved.units.iter().filter(|(id, _)| id.0 >= 3).collect();
        assert_eq!(resolved.units.len(), 3);
        assert!(resolved.cities[&CityId(1)].unit_queue.is_empty());
        assert_eq!(spawned.len(), 1);
        assert_eq!(spawned[0].1.tile, second);
        // Occupying the second tile moves the spawn on to the next free land tile (or waits).
        initial.units.get_mut(&UnitId(2)).unwrap().tile = second;
        let mut again = initial.clone();
        resolve_unit_production(&mut again);
        let tile_of_new = again.units.iter().find(|(id, _)| id.0 >= 3).map(|(_, unit)| unit.tile);
        assert_ne!(tile_of_new, Some(second));
        assert_ne!(tile_of_new, Some(first));
    }

    #[test]
    fn queue_commands_replay_deterministically() {
        let initial = queue_state(&["unit.scout", "unit.settler"]);
        let commands = [command(1, 1, CivId(1), CommandPayload::MoveQueuedUnit { city_id: CityId(1), from: 1, to: 0 }), command(2, 2, CivId(1), CommandPayload::RemoveQueuedUnit { city_id: CityId(1), index: 1 })];
        let a = step(&initial, &commands, 99, &versions());
        let b = step(&initial, &commands, 99, &versions());
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(a.state, b.state);
        assert_eq!(queue_of(&a.state), vec!["unit.settler"]);
    }

    #[test]
    fn queue_command_wire_format_is_adjacent_tagged() {
        let json = serde_json::to_string(&CommandPayload::MoveQueuedUnit { city_id: CityId(1), from: 2, to: 0 }).unwrap();
        assert!(json.contains("\"type\":\"move_queued_unit\"") && json.contains("\"data\""), "{json}");
        let json = serde_json::to_string(&CommandPayload::RemoveQueuedUnit { city_id: CityId(1), index: 0 }).unwrap();
        assert!(json.contains("\"type\":\"remove_queued_unit\""), "{json}");
    }

    #[test]
    fn workplace_allocation_covers_essential_upkeep_before_build_focus() {
        let mut initial = state();
        let mut producer = city(CivId(1), 0, 2);
        producer.focus = CityFocus::Build;
        initial.tiles[0].yields.food = 2;
        initial.tiles[1].yields.production = 6;
        initial.tiles[2].yields.wealth = 2;
        assert_eq!(allocate_workplaces(&initial, &producer, &BTreeSet::new()), vec![TileIndex(0), TileIndex(2)]);
    }

    #[test]
    fn replay_reproduces_each_recorded_turn_hash() {
        let initial = state();
        let snapshot = WorldSnapshot::new(initial.clone(), versions());
        let first = step(&initial, &[move_command(1, 10, CivId(1), UnitId(1), 1)], 99, &versions());
        let second_command = AcceptedCommand {
            command_id: 2,
            world_id: WorldId(9),
            turn: TurnNumber(1),
            accepted_sequence: 11,
            actor_id: CivId(2),
            origin: CommandOrigin::Bot,
            kind: CommandKind::SetResearch,
            payload: CommandPayload::SetResearch { research: "pottery".into() },
            grounding: Vec::new(),
            intent_evidence: None,
            mandate: None,
        };
        let second = step(&first.state, &[second_command.clone()], 99, &versions());
        let mut log = CommandLog::default();
        log.append(move_command(1, 10, CivId(1), UnitId(1), 1));
        log.append(second_command);
        log.record_turn(TurnNumber::ZERO, first.state_hash);
        log.record_turn(TurnNumber(1), second.state_hash);

        let replayed = replay(&snapshot, &log).unwrap();
        assert_eq!(replayed.state, second.state);
        assert_eq!(replayed.state_hashes.get(&TurnNumber::ZERO), Some(&first.state_hash));
        assert_eq!(replayed.state_hashes.get(&TurnNumber(1)), Some(&second.state_hash));
        let json = serde_json::to_string(&snapshot).unwrap();
        assert_eq!(serde_json::from_str::<WorldSnapshot>(&json).unwrap(), snapshot);
    }

    #[test]
    fn invalid_command_only_emits_rejection_and_does_not_change_mechanical_state() {
        let initial = state();
        let invalid = move_command(1, 10, CivId(1), UnitId(1), 99);
        let result = step(&initial, &[invalid], 99, &versions());
        let mut expected = initial.clone();
        reset_unit_movement(&mut expected);
        resolve_visibility(&mut expected);
        // Entropy bookkeeping (era budgets) is a resolution phase, not a command effect.
        resolve_entropy(&mut expected, 99);
        resolve_diplomacy(&mut expected, 99);
        expected.turn = TurnNumber(1);
        assert_eq!(result.state, expected);
        assert_eq!(result.events, vec![DomainEvent::CommandRejected {
            command_id: 1,
            reason: RejectionReason::InvalidTile,
        }]);
    }

    #[test]
    fn accepted_sequence_changes_conflicts_but_not_independent_commands() {
        let initial = state();
        let left_first = [
            move_command(1, 10, CivId(1), UnitId(1), 1),
            move_command(2, 11, CivId(2), UnitId(2), 1),
        ];
        let right_first = [
            move_command(1, 11, CivId(1), UnitId(1), 1),
            move_command(2, 10, CivId(2), UnitId(2), 1),
        ];
        let left_result = step(&initial, &left_first, 99, &versions());
        let right_result = step(&initial, &right_first, 99, &versions());
        assert_ne!(left_result.state_hash, right_result.state_hash);

        let independent_a = [
            move_command(1, 10, CivId(1), UnitId(1), 1),
            move_command(2, 11, CivId(2), UnitId(2), 3),
        ];
        let independent_b = [
            move_command(1, 11, CivId(1), UnitId(1), 1),
            move_command(2, 10, CivId(2), UnitId(2), 3),
        ];
        assert_eq!(
            step(&initial, &independent_a, 99, &versions()).state_hash,
            step(&initial, &independent_b, 99, &versions()).state_hash,
        );
    }

    #[test]
    fn starvation_reduces_population_and_never_underflows_stocks() {
        let mut initial = state();
        initial.units.clear();
        initial.cities.insert(CityId(1), city(CivId(1), 0, 2));
        let first = step(&initial, &[], 99, &versions());
        let second = step(&first.state, &[], 99, &versions());
        let starved = second.state.cities.get(&CityId(1)).unwrap();
        assert_eq!(starved.population, 1);
        assert_eq!(starved.food_stock, 0);
        assert_eq!(starved.groups.iter().map(|group| group.population).sum::<u32>(), starved.population);
    }

    #[test]
    fn crisis_pressure_is_clamped_and_collapse_freezes_after_two_zero_cohesion_turns() {
        let mut initial = state();
        initial.units.clear();
        initial.civilizations.get_mut(&CivId(1)).unwrap().cohesion = 0;
        let mut unstable = city(CivId(1), 0, 2);
        unstable.stability = 0;
        unstable.groups[0].satisfaction = 0;
        unstable.war_threat = 20;
        unstable.environmental_exposure = 20;
        initial.cities.insert(CityId(1), unstable);
        let first = step(&initial, &[], 99, &versions());
        let second = step(&first.state, &[], 99, &versions());
        let city = second.state.cities.get(&CityId(1)).unwrap();
        let civilization = second.state.civilizations.get(&CivId(1)).unwrap();
        assert!((70..=100).contains(&city.crisis_pressure));
        assert!((0..=100).contains(&civilization.crisis_pressure));
        assert!(civilization.frozen);
        assert!(second.events.contains(&DomainEvent::CollapseTriggered { civilization: CivId(1) }));
    }

    #[test]
    fn identical_economic_inputs_produce_the_same_hash() {
        let mut initial = state();
        initial.units.clear();
        initial.tiles[0].yields = TileYields { food: 6, production: 2, wealth: 3, knowledge: 1, culture: 1 };
        initial.cities.insert(CityId(1), city(CivId(1), 0, 1));
        assert_eq!(step(&initial, &[], 99, &versions()).state_hash, step(&initial, &[], 99, &versions()).state_hash);
    }

    #[test]
    fn research_without_prerequisites_is_rejected() {
        let result = step(&state(), &[command(1, 1, CivId(1), CommandPayload::SetResearch { research: "tech.storage".into() })], 99, &versions());
        assert_eq!(result.events, vec![DomainEvent::CommandRejected { command_id: 1, reason: RejectionReason::MissingTechnologyPrerequisite }]);
    }

    #[test]
    fn fourth_active_practice_is_rejected() {
        let mut initial = state();
        let civ = initial.civilizations.get_mut(&CivId(1)).unwrap();
        civ.researched_technologies.extend(["tech.foraging", "tech.council", "tech.paths", "tech.storage"].map(str::to_owned));
        civ.active_practices.extend(["practice.foraging", "practice.council", "practice.paths"].map(str::to_owned));
        let result = step(&initial, &[command(1, 1, CivId(1), CommandPayload::ActivatePractice { practice: "practice.storage".into() })], 99, &versions());
        assert_eq!(result.events, vec![DomainEvent::CommandRejected { command_id: 1, reason: RejectionReason::PracticeLimitReached }]);
    }

    #[test]
    fn research_uses_the_selected_integer_investment_and_conversion() {
        let mut initial = state();
        initial.units.clear();
        initial.cities.insert(CityId(1), city(CivId(1), 0, 2));
        initial.tiles[0].yields.production = 6;
        initial.tiles[1].yields.production = 6;
        let commands = [
            command(1, 1, CivId(1), CommandPayload::SetResearch { research: "tech.foraging".into() }),
            command(2, 2, CivId(1), CommandPayload::SetResearchInvestment { percent: 20 }),
        ];
        let result = step(&initial, &commands, 99, &versions());
        assert_eq!(result.state.civilizations[&CivId(1)].research_progress["tech.foraging"], 2);
    }

    #[test]
    fn attacks_against_one_hex_are_resolved_as_one_order_independent_confrontation() {
        let mut initial = state();
        initial.units.insert(UnitId(3), UnitState { owner: CivId(1), tile: TileIndex(3), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 0, explored: false, order: UnitOrder::Idle, skipped_turn: None });
        let first = command(1, 10, CivId(1), CommandPayload::DeclareAttack { attacker: UnitId(1), target: UnitId(2) });
        let second = command(2, 11, CivId(1), CommandPayload::DeclareAttack { attacker: UnitId(3), target: UnitId(2) });
        let forward = step(&initial, &[first.clone(), second.clone()], 99, &versions());
        let reverse = step(&initial, &[second, first], 99, &versions());
        assert_eq!(forward.state_hash, reverse.state_hash);
        assert_eq!(forward.state.units[&UnitId(2)].hit_points, 80);
        assert_eq!(forward.state.units[&UnitId(1)].hit_points, 90);
    }

    #[test]
    fn same_inputs_are_deterministic_with_visibility_and_combat() {
        let initial = state();
        let attack = command(1, 1, CivId(1), CommandPayload::DeclareAttack { attacker: UnitId(1), target: UnitId(2) });
        let left = step(&initial, &[attack.clone()], 99, &versions());
        let right = step(&initial, &[attack], 99, &versions());
        assert_eq!(left.state_hash, right.state_hash);
        assert_eq!(left.state.visibility[&CivId(1)][&TileIndex(0)], Visibility::Visible);
    }

    fn order_world() -> WorldState {
        let mut initial = state();
        initial.units.clear();
        initial.map_width = 20;
        initial.tiles = vec![TileState::default(); 200];
        initial.units.insert(UnitId(1), UnitState { owner: CivId(1), tile: TileIndex(0), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 3, explored: false, order: UnitOrder::Idle, skipped_turn: None });
        initial.units.insert(UnitId(2), UnitState { owner: CivId(2), tile: TileIndex(50), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 3, explored: false, order: UnitOrder::Idle, skipped_turn: None });
        initial
    }

    fn at_turn(mut command: AcceptedCommand, turn: u32) -> AcceptedCommand { command.turn = TurnNumber(turn); command }

    fn rejection_of(result: &StepResult, id: u64) -> Option<RejectionReason> {
        result.events.iter().find_map(|event| match event {
            DomainEvent::CommandRejected { command_id, reason } if *command_id == id => Some(*reason),
            _ => None,
        })
    }

    #[test]
    fn move_to_advances_each_turn_arrives_and_returns_to_idle() {
        let initial = order_world();
        let order = command(1, 1, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::MoveTo { target: TileIndex(7) } });
        let first = step(&initial, &[order], 99, &versions());
        assert_eq!(rejection_of(&first, 1), None);
        assert_eq!(first.state.units[&UnitId(1)].tile, TileIndex(3));
        assert_eq!(first.state.units[&UnitId(1)].order, UnitOrder::MoveTo { target: TileIndex(7) });
        let second = step(&first.state, &[], 99, &versions());
        assert_eq!(second.state.units[&UnitId(1)].tile, TileIndex(6));
        assert!(idle_units(&second.state, CivId(1)).is_empty());
        let third = step(&second.state, &[], 99, &versions());
        assert_eq!(third.state.units[&UnitId(1)].tile, TileIndex(7));
        assert_eq!(third.state.units[&UnitId(1)].order, UnitOrder::Idle);
        assert_eq!(idle_units(&third.state, CivId(1)), vec![UnitId(1)]);
    }

    #[test]
    fn move_to_becomes_idle_when_the_route_is_blocked() {
        let mut initial = order_world();
        initial.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::MoveTo { target: TileIndex(7) };
        initial.units.get_mut(&UnitId(2)).unwrap().tile = TileIndex(7);
        let result = step(&initial, &[], 99, &versions());
        assert_eq!(result.state.units[&UnitId(1)].tile, TileIndex(0));
        assert_eq!(result.state.units[&UnitId(1)].order, UnitOrder::Idle);
    }

    #[test]
    fn set_unit_order_validates_owner_unit_and_target() {
        let initial = order_world();
        let foreign = command(1, 1, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(2), order: UnitOrder::Fortify });
        let missing = command(2, 2, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(9), order: UnitOrder::Fortify });
        let occupied = command(3, 3, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::MoveTo { target: TileIndex(50) } });
        let off_map = command(4, 4, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::MoveTo { target: TileIndex(5000) } });
        let skip_foreign = command(5, 5, CivId(1), CommandPayload::SkipUnit { unit_id: UnitId(2) });
        let result = step(&initial, &[foreign, missing, occupied, off_map, skip_foreign], 99, &versions());
        assert_eq!(rejection_of(&result, 1), Some(RejectionReason::NotCommandOwner));
        assert_eq!(rejection_of(&result, 2), Some(RejectionReason::UnknownUnit));
        assert_eq!(rejection_of(&result, 3), Some(RejectionReason::DestinationOccupied));
        assert_eq!(rejection_of(&result, 4), Some(RejectionReason::InvalidTile));
        assert_eq!(rejection_of(&result, 5), Some(RejectionReason::NotCommandOwner));

        // A guard (movement 1) can never pay for a forest tile (cost 2).
        let mut hard = order_world();
        hard.units.get_mut(&UnitId(1)).unwrap().unit_type = "unit.guard".into();
        hard.tiles[1].terrain = TERRAIN_FOREST;
        let blocked = command(6, 1, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::MoveTo { target: TileIndex(1) } });
        assert_eq!(rejection_of(&step(&hard, &[blocked], 99, &versions()), 6), Some(RejectionReason::InvalidTile));
    }

    #[test]
    fn skip_unit_frees_the_unit_only_for_the_current_turn() {
        let initial = order_world();
        assert_eq!(idle_units(&initial, CivId(1)), vec![UnitId(1)]);
        let skip = command(1, 1, CivId(1), CommandPayload::SkipUnit { unit_id: UnitId(1) });
        assert!(idle_units_with(&initial, CivId(1), &[skip.clone()]).is_empty());
        let result = step(&initial, &[skip], 99, &versions());
        assert_eq!(result.state.units[&UnitId(1)].skipped_turn, Some(0));
        assert_eq!(result.state.turn, TurnNumber(1));
        assert_eq!(idle_units(&result.state, CivId(1)), vec![UnitId(1)]);
        let fortify = at_turn(command(2, 1, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::Fortify }), 1);
        assert!(idle_units_with(&result.state, CivId(1), &[fortify]).is_empty());
    }

    fn sentry_set(initial: &WorldState, foreign_tile: u32) -> StepResult {
        let mut initial = initial.clone();
        initial.units.get_mut(&UnitId(2)).unwrap().tile = TileIndex(foreign_tile);
        initial.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Sentry;
        step(&initial, &[], 99, &versions())
    }

    #[test]
    fn sentry_is_not_idle_and_stays_while_no_foreign_unit_is_in_range() {
        let initial = order_world();
        let order = command(1, 1, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::Sentry });
        assert!(idle_units_with(&initial, CivId(1), &[order.clone()]).is_empty());
        let set = step(&initial, &[order], 99, &versions());
        assert_eq!(rejection_of(&set, 1), None);
        assert_eq!(set.state.units[&UnitId(1)].order, UnitOrder::Sentry);
        assert!(idle_units(&set.state, CivId(1)).is_empty());
        let next = step(&set.state, &[], 99, &versions());
        assert_eq!(next.state.units[&UnitId(1)].order, UnitOrder::Sentry);
        assert!(idle_units(&next.state, CivId(1)).is_empty());
    }

    #[test]
    fn sentry_wakes_when_a_foreign_unit_is_in_range_and_not_for_own_units() {
        let far = sentry_set(&order_world(), 50);
        assert_eq!(far.state.units[&UnitId(1)].order, UnitOrder::Sentry);
        let near = sentry_set(&order_world(), 1);
        assert_eq!(near.state.units[&UnitId(1)].order, UnitOrder::Idle);
        assert_eq!(idle_units(&near.state, CivId(1)), vec![UnitId(1)]);
        let mut own = order_world();
        own.units.get_mut(&UnitId(2)).unwrap().owner = CivId(1);
        own.units.get_mut(&UnitId(2)).unwrap().tile = TileIndex(1);
        own.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Sentry;
        assert_eq!(step(&own, &[], 99, &versions()).state.units[&UnitId(1)].order, UnitOrder::Sentry);
    }

    #[test]
    fn sentry_wake_is_deterministic_and_wire_format_is_tagged() {
        let (a, b) = (sentry_set(&order_world(), 1), sentry_set(&order_world(), 1));
        assert_eq!(a.state_hash, b.state_hash);
        assert_eq!(serde_json::to_value(UnitOrder::Sentry).unwrap(), serde_json::json!({ "type": "sentry" }));
        let mut asleep = order_world();
        asleep.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Sentry;
        assert_ne!(asleep.state_hash(), order_world().state_hash());
    }

    #[test]
    #[test]
    fn a_unit_that_spent_all_movement_is_idle_again_next_turn() {
        let initial = order_world();
        let walk = command(1, 1, CivId(1), CommandPayload::MoveUnit { unit_id: UnitId(1), target: TileIndex(3) });
        // An accepted one-turn action counts as the unit's order for the open turn.
        assert!(idle_units_with(&initial, CivId(1), &[walk.clone()]).is_empty());
        let result = step(&initial, &[walk], 99, &versions());
        assert_eq!(rejection_of(&result, 1), None);
        assert_eq!(result.state.units[&UnitId(1)].movement_left, 0);
        assert_eq!(idle_units(&result.state, CivId(1)), vec![UnitId(1)]);
    }

    #[test]
    fn explore_walks_toward_unknown_tiles_and_goes_idle_when_none_remain() {
        let mut initial = order_world();
        initial.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Explore;
        assert!(can_explore(&initial, UnitId(1)));
        let moved = step(&initial, &[], 99, &versions());
        assert_ne!(moved.state.units[&UnitId(1)].tile, TileIndex(0));
        assert_eq!(moved.state.units[&UnitId(1)].order, UnitOrder::Explore);

        let known: BTreeMap<TileIndex, Visibility> = (0..200).map(|tile| (TileIndex(tile), Visibility::Remembered)).collect();
        initial.visibility.insert(CivId(1), known);
        assert!(!can_explore(&initial, UnitId(1)));
        let done = step(&initial, &[], 99, &versions());
        assert_eq!(done.state.units[&UnitId(1)].tile, TileIndex(0));
        assert_eq!(done.state.units[&UnitId(1)].order, UnitOrder::Idle);
    }

    #[test]
    fn orders_are_part_of_the_hash_and_replay_identically() {
        let initial = order_world();
        let snapshot = WorldSnapshot::new(initial.clone(), versions());
        let explore = command(1, 1, CivId(1), CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::Explore });
        let travel = command(2, 2, CivId(2), CommandPayload::SetUnitOrder { unit_id: UnitId(2), order: UnitOrder::MoveTo { target: TileIndex(53) } });
        let first = step(&initial, &[explore.clone(), travel.clone()], 99, &versions());
        let again = step(&initial, &[explore.clone(), travel.clone()], 99, &versions());
        assert_eq!(first.state_hash, again.state_hash);
        let skip = command(1, 1, CivId(1), CommandPayload::SkipUnit { unit_id: UnitId(1) });
        assert_ne!(step(&initial, &[skip], 99, &versions()).state_hash, step(&initial, &[], 99, &versions()).state_hash);
        let second = step(&first.state, &[], 99, &versions());
        let mut log = CommandLog::default();
        log.append(explore);
        log.append(travel);
        log.record_turn(TurnNumber::ZERO, first.state_hash);
        log.record_turn(TurnNumber(1), second.state_hash);
        let replayed = replay(&snapshot, &log).unwrap();
        assert_eq!(replayed.state, second.state);
    }
}
