//! Entropy: the world event director (GDD 08, SDD 06).
//!
//! The director is outside `step`. It reads the authoritative state, evaluates the closed catalog DSL and
//! proposes `ApplyEvent` commands with origin `Entropy`. The engine re-validates budget, phase, cooldown,
//! protection and target ownership when it applies them, so a bad proposal is rejected rather than trusted.
//! Everything here is integer arithmetic over ordered collections and the versioned PRNG.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    ai::{ActionIntent, ChoiceRequest, DecisionPort, IntentValidator, ACTION_INTENT_SCHEMA_VERSION},
    dsl::{eligible_targets, EffectOp, EntityRef, EventCatalog, EventChoice, Fact, Resource, StateView, TargetKind, Template, MAX_ADJUST_AMOUNT, MAX_OPERATIONS},
    governor::Mandate,
    hash::StateHasher,
    ids::{CityId, CivId},
    rng::Rng,
    world::{
        AcceptedCommand, AiLayer, AiStatus, CityState, CommandKind, CommandOrigin, CommandPayload, GroundingRef, IntentEvidence, LedgerEntry, RejectionReason, WorldState,
        TERRAIN_COAST, TERRAIN_DESERT, TERRAIN_FOREST, TERRAIN_JUNGLE, TERRAIN_OCEAN, TERRAIN_PLAINS, TERRAIN_STEPPE, TERRAIN_SWAMP,
    },
};

