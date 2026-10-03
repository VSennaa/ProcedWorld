//! Pure interpreter for the closed catalog DSL (docs/sdd/18-dsl-catalogos.md).
//!
//! The grammar is a closed set of enums. `EventCatalog::parse` validates strictly and fails closed:
//! unknown keys, unknown operations or facts, out-of-range numbers and oversized templates produce
//! diagnostics and no catalog. Evaluation reads a `StateView`, never mutates it, performs no I/O and
//! contains no randomness; effects are returned as typed values for the engine to validate and apply.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    hash::fnv1a,
    ids::{CityId, CivId, TileIndex},
};

/// Provisional limits (SDD 18 section 4), to be calibrated by the harness.
pub const MAX_PREDICATE_DEPTH: usize = 16;
pub const MAX_PREDICATE_NODES: u32 = 256;
pub const MAX_OPERATIONS: usize = 64;
pub const MAX_TARGETS: usize = 1024;
pub const MAX_TEMPLATE_BYTES: usize = 32 * 1024;
pub const MAX_ADJUST_AMOUNT: i32 = 8;
pub const MAX_TENSION_COST: u8 = 4;
pub const MAX_DURATION_TURNS: u8 = 6;
pub const MAX_COOLDOWN_TURNS: u32 = 24;
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind { Civilization, City, Tile, Route }

impl TargetKind {
    fn parse(text: &str) -> Option<Self> {
        match text { "civilization" => Some(Self::Civilization), "city" => Some(Self::City), "tile" => Some(Self::Tile), "route" => Some(Self::Route), _ => None }
    }
}

/// A concrete entity. There is no abstract target (GDD 08).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "id")]
pub enum EntityRef { Civilization(CivId), City(CityId), Tile(TileIndex), Route(u32) }

impl EntityRef {
    pub const fn kind(self) -> TargetKind {
        match self { Self::Civilization(_) => TargetKind::Civilization, Self::City(_) => TargetKind::City, Self::Tile(_) => TargetKind::Tile, Self::Route(_) => TargetKind::Route }
    }
}

/// Closed vocabulary of queryable facts. A fact the engine cannot supply evaluates to `UnknownFact`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fact {
    CityExposedFarms, CityDeprivation, CityProduction, CityGroupTension, CityDensity, CitySanitaryStock,
    CivCohesion, CivCulture, CivKnowledge, CivRecentDeprivation, CivCrisisPressure, RouteActive,
}

