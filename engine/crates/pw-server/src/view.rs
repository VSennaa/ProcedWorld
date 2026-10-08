//! Per-civilization view of the world. Uses the engine visibility map: a civilization sees
//! terrain for Visible/Remembered tiles, its own cities, units and state in full, and foreign
//! cities and units only on currently Visible tiles. Foreign civilization internals are never
//! sent, and of the relations ledger only the viewer's own entries with civilizations it has
//! already met (SDD 07, SDD 16).

use pw_engine::{
    diplomacy::{EntryStatus, LedgerCategory, RelationState},
    ids::{CivId, TileIndex, TurnNumber},
    improvements,
    world::{effective_unit_orders, idle_units_with, AcceptedCommand, DomainEvent, Visibility, WorldState},
};
use serde_json::{json, Value};

pub fn state_hash_string(state: &WorldState) -> String {
    format!("{:016x}", state.state_hash().0)
}

/// `pending` are the open turn's accepted commands: own units show the order they will have once
/// the turn resolves, and `idle_units` already accounts for them. `home` is the civilization's
/// starting tile; it is only published while the civilization has no city (`home_tile`), because
/// that is where the first city is founded without a settler.
pub fn view_for(state: &WorldState, civ: CivId, home: Option<TileIndex>, pending: &[AcceptedCommand]) -> Value {
    let effective = effective_unit_orders(state, civ, pending);
    let vis = state.visibility.get(&civ);
    let seen = |tile| vis.and_then(|map| map.get(&tile)).copied().unwrap_or(Visibility::Unknown);
    let tiles: Vec<Value> = vis
        .into_iter()
        .flat_map(|map| map.iter())
        .filter(|(_, v)| **v != Visibility::Unknown)
        .filter_map(|(index, v)| {
            let tile = state.tiles.get(index.0 as usize)?;
            // `yields` is the effective per-worker output (base + improvement, 0-6); `biome` is the
            // catalog biome of the terrain, matching `improvements[].biomes` in the catalog.
            // Improvements, works and effective yields change over time: they are only sent for
            // tiles in sight. Remembered tiles show the base terrain (SDD 16, no fog leak).
            let visible = *v == Visibility::Visible;
            Some(json!({
                "tile": index, "visibility": v, "terrain": tile.terrain,
                "biome": improvements::terrain_biome(tile.terrain),
                "river": tile.river, "yields": if visible { improvements::tile_output(tile) } else { tile.yields }, "owner": state.control.get(index),
                "improvement": tile.improvement.as_ref().filter(|_| visible),
                "build_progress": tile.build.as_ref().filter(|_| visible).map(|build| json!({
                    "improvement": build.improvement, "progress": build.progress,
                    "required": improvements::improvement(&build.improvement).map(|definition| definition.work_required()),
                })),
            }))
        })
        .collect();
    let cities: Vec<Value> = state
        .cities
        .iter()
        .filter(|(_, c)| c.owner == civ || seen(c.tile) == Visibility::Visible)
        .map(|(id, c)| json!({ "id": id, "city": c }))
        .collect();
    let units: Vec<Value> = state
        .units
        .iter()
        .filter(|(_, u)| u.owner == civ || seen(u.tile) == Visibility::Visible)
        .map(|(id, u)| {
            let mut unit = serde_json::to_value(u).unwrap_or(Value::Null);
            if let Some(object) = unit.as_object_mut() {
                if u.owner != civ {
                    // Orders are private to the owner.
                    object.remove("order");
                    object.remove("skipped_turn");
                } else {
                    if let Some((order, skipped)) = effective.get(id) {
                        object.insert("order".to_string(), json!(order));
                        object.insert("skipped_turn".to_string(), json!(skipped));
                    }
                    // Improvements this own worker may start on its tile now (engine validation).
                    let buildable = improvements::buildable_improvements(state, *id);
                    if !buildable.is_empty() { object.insert("buildable".to_string(), json!(buildable)); }
                }
            }
            json!({ "id": id, "unit": unit })
        })
        .collect();
    let has_city = state.cities.values().any(|c| c.owner == civ);
    json!({
        "world_id": state.world_id,
        "turn": state.turn,
        "state_hash": state_hash_string(state),
        "map_width": state.map_width,
        "civ": civ,
        "civilization": state.civilizations.get(&civ),
        // Engine rule (research_points_per_turn); the client's estimate never guesses it.
        "research_per_turn": pw_engine::world::research_points_per_turn(state, civ),
        "research_percentages": pw_engine::world::RESEARCH_PERCENTAGES,
        "home_tile": if has_city { None } else { home },
        "tiles": tiles,
        "cities": cities,
        "units": units,
        // Known civilizations only: one entry per civilization the engine has put this one in
        // contact with, carrying the viewer's directional balances and the recent Ledger entries
        // between the two (never third-party relations).
        "relations": relations_for(state, civ),
        // Own units awaiting an order: they block `ready` until ordered or skipped this turn.
        "idle_units": idle_units_with(state, civ, pending),
        // Entropy events awaiting this civilization's response (other civilizations' are never sent).
        "pending_events": state.entropy.pending.iter().filter(|(_, event)| event.civilization == civ).map(|(id, event)| json!({ "id": id, "event": event })).collect::<Vec<Value>>(),
    })
}

