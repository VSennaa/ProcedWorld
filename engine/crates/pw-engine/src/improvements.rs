//! Terrain improvements built by workers (docs/sdd/15 section 4.2.2).
//!
//! Catalog: `data/catalogs/improvements.json` (`core.improvements`). The engine keeps one
//! terrain code per tile and no per-tile resource deposits, so two documented simplifications
//! apply (docs/sdd/15 section 4.2.2 and engine/DIAGNOSTICO-P7.md, "Melhorias"):
//! - each engine terrain maps to exactly one catalog biome (`terrain_biome`);
//! - a tile offers the resources listed in its biome's `resource_tags` (`core.biomes`).
//!
//! Non-production costs (stone, timber) have no stockpile in the engine yet: every cost entry is
//! converted 1:1 into worker labour (`work_required`). A worker adds `WORK_PER_TURN` per turn.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::{
    ids::{CivId, TileIndex, UnitId},
    world::{
        grid_for, unit_role, RejectionReason, TileState, TileYields, UnitOrder, WorldState,
        TERRAIN_COAST, TERRAIN_DESERT, TERRAIN_FOREST, TERRAIN_JUNGLE, TERRAIN_OCEAN,
        TERRAIN_PLAINS, TERRAIN_STEPPE, TERRAIN_SWAMP,
    },
};

/// Labour a worker adds to its tile's construction each turn (balance proposal, 2026-10-03).
pub const WORK_PER_TURN: u32 = 2;
/// Work radius of a city (the same radius `allocate_workplaces` uses).
const CITY_WORK_RADIUS: u32 = 2;

/// Construction in progress on a tile. Kept when the worker leaves; resumed only by an order for
/// the same improvement, reset by an order for a different one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileBuild {
    pub improvement: String,
    pub progress: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImprovementDefinition {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub biomes: Vec<String>,
    #[serde(default)]
    pub resources: Vec<String>,
    pub cost: BTreeMap<String, u32>,
    #[serde(default)]
    pub maintenance: BTreeMap<String, u32>,
    pub requires_technology: String,
    #[serde(default)]
    pub effects: Vec<EffectDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectDefinition {
    pub op: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub amount: i32,
}

impl ImprovementDefinition {
    /// Total labour: production plus every non-production cost, converted 1:1 (simplification).
    pub fn work_required(&self) -> u32 { self.cost.values().copied().sum::<u32>().max(1) }

    /// Wealth maintenance per turn.
    pub fn upkeep(&self) -> u32 { self.maintenance.get("wealth").copied().unwrap_or(0) }

    /// Additive yield bonus from `AdjustResource` effects on the tile. Tag effects
    /// (`AddTag`) have no engine rule yet; the tag is implied by the improvement id.
    fn bonus(&self) -> [i32; 5] {
        let mut bonus = [0; 5];
        for effect in self.effects.iter().filter(|effect| effect.op == "AdjustResource" && effect.target == "tile") {
            let slot = match effect.resource_id.as_deref() {
                Some("food") => 0, Some("production") => 1, Some("wealth") => 2,
                Some("knowledge") => 3, Some("culture") => 4, _ => continue,
            };
            bonus[slot] += effect.amount;
        }
        bonus
    }
}

#[derive(Deserialize)]
struct ImprovementFile { improvements: Vec<ImprovementDefinition> }

#[derive(Deserialize)]
struct BiomeFile { biomes: Vec<BiomeEntry> }

#[derive(Deserialize)]
struct BiomeEntry {
    id: String,
    #[serde(default)]
    resource_tags: Vec<String>,
}

/// Bundled catalog in file order (the order used for every deterministic iteration).
pub fn improvements() -> &'static [ImprovementDefinition] {
    static CATALOG: OnceLock<Vec<ImprovementDefinition>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str::<ImprovementFile>(include_str!("../../../../data/catalogs/improvements.json"))
            .expect("the bundled improvement catalog is valid")
            .improvements
    })
}

pub fn improvement(id: &str) -> Option<&'static ImprovementDefinition> {
    improvements().iter().find(|definition| definition.id == id)
}

