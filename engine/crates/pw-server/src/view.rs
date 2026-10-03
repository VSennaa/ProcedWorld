//! Per-civilization view of the world. Uses the engine visibility map: a civilization sees
//! terrain for Visible/Remembered tiles, its own cities, units and state in full, and foreign
//! cities and units only on currently Visible tiles. Foreign civilization internals and the
//! relations ledger are never sent.

use pw_engine::{
    ids::{CivId, TileIndex},
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
            Some(json!({
                "tile": index, "visibility": v, "terrain": tile.terrain,
                "river": tile.river, "yields": tile.yields, "owner": state.control.get(index),
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
                } else if let Some((order, skipped)) = effective.get(id) {
                    object.insert("order".to_string(), json!(order));
                    object.insert("skipped_turn".to_string(), json!(skipped));
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
        "home_tile": if has_city { None } else { home },
        "tiles": tiles,
        "cities": cities,
        "units": units,
        // Own units awaiting an order: they block `ready` until ordered or skipped this turn.
        "idle_units": idle_units_with(state, civ, pending),
        // Entropy events awaiting this civilization's response (other civilizations' are never sent).
        "pending_events": state.entropy.pending.iter().filter(|(_, event)| event.civilization == civ).map(|(id, event)| json!({ "id": id, "event": event })).collect::<Vec<Value>>(),
    })
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