/// Ledger entries published per known civilization, most recent first (SDD 07).
const RECENT_LEDGER_LIMIT: usize = 5;

/// One row per Ledger fact between the viewer and `other`. `subject` is the side the entry
/// names (`LedgerEntry::subject`) and `counterpart` the other side of the pair, so
/// `subject -> counterpart` reads the recorded direction. A symmetric fact is written in both
/// directions by `push_pair`; only the newest row is published, so the same fact is never listed
/// twice (SDD 07, C2a).
fn recent_ledger(state: &WorldState, civ: CivId, other: CivId) -> Vec<Value> {
    let mut seen: Vec<(LedgerCategory, TurnNumber, EntryStatus, Option<u64>)> = Vec::new();
    let mut ledger: Vec<Value> = Vec::new();
    for entry in state.diplomacy.entries.iter().rev() {
        if !((entry.holder == civ && entry.subject == other) || (entry.holder == other && entry.subject == civ)) {
            continue;
        }
        let fact = (entry.category, entry.turn, entry.status, entry.cause_command);
        if seen.contains(&fact) { continue; }
        seen.push(fact);
        ledger.push(json!({
            "id": entry.id,
            "turn": entry.turn,
            "category": entry.category,
            "status": entry.status,
            "subject": entry.subject,
            "counterpart": entry.holder,
        }));
        if ledger.len() >= RECENT_LEDGER_LIMIT { break; }
    }
    ledger
}

/// Relations visible to `civ`: one entry per civilization the engine has put it in contact with
/// (`relation_state != Unknown`, i.e. contact was established). Each entry carries the viewer's
/// directional balance (`Cf`/`R`/`Dv`) and the most recent Ledger entries **between the two**.
/// Entries between third parties are never included, and a civilization never met stays absent.
fn relations_for(state: &WorldState, civ: CivId) -> Vec<Value> {
    state
        .civilizations
        .keys()
        .copied()
        .filter(|other| *other != civ)
        .filter_map(|other| {
            let relation_state = state.diplomacy.state(civ, other);
            if relation_state == RelationState::Unknown { return None; }
            // Directional: the viewer's own view of `other` (SDD 07, GDD 07).
            let balance = state.diplomacy.balance(civ, other);
            let ledger = recent_ledger(state, civ, other);
            Some(json!({
                "civ": other,
                "state": relation_state,
                "confidence": balance.cf,
                "resentment": balance.r,
                "debt": balance.dv,
                "ledger": ledger,
            }))
        })
        .collect()
}