/// Initial balance values (GDD 08, "sujeito a balanceamento"). An era window has a fixed length until
/// milestone-based eras (GDD 10) exist.
pub const ERA_TURNS: u32 = 8;
pub const BASE_WORLD_BUDGET: u32 = 12;
pub const BUDGET_PER_LIVING_CIVILIZATION: u32 = 2;
pub const RESERVE_BASE: u32 = 2;
pub const RESERVE_MIN: u32 = 2;
pub const RESERVE_MAX: u32 = 6;
pub const RESERVE_PRESSURE_STEP: u32 = 25;
/// Curve over an era of `ERA_TURNS`: reading 25%, crisis 55% (cumulative 80%), respite the last 20%.
pub const READING_TURNS: u32 = 2;
pub const CRISIS_END_TURN: u32 = 6;
pub const READING_CAP_PERCENT: u32 = 25;
pub const CRISIS_CAP_PERCENT: u32 = 80;
/// A civilization with less cohesion than this is never targeted: no event may push it into collapse.
pub const FRAGILE_COHESION: u8 = 25;
pub const MAX_PENDING_PER_CIVILIZATION: usize = 1;
pub const MAX_EVENTS_PER_TURN: usize = 2;
pub const SAME_CATEGORY_PROTECTION_TURNS: u32 = 3;
pub const OTHER_CRISIS_PROTECTION_TURNS: u32 = 2;
pub const PROTECTION_MIN_COST: u8 = 2;
pub const SEVERE_COST: u8 = 3;
/// Tags added by a chosen response last this many turns.
pub const CHOICE_TAG_TURNS: u32 = 4;
pub const MAX_ENVIRONMENTAL_EXPOSURE: u32 = 20;
const RESPITE_CATEGORIES: [&str; 3] = ["discovery", "technology", "renewal"];
/// Categories whose active events act as negative climate modifiers: `E` in GDD 12.
const ENVIRONMENTAL_CATEGORIES: [&str; 4] = ["climate", "epidemic", "terrain", "magic"];
/// The collapse category belongs to the society systems; an event never causes collapse (GDD 08).
const EXCLUDED_CATEGORIES: [&str; 1] = ["collapse"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Personality { Cyclical, Contentious, Transformative }

impl Personality {
    const fn tag(self) -> u8 { match self { Self::Cyclical => 0, Self::Contentious => 1, Self::Transformative => 2 } }
    /// Personality only weights templates that are already valid; it never changes limits or protections.
    pub fn prefers(self, category: &str) -> bool {
        match self {
            Self::Cyclical => matches!(category, "climate" | "epidemic" | "renewal"),
            Self::Contentious => matches!(category, "diplomacy" | "revolt" | "social"),
            Self::Transformative => matches!(category, "discovery" | "technology" | "magic" | "terrain"),
        }
    }
    pub fn weight(self, category: &str) -> u32 { if self.prefers(category) { 2 } else { 1 } }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protection {
    pub civilization: CivId,
    /// `None` protects against any event of at least `min_cost`.
    pub category: Option<String>,
    pub min_cost: u8,
    pub until: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagMark { pub entity: EntityRef, pub tag: String, pub until: u32 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveEvent {
    pub template_id: String,
    pub category: String,
    pub civilization: CivId,
    pub target: EntityRef,
    pub cost: u8,
    pub started: u32,
    /// Last turn on which a response is accepted.
    pub expires: u32,
    pub related: Option<CivId>,
    pub choices: Vec<EventChoice>,
}

/// Director state that the pure `step` owns: budgets, cooldowns, protections, pending events and tags.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntropyState {
    pub personality: Option<Personality>,
    pub era: Option<u32>,
    pub world_budget: u32,
    pub world_spent: u32,
    pub civ_reserve: BTreeMap<CivId, u32>,
    pub civ_spent: BTreeMap<CivId, u32>,
    /// First turn on which a template may fire again.
    pub cooldowns: BTreeMap<String, u32>,
    pub protections: Vec<Protection>,
    pub pending: BTreeMap<u64, ActiveEvent>,
    pub tags: Vec<TagMark>,
    pub next_event_id: u64,
}

impl EntropyState {
    pub(crate) fn write_hash(&self, hasher: &mut StateHasher) {
        hasher.write_u8(self.personality.map_or(0, |personality| personality.tag() + 1));
        hasher.write_bool(self.era.is_some());
        hasher.write_u32(self.era.unwrap_or(0));
        hasher.write_u32(self.world_budget);
        hasher.write_u32(self.world_spent);
        hasher.write_u64(self.civ_reserve.len() as u64);
        for (civ, value) in &self.civ_reserve { hasher.write_u32(civ.0); hasher.write_u32(*value); }
        hasher.write_u64(self.civ_spent.len() as u64);
        for (civ, value) in &self.civ_spent { hasher.write_u32(civ.0); hasher.write_u32(*value); }
        hasher.write_u64(self.cooldowns.len() as u64);
        for (id, until) in &self.cooldowns { write_text(hasher, id); hasher.write_u32(*until); }
        hasher.write_u64(self.protections.len() as u64);
        for protection in &self.protections {
            hasher.write_u32(protection.civilization.0);
            hasher.write_bool(protection.category.is_some());
            write_text(hasher, protection.category.as_deref().unwrap_or(""));
            hasher.write_u8(protection.min_cost);
            hasher.write_u32(protection.until);
        }
        hasher.write_u64(self.pending.len() as u64);
        for (id, event) in &self.pending {
            hasher.write_u64(*id);
            write_text(hasher, &event.template_id);
            write_text(hasher, &event.category);
            hasher.write_u32(event.civilization.0);
            write_entity(hasher, event.target);
            hasher.write_u8(event.cost);
            hasher.write_u32(event.started);
            hasher.write_u32(event.expires);
            hasher.write_bool(event.related.is_some());
            hasher.write_u32(event.related.map_or(0, |civ| civ.0));
            hasher.write_u64(event.choices.len() as u64);
            for choice in &event.choices { write_text(hasher, &choice.id); hasher.write_u64(choice.effects.len() as u64); }
        }
        hasher.write_u64(self.tags.len() as u64);
        for mark in &self.tags { write_entity(hasher, mark.entity); write_text(hasher, &mark.tag); hasher.write_u32(mark.until); }
        hasher.write_u64(self.next_event_id);
    }
}

fn write_text(hasher: &mut StateHasher, value: &str) {
    hasher.write_u64(value.len() as u64);
    hasher.write_bytes(value.as_bytes());
}

fn write_entity(hasher: &mut StateHasher, entity: EntityRef) {
    match entity {
        EntityRef::Civilization(id) => { hasher.write_u8(0); hasher.write_u32(id.0); }
        EntityRef::City(id) => { hasher.write_u8(1); hasher.write_u32(id.0); }
        EntityRef::Tile(id) => { hasher.write_u8(2); hasher.write_u32(id.0); }
        EntityRef::Route(id) => { hasher.write_u8(3); hasher.write_u32(id); }
    }
}

/// Catalog bundled with the engine. Its validity is covered by tests; the engine fails closed otherwise.
pub fn bundled_catalog() -> EventCatalog {
    EventCatalog::parse(include_str!("../../../../data/catalogs/event_templates.json")).expect("the bundled event catalog is valid")
}

pub const fn era_position(turn: u32) -> u32 { turn % ERA_TURNS }

/// Phase rule of the reading / crisis / respite curve. Respite never starts a grave event.
pub fn phase_allows(position: u32, category: &str, cost: u8) -> bool {
    if position < READING_TURNS { cost <= 1 } else if position < CRISIS_END_TURN { true } else { cost <= 2 && RESPITE_CATEGORIES.contains(&category) }
}

/// Cumulative share of the world budget that may be spent by the end of this phase.
pub const fn phase_cap(budget: u32, position: u32) -> u32 {
    let percent = if position < READING_TURNS { READING_CAP_PERCENT } else if position < CRISIS_END_TURN { CRISIS_CAP_PERCENT } else { 100 };
    budget * percent / 100
}

/// The civilization that bears an event aimed at `entity`. A tile belongs to the city that works it.
pub fn owner_of(state: &WorldState, entity: EntityRef) -> Option<CivId> {
    match entity {
        EntityRef::Civilization(civ) => state.civilizations.contains_key(&civ).then_some(civ),
        EntityRef::City(city) => state.cities.get(&city).map(|city| city.owner),
        EntityRef::Tile(tile) => state.cities.values().find(|city| city.workplaces.contains(&tile)).map(|city| city.owner),
        EntityRef::Route(_) => None,
    }
}

fn is_protected(entropy: &EntropyState, civilization: CivId, category: &str, cost: u8, turn: u32) -> bool {
    entropy.protections.iter().any(|protection| {
        protection.civilization == civilization && protection.until > turn && cost >= protection.min_cost
            && protection.category.as_deref().map_or(true, |protected| protected == category)
    })
}

fn tile_tags(terrain: u8, river: bool) -> Vec<&'static str> {
    let mut tags = match terrain {
        TERRAIN_PLAINS => vec!["biome.plains"],
        TERRAIN_FOREST => vec!["biome.forest"],
        TERRAIN_JUNGLE => vec!["biome.jungle"],
        TERRAIN_SWAMP => vec!["biome.swamp"],
        TERRAIN_DESERT => vec!["biome.desert", "terrain.prospectable"],
        TERRAIN_STEPPE => vec!["biome.steppe", "terrain.prospectable"],
        TERRAIN_COAST => vec!["biome.coast"],
        TERRAIN_OCEAN => vec!["biome.ocean"],
        _ => Vec::new(),
    };
    if river { tags.push("route.crossing"); }
    tags
}

/// The read-only view the DSL interpreter sees. Facts the engine does not model yet are `None`.
pub struct WorldView<'a> { pub state: &'a WorldState }

fn civilization_is_active(state: &WorldState, civilization: CivId) -> bool {
    state.civilizations.get(&civilization).is_some_and(|civ| !civ.frozen)
}

fn city_fact(state: &WorldState, city: &CityState, fact: Fact) -> i64 {
    match fact {
        Fact::CityExposedFarms => city.workplaces.iter().filter(|tile| state.tiles.get(tile.0 as usize).is_some_and(|tile| tile.yields.food > 0)).count() as i64,
        Fact::CityDeprivation => i64::from(city.deprivation),
        Fact::CityProduction => i64::from(city.last_yields.production),
        Fact::CityGroupTension => i64::from(city.group_tension),
        Fact::CityDensity => i64::from(city.population),
        _ => 0,
    }
}

impl StateView for WorldView<'_> {
    fn entities(&self, kind: TargetKind) -> Vec<EntityRef> {
        let state = self.state;
        match kind {
            TargetKind::Civilization => state.civilizations.iter().filter(|(_, civ)| !civ.frozen).map(|(id, _)| EntityRef::Civilization(*id)).collect(),
            TargetKind::City => state.cities.iter().filter(|(_, city)| civilization_is_active(state, city.owner)).map(|(id, _)| EntityRef::City(*id)).collect(),
            TargetKind::Tile => {
                let mut tiles = BTreeSet::new();
                for city in state.cities.values().filter(|city| civilization_is_active(state, city.owner)) { tiles.extend(city.workplaces.iter().copied()); }
                tiles.into_iter().map(EntityRef::Tile).collect()
            }
            TargetKind::Route => Vec::new(),
        }
    }

    fn fact(&self, entity: EntityRef, fact: Fact) -> Option<i64> {
        let state = self.state;
        let owner = owner_of(state, entity)?;
        let civ = state.civilizations.get(&owner)?;
        match fact {
            Fact::CivCohesion => Some(i64::from(civ.cohesion)),
            Fact::CivCulture => Some(i64::from(civ.culture)),
            Fact::CivKnowledge => Some(i64::from(civ.knowledge)),
            Fact::CivRecentDeprivation => Some(i64::from(civ.deprivation)),
            Fact::CivCrisisPressure => Some(i64::from(civ.crisis_pressure)),
            Fact::CitySanitaryStock | Fact::RouteActive => None,
            Fact::CityExposedFarms | Fact::CityDeprivation | Fact::CityProduction | Fact::CityGroupTension | Fact::CityDensity => match entity {
                EntityRef::City(id) => state.cities.get(&id).map(|city| city_fact(state, city, fact)),
                // A civilization satisfies a city fact when its best city does (only `gte` tests exist).
                EntityRef::Civilization(id) => state.cities.values().filter(|city| city.owner == id).map(|city| city_fact(state, city, fact)).max(),
                _ => None,
            },
        }
    }

    fn has_tag(&self, entity: EntityRef, tag: &str) -> bool {
        let state = self.state;
        let turn = state.turn.0;
        if state.entropy.tags.iter().any(|mark| mark.entity == entity && mark.tag == tag && mark.until > turn) { return true; }
        match entity {
            EntityRef::Tile(tile) => state.tiles.get(tile.0 as usize).is_some_and(|tile| tile_tags(tile.terrain, tile.river).contains(&tag)),
            EntityRef::City(id) => state.cities.get(&id).and_then(|city| state.tiles.get(city.tile.0 as usize)).is_some_and(|tile| tile_tags(tile.terrain, tile.river).contains(&tag)),
            _ => false,
        }
    }

    /// No diplomacy system supplies a traceable counterparty yet, so there is none.
    fn related(&self, _: EntityRef) -> Option<CivId> { None }
}