fn biome_resources(biome: &str) -> &'static [String] {
    static BIOMES: OnceLock<BTreeMap<String, Vec<String>>> = OnceLock::new();
    BIOMES.get_or_init(|| {
        serde_json::from_str::<BiomeFile>(include_str!("../../../../data/catalogs/biomes.json"))
            .expect("the bundled biome catalog is valid")
            .biomes.into_iter().map(|biome| (biome.id, biome.resource_tags)).collect()
    }).get(biome).map_or(&[], Vec::as_slice)
}

/// Catalog biome represented by an engine terrain code. One biome per terrain: the harness
/// collapses savanna/tundra into steppe and mountain/glacier into plains, but those source
/// biomes are not kept in `TileState`, so they cannot be told apart here (no quarry for now).
pub const fn terrain_biome(terrain: u8) -> Option<&'static str> {
    match terrain {
        TERRAIN_PLAINS => Some("plains"),
        TERRAIN_FOREST => Some("forest"),
        TERRAIN_JUNGLE => Some("jungle"),
        TERRAIN_SWAMP => Some("swamp"),
        TERRAIN_DESERT => Some("desert"),
        TERRAIN_STEPPE => Some("steppe"),
        TERRAIN_COAST => Some("coast"),
        TERRAIN_OCEAN => Some("ocean"),
        _ => None,
    }
}

/// True when the catalog allows `definition` on `terrain`: the biome is listed and, if the
/// improvement needs a resource, the biome offers one of them.
pub fn allowed_on_terrain(definition: &ImprovementDefinition, terrain: u8) -> bool {
    let Some(biome) = terrain_biome(terrain) else { return false };
    definition.biomes.iter().any(|listed| listed == biome)
        && (definition.resources.is_empty() || definition.resources.iter().any(|resource| biome_resources(biome).contains(resource)))
}

fn clamp_yield(base: u8, bonus: i32) -> u8 { (i32::from(base.min(6)) + bonus).clamp(0, 6) as u8 }

/// Per-worker output of a tile: base yields plus the improvement bonus, each capped to 0-6 (GDD 03).
pub fn tile_output(tile: &TileState) -> TileYields {
    let bonus = tile.improvement.as_deref().and_then(improvement).map_or([0; 5], ImprovementDefinition::bonus);
    let base = tile.yields;
    TileYields {
        food: clamp_yield(base.food, bonus[0]),
        production: clamp_yield(base.production, bonus[1]),
        wealth: clamp_yield(base.wealth, bonus[2]),
        knowledge: clamp_yield(base.knowledge, bonus[3]),
        culture: clamp_yield(base.culture, bonus[4]),
    }
}

/// Output of a tile whose improvement is suspended (unpaid maintenance): base yields only.
pub fn base_output(tile: &TileState) -> TileYields {
    tile.yields.capped()
}

/// Territory of a tile: an explicit `control` entry, otherwise the owner of the nearest city whose
/// work radius covers it (ties by city id). `None` is neutral land.
pub fn territory_owner(state: &WorldState, tile: TileIndex) -> Option<CivId> {
    if let Some(owner) = state.control.get(&tile) { return Some(*owner); }
    let grid = grid_for(state).ok()?;
    let cell = grid.cell(tile).ok()?;
    state.cities.iter()
        .filter_map(|(id, city)| {
            let distance = grid.distance(cell, grid.cell(city.tile).ok()?).ok()?;
            (distance <= CITY_WORK_RADIUS).then_some((distance, *id, city.owner))
        })
        .min()
        .map(|(_, _, owner)| owner)
}

/// Own territory, or neutral land adjacent to own territory.
fn buildable_territory(state: &WorldState, tile: TileIndex, civ: CivId) -> bool {
    match territory_owner(state, tile) {
        Some(owner) => owner == civ,
        None => grid_for(state).ok()
            .and_then(|grid| grid.neighbors(grid.cell(tile).ok()?).ok().map(|cells| (grid, cells)))
            .is_some_and(|(grid, cells)| cells.into_iter().filter_map(|cell| grid.tile_index(cell).ok())
                .any(|neighbor| territory_owner(state, neighbor) == Some(civ))),
    }
}