/// Domain events relevant to `civ`: results of its own commands, migrations between its cities
/// and collapses (public). `turn_commands` are all commands of the resolved turn.
pub fn events_for(state: &WorldState, civ: CivId, turn_commands: &[AcceptedCommand], events: &[DomainEvent]) -> Vec<Value> {
    let own = |command_id: u64| turn_commands.iter().any(|c| c.command_id == command_id && c.actor_id == civ);
    let own_city = |id| state.cities.get(id).is_some_and(|city| city.owner == civ);
    events
        .iter()
        .filter(|event| match event {
            DomainEvent::CommandApplied { command_id } | DomainEvent::CommandRejected { command_id, .. } => own(*command_id),
            DomainEvent::PopulationMigrated { from, to, .. } => own_city(from) || own_city(to),
            DomainEvent::CollapseTriggered { .. } => true,
            // A diplomatic resolution is visible to both parties.
            DomainEvent::DiplomacyResolved { actor, other, .. } => *actor == civ || *other == civ,
        })
        .map(|event| serde_json::to_value(event).unwrap_or(Value::Null))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pw_engine::diplomacy::{Balance, EntryStatus, LedgerCategory, LedgerEntry, RelationRecord, RelationState};
    use pw_engine::ids::TurnNumber;
    use pw_engine::improvements::TileBuild;
    use std::collections::BTreeMap;

    #[test]
    fn tiles_carry_improvement_build_progress_and_effective_yields() {
        let (mut state, _, _) = pw_harness::initial_world(5, 2).expect("world builds");
        let civ = *state.civilizations.keys().next().expect("a civilization");
        state.visibility.insert(civ, BTreeMap::from([(TileIndex(0), Visibility::Visible), (TileIndex(1), Visibility::Remembered)]));
        state.tiles[0].improvement = Some("improvement.farm".into());
        state.tiles[0].yields.food = 2;
        state.tiles[1].build = Some(TileBuild { improvement: "improvement.farm".into(), progress: 2 });
        state.visibility.get_mut(&civ).unwrap().insert(TileIndex(2), Visibility::Visible);
        state.tiles[2].build = Some(TileBuild { improvement: "improvement.farm".into(), progress: 2 });
        let view = view_for(&state, civ, None, &[]);
        let tiles = view["tiles"].as_array().expect("tiles");
        assert_eq!(tiles[0]["improvement"], json!("improvement.farm"));
        assert_eq!(tiles[0]["build_progress"], Value::Null);
        assert_eq!(tiles[0]["yields"]["food"], json!(3));
        assert_eq!(tiles[2]["build_progress"], json!({ "improvement": "improvement.farm", "progress": 2, "required": 8 }));
        // Fog: a remembered tile never reveals its current work or improvement.
        assert_eq!(tiles[1]["improvement"], Value::Null);
        assert_eq!(tiles[1]["build_progress"], Value::Null);
        assert!(tiles[0]["biome"].is_string());
    }

    fn ledger_entry(id: u64, turn: u32, holder: CivId, subject: CivId, category: LedgerCategory) -> LedgerEntry {
        LedgerEntry {
            id, turn: TurnNumber(turn), holder, subject, category, status: EntryStatus::Active,
            intensity: 1, cf_delta: 0, resentment_delta: 0, debt_delta: 0,
            due: None, cause_command: None, supersedes: None,
        }
    }

    #[test]
    fn relations_only_include_met_civilizations_and_their_own_ledger_entries() {
        let (mut state, _, _) = pw_harness::initial_world(5, 3).expect("world builds");
        let a = CivId(0);
        let b = CivId(1);
        let c = CivId(2);
        // A has met B; B and C are at war with each other. A has never made contact with C.
        state.diplomacy.relations.insert(a, BTreeMap::from([(b, RelationRecord { state: RelationState::Peace, since: TurnNumber(3), until: None, objective: None })]));
        state.diplomacy.relations.insert(b, BTreeMap::from([(c, RelationRecord { state: RelationState::War, since: TurnNumber(4), until: None, objective: None })]));
        // Balances are directional: A's view of B differs from B's view of A, and B also views C.
        state.diplomacy.balances.insert(a, BTreeMap::from([(b, Balance { cf: 71, r: 12, dv: -8 })]));
        state.diplomacy.balances.insert(b, BTreeMap::from([(a, Balance { cf: 30, r: 40, dv: 5 }), (c, Balance { cf: 10, r: 90, dv: 0 })]));
        state.diplomacy.entries = vec![
            ledger_entry(0, 1, a, b, LedgerCategory::Border),
            ledger_entry(1, 2, b, a, LedgerCategory::Offense),
            // Third-party facts shared between B and C: never visible to A.
            ledger_entry(2, 5, b, c, LedgerCategory::Trade),
            ledger_entry(3, 6, c, b, LedgerCategory::Incident),
            // One symmetric fact mirrored in both directions (as `push_pair` records it).
            ledger_entry(4, 7, a, b, LedgerCategory::Promise),
            ledger_entry(5, 7, b, a, LedgerCategory::Promise),
        ];

        let view = view_for(&state, a, None, &[]);
        let relations = view["relations"].as_array().expect("relations");
        assert_eq!(relations.len(), 1, "only the civilization A has met is listed; C stays unknown");
        let relation = &relations[0];
        assert_eq!(relation["civ"], json!(1));
        assert_eq!(relation["state"], json!("peace"));
        // The viewer's own directional balances.
        assert_eq!(relation["confidence"], json!(71));
        assert_eq!(relation["resentment"], json!(12));
        assert_eq!(relation["debt"], json!(-8));

        let ledger = relation["ledger"].as_array().expect("ledger");
        assert_eq!(ledger.len(), 3, "the mirrored fact is listed once, one row per fact");
        // Most recent first: `subject` names the entry's side and `counterpart` the other side.
        assert_eq!(ledger[0]["id"], json!(5));
        assert_eq!(ledger[0]["category"], json!("promise"));
        assert_eq!(ledger[0]["subject"], json!(0));
        assert_eq!(ledger[0]["counterpart"], json!(1));
        assert_eq!(ledger[1]["id"], json!(1));
        assert_eq!(ledger[1]["category"], json!("offense"));
        assert_eq!(ledger[1]["subject"], json!(0));
        assert_eq!(ledger[1]["counterpart"], json!(1));
        assert_eq!(ledger[2]["id"], json!(0));
        assert_eq!(ledger[2]["category"], json!("border"));
        assert_eq!(ledger[2]["subject"], json!(1));
        assert_eq!(ledger[2]["counterpart"], json!(0));

        // No leak: A's relations carry no trace of the B<->C war or its ledger.
        let visible = serde_json::to_string(relations).expect("serializes");
        assert!(!visible.contains("\"counterpart\":2"), "third-party counterpart leaks: {visible}");
        assert!(!visible.contains("trade") && !visible.contains("incident"), "third-party ledger leaks: {visible}");
        assert!(!visible.contains("war"), "third-party state leaks: {visible}");
    }
}