fn food_available(state: &WorldState, actor: CivId, target: EntityRef) -> u32 {
    match target {
        EntityRef::City(id) => state.cities.get(&id).map_or(0, |city| city.food_stock),
        _ => state.cities.values().filter(|city| city.owner == actor).map(|city| city.food_stock).sum(),
    }
}

/// True when `actor` can pay every cost in `ops` and every diplomatic party exists.
pub fn ops_affordable(state: &WorldState, actor: CivId, target: EntityRef, ops: &[EffectOp], related: Option<CivId>) -> bool {
    let Some(civ) = state.civilizations.get(&actor) else { return false; };
    ops.iter().all(|op| match op {
        EffectOp::AdjustResource { resource, amount } if *amount < 0 => {
            let need = amount.unsigned_abs();
            match resource {
                Resource::Wealth => civ.treasury_wealth >= need,
                Resource::Knowledge => civ.knowledge >= need,
                Resource::Culture => civ.culture >= need,
                Resource::Food => food_available(state, actor, target) >= need,
                Resource::Production => true,
            }
        }
        EffectOp::CreateLedgerEntry { .. } => related.is_some(),
        _ => true,
    })
}

fn adjusted(value: u32, amount: i32) -> u32 {
    if amount >= 0 { value.saturating_add(amount as u32) } else { value.saturating_sub(amount.unsigned_abs()) }
}

fn adjust_resource(state: &mut WorldState, actor: CivId, target: EntityRef, resource: Resource, amount: i32) {
    match resource {
        Resource::Wealth => if let Some(civ) = state.civilizations.get_mut(&actor) { civ.treasury_wealth = adjusted(civ.treasury_wealth, amount); },
        Resource::Knowledge => if let Some(civ) = state.civilizations.get_mut(&actor) { civ.knowledge = adjusted(civ.knowledge, amount); },
        Resource::Culture => if let Some(civ) = state.civilizations.get_mut(&actor) { civ.culture = adjusted(civ.culture, amount); },
        Resource::Food | Resource::Production => {
            let owned: Vec<CityId> = state.cities.iter()
                .filter(|(id, city)| city.owner == actor && match target { EntityRef::City(wanted) => **id == wanted, _ => true })
                .map(|(id, _)| *id).collect();
            let mut remaining = amount.unsigned_abs();
            for id in owned {
                let Some(city) = state.cities.get_mut(&id) else { continue; };
                let stock = if resource == Resource::Food { &mut city.food_stock } else { &mut city.unit_production };
                if amount >= 0 {
                    // A gain goes to one city only: the targeted one, or the lowest id for a civilization-wide event.
                    *stock = stock.saturating_add(remaining);
                    break;
                }
                let taken = (*stock).min(remaining);
                *stock -= taken;
                remaining -= taken;
                if remaining == 0 { break; }
            }
        }
    }
}