/// Validates a `Build` order for `unit_id` on its current tile (docs/sdd/15 section 4.2.2).
pub fn check_build(state: &WorldState, unit_id: UnitId, improvement_id: &str) -> Result<&'static ImprovementDefinition, RejectionReason> {
    let unit = state.units.get(&unit_id).ok_or(RejectionReason::UnknownUnit)?;
    if unit_role(&unit.unit_type) != "worker" { return Err(RejectionReason::NotAWorker); }
    let definition = improvement(improvement_id).ok_or(RejectionReason::UnknownImprovement)?;
    let civilization = state.civilizations.get(&unit.owner).ok_or(RejectionReason::UnknownCivilization)?;
    if !civilization.researched_technologies.contains(&definition.requires_technology) {
        return Err(RejectionReason::ImprovementTechnologyNotResearched);
    }
    let tile = state.tiles.get(unit.tile.0 as usize).ok_or(RejectionReason::InvalidTile)?;
    if !allowed_on_terrain(definition, tile.terrain) { return Err(RejectionReason::ImprovementTerrainInvalid); }
    if tile.improvement.is_some() { return Err(RejectionReason::TileAlreadyImproved); }
    if state.cities.values().any(|city| city.tile == unit.tile) { return Err(RejectionReason::CityTileNotImprovable); }
    // A work with an active builder blocks every other order on the tile. An abandoned work (no unit
    // building it) is resumed by an order for the same improvement and replaced, losing its
    // progress, by an order for a different one (docs/sdd/15 section 4.2.2).
    if state.units.iter().any(|(id, other)| *id != unit_id && other.tile == unit.tile && matches!(other.order, UnitOrder::Build { .. })) {
        return Err(RejectionReason::TileWorkInProgress);
    }
    if !buildable_territory(state, unit.tile, unit.owner) { return Err(RejectionReason::TileOutsideTerritory); }
    Ok(definition)
}

/// Improvements the worker could start on its current tile, in catalog order.
pub fn buildable_improvements(state: &WorldState, unit_id: UnitId) -> Vec<String> {
    improvements().iter()
        .filter(|definition| check_build(state, unit_id, &definition.id).is_ok())
        .map(|definition| definition.id.clone())
        .collect()
}

/// Advances every `Build` order in unit-id order. An order that is no longer legal on the unit's
/// tile returns the unit to `Idle` without progress; a finished one places the improvement.
pub(crate) fn resolve_works(state: &mut WorldState) {
    let builders: Vec<(UnitId, String)> = state.units.iter()
        .filter_map(|(id, unit)| match &unit.order { UnitOrder::Build { improvement } => Some((*id, improvement.clone())), _ => None })
        .collect();
    for (unit_id, improvement_id) in builders {
        let Some(tile_index) = state.units.get(&unit_id).map(|unit| unit.tile) else { continue };
        let Ok(definition) = check_build(state, unit_id, &improvement_id) else {
            if let Some(unit) = state.units.get_mut(&unit_id) { unit.order = UnitOrder::Idle; }
            continue;
        };
        let tile = &mut state.tiles[tile_index.0 as usize];
        let progress = match &tile.build {
            Some(build) if build.improvement == improvement_id => build.progress,
            _ => 0,
        }.saturating_add(WORK_PER_TURN);
        if progress >= definition.work_required() {
            tile.improvement = Some(improvement_id);
            tile.build = None;
            if let Some(unit) = state.units.get_mut(&unit_id) { unit.order = UnitOrder::Idle; }
        } else {
            tile.build = Some(TileBuild { improvement: improvement_id, progress });
        }
    }
}

/// Pays improvement maintenance after essential city upkeep (GDD 03 priority: city sustenance,
/// then improvements by stable id = tile index). The payer is the tile's territory owner; neutral
/// improvements are dormant (nobody works them) and cost nothing. `available` holds only base
/// income: a paid, worked improvement then credits its wealth bonus to the civilization working the
/// tile (`worked_by`), so it can fund improvements with a higher tile index but never its own
/// upkeep. Returns the tiles whose bonus is suspended this turn.
pub(crate) fn pay_upkeep(state: &WorldState, worked_by: &BTreeMap<TileIndex, CivId>, available: &mut BTreeMap<CivId, u32>) -> BTreeSet<TileIndex> {
    let mut suspended = BTreeSet::new();
    for (index, tile) in state.tiles.iter().enumerate() {
        let Some(definition) = tile.improvement.as_deref().and_then(improvement) else { continue };
        let tile_index = TileIndex(index as u32);
        let Some(payer) = territory_owner(state, tile_index) else { continue };
        let Some(funds) = available.get_mut(&payer) else { continue };
        if *funds < definition.upkeep() { suspended.insert(tile_index); continue; }
        *funds -= definition.upkeep();
        if let Some(worker) = worked_by.get(&tile_index) {
            let bonus = tile_output(tile).wealth.saturating_sub(base_output(tile).wealth);
            let funds = available.entry(*worker).or_default();
            *funds = funds.saturating_add(u32::from(bonus));
        }
    }
    suspended
}

