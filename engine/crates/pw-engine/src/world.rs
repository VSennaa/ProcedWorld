//! Deterministic world state, accepted commands, turn resolution, and replay.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    diplomacy::{DiplomacyState, DiplomaticAction, DiplomaticResolution, ProposalKind},
    hex::Grid,
    hash::{StateHash, StateHasher},
    ids::{CivId, CityId, TileIndex, TurnNumber, UnitId},
    rng::Rng,
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
    fn capped(self) -> Self {
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

/// Minimal unit state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitState {
    pub owner: CivId,
    pub tile: TileIndex,
    pub unit_type: String,
    pub hit_points: u8,
    pub movement_left: u8,
    pub explored: bool,
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
    DeclareAttack,
    Explore,
    KeepPlan,
    ProposeDiplomacy,
    BreakTreaty,
    DeclareWar,
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
    DeclareAttack { attacker: UnitId, target: UnitId },
    Explore { unit_id: UnitId },
    KeepPlan,
    /// Structured proposal; the engine evaluates the automatic counterpart with `A`.
    ProposeDiplomacy { recipient: CivId, kind: ProposalKind },
    BreakTreaty { counterpart: CivId },
    /// `objective` is a catalog id (`objective.*`); `cause` is the Ledger entry that justifies the war.
    DeclareWar { target: CivId, objective: String, cause: u64 },
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
            Self::DeclareAttack { .. } => CommandKind::DeclareAttack,
            Self::Explore { .. } => CommandKind::Explore,
            Self::KeepPlan => CommandKind::KeepPlan,
            Self::ProposeDiplomacy { .. } => CommandKind::ProposeDiplomacy,
            Self::BreakTreaty { .. } => CommandKind::BreakTreaty,
            Self::DeclareWar { .. } => CommandKind::DeclareWar,
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
}

#[derive(Deserialize)]
struct TechCatalog { technologies: Vec<TechDefinition> }