fn add_tag(state: &mut WorldState, target: EntityRef, tag: &str, until: u32) {
    if let Some(mark) = state.entropy.tags.iter_mut().find(|mark| mark.entity == target && mark.tag == tag) {
        mark.until = mark.until.max(until);
    } else {
        state.entropy.tags.push(TagMark { entity: target, tag: tag.to_owned(), until });
    }
}

/// Applies validated operations. None of them can remove a city, change cohesion or freeze a civilization.
fn apply_ops(state: &mut WorldState, actor: CivId, target: EntityRef, ops: &[EffectOp], related: Option<CivId>, tag_until: u32) {
    for op in ops {
        match op {
            EffectOp::AdjustResource { resource, amount } => adjust_resource(state, actor, target, *resource, *amount),
            EffectOp::AddTag { tag } => add_tag(state, target, tag, tag_until),
            EffectOp::RemoveTag { tag } => state.entropy.tags.retain(|mark| !(mark.entity == target && mark.tag == *tag)),
            EffectOp::CreateLedgerEntry { duration, .. } => {
                if let Some(counterparty) = related { state.ledger.push(LedgerEntry { from: actor, to: counterparty, value: i64::from(*duration) }); }
            }
            // The chronicle lives outside deterministic state; the accepted command is its durable record.
            EffectOp::EmitChronicle { .. } => {}
        }
    }
}

fn ops_are_bounded(ops: &[EffectOp]) -> bool {
    ops.len() <= MAX_OPERATIONS && ops.iter().all(|op| match op {
        EffectOp::AdjustResource { amount, .. } => amount.abs() <= MAX_ADJUST_AMOUNT,
        EffectOp::AddTag { tag } | EffectOp::RemoveTag { tag } => !tag.is_empty() && tag.len() <= 96,
        EffectOp::CreateLedgerEntry { term, .. } => !term.is_empty() && term.len() <= 96,
        EffectOp::EmitChronicle { template, parameters } => !template.is_empty() && template.len() <= 96 && parameters.len() <= 8,
    })
}

/// Applies an `ApplyEvent` command. Every rule is re-checked here; the director is not trusted.
pub(crate) fn apply_event(state: &mut WorldState, command: &AcceptedCommand) -> Result<(), RejectionReason> {
    let CommandPayload::ApplyEvent { template_id, category, cost, duration, cooldown, target, related, ops, choices, .. } = &command.payload else {
        return Err(RejectionReason::KindPayloadMismatch);
    };
    if command.origin != CommandOrigin::Entropy { return Err(RejectionReason::EventForbidden); }
    let actor = command.actor_id;
    let turn = state.turn.0;
    if owner_of(state, *target) != Some(actor) { return Err(RejectionReason::EventInvalid); }
    if *cost == 0 || *cost > crate::dsl::MAX_TENSION_COST || *duration == 0 || choices.len() < 2 || choices.len() > 3 { return Err(RejectionReason::EventInvalid); }
    if !ops_are_bounded(ops) || choices.iter().any(|choice| !ops_are_bounded(&choice.effects)) { return Err(RejectionReason::EventInvalid); }
    if related.is_none() && ops.iter().any(|op| matches!(op, EffectOp::CreateLedgerEntry { .. })) { return Err(RejectionReason::EventInvalid); }
    let entropy = &state.entropy;
    if entropy.era != Some(turn / ERA_TURNS) { return Err(RejectionReason::EventOverBudget); }
    let position = era_position(turn);
    if !phase_allows(position, category, *cost) { return Err(RejectionReason::EventOverBudget); }
    if entropy.world_spent + u32::from(*cost) > phase_cap(entropy.world_budget, position) { return Err(RejectionReason::EventOverBudget); }
    let civ_spent = entropy.civ_spent.get(&actor).copied().unwrap_or(0);
    if civ_spent + u32::from(*cost) > entropy.civ_reserve.get(&actor).copied().unwrap_or(0) { return Err(RejectionReason::EventOverBudget); }
    if entropy.cooldowns.get(template_id).is_some_and(|until| turn < *until) { return Err(RejectionReason::EventOnCooldown); }
    if is_protected(entropy, actor, category, *cost, turn) { return Err(RejectionReason::EventProtected); }
    if entropy.pending.values().filter(|event| event.civilization == actor).count() >= MAX_PENDING_PER_CIVILIZATION { return Err(RejectionReason::EventBusy); }
    if EXCLUDED_CATEGORIES.contains(&category.as_str()) { return Err(RejectionReason::EventUnsafeTarget); }
    if state.civilizations.get(&actor).map_or(true, |civ| civ.cohesion < FRAGILE_COHESION) { return Err(RejectionReason::EventUnsafeTarget); }
    // Useful-response rule: some answer must be payable now (GDD 08).
    if !choices.iter().any(|choice| ops_affordable(state, actor, *target, &choice.effects, *related)) { return Err(RejectionReason::EventNoUsefulResponse); }

    let duration_turns = u32::from(*duration);
    apply_ops(state, actor, *target, ops, *related, turn + duration_turns);
    let entropy = &mut state.entropy;
    let id = entropy.next_event_id;
    entropy.next_event_id = entropy.next_event_id.saturating_add(1);
    entropy.world_spent += u32::from(*cost);
    *entropy.civ_spent.entry(actor).or_default() += u32::from(*cost);
    entropy.cooldowns.insert(template_id.clone(), turn + *cooldown);
    if *cost >= PROTECTION_MIN_COST {
        entropy.protections.push(Protection { civilization: actor, category: Some(category.clone()), min_cost: 0, until: turn + SAME_CATEGORY_PROTECTION_TURNS });
        entropy.protections.push(Protection { civilization: actor, category: None, min_cost: SEVERE_COST, until: turn + OTHER_CRISIS_PROTECTION_TURNS });
    }
    entropy.pending.insert(id, ActiveEvent {
        template_id: template_id.clone(), category: category.clone(), civilization: actor, target: *target, cost: *cost,
        started: turn, expires: turn + duration_turns, related: *related, choices: choices.clone(),
    });
    Ok(())
}