/// Maintenance the civilization currently owes for improvements in its territory.
pub fn civilization_upkeep(state: &WorldState, civ: CivId) -> u32 {
    state.tiles.iter().enumerate()
        .filter_map(|(index, tile)| tile.improvement.as_deref().and_then(improvement).map(|definition| (index, definition)))
        .filter(|(index, _)| territory_owner(state, TileIndex(*index as u32)) == Some(civ))
        .map(|(_, definition)| definition.upkeep())
        .sum()
}

/// Gain of `definition` on `tile` after the 0-6 cap, weighted for the T0 bot
/// (food 3, production 2, wealth 2, knowledge 1, culture 1). Zero for tag-only improvements.
pub fn improvement_value(tile: &TileState, definition: &ImprovementDefinition) -> u32 {
    let mut improved = tile.clone();
    improved.improvement = Some(definition.id.clone());
    let (after, before) = (tile_output(&improved), base_output(tile));
    let gain = |a: u8, b: u8| u32::from(a.saturating_sub(b));
    3 * gain(after.food, before.food) + 2 * gain(after.production, before.production) + 2 * gain(after.wealth, before.wealth)
        + gain(after.knowledge, before.knowledge) + gain(after.culture, before.culture)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ids::{CityId, TurnNumber},
        world::{
            replay, step, AcceptedCommand, CityFocus, CityGroup, CityState, CivilizationState, CommandKind, CommandLog,
            CommandOrigin, CommandPayload, DomainEvent, RulesetRef, SimulationVersions, UnitState,
            WorldId, WorldSnapshot,
        },
    };

    const WIDTH: u32 = 20;
    const CITY_TILE: u32 = 5 * WIDTH + 5;

    fn versions(state: &WorldState) -> SimulationVersions { SimulationVersions { ruleset: state.ruleset.clone(), resolver_version: 1 } }

    fn unit(owner: u32, tile: u32, unit_type: &str) -> UnitState {
        UnitState { owner: CivId(owner), tile: TileIndex(tile), unit_type: unit_type.into(), hit_points: 100, movement_left: 2, explored: false, order: UnitOrder::Idle, skipped_turn: None }
    }

    fn city(owner: u32, tile: u32, population: u32) -> CityState {
        CityState {
            owner: CivId(owner), tile: TileIndex(tile), focus: CityFocus::Supply, population, housing: population + 2,
            food_stock: 10, growth_progress: 0, consecutive_food_shortages: 0, stability: 50,
            groups: vec![CityGroup { function: crate::world::GroupFunction::Cultivators, population, satisfaction: 50 },
                CityGroup { function: crate::world::GroupFunction::Crafts, population: 0, satisfaction: 50 },
                CityGroup { function: crate::world::GroupFunction::Merchants, population: 0, satisfaction: 50 }],
            workplaces: Vec::new(), last_yields: TileYields::default(), deprivation: 0, group_tension: 0,
            war_threat: 0, environmental_exposure: 0, crisis_pressure: 0, crisis_turns: 0,
            essential_maintenance_unpaid: false, unit_queue: Vec::new(), unit_production: 0,
        }
    }

    /// Plains map 20x10; civ 1 has a city in the middle and a worker on an adjacent tile.
    fn world() -> WorldState {
        let mut civ = CivilizationState::default();
        civ.researched_technologies.insert("tech.foraging".into());
        let tile = TileState { terrain: TERRAIN_PLAINS, river: false, yields: TileYields { food: 2, production: 1, wealth: 1, knowledge: 0, culture: 0 }, ..TileState::default() };
        WorldState {
            world_id: WorldId(3), turn: TurnNumber::ZERO, seed: 5,
            ruleset: RulesetRef { id: "test".into(), version: "1".into(), content_hash: 1 },
            schema_version: 1, map_width: WIDTH, tiles: vec![tile; (WIDTH * 10) as usize],
            civilizations: BTreeMap::from([(CivId(1), civ), (CivId(2), CivilizationState::default())]),
            cities: BTreeMap::from([(CityId(1), city(1, CITY_TILE, 1))]),
            units: BTreeMap::from([(UnitId(1), unit(1, CITY_TILE + 1, "unit.worker"))]),
            attacks: BTreeMap::new(), control: BTreeMap::new(), visibility: BTreeMap::new(),
            diplomacy: Default::default(), entropy: Default::default(),
        }
    }

    fn build(id: u64, actor: u32, unit_id: u32, improvement: &str, turn: u32) -> AcceptedCommand {
        let payload = CommandPayload::SetUnitOrder { unit_id: UnitId(unit_id), order: UnitOrder::Build { improvement: improvement.into() } };
        AcceptedCommand { command_id: id, world_id: WorldId(3), turn: TurnNumber(turn), accepted_sequence: id, actor_id: CivId(actor), origin: CommandOrigin::Player, kind: CommandKind::SetUnitOrder, payload, grounding: Vec::new(), intent_evidence: None, mandate: None }
    }

    fn rejection(state: &WorldState, command: AcceptedCommand) -> Option<RejectionReason> {
        let id = command.command_id;
        step(state, &[command], state.seed, &versions(state)).events.into_iter().find_map(|event| match event {
            DomainEvent::CommandRejected { command_id, reason } if command_id == id => Some(reason),
            _ => None,
        })
    }

    /// The tile `steps` hexes east of the city along its row (offset grid, same row).
    fn east(steps: u32) -> u32 { CITY_TILE + steps }

    #[test]
    fn wire_format_of_the_build_order() {
        let order = UnitOrder::Build { improvement: "improvement.farm".into() };
        assert_eq!(serde_json::to_value(&order).unwrap(), serde_json::json!({ "type": "build", "data": { "improvement": "improvement.farm" } }));
        let parsed: UnitOrder = serde_json::from_str(r#"{"type":"build","data":{"improvement":"improvement.farm"}}"#).unwrap();
        assert_eq!(parsed, order);
    }

    #[test]
    fn catalog_biomes_map_to_engine_terrains_explicitly() {
        let farm = improvement("improvement.farm").unwrap();
        assert!(allowed_on_terrain(farm, TERRAIN_PLAINS));
        assert!(!allowed_on_terrain(farm, TERRAIN_STEPPE), "steppe offers no crops");
        assert!(allowed_on_terrain(improvement("improvement.pasture").unwrap(), TERRAIN_STEPPE));
        assert!(allowed_on_terrain(improvement("improvement.lumber_camp").unwrap(), TERRAIN_FOREST));
        assert!(allowed_on_terrain(improvement("improvement.lumber_camp").unwrap(), TERRAIN_JUNGLE));
        assert!(allowed_on_terrain(improvement("improvement.fishing_wharf").unwrap(), TERRAIN_COAST));
        assert!(allowed_on_terrain(improvement("improvement.saltworks").unwrap(), TERRAIN_DESERT));
        assert!(allowed_on_terrain(improvement("improvement.mine").unwrap(), TERRAIN_DESERT), "desert offers copper");
        assert!(!allowed_on_terrain(improvement("improvement.irrigated_field").unwrap(), TERRAIN_DESERT), "desert offers no crops");
        assert!(allowed_on_terrain(improvement("improvement.dryland_cistern").unwrap(), TERRAIN_STEPPE), "no resource needed");
        // The engine has no mountain terrain: the quarry cannot be built anywhere for now.
        assert!((0..8).all(|terrain| !allowed_on_terrain(improvement("improvement.quarry").unwrap(), terrain)));
        assert_eq!(terrain_biome(42), None);
        assert_eq!(improvement("improvement.irrigated_field").unwrap().work_required(), 22, "stone converted to labour");
    }

    #[test]
    fn build_validation_rejects_illegal_orders() {
        let base = world();
        assert_eq!(rejection(&base, build(1, 1, 1, "improvement.farm", 0)), None);
        assert_eq!(rejection(&base, build(1, 2, 1, "improvement.farm", 0)), Some(RejectionReason::NotCommandOwner));
        assert_eq!(rejection(&base, build(1, 1, 1, "improvement.nope", 0)), Some(RejectionReason::UnknownImprovement));
        assert_eq!(rejection(&base, build(1, 1, 1, "improvement.irrigated_field", 0)), Some(RejectionReason::ImprovementTechnologyNotResearched));
        let mut teched = base.clone();
        teched.civilizations.get_mut(&CivId(1)).unwrap().researched_technologies.insert("tech.public_works".into());
        assert_eq!(rejection(&teched, build(1, 1, 1, "improvement.lumber_camp", 0)), Some(RejectionReason::ImprovementTerrainInvalid));

        let mut scout = base.clone();
        scout.units.get_mut(&UnitId(1)).unwrap().unit_type = "unit.scout".into();
        assert_eq!(rejection(&scout, build(1, 1, 1, "improvement.farm", 0)), Some(RejectionReason::NotAWorker));

        let mut improved = base.clone();
        improved.tiles[east(1) as usize].improvement = Some("improvement.pasture".into());
        assert_eq!(rejection(&improved, build(1, 1, 1, "improvement.farm", 0)), Some(RejectionReason::TileAlreadyImproved));

        let mut on_city = base.clone();
        on_city.units.get_mut(&UnitId(1)).unwrap().tile = TileIndex(CITY_TILE);
        assert_eq!(rejection(&on_city, build(1, 1, 1, "improvement.farm", 0)), Some(RejectionReason::CityTileNotImprovable));

        // Distance 3: neutral land adjacent to the work radius is allowed; distance 4 is not.
        let mut edge = base.clone();
        edge.units.get_mut(&UnitId(1)).unwrap().tile = TileIndex(east(3));
        assert_eq!(rejection(&edge, build(1, 1, 1, "improvement.farm", 0)), None);
        edge.units.get_mut(&UnitId(1)).unwrap().tile = TileIndex(east(4));
        assert_eq!(rejection(&edge, build(1, 1, 1, "improvement.farm", 0)), Some(RejectionReason::TileOutsideTerritory));

        let mut foreign = base.clone();
        foreign.cities.insert(CityId(2), city(2, east(3), 1));
        foreign.units.get_mut(&UnitId(1)).unwrap().tile = TileIndex(east(2));
        // Distance 2 from our city, 1 from the foreign one: the nearer city owns the tile.
        assert_eq!(rejection(&foreign, build(1, 1, 1, "improvement.farm", 0)), Some(RejectionReason::TileOutsideTerritory));
        foreign.control.insert(TileIndex(east(2)), CivId(1));
        assert_eq!(rejection(&foreign, build(1, 1, 1, "improvement.farm", 0)), None, "explicit control wins");
    }

    #[test]
    fn work_progresses_each_turn_and_completes_into_an_improvement() {
        let mut state = world();
        let first = step(&state, &[build(1, 1, 1, "improvement.farm", 0)], state.seed, &versions(&state));
        state = first.state;
        let tile = &state.tiles[east(1) as usize];
        assert_eq!(tile.build, Some(TileBuild { improvement: "improvement.farm".into(), progress: WORK_PER_TURN }));
        assert_eq!(tile.improvement, None);
        assert!(matches!(state.units[&UnitId(1)].order, UnitOrder::Build { .. }));
        // Farm costs 8 production: 4 turns at 2 labour per turn.
        for _ in 1..4 { state = step(&state, &[], state.seed, &versions(&state)).state; }
        let tile = &state.tiles[east(1) as usize];
        assert_eq!(tile.improvement.as_deref(), Some("improvement.farm"));
        assert_eq!(tile.build, None);
        assert_eq!(state.units[&UnitId(1)].order, UnitOrder::Idle);
        // The finished tile no longer accepts another improvement.
        assert_eq!(rejection(&state, build(9, 1, 1, "improvement.pasture", state.turn.0)), Some(RejectionReason::TileAlreadyImproved));
    }

    #[test]
    fn leaving_keeps_progress_and_a_different_improvement_restarts_it() {
        let mut state = world();
        state = step(&state, &[build(1, 1, 1, "improvement.farm", 0)], state.seed, &versions(&state)).state;
        let idle = AcceptedCommand { payload: CommandPayload::SetUnitOrder { unit_id: UnitId(1), order: UnitOrder::Idle }, ..build(2, 1, 1, "x", 1) };
        state = step(&state, &[idle], state.seed, &versions(&state)).state;
        assert_eq!(state.tiles[east(1) as usize].build.as_ref().map(|build| build.progress), Some(2));
        state = step(&state, &[build(3, 1, 1, "improvement.pasture", 2)], state.seed, &versions(&state)).state;
        assert_eq!(state.tiles[east(1) as usize].build, Some(TileBuild { improvement: "improvement.pasture".into(), progress: 2 }));
    }

    #[test]
    fn an_actively_built_tile_rejects_other_orders_but_an_abandoned_work_can_be_replaced() {
        let mut state = world();
        state.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Build { improvement: "improvement.farm".into() };
        state.tiles[east(1) as usize].build = Some(TileBuild { improvement: "improvement.farm".into(), progress: 4 });
        // A second worker on the same tile (only constructible directly; movement forbids stacking).
        state.units.insert(UnitId(2), unit(1, east(1), "unit.worker"));
        assert_eq!(check_build(&state, UnitId(2), "improvement.pasture").err(), Some(RejectionReason::TileWorkInProgress));
        assert_eq!(check_build(&state, UnitId(2), "improvement.farm").err(), Some(RejectionReason::TileWorkInProgress));
        // Abandoned (no active builder): the same improvement resumes, a different one replaces it.
        state.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Idle;
        state.units.remove(&UnitId(2));
        let resumed = step(&state, &[build(1, 1, 1, "improvement.farm", 0)], state.seed, &versions(&state)).state;
        assert_eq!(resumed.tiles[east(1) as usize].build, Some(TileBuild { improvement: "improvement.farm".into(), progress: 6 }));
        let replaced = step(&state, &[build(1, 1, 1, "improvement.pasture", 0)], state.seed, &versions(&state)).state;
        assert_eq!(replaced.tiles[east(1) as usize].build, Some(TileBuild { improvement: "improvement.pasture".into(), progress: 2 }));
    }

    #[test]
    fn an_order_that_became_illegal_returns_to_idle_without_progress() {
        let mut state = world();
        state.units.get_mut(&UnitId(1)).unwrap().order = UnitOrder::Build { improvement: "improvement.farm".into() };
        state.control.insert(TileIndex(east(1)), CivId(2));
        let next = step(&state, &[], state.seed, &versions(&state)).state;
        assert_eq!(next.units[&UnitId(1)].order, UnitOrder::Idle);
        assert_eq!(next.tiles[east(1) as usize].build, None);
    }

    #[test]
    fn improvement_adds_yield_and_respects_the_cap_of_six() {
        let mut tile = world().tiles[0].clone();
        tile.improvement = Some("improvement.farm".into());
        assert_eq!(tile_output(&tile).food, 3);
        tile.yields.food = 6;
        assert_eq!(tile_output(&tile).food, 6);
        tile.yields.food = 9;
        assert_eq!(tile_output(&tile).food, 6, "base above six is capped before the bonus");
        tile.improvement = Some("improvement.mine".into());
        tile.yields.food = 2;
        assert_eq!(tile_output(&tile), base_output(&tile), "tag-only improvements add no yield");
    }

    fn economy_world(treasury: u32) -> WorldState {
        let mut state = world();
        state.units.clear();
        // Only the improved tile yields food, so the single worker of the city works it.
        for tile in &mut state.tiles { tile.yields = TileYields { food: 0, production: 0, wealth: 0, knowledge: 0, culture: 0 }; }
        state.tiles[east(1) as usize].yields = TileYields { food: 2, production: 0, wealth: 2, knowledge: 0, culture: 0 };
        state.tiles[east(1) as usize].improvement = Some("improvement.farm".into());
        state.civilizations.get_mut(&CivId(1)).unwrap().treasury_wealth = treasury;
        state
    }

    #[test]
    fn worked_improvement_raises_city_yield_and_charges_maintenance_after_city_upkeep() {
        let state = economy_world(10);
        let next = step(&state, &[], state.seed, &versions(&state)).state;
        let city = &next.cities[&CityId(1)];
        assert_eq!(city.workplaces, vec![TileIndex(east(1))]);
        assert_eq!(city.last_yields.food, 3);
        // 10 + 2 income - 2 city upkeep (1 + ceil(1/4)) - 1 farm upkeep.
        assert_eq!(next.civilizations[&CivId(1)].treasury_wealth, 9);
        assert!(!city.essential_maintenance_unpaid);
        assert_eq!(civilization_upkeep(&next, CivId(1)), 1);
    }

    #[test]
    fn unpaid_maintenance_suspends_the_bonus_without_touching_essential_upkeep() {
        // Income 2 covers exactly the city's essential upkeep (2); nothing is left for the farm.
        let state = economy_world(0);
        let next = step(&state, &[], state.seed, &versions(&state)).state;
        let city = &next.cities[&CityId(1)];
        assert!(!city.essential_maintenance_unpaid);
        assert_eq!(city.last_yields.food, 2, "suspended farm yields only the base");
        assert_eq!(next.civilizations[&CivId(1)].treasury_wealth, 0);
        assert_eq!(next.tiles[east(1) as usize].improvement.as_deref(), Some("improvement.farm"), "suspension is not removal");
    }

    /// A worked desert saltworks (+1 wealth, upkeep 1) next to a city of population 1 (upkeep 2).
    fn saltworks_world(treasury: u32) -> WorldState {
        let mut state = economy_world(treasury);
        let tile = &mut state.tiles[east(1) as usize];
        tile.terrain = TERRAIN_DESERT;
        tile.improvement = Some("improvement.saltworks".into());
        state
    }

    #[test]
    fn a_saltworks_cannot_pay_its_own_upkeep_with_its_own_wealth() {
        // Base income 2 + treasury 0 covers exactly the essential upkeep: nothing left for the
        // saltworks, whose own +1 must not count before it is paid.
        let next = step(&saltworks_world(0), &[], 5, &versions(&saltworks_world(0))).state;
        let city = &next.cities[&CityId(1)];
        assert!(!city.essential_maintenance_unpaid);
        assert_eq!(city.last_yields.wealth, 2, "suspended saltworks yields only the base wealth");
        assert_eq!(next.civilizations[&CivId(1)].treasury_wealth, 0);
        // One more wealth in the treasury pays it; the bonus then arrives: 1 + 2 - 2 - 1 + 1.
        let paid = step(&saltworks_world(1), &[], 5, &versions(&saltworks_world(1))).state;
        assert_eq!(paid.cities[&CityId(1)].last_yields.wealth, 3);
        assert_eq!(paid.civilizations[&CivId(1)].treasury_wealth, 1);
    }

    #[test]
    fn improvements_are_hashed_and_replay_identically() {
        let state = world();
        let mut improved = state.clone();
        improved.tiles[east(1) as usize].improvement = Some("improvement.farm".into());
        assert_ne!(improved.state_hash(), state.state_hash());
        let mut building = state.clone();
        building.tiles[east(1) as usize].build = Some(TileBuild { improvement: "improvement.farm".into(), progress: 2 });
        assert_ne!(building.state_hash(), state.state_hash());

        let snapshot = WorldSnapshot::new(state.clone(), versions(&state));
        let mut log = CommandLog::default();
        let mut current = state;
        for turn in 0..5 {
            let commands = if turn == 0 { vec![build(1, 1, 1, "improvement.farm", 0)] } else { Vec::new() };
            let a = step(&current, &commands, current.seed, &versions(&current));
            let b = step(&current, &commands, current.seed, &versions(&current));
            assert_eq!(a.state_hash, b.state_hash);
            for command in commands { log.append(command); }
            log.record_turn(current.turn, a.state_hash);
            current = a.state;
        }
        assert_eq!(current.tiles[east(1) as usize].improvement.as_deref(), Some("improvement.farm"));
        let replayed = replay(&snapshot, &log).unwrap();
        assert_eq!(replayed.state, current);
        let json = serde_json::to_string(&WorldSnapshot::new(current.clone(), versions(&current))).unwrap();
        assert_eq!(serde_json::from_str::<WorldSnapshot>(&json).unwrap().state, current);
    }

    #[test]
    fn buildable_list_follows_the_validation() {
        let state = world();
        assert_eq!(buildable_improvements(&state, UnitId(1)), vec!["improvement.farm".to_string(), "improvement.pasture".to_string()]);
        let mut scout = state.clone();
        scout.units.get_mut(&UnitId(1)).unwrap().unit_type = "unit.scout".into();
        assert!(buildable_improvements(&scout, UnitId(1)).is_empty());
    }
}
