//! Per-civilization view of the world. Uses the engine visibility map: a civilization sees
//! terrain for Visible/Remembered tiles, its own cities, units and state in full, and foreign
//! cities and units only on currently Visible tiles. Foreign civilization internals and the
//! relations ledger are never sent.

use pw_engine::{
    ids::{CivId, TileIndex},
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
            Some(json!({
                "tile": index, "visibility": v, "terrain": tile.terrain,
                "biome": improvements::terrain_biome(tile.terrain),
                "river": tile.river, "yields": improvements::tile_output(tile), "owner": state.control.get(index),
                "improvement": tile.improvement,
                "build_progress": tile.build.as_ref().map(|build| json!({
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

#[cfg(test)]
mod tests {
    use super::*;
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
        let view = view_for(&state, civ, None, &[]);
        let tiles = view["tiles"].as_array().expect("tiles");
        assert_eq!(tiles[0]["improvement"], json!("improvement.farm"));
        assert_eq!(tiles[0]["build_progress"], Value::Null);
        assert_eq!(tiles[0]["yields"]["food"], json!(3));
        assert_eq!(tiles[1]["improvement"], Value::Null);
        assert_eq!(tiles[1]["build_progress"], json!({ "improvement": "improvement.farm", "progress": 2, "required": 8 }));
        assert!(tiles[0]["biome"].is_string());
    }
}