/// Applies the response of a player or Governor to a pending event.
pub(crate) fn respond_to_event(state: &mut WorldState, command: &AcceptedCommand) -> Result<(), RejectionReason> {
    let CommandPayload::RespondToEvent { event_id, choice_id } = &command.payload else { return Err(RejectionReason::KindPayloadMismatch); };
    let turn = state.turn.0;
    let event = state.entropy.pending.get(event_id).cloned().ok_or(RejectionReason::EventUnknown)?;
    if event.civilization != command.actor_id { return Err(RejectionReason::NotCommandOwner); }
    if turn > event.expires { return Err(RejectionReason::EventUnknown); }
    let choice = event.choices.iter().find(|choice| choice.id == *choice_id).ok_or(RejectionReason::EventChoiceInvalid)?;
    if !ops_affordable(state, command.actor_id, event.target, &choice.effects, event.related) { return Err(RejectionReason::EventUnaffordable); }
    apply_ops(state, command.actor_id, event.target, &choice.effects, event.related, turn + CHOICE_TAG_TURNS);
    state.entropy.pending.remove(event_id);
    Ok(())
}

fn personality_from_seed(seed: u64) -> Personality {
    match Rng::derive(seed, "entropy-personality").below(3) { 0 => Personality::Cyclical, 1 => Personality::Contentious, _ => Personality::Transformative }
}

/// End-of-turn bookkeeping: expiry, era rollover with fresh budgets, and the exposure `E` of active events.
pub(crate) fn resolve(state: &mut WorldState, seed: u64) {
    let turn = state.turn.0;
    let entropy = &mut state.entropy;
    entropy.pending.retain(|_, event| turn < event.expires);
    entropy.tags.retain(|mark| mark.until > turn);
    entropy.protections.retain(|protection| protection.until > turn);
    entropy.cooldowns.retain(|_, until| *until > turn);

    let next_era = turn.saturating_add(1) / ERA_TURNS;
    if state.entropy.era != Some(next_era) {
        let living: Vec<(CivId, u32)> = state.civilizations.iter().filter(|(_, civ)| !civ.frozen).map(|(id, civ)| (*id, u32::from(civ.crisis_pressure))).collect();
        let entropy = &mut state.entropy;
        if entropy.personality.is_none() { entropy.personality = Some(personality_from_seed(seed)); }
        entropy.era = Some(next_era);
        entropy.world_budget = BASE_WORLD_BUDGET + BUDGET_PER_LIVING_CIVILIZATION * living.len() as u32;
        entropy.world_spent = 0;
        entropy.civ_spent.clear();
        entropy.civ_reserve = living.into_iter().map(|(id, pressure)| (id, (RESERVE_BASE + pressure / RESERVE_PRESSURE_STEP).clamp(RESERVE_MIN, RESERVE_MAX))).collect();
    }

    let mut exposure: BTreeMap<CityId, u32> = BTreeMap::new();
    for event in state.entropy.pending.values().filter(|event| ENVIRONMENTAL_CATEGORIES.contains(&event.category.as_str())) {
        for (id, city) in &state.cities {
            let affected = match event.target {
                EntityRef::City(target) => *id == target,
                EntityRef::Tile(tile) => city.owner == event.civilization && city.workplaces.contains(&tile),
                EntityRef::Civilization(target) => city.owner == target,
                EntityRef::Route(_) => false,
            };
            if affected { *exposure.entry(*id).or_default() += u32::from(event.cost); }
        }
    }
    for (id, city) in &mut state.cities {
        city.environmental_exposure = exposure.get(id).copied().unwrap_or(0).min(MAX_ENVIRONMENTAL_EXPOSURE) as u8;
    }
}

/// One selectable (template, target) pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate { pub template: usize, pub target: EntityRef, pub actor: CivId, pub cost: u8, pub weight: u32 }

/// Proposes events. Without a decision port, or when it fails, the choice is a deterministic seeded T0 draw.
pub struct EntropyDirector<'a> { pub catalog: &'a EventCatalog, pub decision_port: Option<&'a dyn DecisionPort> }