impl Fact {
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "city.exposed_farms" => Self::CityExposedFarms, "city.deprivation" => Self::CityDeprivation,
            "city.production" => Self::CityProduction, "city.group_tension" => Self::CityGroupTension,
            "city.density" => Self::CityDensity, "city.sanitary_stock" => Self::CitySanitaryStock,
            "civilization.cohesion" => Self::CivCohesion, "civilization.culture" => Self::CivCulture,
            "civilization.knowledge" => Self::CivKnowledge, "civilization.recent_deprivation" => Self::CivRecentDeprivation,
            "civilization.crisis_pressure" => Self::CivCrisisPressure, "route.active" => Self::RouteActive,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparator { Eq, Ne, Gt, Gte, Lt, Lte }

impl Comparator {
    fn parse(text: &str) -> Option<Self> {
        Some(match text { "eq" => Self::Eq, "ne" => Self::Ne, "gt" => Self::Gt, "gte" => Self::Gte, "lt" => Self::Lt, "lte" => Self::Lte, _ => return None })
    }
    pub const fn holds(self, actual: i64, expected: i64) -> bool {
        match self { Self::Eq => actual == expected, Self::Ne => actual != expected, Self::Gt => actual > expected, Self::Gte => actual >= expected, Self::Lt => actual < expected, Self::Lte => actual <= expected }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Predicate {
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    Not(Box<Predicate>),
    Compare { fact: Fact, comparator: Comparator, value: i64 },
    /// Tag test on the selected entity.
    HasTag(String),
    Exists(TargetKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource { Food, Production, Wealth, Knowledge, Culture }

impl Resource {
    fn parse(text: &str) -> Option<Self> {
        Some(match text { "food" => Self::Food, "production" => Self::Production, "wealth" => Self::Wealth, "knowledge" => Self::Knowledge, "culture" => Self::Culture, _ => return None })
    }
}

/// A validated effect. The engine still checks pre-conditions and limits when it applies one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum EffectOp {
    AdjustResource { resource: Resource, amount: i32 },
    AddTag { tag: String },
    RemoveTag { tag: String },
    CreateLedgerEntry { term: String, duration: u32 },
    EmitChronicle { template: String, parameters: BTreeMap<String, i64> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventChoice { pub id: String, pub effects: Vec<EffectOp> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub id: String,
    pub category: String,
    pub tension_cost: u8,
    pub duration_turns: u8,
    pub cooldown_turns: u32,
    pub when: Predicate,
    pub target: TargetKind,
    pub effects: Vec<EffectOp>,
    pub choices: Vec<EventChoice>,
    /// Player-initiated interference (predict, appease, divert). The director never selects these.
    pub interference: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventCatalog {
    pub catalog_id: String,
    pub catalog_version: u32,
    /// FNV-1a of the source bytes. A recorded command carries it for audit.
    pub content_hash: u64,
    /// Sorted by id, so iteration order never depends on the source file layout.
    pub templates: Vec<Template>,
}

impl EventCatalog {
    pub fn get(&self, id: &str) -> Option<&Template> { self.templates.iter().find(|template| template.id == id) }

    pub fn parse(raw: &str) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let value: Value = match serde_json::from_str(raw) {
            Ok(value) => value,
            Err(_) => return Err(vec![Diagnostic { code: "syntax", path: "$".into() }]),
        };
        let Some(root) = object(&value, "$", &mut diagnostics) else { return Err(diagnostics); };
        check_keys(root, &["catalog_id", "catalog_version", "schema_version", "status", "entropy_rules", "templates"], "$", &mut diagnostics);
        let catalog_id = text(root, "catalog_id", "$", &mut diagnostics).unwrap_or_default();
        let catalog_version = integer(root, "catalog_version", 0, i64::from(u32::MAX), "$", &mut diagnostics).unwrap_or(0) as u32;
        if integer(root, "schema_version", 0, 1_000, "$", &mut diagnostics) != Some(i64::from(SUPPORTED_SCHEMA_VERSION)) {
            diagnostics.push(Diagnostic { code: "unsupported_schema_version", path: "$.schema_version".into() });
        }
        let mut templates = Vec::new();
        match root.get("templates").and_then(Value::as_array) {
            Some(items) => {
                for (index, item) in items.iter().enumerate() {
                    let path = format!("$.templates[{index}]");
                    if item.to_string().len() > MAX_TEMPLATE_BYTES { diagnostics.push(Diagnostic { code: "template_too_large", path: path.clone() }); continue; }
                    if let Some(template) = parse_template(item, &path, &mut diagnostics) { templates.push(template); }
                }
            }
            None => diagnostics.push(Diagnostic { code: "missing_templates", path: "$.templates".into() }),
        }
        templates.sort_by(|left, right| left.id.cmp(&right.id));
        for pair in templates.windows(2) {
            if pair[0].id == pair[1].id { diagnostics.push(Diagnostic { code: "duplicate_template_id", path: pair[0].id.clone() }); }
        }
        if diagnostics.is_empty() { Ok(Self { catalog_id, catalog_version, content_hash: fnv1a(raw.as_bytes()), templates }) } else { Err(diagnostics) }
    }
}

fn object<'a>(value: &'a Value, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<&'a Map<String, Value>> {
    let map = value.as_object();
    if map.is_none() { diagnostics.push(Diagnostic { code: "expected_object", path: path.into() }); }
    map
}

fn check_keys(map: &Map<String, Value>, allowed: &[&str], path: &str, diagnostics: &mut Vec<Diagnostic>) {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) { diagnostics.push(Diagnostic { code: "unknown_key", path: format!("{path}.{key}") }); }
    }
}

fn text(map: &Map<String, Value>, key: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    match map.get(key).and_then(Value::as_str) {
        Some(value) if !value.is_empty() && value.len() <= 96 => Some(value.to_owned()),
        _ => { diagnostics.push(Diagnostic { code: "expected_text", path: format!("{path}.{key}") }); None }
    }
}

fn integer(map: &Map<String, Value>, key: &str, min: i64, max: i64, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<i64> {
    match map.get(key).and_then(Value::as_i64) {
        Some(value) if value >= min && value <= max => Some(value),
        _ => { diagnostics.push(Diagnostic { code: "expected_integer_in_range", path: format!("{path}.{key}") }); None }
    }
}

fn identifier(map: &Map<String, Value>, key: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    let value = text(map, key, path, diagnostics)?;
    if value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'_') { Some(value) } else {
        diagnostics.push(Diagnostic { code: "invalid_identifier", path: format!("{path}.{key}") });
        None
    }
}

fn parse_template(value: &Value, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<Template> {
    let map = object(value, path, diagnostics)?;
    check_keys(map, &["id", "name", "category", "reference_event", "tension_cost", "duration_turns", "when", "targets", "effects", "choices", "interference", "cooldown_turns"], path, diagnostics);
    let before = diagnostics.len();
    let id = identifier(map, "id", path, diagnostics);
    let category = identifier(map, "category", path, diagnostics);
    let tension_cost = integer(map, "tension_cost", 1, i64::from(MAX_TENSION_COST), path, diagnostics);
    let duration_turns = integer(map, "duration_turns", 1, i64::from(MAX_DURATION_TURNS), path, diagnostics);
    let cooldown_turns = integer(map, "cooldown_turns", 0, i64::from(MAX_COOLDOWN_TURNS), path, diagnostics);
    let mut nodes = 0u32;
    let when = map.get("when").and_then(|item| parse_predicate(item, &format!("{path}.when"), 1, &mut nodes, diagnostics));
    if when.is_none() && diagnostics.len() == before { diagnostics.push(Diagnostic { code: "missing_when", path: format!("{path}.when") }); }
    let target = match map.get("targets").and_then(Value::as_object) {
        Some(targets) => {
            check_keys(targets, &["kind", "scope"], &format!("{path}.targets"), diagnostics);
            if targets.get("scope").and_then(Value::as_str) != Some("eligible") { diagnostics.push(Diagnostic { code: "unsupported_scope", path: format!("{path}.targets.scope") }); }
            match targets.get("kind").and_then(Value::as_str).and_then(TargetKind::parse) {
                Some(kind) => Some(kind),
                None => { diagnostics.push(Diagnostic { code: "unknown_target_kind", path: format!("{path}.targets.kind") }); None }
            }
        }
        None => { diagnostics.push(Diagnostic { code: "missing_targets", path: format!("{path}.targets") }); None }
    };
    let effects = parse_effects(map.get("effects"), &format!("{path}.effects"), diagnostics);
    let mut choices = Vec::new();
    match map.get("choices").and_then(Value::as_array) {
        Some(items) if (2..=3).contains(&items.len()) => {
            for (index, item) in items.iter().enumerate() {
                let choice_path = format!("{path}.choices[{index}]");
                let Some(choice) = object(item, &choice_path, diagnostics) else { continue; };
                check_keys(choice, &["id", "effects"], &choice_path, diagnostics);
                let choice_id = identifier(choice, "id", &choice_path, diagnostics);
                let choice_effects = parse_effects(choice.get("effects"), &format!("{choice_path}.effects"), diagnostics);
                if let (Some(choice_id), Some(choice_effects)) = (choice_id, choice_effects) { choices.push(EventChoice { id: choice_id, effects: choice_effects }); }
            }
            for (index, choice) in choices.iter().enumerate() {
                if choices[..index].iter().any(|other| other.id == choice.id) { diagnostics.push(Diagnostic { code: "duplicate_choice_id", path: format!("{path}.choices[{index}]") }); }
            }
        }
        _ => diagnostics.push(Diagnostic { code: "choices_must_be_two_or_three", path: format!("{path}.choices") }),
    }
    let interference = match map.get("interference") {
        None => false,
        Some(item) if item.is_object() => true,
        Some(_) => { diagnostics.push(Diagnostic { code: "expected_object", path: format!("{path}.interference") }); false }
    };
    if diagnostics.len() != before { return None; }
    Some(Template {
        id: id?, category: category?, tension_cost: tension_cost? as u8, duration_turns: duration_turns? as u8, cooldown_turns: cooldown_turns? as u32,
        when: when?, target: target?, effects: effects?, choices, interference,
    })
}

fn parse_predicate(value: &Value, path: &str, depth: usize, nodes: &mut u32, diagnostics: &mut Vec<Diagnostic>) -> Option<Predicate> {
    *nodes += 1;
    if depth > MAX_PREDICATE_DEPTH || *nodes > MAX_PREDICATE_NODES {
        diagnostics.push(Diagnostic { code: "predicate_too_large", path: path.into() });
        return None;
    }
    let map = object(value, path, diagnostics)?;
    let op = map.get("op").and_then(Value::as_str).unwrap_or("");
    match op {
        "all" | "any" => {
            check_keys(map, &["op", "args"], path, diagnostics);
            let Some(items) = map.get("args").and_then(Value::as_array) else { diagnostics.push(Diagnostic { code: "expected_args", path: format!("{path}.args") }); return None; };
            let mut args = Vec::new();
            let mut valid = true;
            for (index, item) in items.iter().enumerate() {
                match parse_predicate(item, &format!("{path}.args[{index}]"), depth + 1, nodes, diagnostics) { Some(arg) => args.push(arg), None => valid = false }
            }
            if !valid { return None; }
            Some(if op == "all" { Predicate::All(args) } else { Predicate::Any(args) })
        }
        "not" => {
            check_keys(map, &["op", "arg"], path, diagnostics);
            let arg = map.get("arg").and_then(|item| parse_predicate(item, &format!("{path}.arg"), depth + 1, nodes, diagnostics))?;
            Some(Predicate::Not(Box::new(arg)))
        }
        "compare" => {
            check_keys(map, &["op", "fact", "comparator", "value"], path, diagnostics);
            let fact = map.get("fact").and_then(Value::as_str).and_then(Fact::parse);
            let comparator = map.get("comparator").and_then(Value::as_str).and_then(Comparator::parse);
            let value = map.get("value").and_then(Value::as_i64).filter(|number| number.abs() <= 1_000_000);
            if fact.is_none() { diagnostics.push(Diagnostic { code: "unknown_fact", path: format!("{path}.fact") }); }
            if comparator.is_none() { diagnostics.push(Diagnostic { code: "unknown_comparator", path: format!("{path}.comparator") }); }
            if value.is_none() { diagnostics.push(Diagnostic { code: "expected_integer_literal", path: format!("{path}.value") }); }
            Some(Predicate::Compare { fact: fact?, comparator: comparator?, value: value? })
        }
        "has_tag" => {
            check_keys(map, &["op", "entity", "tag_id"], path, diagnostics);
            if map.get("entity").and_then(Value::as_str) != Some("selected") { diagnostics.push(Diagnostic { code: "unsupported_entity", path: format!("{path}.entity") }); return None; }
            Some(Predicate::HasTag(identifier(map, "tag_id", path, diagnostics)?))
        }
        "exists" => {
            check_keys(map, &["op", "kind"], path, diagnostics);
            match map.get("kind").and_then(Value::as_str).and_then(TargetKind::parse) {
                Some(kind) => Some(Predicate::Exists(kind)),
                None => { diagnostics.push(Diagnostic { code: "unknown_target_kind", path: format!("{path}.kind") }); None }
            }
        }
        _ => { diagnostics.push(Diagnostic { code: "unknown_op", path: format!("{path}.op") }); None }
    }
}

fn parse_effects(value: Option<&Value>, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<Vec<EffectOp>> {
    let Some(items) = value.and_then(Value::as_array) else { diagnostics.push(Diagnostic { code: "expected_array", path: path.into() }); return None; };
    if items.len() > MAX_OPERATIONS { diagnostics.push(Diagnostic { code: "too_many_operations", path: path.into() }); return None; }
    let mut effects = Vec::new();
    let mut valid = true;
    for (index, item) in items.iter().enumerate() {
        match parse_effect(item, &format!("{path}[{index}]"), diagnostics) { Some(effect) => effects.push(effect), None => valid = false }
    }
    // Operations that touch the same field conflict; the validator rejects them (SDD 18 section 4).
    for (index, effect) in effects.iter().enumerate() {
        let conflict = effects[..index].iter().any(|other| match (other, effect) {
            (EffectOp::AdjustResource { resource: left, .. }, EffectOp::AdjustResource { resource: right, .. }) => left == right,
            (EffectOp::AddTag { tag: left }, EffectOp::AddTag { tag: right }) => left == right,
            (EffectOp::AddTag { tag: left }, EffectOp::RemoveTag { tag: right }) | (EffectOp::RemoveTag { tag: left }, EffectOp::AddTag { tag: right }) => left == right,
            _ => false,
        });
        if conflict { diagnostics.push(Diagnostic { code: "conflicting_operations", path: format!("{path}[{index}]") }); valid = false; }
    }
    valid.then_some(effects)
}

fn parse_effect(value: &Value, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<EffectOp> {
    let map = object(value, path, diagnostics)?;
    let op = map.get("op").and_then(Value::as_str).unwrap_or("");
    let selected = |key: &str, diagnostics: &mut Vec<Diagnostic>| {
        let ok = map.get(key).and_then(Value::as_str) == Some("selected");
        if !ok { diagnostics.push(Diagnostic { code: "unsupported_target_ref", path: format!("{path}.{key}") }); }
        ok
    };
    match op {
        "AdjustResource" => {
            check_keys(map, &["op", "target", "resource_id", "amount"], path, diagnostics);
            let target_ok = selected("target", diagnostics);
            let resource = map.get("resource_id").and_then(Value::as_str).and_then(Resource::parse);
            if resource.is_none() { diagnostics.push(Diagnostic { code: "unknown_resource", path: format!("{path}.resource_id") }); }
            let amount = integer(map, "amount", -i64::from(MAX_ADJUST_AMOUNT), i64::from(MAX_ADJUST_AMOUNT), path, diagnostics);
            if !target_ok { return None; }
            Some(EffectOp::AdjustResource { resource: resource?, amount: amount? as i32 })
        }
        "AddTag" | "RemoveTag" => {
            check_keys(map, &["op", "target", "tag_id"], path, diagnostics);
            let target_ok = selected("target", diagnostics);
            let tag = identifier(map, "tag_id", path, diagnostics);
            if !target_ok { return None; }
            Some(if op == "AddTag" { EffectOp::AddTag { tag: tag? } } else { EffectOp::RemoveTag { tag: tag? } })
        }
        "CreateLedgerEntry" => {
            check_keys(map, &["op", "from", "to", "term_id", "duration"], path, diagnostics);
            let from_ok = map.get("from").and_then(Value::as_str) == Some("self");
            let to_ok = map.get("to").and_then(Value::as_str) == Some("related");
            if !from_ok || !to_ok { diagnostics.push(Diagnostic { code: "unsupported_party_ref", path: path.into() }); }
            let term = identifier(map, "term_id", path, diagnostics);
            let duration = integer(map, "duration", 1, i64::from(MAX_DURATION_TURNS), path, diagnostics);
            if !from_ok || !to_ok { return None; }
            Some(EffectOp::CreateLedgerEntry { term: term?, duration: duration? as u32 })
        }
        "EmitChronicle" => {
            check_keys(map, &["op", "template_id", "parameters"], path, diagnostics);
            let template = identifier(map, "template_id", path, diagnostics);
            let mut parameters = BTreeMap::new();
            match map.get("parameters").and_then(Value::as_object) {
                Some(items) if items.len() <= 8 => {
                    for (key, item) in items {
                        match item.as_i64().filter(|number| number.abs() <= 1_000_000) {
                            Some(number) => { parameters.insert(key.clone(), number); }
                            None => diagnostics.push(Diagnostic { code: "expected_integer_literal", path: format!("{path}.parameters.{key}") }),
                        }
                    }
                }
                _ => diagnostics.push(Diagnostic { code: "invalid_parameters", path: format!("{path}.parameters") }),
            }
            Some(EffectOp::EmitChronicle { template: template?, parameters })
        }
        _ => { diagnostics.push(Diagnostic { code: "unknown_op", path: format!("{path}.op") }); None }
    }
}

/// Read-only access to simulation facts. Implementations must be deterministic and side-effect free.
pub trait StateView {
    /// Entities of one kind. The interpreter sorts and deduplicates the result.
    fn entities(&self, kind: TargetKind) -> Vec<EntityRef>;
    /// `None` means the engine cannot supply this fact for this entity (never zero or false).
    fn fact(&self, entity: EntityRef, fact: Fact) -> Option<i64>;
    fn has_tag(&self, entity: EntityRef, tag: &str) -> bool;
    /// The counterparty civilization of a diplomatic effect, if one is traceable.
    fn related(&self, entity: EntityRef) -> Option<CivId>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalError { UnknownFact(Fact), BudgetExceeded, TooManyTargets }

/// Evaluates a predicate against one selected entity. `All`/`Any` short-circuit in canonical order.
pub fn evaluate(predicate: &Predicate, view: &dyn StateView, selected: EntityRef) -> Result<bool, EvalError> {
    let mut budget = MAX_PREDICATE_NODES;
    evaluate_node(predicate, view, selected, &mut budget)
}

fn evaluate_node(predicate: &Predicate, view: &dyn StateView, selected: EntityRef, budget: &mut u32) -> Result<bool, EvalError> {
    if *budget == 0 { return Err(EvalError::BudgetExceeded); }
    *budget -= 1;
    match predicate {
        Predicate::All(args) => { for arg in args { if !evaluate_node(arg, view, selected, budget)? { return Ok(false); } } Ok(true) }
        Predicate::Any(args) => { for arg in args { if evaluate_node(arg, view, selected, budget)? { return Ok(true); } } Ok(false) }
        Predicate::Not(arg) => Ok(!evaluate_node(arg, view, selected, budget)?),
        Predicate::Compare { fact, comparator, value } => {
            let actual = view.fact(selected, *fact).ok_or(EvalError::UnknownFact(*fact))?;
            Ok(comparator.holds(actual, *value))
        }
        Predicate::HasTag(tag) => Ok(view.has_tag(selected, tag)),
        Predicate::Exists(kind) => Ok(!view.entities(*kind).is_empty()),
    }
}

/// Entities of the template's kind that satisfy `when`, sorted and deduplicated. A selector that
/// exceeds its limit fails as a whole instead of truncating silently.
pub fn eligible_targets(template: &Template, view: &dyn StateView) -> Result<Vec<EntityRef>, EvalError> {
    let mut candidates = view.entities(template.target);
    candidates.sort_unstable();
    candidates.dedup();
    if candidates.len() > MAX_TARGETS { return Err(EvalError::TooManyTargets); }
    let mut eligible = Vec::new();
    for entity in candidates { if evaluate(&template.when, view, entity)? { eligible.push(entity); } }
    Ok(eligible)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture { cities: Vec<(u32, i64, &'static [&'static str])> }
    impl StateView for Fixture {
        fn entities(&self, kind: TargetKind) -> Vec<EntityRef> {
            if kind == TargetKind::City { self.cities.iter().rev().map(|city| EntityRef::City(CityId(city.0))).collect() } else { Vec::new() }
        }
        fn fact(&self, entity: EntityRef, fact: Fact) -> Option<i64> {
            let EntityRef::City(id) = entity else { return None; };
            let city = self.cities.iter().find(|city| city.0 == id.0)?;
            (fact == Fact::CityDeprivation).then_some(city.1)
        }
        fn has_tag(&self, entity: EntityRef, tag: &str) -> bool {
            let EntityRef::City(id) = entity else { return false; };
            self.cities.iter().find(|city| city.0 == id.0).is_some_and(|city| city.2.contains(&tag))
        }
        fn related(&self, _: EntityRef) -> Option<CivId> { None }
    }

    fn template_json(when: &str) -> String {
        format!(r#"{{"catalog_id":"t","catalog_version":1,"schema_version":1,"templates":[{{"id":"event.t","category":"climate","tension_cost":2,"duration_turns":2,"when":{when},"targets":{{"kind":"city","scope":"eligible"}},"effects":[{{"op":"AdjustResource","target":"selected","resource_id":"food","amount":-1}}],"choices":[{{"id":"a","effects":[]}},{{"id":"b","effects":[{{"op":"AddTag","target":"selected","tag_id":"x.y"}}]}}],"cooldown_turns":3}}]}}"#)
    }

    #[test]
    fn bundled_event_catalog_validates() {
        let catalog = EventCatalog::parse(include_str!("../../../../data/catalogs/event_templates.json")).expect("catalog is valid");
        assert!(catalog.templates.len() >= 20);
        assert!(catalog.templates.windows(2).all(|pair| pair[0].id < pair[1].id));
    }

    #[test]
    fn parse_is_fail_closed() {
        let good = template_json(r#"{"op":"compare","fact":"city.deprivation","comparator":"gte","value":5}"#);
        assert!(EventCatalog::parse(&good).is_ok());
        for bad in [
            template_json(r#"{"op":"compare","fact":"city.secret","comparator":"gte","value":5}"#),
            template_json(r#"{"op":"eval","code":"1+1"}"#),
            template_json(r#"{"op":"compare","fact":"city.deprivation","comparator":"gte","value":1.5}"#),
            template_json(r#"{"op":"compare","fact":"city.deprivation","comparator":"gte","value":5,"extra":1}"#),
            good.replace("\"amount\":-1", "\"amount\":-99"),
            good.replace("\"AdjustResource\"", "\"SpawnArmy\""),
            good.replace("\"tension_cost\":2", "\"tension_cost\":9"),
            good.replace("\"catalog_version\":1", "\"catalog_version\":-1"),
            good.replace("\"schema_version\":1", "\"schema_version\":2"),
            "not json".to_owned(),
        ] {
            assert!(EventCatalog::parse(&bad).is_err(), "must reject: {bad}");
        }
    }

    #[test]
    fn duplicate_template_ids_and_conflicting_ops_are_rejected() {
        let good = template_json(r#"{"op":"all","args":[]}"#);
        let duplicated = good.replace("\"cooldown_turns\":3}]}", "\"cooldown_turns\":3},{\"id\":\"event.t\",\"category\":\"climate\",\"tension_cost\":1,\"duration_turns\":1,\"when\":{\"op\":\"all\",\"args\":[]},\"targets\":{\"kind\":\"city\",\"scope\":\"eligible\"},\"effects\":[],\"choices\":[{\"id\":\"a\",\"effects\":[]},{\"id\":\"b\",\"effects\":[]}],\"cooldown_turns\":0}]}");
        assert!(EventCatalog::parse(&duplicated).unwrap_err().iter().any(|d| d.code == "duplicate_template_id"));
        let conflicting = good.replace("\"effects\":[{\"op\":\"AdjustResource\"", "\"effects\":[{\"op\":\"AdjustResource\",\"target\":\"selected\",\"resource_id\":\"food\",\"amount\":1},{\"op\":\"AdjustResource\"");
        assert!(EventCatalog::parse(&conflicting).unwrap_err().iter().any(|d| d.code == "conflicting_operations"));
    }

    #[test]
    fn depth_limit_is_enforced() {
        let mut when = String::from(r#"{"op":"all","args":[]}"#);
        for _ in 0..20 { when = format!(r#"{{"op":"not","arg":{when}}}"#); }
        assert!(EventCatalog::parse(&template_json(&when)).unwrap_err().iter().any(|d| d.code == "predicate_too_large"));
    }

    #[test]
    fn evaluation_is_pure_sorted_and_respects_predicates() {
        let catalog = EventCatalog::parse(&template_json(r#"{"op":"all","args":[{"op":"compare","fact":"city.deprivation","comparator":"gte","value":5},{"op":"not","arg":{"op":"has_tag","entity":"selected","tag_id":"protected"}}]}"#)).unwrap();
        let view = Fixture { cities: vec![(3, 9, &[]), (1, 5, &[]), (2, 9, &["protected"]), (4, 1, &[])] };
        let first = eligible_targets(&catalog.templates[0], &view).unwrap();
        assert_eq!(first, vec![EntityRef::City(CityId(1)), EntityRef::City(CityId(3))]);
        assert_eq!(first, eligible_targets(&catalog.templates[0], &view).unwrap());
    }

    #[test]
    fn unknown_fact_is_an_error_not_zero() {
        let catalog = EventCatalog::parse(&template_json(r#"{"op":"compare","fact":"city.sanitary_stock","comparator":"lte","value":0}"#)).unwrap();
        let view = Fixture { cities: vec![(1, 0, &[])] };
        assert_eq!(eligible_targets(&catalog.templates[0], &view), Err(EvalError::UnknownFact(Fact::CitySanitaryStock)));
    }

    #[test]
    fn any_and_empty_combinators_follow_their_definition() {
        let view = Fixture { cities: vec![(1, 0, &[])] };
        let entity = EntityRef::City(CityId(1));
        assert_eq!(evaluate(&Predicate::All(Vec::new()), &view, entity), Ok(true));
        assert_eq!(evaluate(&Predicate::Any(Vec::new()), &view, entity), Ok(false));
        assert_eq!(evaluate(&Predicate::Exists(TargetKind::City), &view, entity), Ok(true));
        assert_eq!(evaluate(&Predicate::Exists(TargetKind::Route), &view, entity), Ok(false));
    }
}
