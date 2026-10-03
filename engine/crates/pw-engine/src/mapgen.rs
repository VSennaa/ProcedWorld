//! Deterministic, catalog-backed initial world generation.
//!
//! Catalog loading is deliberately kept at the boundary. Once a [`Catalog`] has
//! been constructed, [`generate`] is a pure integer-only operation.

use std::{fs, path::Path};

use serde::Deserialize;

use crate::{
    hash::{fnv1a, StateHash, StateHasher},
    hex::Grid,
    ids::TileIndex,
    rng::Rng,
    world::RulesetRef,
};

pub const GENERATOR_VERSION: u32 = 1;
pub const MIN_TOTAL_TILES: u32 = 600;
pub const LAND_PERCENT_MIN: u32 = 30;
pub const LAND_PERCENT_MAX: u32 = 40;
pub const LAND_PERCENT_TARGET: u32 = 35;
pub const LAND_TILES_PER_CIVILIZATION: u32 = 70;
pub const CAPITAL_MIN_DISTANCE: u32 = 8;
pub const FAIRNESS_MAX_DIFFERENCE_PERCENT: u32 = 15;
pub const MAX_GENERATION_ATTEMPTS: u32 = 24;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogError {
    Read,
    Parse,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerationError {
    InvalidParameters,
    InvalidCatalog,
    AttemptsExhausted,
}

#[derive(Clone, Debug, Deserialize)]
struct CatalogFile<T> {
    catalog_id: String,
    catalog_version: u32,
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    status: String,
    #[serde(flatten)]
    contents: T,
}

#[derive(Clone, Debug, Deserialize)]
struct BiomeFile {
    biomes: Vec<BiomeDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
struct ResourceFile {
    resources: Vec<ResourceDefinition>,
}

#[derive(Clone, Debug, Deserialize)]
struct YieldDefinition {
    food: i16,
    production: i16,
    wealth: i16,
}

#[derive(Clone, Debug, Deserialize)]
struct BiomeDefinition {
    id: String,
    yields: YieldDefinition,
}

#[derive(Clone, Debug, Deserialize)]
struct ResourceDefinition {
    id: String,
    category: String,
    #[serde(default)]
    biomes: Vec<String>,
    #[serde(default)]
    stock_min: u16,
    #[serde(default)]
    stock_max: u16,
}

/// Validated data required by map generation, with a stable content identity.
#[derive(Clone, Debug)]
pub struct Catalog {
    biomes: Vec<BiomeDefinition>,
    resources: Vec<ResourceDefinition>,
    pub ruleset_ref: RulesetRef,
}

impl Catalog {
    pub fn load(data_root: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let root = data_root.as_ref();
        let biome_source = fs::read_to_string(root.join("biomes.json")).map_err(|_| CatalogError::Read)?;
        let resource_source = fs::read_to_string(root.join("resources.json")).map_err(|_| CatalogError::Read)?;
        Self::from_json(&biome_source, &resource_source)
    }

    pub fn from_json(biome_source: &str, resource_source: &str) -> Result<Self, CatalogError> {
        let biome_file: CatalogFile<BiomeFile> = serde_json::from_str(biome_source).map_err(|_| CatalogError::Parse)?;
        let resource_file: CatalogFile<ResourceFile> = serde_json::from_str(resource_source).map_err(|_| CatalogError::Parse)?;
        if biome_file.catalog_id.is_empty()
            || resource_file.catalog_id.is_empty()
            || biome_file.contents.biomes.is_empty()
            || resource_file.contents.resources.is_empty()
            || !biome_file.contents.biomes.iter().any(|biome| biome.id == "ocean")
        {
            return Err(CatalogError::Invalid);
        }
        let mut content = Vec::new();
        content.extend_from_slice(biome_file.catalog_id.as_bytes());
        content.extend_from_slice(&biome_file.catalog_version.to_le_bytes());
        content.extend_from_slice(&biome_file.schema_version.to_le_bytes());
        content.extend_from_slice(biome_file.status.as_bytes());
        content.extend_from_slice(biome_source.as_bytes());
        content.extend_from_slice(resource_file.catalog_id.as_bytes());
        content.extend_from_slice(&resource_file.catalog_version.to_le_bytes());
        content.extend_from_slice(&resource_file.schema_version.to_le_bytes());
        content.extend_from_slice(resource_file.status.as_bytes());
        content.extend_from_slice(resource_source.as_bytes());
        Ok(Self {
            biomes: biome_file.contents.biomes,
            resources: resource_file.contents.resources,
            ruleset_ref: RulesetRef {
                id: "core.mapgen".into(),
                version: format!("{}.{}", biome_file.catalog_version, resource_file.catalog_version),
                content_hash: fnv1a(&content),
            },
        })
    }

    fn biome(&self, id: &str) -> Option<&BiomeDefinition> {
        self.biomes.iter().find(|biome| biome.id == id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldParams {
    pub civilizations: u32,
    pub max_attempts: u32,
}

impl WorldParams {
    pub fn for_civilizations(civilizations: u32) -> Self {
        Self { civilizations, max_attempts: MAX_GENERATION_ATTEMPTS }
    }

    /// Computes a near-square shape with `max(600, ceil(civs * 70 / 0.35))` tiles.
    pub fn shape(self) -> Result<Grid, GenerationError> {
        if self.civilizations == 0 {
            return Err(GenerationError::InvalidParameters);
        }
        let land = self.civilizations.checked_mul(LAND_TILES_PER_CIVILIZATION).ok_or(GenerationError::InvalidParameters)?;
        let total = MIN_TOTAL_TILES.max(ceil_div(land.checked_mul(100).ok_or(GenerationError::InvalidParameters)?, LAND_PERCENT_TARGET));
        let width = integer_sqrt(total).max(1);
        let height = ceil_div(total, width);
        Grid::new(width, height).map_err(|_| GenerationError::InvalidParameters)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDeposit {
    pub id: String,
    pub category: String,
    pub stock: u16,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedTile {
    pub elevation: i8,
    pub temperature: u8,
    pub moisture: u8,
    pub biome_id: String,
    pub resource: Option<ResourceDeposit>,
}

impl GeneratedTile {
    pub fn is_land(&self) -> bool { self.elevation >= 0 }
}

/// A river is represented authoritatively by its tile-pair edges, never a tile flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RiverEdge {
    pub low: TileIndex,
    pub high: TileIndex,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct River {
    pub edges: Vec<RiverEdge>,
    pub mouth: TileIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartingPoint {
    pub civilization: u32,
    pub tile: TileIndex,
    pub fairness_score: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedMap {
    pub grid: Grid,
    pub generator_version: u32,
    pub generation_attempt: u32,
    pub ruleset_ref: RulesetRef,
    pub tiles: Vec<GeneratedTile>,
    pub rivers: Vec<River>,
    pub starting_points: Vec<StartingPoint>,
}

impl GeneratedMap {
    pub fn state_hash(&self) -> StateHash {
        let mut hash = StateHasher::with_capacity(self.tiles.len() * 12);
        hash.write_u32(self.grid.width);
        hash.write_u32(self.grid.height);
        hash.write_u32(self.generator_version);
        hash.write_u32(self.generation_attempt);
        write_string(&mut hash, &self.ruleset_ref.id);
        write_string(&mut hash, &self.ruleset_ref.version);
        hash.write_u64(self.ruleset_ref.content_hash);
        hash.write_u32(self.tiles.len() as u32);
        for tile in &self.tiles {
            hash.write_i8(tile.elevation);
            hash.write_u8(tile.temperature);
            hash.write_u8(tile.moisture);
            write_string(&mut hash, &tile.biome_id);
            hash.write_bool(tile.resource.is_some());
            if let Some(resource) = &tile.resource {
                write_string(&mut hash, &resource.id);
                write_string(&mut hash, &resource.category);
                hash.write_u16(resource.stock);
                hash.write_bool(resource.hidden);
            }
        }
        hash.write_u32(self.rivers.len() as u32);
        for river in &self.rivers {
            hash.write_u32(river.edges.len() as u32);
            for edge in &river.edges { hash.write_u32(edge.low.0); hash.write_u32(edge.high.0); }
            hash.write_u32(river.mouth.0);
        }
        hash.write_u32(self.starting_points.len() as u32);
        for start in &self.starting_points {
            hash.write_u32(start.civilization);
            hash.write_u32(start.tile.0);
            hash.write_u32(start.fairness_score);
        }
        hash.finish()
    }

    pub fn land_percent(&self) -> u32 {
        (self.tiles.iter().filter(|tile| tile.is_land()).count() as u32) * 100 / self.tiles.len() as u32
    }
}

pub fn generate(params: WorldParams, seed: u64, catalog: &Catalog) -> Result<GeneratedMap, GenerationError> {
    let grid = params.shape()?;
    if catalog.biome("ocean").is_none() { return Err(GenerationError::InvalidCatalog); }
    for attempt in 0..params.max_attempts.max(1) {
        if let Some(map) = generate_attempt(grid, params.civilizations, seed, attempt, catalog) {
            return Ok(map);
        }
    }
    Err(GenerationError::AttemptsExhausted)
}

fn generate_attempt(grid: Grid, civilizations: u32, seed: u64, attempt: u32, catalog: &Catalog) -> Option<GeneratedMap> {
    let mut land_rng = stage_rng(seed, attempt, "continents");
    let mut noise: Vec<u32> = (0..grid.len()).map(|_| land_rng.below(101)).collect();
    for _ in 0..4 {
        let previous = noise.clone();
        for index in 0..grid.len() {
            let cell = grid.cell(TileIndex(index as u32)).ok()?;
            let mut sum = previous[index];
            let mut count = 1_u32;
            for neighbor in grid.neighbors(cell).ok()? {
                sum += previous[grid.tile_index(neighbor).ok()?.0 as usize];
                count += 1;
            }
            noise[index] = sum / count;
        }
    }
    let target_percent = LAND_PERCENT_MIN + land_rng.below(LAND_PERCENT_MAX - LAND_PERCENT_MIN + 1);
    let land_count = ceil_div(grid.len() as u32 * target_percent, 100) as usize;
    let mut ranked: Vec<(u32, usize)> = noise.iter().copied().enumerate().map(|(index, value)| (value, index)).collect();
    ranked.sort_unstable_by(|left, right| right.cmp(left));
    let mut land = vec![false; grid.len()];
    for (_, index) in ranked.into_iter().take(land_count) { land[index] = true; }

    let mut relief_rng = stage_rng(seed, attempt, "relief");
    let mut tiles = Vec::with_capacity(grid.len());
    for index in 0..grid.len() {
        let elevation = if land[index] { 1 + (noise[index] / 20) as i8 + relief_rng.below(3) as i8 } else { -2 };
        tiles.push(GeneratedTile { elevation, temperature: 0, moisture: 0, biome_id: "ocean".into(), resource: None });
    }
    apply_climate(grid, &mut tiles, seed, attempt);
    classify_biomes(grid, &mut tiles);
    let rivers = generate_rivers(grid, &tiles, seed, attempt);
    place_resources(&mut tiles, seed, attempt, catalog);
    let starting_points = choose_starts(grid, &tiles, civilizations, seed, attempt, catalog)?;
    Some(GeneratedMap { grid, generator_version: GENERATOR_VERSION, generation_attempt: attempt, ruleset_ref: catalog.ruleset_ref.clone(), tiles, rivers, starting_points })
}

fn apply_climate(grid: Grid, tiles: &mut [GeneratedTile], seed: u64, attempt: u32) {
    let mut rng = stage_rng(seed, attempt, "climate");
    let latitude_denominator = (grid.height.saturating_sub(1)).max(1);
    for index in 0..tiles.len() {
        let row = index as u32 / grid.width;
        let pole_distance = (row.saturating_mul(2)).abs_diff(latitude_denominator);
        let latitude_cold = pole_distance.saturating_mul(55) / latitude_denominator;
        let altitude_cold = tiles[index].elevation.max(0) as u32 * 6;
        tiles[index].temperature = (100_i32 - latitude_cold as i32 - altitude_cold as i32 + rng.below(11) as i32 - 5).clamp(0, 100) as u8;
    }
    let mut moisture_rng = stage_rng(seed, attempt, "moisture");
    for index in 0..tiles.len() {
        if !tiles[index].is_land() { tiles[index].moisture = 100; continue; }
        let cell = grid.cell(TileIndex(index as u32)).expect("index is in the grid");
        let mut nearest_water = grid.len() as u32;
        for water_index in 0..tiles.len() {
            if !tiles[water_index].is_land() {
                let water = grid.cell(TileIndex(water_index as u32)).expect("index is in the grid");
                nearest_water = nearest_water.min(grid.distance(cell, water).expect("valid cells"));
            }
        }
        let value = 88_i32 - nearest_water as i32 * 12 - tiles[index].elevation.max(0) as i32 * 3 + moisture_rng.below(17) as i32 - 8;
        tiles[index].moisture = value.clamp(0, 100) as u8;
    }
}

fn classify_biomes(grid: Grid, tiles: &mut [GeneratedTile]) {
    for index in 0..tiles.len() {
        if !tiles[index].is_land() { tiles[index].biome_id = "ocean".into(); continue; }
        let cell = grid.cell(TileIndex(index as u32)).expect("index is in the grid");
        let coastal = grid.neighbors(cell).expect("valid cell").into_iter().any(|neighbor| !tiles[grid.tile_index(neighbor).expect("valid neighbor").0 as usize].is_land());
        let tile = &mut tiles[index];
        tile.biome_id = if coastal { "coast" } else if tile.elevation >= 6 { "mountain" } else if tile.temperature <= 12 { "glacier" } else if tile.temperature <= 25 { "tundra" } else if tile.moisture >= 82 { "swamp" } else if tile.temperature >= 70 && tile.moisture >= 62 { "jungle" } else if tile.moisture >= 58 { "forest" } else if tile.moisture <= 22 { "desert" } else if tile.temperature >= 62 { "savanna" } else if tile.moisture <= 38 { "steppe" } else { "plains" }.into();
    }
}

fn generate_rivers(grid: Grid, tiles: &[GeneratedTile], seed: u64, attempt: u32) -> Vec<River> {
    let mut rng = stage_rng(seed, attempt, "rivers");
    let mut rivers = Vec::new();
    for index in 0..tiles.len() {
        if !tiles[index].is_land() || tiles[index].elevation < 4 || tiles[index].moisture < 55 || rng.below(100) >= 22 { continue; }
        let mut current = TileIndex(index as u32);
        let mut visited = vec![current];
        let mut edges = Vec::new();
        for _ in 0..tiles.len() {
            if !tiles[current.0 as usize].is_land() { break; }
            let cell = grid.cell(current).expect("valid current tile");
            let next = grid.neighbors(cell).expect("valid current tile").into_iter()
                .map(|neighbor| grid.tile_index(neighbor).expect("valid neighbor"))
                .filter(|neighbor| tiles[neighbor.0 as usize].elevation < tiles[current.0 as usize].elevation)
                .min_by_key(|neighbor| (tiles[neighbor.0 as usize].elevation, neighbor.0));
            let Some(next) = next else { break; };
            if visited.contains(&next) { break; }
            edges.push(normalize_edge(current, next));
            current = next;
            visited.push(current);
        }
        if !edges.is_empty() && !tiles[current.0 as usize].is_land() {
            rivers.push(River { edges, mouth: current });
        }
    }
    rivers
}

fn place_resources(tiles: &mut [GeneratedTile], seed: u64, attempt: u32, catalog: &Catalog) {
    let mut rng = stage_rng(seed, attempt, "resources");
    let land_count = tiles.iter().filter(|tile| tile.is_land()).count() as u32;
    let scarce_limit = (land_count / 40).max(1);
    let mut scarce_count = 0_u32;
    for tile in tiles.iter_mut() {
        if !tile.is_land() || rng.below(100) >= 38 { continue; }
        let choices: Vec<&ResourceDefinition> = catalog.resources.iter().filter(|resource| resource.biomes.iter().any(|biome| biome == &tile.biome_id)).collect();
        if choices.is_empty() { continue; }
        let resource = choices[rng.below(choices.len() as u32) as usize];
        let scarce = resource.category == "strategic" || resource.category == "luxury";
        if scarce && scarce_count >= scarce_limit { continue; }
        if scarce { scarce_count += 1; }
        let stock = if resource.stock_max >= resource.stock_min && resource.stock_max > 0 { resource.stock_min + rng.below(u32::from(resource.stock_max - resource.stock_min) + 1) as u16 } else { 0 };
        tile.resource = Some(ResourceDeposit { id: resource.id.clone(), category: resource.category.clone(), stock, hidden: resource.category == "finite" || resource.category == "strategic" });
    }
}

fn choose_starts(grid: Grid, tiles: &[GeneratedTile], civilizations: u32, seed: u64, attempt: u32, catalog: &Catalog) -> Option<Vec<StartingPoint>> {
    let mut candidates = Vec::new();
    for index in 0..tiles.len() {
        if !tiles[index].is_land() { continue; }
        let score = start_score(grid, tiles, TileIndex(index as u32), catalog)?;
        if score > 0 { candidates.push((score, TileIndex(index as u32))); }
    }
    candidates.sort_unstable_by(|left, right| right.cmp(left));
    let best = candidates.first()?.0;
    let eligible: Vec<_> = candidates.into_iter().filter(|(score, _)| (best - *score) * 100 <= best * FAIRNESS_MAX_DIFFERENCE_PERCENT).collect();
    let mut rng = stage_rng(seed, attempt, "starting-points");
    let mut picked = Vec::new();
    let offset = rng.below(eligible.len() as u32) as usize;
    picked.push(eligible[offset]);
    while picked.len() < civilizations as usize {
        let next = eligible.iter().copied()
            .filter(|candidate| !picked.contains(candidate))
            .filter_map(|candidate| {
                let candidate_cell = grid.cell(candidate.1).ok()?;
                let nearest = picked.iter().map(|(_, other)| grid.distance(candidate_cell, grid.cell(*other).expect("valid selected tile")).expect("valid selected tile")).min()?;
                (nearest >= CAPITAL_MIN_DISTANCE).then_some((nearest, candidate))
            })
            .max_by_key(|(nearest, candidate)| (*nearest, candidate.0, std::cmp::Reverse(candidate.1.0)))
            .map(|(_, candidate)| candidate);
        let Some(next) = next else { return None; };
        picked.push(next);
    }
    if picked.len() != civilizations as usize { return None; }
    let min_score = picked.iter().map(|(score, _)| *score).min()?;
    let max_score = picked.iter().map(|(score, _)| *score).max()?;
    if (max_score - min_score) * 100 > max_score * FAIRNESS_MAX_DIFFERENCE_PERCENT { return None; }
    Some(picked.into_iter().enumerate().map(|(civilization, (fairness_score, tile))| StartingPoint { civilization: civilization as u32, tile, fairness_score }).collect())
}

fn start_score(grid: Grid, tiles: &[GeneratedTile], center: TileIndex, catalog: &Catalog) -> Option<u32> {
    let center_cell = grid.cell(center).ok()?;
    let mut score = 0_u32;
    for cell in grid.area(center_cell, 2).ok()? {
        let tile = &tiles[grid.tile_index(cell).ok()?.0 as usize];
        let biome = catalog.biome(&tile.biome_id)?;
        score += biome.yields.food.max(0) as u32 * 3;
        score += biome.yields.production.max(0) as u32 * 2;
        score += biome.yields.wealth.max(0) as u32;
    }
    Some(score)
}

fn stage_rng(seed: u64, attempt: u32, stage: &str) -> Rng {
    Rng::derive(seed ^ u64::from(attempt).wrapping_mul(0x9e37_79b9_7f4a_7c15), stage)
}

fn normalize_edge(left: TileIndex, right: TileIndex) -> RiverEdge {
    RiverEdge { low: left.min(right), high: left.max(right) }
}

fn integer_sqrt(value: u32) -> u32 {
    let mut root = 0_u32;
    while (root + 1).checked_mul(root + 1).is_some_and(|square| square <= value) { root += 1; }
    root
}

fn ceil_div(value: u32, divisor: u32) -> u32 {
    value / divisor + u32::from(value % divisor != 0)
}

fn write_string(hash: &mut StateHasher, value: &str) {
    hash.write_u32(value.len() as u32);
    hash.write_bytes(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> Catalog {
        Catalog::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data/catalogs")).unwrap()
    }

    fn map() -> GeneratedMap {
        generate(WorldParams::for_civilizations(4), 0x5eed, &catalog()).unwrap()
    }

    #[test]
    fn same_seed_produces_same_map_hash() {
        let catalog = catalog();
        let parameters = WorldParams::for_civilizations(4);
        assert_eq!(generate(parameters, 77, &catalog).unwrap().state_hash(), generate(parameters, 77, &catalog).unwrap().state_hash());
    }

    #[test]
    fn land_fraction_is_within_the_gdd_range() {
        let generated = map();
        assert!((LAND_PERCENT_MIN..=LAND_PERCENT_MAX).contains(&generated.land_percent()));
    }

    #[test]
    fn capitals_are_land_and_far_apart() {
        let generated = map();
        for start in &generated.starting_points {
            assert!(generated.tiles[start.tile.0 as usize].is_land());
            for other in &generated.starting_points {
                if start.civilization != other.civilization {
                    assert!(generated.grid.distance(generated.grid.cell(start.tile).unwrap(), generated.grid.cell(other.tile).unwrap()).unwrap() >= CAPITAL_MIN_DISTANCE);
                }
            }
        }
    }

    #[test]
    fn capital_scores_obey_the_fairness_limit() {
        let generated = map();
        let minimum = generated.starting_points.iter().map(|start| start.fairness_score).min().unwrap();
        let maximum = generated.starting_points.iter().map(|start| start.fairness_score).max().unwrap();
        assert!((maximum - minimum) * 100 <= maximum * FAIRNESS_MAX_DIFFERENCE_PERCENT);
    }

    #[test]
    fn rivers_end_in_water() {
        let generated = map();
        for river in &generated.rivers { assert!(!generated.tiles[river.mouth.0 as usize].is_land()); }
    }

    #[test]
    fn every_capital_has_food_in_radius_two() {
        let generated = map();
        let catalog = catalog();
        for start in &generated.starting_points {
            let food = generated.grid.area(generated.grid.cell(start.tile).unwrap(), 2).unwrap().into_iter().map(|cell| {
                let tile = &generated.tiles[generated.grid.tile_index(cell).unwrap().0 as usize];
                catalog.biome(&tile.biome_id).unwrap().yields.food.max(0) as u32
            }).sum::<u32>();
            assert!(food > 0);
        }
    }
}