impl EntropyDirector<'_> {
    fn candidate_id(&self, candidate: &Candidate) -> String { format!("{}@{:?}", self.catalog.templates[candidate.template].id, candidate.target) }

    /// Eligible candidates for the current turn, in canonical order (template id, then entity).
    /// `extra_world` and `extra_civ` account for events already proposed this turn.
    pub fn candidates(&self, state: &WorldState, extra_world: u32, extra_civ: &BTreeMap<CivId, u32>, used: &BTreeSet<CivId>, used_templates: &BTreeSet<usize>) -> Vec<Candidate> {
        let entropy = &state.entropy;
        let turn = state.turn.0;
        if entropy.era != Some(turn / ERA_TURNS) { return Vec::new(); }
        let personality = entropy.personality.unwrap_or(Personality::Cyclical);
        let position = era_position(turn);
        let cap = phase_cap(entropy.world_budget, position);
        let view = WorldView { state };
        let mut candidates = Vec::new();
        for (index, template) in self.catalog.templates.iter().enumerate() {
            if template.interference || EXCLUDED_CATEGORIES.contains(&template.category.as_str()) { continue; }
            if !phase_allows(position, &template.category, template.tension_cost) { continue; }
            if entropy.world_spent + extra_world + u32::from(template.tension_cost) > cap { continue; }
            if used_templates.contains(&index) || entropy.cooldowns.get(&template.id).is_some_and(|until| turn < *until) { continue; }
            let Ok(targets) = eligible_targets(template, &view) else { continue; };
            for target in targets {
                let Some(actor) = owner_of(state, target) else { continue; };
                if used.contains(&actor) || !self.civilization_can_receive(state, actor, template, extra_civ) { continue; }
                if !template.choices.iter().any(|choice| ops_affordable(state, actor, target, &choice.effects, view.related(target))) { continue; }
                candidates.push(Candidate { template: index, target, actor, cost: template.tension_cost, weight: personality.weight(&template.category) });
            }
        }
        candidates
    }

    fn civilization_can_receive(&self, state: &WorldState, actor: CivId, template: &Template, extra_civ: &BTreeMap<CivId, u32>) -> bool {
        let entropy = &state.entropy;
        let Some(civ) = state.civilizations.get(&actor) else { return false; };
        if civ.frozen || civ.cohesion < FRAGILE_COHESION { return false; }
        if is_protected(entropy, actor, &template.category, template.tension_cost, state.turn.0) { return false; }
        if entropy.pending.values().filter(|event| event.civilization == actor).count() >= MAX_PENDING_PER_CIVILIZATION { return false; }
        let spent = entropy.civ_spent.get(&actor).copied().unwrap_or(0) + extra_civ.get(&actor).copied().unwrap_or(0);
        spent + u32::from(template.tension_cost) <= entropy.civ_reserve.get(&actor).copied().unwrap_or(0)
    }

    fn pick(&self, state: &WorldState, candidates: &[Candidate], pick_index: usize) -> (usize, AiLayer) {
        if let Some(port) = self.decision_port {
            let choices: Vec<String> = candidates.iter().map(|candidate| self.candidate_id(candidate)).collect();
            let request_id = (u64::from(state.turn.0) << 8) | pick_index as u64;
            if let Ok(response) = port.choice(&ChoiceRequest { request_id, choices: choices.clone() }) {
                if let Some(index) = choices.iter().position(|id| *id == response.choice_id) { return (index, AiLayer::T1); }
            }
        }
        let mut rng = Rng::derive(state.seed, &format!("entropy-pick-{}-{}", state.turn.0, pick_index));
        let total: u32 = candidates.iter().map(|candidate| candidate.weight).sum();
        let mut roll = rng.below(total.max(1));
        for (index, candidate) in candidates.iter().enumerate() {
            if roll < candidate.weight { return (index, AiLayer::T0); }
            roll -= candidate.weight;
        }
        (candidates.len() - 1, AiLayer::T0)
    }

    /// Entropy commands for the current turn. Ids and sequences are assigned consecutively.
    pub fn propose(&self, state: &WorldState, command_id: u64, accepted_sequence: u64) -> Vec<AcceptedCommand> {
        let mut commands: Vec<AcceptedCommand> = Vec::new();
        let mut used = BTreeSet::new();
        let mut used_templates = BTreeSet::new();
        let mut extra_world = 0u32;
        let mut extra_civ: BTreeMap<CivId, u32> = BTreeMap::new();
        let view = WorldView { state };
        for pick_index in 0..MAX_EVENTS_PER_TURN {
            let candidates = self.candidates(state, extra_world, &extra_civ, &used, &used_templates);
            if candidates.is_empty() { break; }
            let (index, layer) = self.pick(state, &candidates, pick_index);
            let chosen = &candidates[index];
            let template = &self.catalog.templates[chosen.template];
            let offset = commands.len() as u64;
            let id = command_id.saturating_add(offset);
            let mut grounding = vec![GroundingRef::Civilization { civilization: chosen.actor }];
            match chosen.target {
                EntityRef::City(city) => grounding.push(GroundingRef::City { city }),
                EntityRef::Tile(tile) => grounding.push(GroundingRef::Tile { tile }),
                _ => {}
            }
            grounding.push(GroundingRef::Turn { turn: state.turn });
            commands.push(AcceptedCommand {
                command_id: id, world_id: state.world_id, turn: state.turn, accepted_sequence: accepted_sequence.saturating_add(offset), actor_id: chosen.actor,
                origin: CommandOrigin::Entropy, kind: CommandKind::ApplyEvent,
                payload: CommandPayload::ApplyEvent {
                    template_id: template.id.clone(), catalog_hash: self.catalog.content_hash, category: template.category.clone(), cost: template.tension_cost,
                    duration: template.duration_turns, cooldown: template.cooldown_turns, target: chosen.target, related: view.related(chosen.target),
                    ops: template.effects.clone(), choices: template.choices.clone(),
                },
                grounding,
                intent_evidence: Some(IntentEvidence { request_id: id, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(), layer, status: AiStatus::Ok, fixture_id: None, fallback_from: None }),
                mandate: None,
            });
            extra_world += u32::from(chosen.cost);
            *extra_civ.entry(chosen.actor).or_default() += u32::from(chosen.cost);
            used.insert(chosen.actor);
            used_templates.insert(chosen.template);
        }
        commands
    }
}

fn response_cost(choice: &EventChoice) -> u32 {
    choice.effects.iter().map(|op| match op { EffectOp::AdjustResource { amount, .. } if *amount < 0 => amount.unsigned_abs(), _ => 0 }).sum()
}