#[derive(Deserialize)]
struct TechDefinition {
    id: String,
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
    resolve_visibility(&mut next);
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

fn grid_for(state: &WorldState) -> Result<Grid, RejectionReason> {
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

fn resolve_movement(_state: &mut WorldState, seed: u64) { let _rng = Rng::derive(seed, "movement"); }

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
    let mut city_yields = BTreeMap::new();
    let mut claimed = BTreeSet::new();
    for (city_id, city) in &state.cities {
        let workplaces = allocate_workplaces(state, city, &claimed);
        let mut total = TileYields::default();
        for tile in &workplaces {
            let yields = state.tiles[tile.0 as usize].yields.capped();
            total.food = total.food.saturating_add(yields.food);
            total.production = total.production.saturating_add(yields.production);
            total.wealth = total.wealth.saturating_add(yields.wealth);
            total.knowledge = total.knowledge.saturating_add(yields.knowledge);
            total.culture = total.culture.saturating_add(yields.culture);
        }
        claimed.extend(workplaces.iter().copied());
        city_yields.insert(*city_id, (workplaces, total));
    }
    let mut wealth_by_civ: BTreeMap<CivId, u32> = BTreeMap::new();
    for (city_id, (_, yields)) in &city_yields {
        *wealth_by_civ.entry(state.cities[city_id].owner).or_default() += u32::from(yields.wealth);
    }
    let mut available_wealth: BTreeMap<_, _> = state.civilizations.iter().map(|(id, civ)| {
        (*id, civ.treasury_wealth.saturating_add(wealth_by_civ.get(id).copied().unwrap_or(0)))
    }).collect();
    for (city_id, city) in &mut state.cities {
        let (workplaces, yields) = city_yields.remove(city_id).expect("all cities were allocated");
        city.workplaces = workplaces;
        city.last_yields = yields;
        city.food_stock = city.food_stock.saturating_add(u32::from(yields.food));
        let maintenance = 1 + (city.population + 3) / 4;
        let available = available_wealth.entry(city.owner).or_default();
        city.essential_maintenance_unpaid = *available < maintenance;
        *available = available.saturating_sub(maintenance);
        let demand = city.population;
        let consumed = city.food_stock.min(demand);
        city.food_stock -= consumed;
        let missing = demand - consumed;
        city.consecutive_food_shortages = if missing > 0 { city.consecutive_food_shortages.saturating_add(1) } else { 0 };
        city.deprivation = (missing.saturating_mul(5) + if city.essential_maintenance_unpaid { 5 } else { 0 }).min(20) as u8;
        city.food_stock = city.food_stock.min(3u32.saturating_mul(city.population).saturating_add(4));
    }
    for (civ_id, civilization) in &mut state.civilizations {
        let income = wealth_by_civ.remove(civ_id).unwrap_or(0);
        let maintenance: u32 = state.cities.values().filter(|city| city.owner == *civ_id).map(|city| 1 + (city.population + 3) / 4).sum();
        civilization.treasury_wealth = civilization.treasury_wealth.saturating_add(income).saturating_sub(maintenance);
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
        let can_spawn = next_id.is_some_and(|id| id <= u32::MAX - SETTLER_CITY_ID_BASE)
            && !state.units.values().any(|unit| unit.tile == tile);
        {
            let city = state.cities.get_mut(&city_id).expect("city id was collected from state");
            city.unit_production = city.unit_production.saturating_add(produced);
            if !complete || !can_spawn { continue; }
            city.unit_production -= cost;
            city.food_stock -= food_cost;
            city.unit_queue.remove(0);
        }
        let next_id = next_id.expect("spawn id was checked before spending resources");
        state.units.insert(UnitId(next_id), UnitState { owner, tile, unit_type, hit_points: 100, movement_left: definition.movement, explored: false });
    }
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

fn allocate_workplaces(state: &WorldState, city: &CityState, claimed: &BTreeSet<TileIndex>) -> Vec<TileIndex> {
    let candidates: Vec<_> = workable_tiles(state, city.tile).into_iter()
        .filter(|tile| !claimed.contains(tile))
        .collect();
    let mut selected = Vec::new();
    let mut food_candidates = candidates.clone();
    food_candidates.sort_unstable_by_key(|tile| (std::cmp::Reverse(state.tiles[tile.0 as usize].yields.capped().food), tile.0));
    let mut food = 0u32;
    for tile in food_candidates {
        if food >= city.population || selected.len() >= city.population as usize { break; }
        let yield_food = u32::from(state.tiles[tile.0 as usize].yields.capped().food);
        if yield_food == 0 { break; }
        food += yield_food;
        selected.push(tile);
    }
    let mut remaining: Vec<_> = candidates.into_iter().filter(|tile| !selected.contains(tile)).collect();
    // Essential upkeep precedes the discretionary focus (GDD 04).
    let maintenance = 1 + (city.population + 3) / 4;
    let mut wealth: u32 = selected.iter().map(|tile| u32::from(state.tiles[tile.0 as usize].yields.capped().wealth)).sum();
    remaining.sort_unstable_by_key(|tile| (std::cmp::Reverse(state.tiles[tile.0 as usize].yields.capped().wealth), tile.0));
    for tile in &remaining {
        if wealth >= maintenance || selected.len() >= city.population as usize { break; }
        let output = u32::from(state.tiles[tile.0 as usize].yields.capped().wealth);
        if output == 0 { break; }
        wealth += output;
        selected.push(*tile);
    }
    remaining.retain(|tile| !selected.contains(tile));
    remaining.sort_unstable_by_key(|tile| {
        let yields = state.tiles[tile.0 as usize].yields.capped();
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
            city.crisis_pressure = (2 * i32::from(city.deprivation) + 2 * i32::from(city.group_tension) + i32::from(city.war_threat) + i32::from(city.environmental_exposure) + (100 - i32::from(city.stability)) / 10 - i32::from(civ.cohesion) / 5).clamp(0, 100) as u8;
            city.crisis_turns = if city.crisis_pressure >= 70 { city.crisis_turns.saturating_add(1) } else { 0 };
        }
        civ.crisis_pressure = if population == 0 { 0 } else { (state.cities.values().filter(|city| city.owner == *civ_id).map(|city| city.population * u32::from(city.crisis_pressure)).sum::<u32>() / population).min(100) as u8 };
        if civ.zero_cohesion_turns >= 2 && !civ.frozen { civ.frozen = true; events.push(DomainEvent::CollapseTriggered { civilization: *civ_id }); }
    }
}
fn resolve_entropy(_state: &mut WorldState, seed: u64) { let _rng = Rng::derive(seed, "entropy"); }
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
        units.insert(UnitId(1), UnitState { owner: CivId(1), tile: TileIndex(0), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 0, explored: false });
        units.insert(UnitId(2), UnitState { owner: CivId(2), tile: TileIndex(2), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 0, explored: false });
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
        // 2*5 + 2*20 + 7 + 3 + 65/10 - 49/5 = 57.
        assert_eq!(local.crisis_pressure, 57);
        assert_eq!(initial.cities[&CityId(2)].crisis_pressure, 0);
        assert_eq!(civ.crisis_pressure, 14);
    }

    #[test]
    fn low_cohesion_alone_does_not_create_pressure() {
        let mut initial = state();
        initial.civilizations.get_mut(&CivId(1)).unwrap().cohesion = 26;
        let mut healthy = city(CivId(1), 0, 4);
        healthy.groups[0].satisfaction = 100;
        healthy.last_yields.food = 4;
        initial.cities.insert(CityId(1), healthy);
        resolve_society(&mut initial, 99, &mut Vec::new());
        assert_eq!(initial.civilizations[&CivId(1)].cohesion, 26);
        assert_eq!(initial.civilizations[&CivId(1)].crisis_pressure, 0);
    }

    #[test]
    fn blocked_production_preserves_queue_food_and_completed_work() {
        let mut initial = state();
        let mut producer = city(CivId(1), 0, 4);
        producer.unit_queue.push("unit.settler".into());
        producer.food_stock = 12;
        producer.unit_production = 22;
        initial.cities.insert(CityId(1), producer);
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
        initial.units.insert(UnitId(3), UnitState { owner: CivId(1), tile: TileIndex(3), unit_type: "unit.scout".into(), hit_points: 100, movement_left: 0, explored: false });
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
}