/// T0 answer to pending events of `civilization`: the payable choice with the lowest cost, ties by listed order.
/// Commands go through `IntentValidator` like any Governor intent, so the Mandate still applies.
pub fn respond_commands(state: &WorldState, civilization: CivId, mandate: &Mandate, command_id: u64, accepted_sequence: u64) -> Vec<AcceptedCommand> {
    let mut commands = Vec::new();
    for (event_id, event) in &state.entropy.pending {
        if event.civilization != civilization || state.turn.0 > event.expires { continue; }
        let best = event.choices.iter().enumerate()
            .filter(|(_, choice)| ops_affordable(state, civilization, event.target, &choice.effects, event.related))
            .min_by_key(|(index, choice)| (response_cost(choice), *index));
        let Some((_, choice)) = best else { continue; };
        let offset = commands.len() as u64;
        let id = command_id.saturating_add(offset);
        let intent = ActionIntent {
            request_id: id, actor_id: civilization, turn: state.turn, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(),
            kind: CommandKind::RespondToEvent, parameters: CommandPayload::RespondToEvent { event_id: *event_id, choice_id: choice.id.clone() },
            grounding: vec![GroundingRef::Civilization { civilization }, GroundingRef::Turn { turn: state.turn }],
        };
        let evidence = IntentEvidence { request_id: id, schema_version: ACTION_INTENT_SCHEMA_VERSION, ruleset_ref: state.ruleset.clone(), layer: AiLayer::T0, status: AiStatus::Ok, fixture_id: None, fallback_from: None };
        if let Ok(command) = IntentValidator::accept(state, intent, CommandOrigin::Governor, Some(mandate), id, accepted_sequence.saturating_add(offset), evidence) { commands.push(command); }
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ids::{TileIndex, TurnNumber},
        world::{step, CityFocus, CityGroup, CivilizationState, GroupFunction, RulesetRef, SimulationVersions, TileState, TileYields, WorldId},
    };

    fn versions(state: &WorldState) -> SimulationVersions { SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 } }

    /// Two civilizations with one city each; civilization 1 works a plains tile that yields food.
    fn world(turn: u32) -> WorldState {
        let yields = TileYields { food: 2, production: 1, wealth: 1, knowledge: 0, culture: 0 };
        let mut state = WorldState {
            world_id: WorldId(1), turn: TurnNumber(turn), seed: 7, ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 1 }, schema_version: 1, map_width: 2,
            tiles: vec![TileState { terrain: TERRAIN_PLAINS, river: false, yields }; 4],
            civilizations: BTreeMap::from([(CivId(1), CivilizationState::default()), (CivId(2), CivilizationState::default())]),
            cities: BTreeMap::new(), units: BTreeMap::new(), attacks: BTreeMap::new(), control: BTreeMap::new(), visibility: BTreeMap::new(), ledger: Vec::new(), entropy: EntropyState::default(),
        };
        for (id, owner, tile) in [(1, 1, 0), (2, 2, 3)] {
            state.cities.insert(CityId(id), CityState {
                owner: CivId(owner), tile: TileIndex(tile), focus: CityFocus::Supply, population: 2, housing: 4, food_stock: 6, growth_progress: 0, consecutive_food_shortages: 0, stability: 60,
                groups: vec![CityGroup { function: GroupFunction::Cultivators, population: 2, satisfaction: 60 }, CityGroup { function: GroupFunction::Crafts, population: 0, satisfaction: 50 }, CityGroup { function: GroupFunction::Merchants, population: 0, satisfaction: 50 }],
                workplaces: vec![TileIndex(tile)], last_yields: yields, deprivation: 0, group_tension: 0, war_threat: 0, environmental_exposure: 0, crisis_pressure: 0, crisis_turns: 0,
                essential_maintenance_unpaid: false, unit_queue: Vec::new(), unit_production: 0,
            });
        }
        state.entropy.era = Some(turn / ERA_TURNS);
        state.entropy.personality = Some(Personality::Cyclical);
        state.entropy.world_budget = 16;
        state.entropy.civ_reserve = BTreeMap::from([(CivId(1), 6), (CivId(2), 6)]);
        state
    }

    fn frost(catalog: &EventCatalog, state: &WorldState, id: u64) -> AcceptedCommand {
        let director = EntropyDirector { catalog, decision_port: None };
        let mut commands = director.propose(state, id, 1);
        assert!(!commands.is_empty(), "frost is eligible on a plains city with a farmed tile");
        commands.remove(0)
    }

    fn applied(result: &crate::world::StepResult, id: u64) -> bool {
        result.events.iter().any(|event| matches!(event, crate::world::DomainEvent::CommandApplied { command_id } if *command_id == id))
    }

    #[test]
    fn rollover_sets_the_world_budget_and_clamped_reserves() {
        let mut state = world(0);
        state.entropy = EntropyState::default();
        state.civilizations.get_mut(&CivId(2)).unwrap().crisis_pressure = 99;
        resolve(&mut state, 7);
        assert_eq!(state.entropy.world_budget, BASE_WORLD_BUDGET + 2 * 2);
        assert_eq!(state.entropy.civ_reserve[&CivId(1)], 2);
        assert_eq!(state.entropy.civ_reserve[&CivId(2)], 5);
        assert_eq!(state.entropy.era, Some(0));
        assert!(state.entropy.personality.is_some());
    }

    #[test]
    fn director_proposes_a_valid_entropy_command_and_the_engine_applies_it() {
        let catalog = bundled_catalog();
        let state = world(3);
        let command = frost(&catalog, &state, 10);
        assert_eq!(command.origin, CommandOrigin::Entropy);
        assert!(!command.grounding.is_empty());
        let result = step(&state, &[command.clone()], state.seed, &versions(&state));
        assert!(applied(&result, 10), "{:?}", result.events);
        assert_eq!(result.state.entropy.pending.len(), 1);
        assert_eq!(result.state.entropy.world_spent, u32::from(match &command.payload { CommandPayload::ApplyEvent { cost, .. } => *cost, _ => 0 }));
    }

    #[test]
    fn same_state_gives_the_same_proposals() {
        let catalog = bundled_catalog();
        let state = world(3);
        let director = EntropyDirector { catalog: &catalog, decision_port: None };
        assert_eq!(director.propose(&state, 1, 1), director.propose(&state, 1, 1));
    }

    #[test]
    fn cooldown_blocks_the_same_template() {
        let catalog = bundled_catalog();
        let state = world(3);
        let command = frost(&catalog, &state, 10);
        let first = step(&state, &[command.clone()], state.seed, &versions(&state));
        assert!(applied(&first, 10));
        let mut again = first.state.clone();
        again.entropy.pending.clear();
        again.entropy.protections.clear();
        let mut repeat = command;
        repeat.command_id = 11;
        repeat.turn = again.turn;
        let second = step(&again, &[repeat], again.seed, &versions(&again));
        assert!(second.events.iter().any(|event| matches!(event, crate::world::DomainEvent::CommandRejected { command_id: 11, reason: RejectionReason::EventOnCooldown })), "{:?}", second.events);
    }

    #[test]
    fn engine_rejects_forged_over_budget_and_unsafe_events() {
        let catalog = bundled_catalog();
        let state = world(3);
        let command = frost(&catalog, &state, 10);
        let reject = |state: &WorldState, command: AcceptedCommand, expected: RejectionReason| {
            let id = command.command_id;
            let result = step(state, &[command], state.seed, &versions(state));
            assert!(result.events.iter().any(|event| matches!(event, crate::world::DomainEvent::CommandRejected { command_id, reason } if *command_id == id && *reason == expected)), "{:?}", result.events);
        };
        let mut forged = command.clone();
        forged.origin = CommandOrigin::Player;
        reject(&state, forged, RejectionReason::EventForbidden);
        let mut broke = state.clone();
        broke.entropy.world_budget = 0;
        reject(&broke, command.clone(), RejectionReason::EventOverBudget);
        let mut fragile = state.clone();
        fragile.civilizations.get_mut(&command.actor_id).unwrap().cohesion = FRAGILE_COHESION - 1;
        reject(&fragile, command.clone(), RejectionReason::EventUnsafeTarget);
        let mut wrong_owner = command.clone();
        wrong_owner.actor_id = if command.actor_id == CivId(1) { CivId(2) } else { CivId(1) };
        reject(&state, wrong_owner, RejectionReason::EventInvalid);
    }

    #[test]
    fn respite_never_starts_a_grave_event() {
        assert!(!phase_allows(7, "climate", 2));
        assert!(!phase_allows(7, "discovery", 3));
        assert!(phase_allows(7, "discovery", 2));
        assert!(!phase_allows(0, "climate", 2));
        assert!(phase_allows(3, "climate", 3));
        let catalog = bundled_catalog();
        let director = EntropyDirector { catalog: &catalog, decision_port: None };
        let state = world(7);
        for candidate in director.candidates(&state, 0, &BTreeMap::new(), &BTreeSet::new(), &BTreeSet::new()) {
            let template = &catalog.templates[candidate.template];
            assert!(template.tension_cost <= 2 && RESPITE_CATEGORIES.contains(&template.category.as_str()));
        }
    }

    #[test]
    fn response_applies_the_cheapest_choice_and_clears_the_event() {
        let catalog = bundled_catalog();
        let state = world(3);
        let command = frost(&catalog, &state, 10);
        let first = step(&state, &[command], state.seed, &versions(&state));
        let mandate = Mandate::balanced(1);
        let actor = *first.state.entropy.pending.values().next().map(|event| &event.civilization).unwrap();
        let responses = respond_commands(&first.state, actor, &mandate, 20, 1);
        assert_eq!(responses.len(), 1);
        let second = step(&first.state, &responses, first.state.seed, &versions(&first.state));
        assert!(applied(&second, 20), "{:?}", second.events);
        assert!(second.state.entropy.pending.is_empty());
    }

    #[test]
    fn unanswered_events_expire_and_stop_adding_exposure() {
        let catalog = bundled_catalog();
        let state = world(3);
        let command = frost(&catalog, &state, 10);
        let mut current = step(&state, &[command], state.seed, &versions(&state)).state;
        assert!(current.cities.values().any(|city| city.environmental_exposure > 0));
        for _ in 0..4 { current = step(&current, &[], current.seed, &versions(&current)).state; }
        assert!(current.entropy.pending.is_empty());
        assert!(current.cities.values().all(|city| city.environmental_exposure == 0));
    }

    #[test]
    fn no_template_effect_can_remove_a_city_move_cohesion_or_freeze_a_civilization() {
        let catalog = bundled_catalog();
        for template in &catalog.templates {
            let mut state = world(3);
            let before = state.clone();
            let target = EntityRef::City(CityId(1));
            let mut all_ops: Vec<EffectOp> = template.effects.clone();
            for choice in &template.choices { all_ops.extend(choice.effects.clone()); }
            assert!(ops_are_bounded(&all_ops), "{}", template.id);
            for op in all_ops.chunks(1) { apply_ops(&mut state, CivId(1), target, op, Some(CivId(2)), 99); }
            assert_eq!(state.cities.len(), before.cities.len(), "{}", template.id);
            for (id, civ) in &state.civilizations {
                assert_eq!(civ.cohesion, before.civilizations[id].cohesion, "{}", template.id);
                assert_eq!(civ.legitimacy, before.civilizations[id].legitimacy, "{}", template.id);
                assert!(!civ.frozen, "{}", template.id);
            }
            assert!(state.cities.values().all(|city| city.population > 0), "{}", template.id);
        }
    }

    #[test]
    fn interference_and_collapse_templates_are_never_proposed() {
        let catalog = bundled_catalog();
        let director = EntropyDirector { catalog: &catalog, decision_port: None };
        let state = world(3);
        for candidate in director.candidates(&state, 0, &BTreeMap::new(), &BTreeSet::new(), &BTreeSet::new()) {
            let template = &catalog.templates[candidate.template];
            assert!(!template.interference && template.category != "collapse", "{}", template.id);
        }
    }

    #[test]
    fn personality_weights_double_the_preferred_categories_only() {
        assert_eq!(Personality::Cyclical.weight("climate"), 2);
        assert_eq!(Personality::Cyclical.weight("diplomacy"), 1);
        assert_eq!(Personality::Contentious.weight("diplomacy"), 2);
        assert_eq!(Personality::Transformative.weight("discovery"), 2);
    }
}
