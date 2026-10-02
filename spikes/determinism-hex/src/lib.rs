//! Throwaway spike for ADR-0007. Not production code: never merged (CLAUDE.md §4).
//!
//! Questions it answers:
//! 1. Does an integer-only simulation produce the same state hash on Linux x86_64, Windows x86_64
//!    and macOS ARM64?
//! 2. Do pointy-top axial hex primitives on a cylinder (horizontal wrap, closed poles) behave?
//! 3. Rough cost per turn for 8 civs on an 8-civ map (ADR-0004, GDD 02 §6).

/// Bumped whenever the PRNG algorithm or seeding changes: replays must pin it (ADR-0006).
pub const RNG_VERSION: u32 = 1;

/// xoshiro256** seeded through splitmix64. Integer-only, platform independent.
#[derive(Clone)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut x = seed;
        let mut next = || {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Rng { s: [next(), next(), next(), next()] }
    }

    /// Derives an independent stream for a named stage, so stages can be added without
    /// shifting the random numbers of the others (GDD 02 §3: `hash(seed, stage)`).
    pub fn derive(seed: u64, stage: &str) -> Self {
        Rng::new(seed ^ fnv1a(stage.as_bytes()))
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform integer in `0..n` without modulo bias (Lemire's method, integer only).
    pub fn below(&mut self, n: u32) -> u32 {
        let mut m = (self.next_u64() >> 32) * n as u64;
        if (m as u32) < n {
            let threshold = n.wrapping_neg() % n;
            while (m as u32) < threshold {
                m = (self.next_u64() >> 32) * n as u64;
            }
        }
        (m >> 32) as u32
    }
}

/// FNV-1a 64: stable across platforms and Rust versions, unlike `std::hash::DefaultHasher`.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// Pointy-top hex grid on a cylinder: columns wrap, rows are bounded (closed poles).
/// Storage uses odd-r offset coordinates; distance converts to axial/cube.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cell {
    pub col: i32,
    pub row: i32,
}

impl Grid {
    pub fn len(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn index(&self, c: Cell) -> usize {
        (c.row * self.width + c.col) as usize
    }

    pub fn cell(&self, i: usize) -> Cell {
        Cell { col: i as i32 % self.width, row: i as i32 / self.width }
    }

    fn wrap(&self, col: i32) -> i32 {
        col.rem_euclid(self.width)
    }

    /// Neighbours in a fixed canonical order (E, NE, NW, W, SW, SE); rows outside the poles
    /// are dropped, so polar cells have fewer than six.
    pub fn neighbors(&self, c: Cell) -> Vec<Cell> {
        let odd = c.row & 1;
        let deltas: [(i32, i32); 6] = if odd == 1 {
            [(1, 0), (1, -1), (0, -1), (-1, 0), (0, 1), (1, 1)]
        } else {
            [(1, 0), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)]
        };
        deltas
            .iter()
            .filter(|(_, dr)| (0..self.height).contains(&(c.row + dr)))
            .map(|(dc, dr)| Cell { col: self.wrap(c.col + dc), row: c.row + dr })
            .collect()
    }

    fn to_axial(c: Cell) -> (i32, i32) {
        let q = c.col - (c.row - (c.row & 1)) / 2;
        (q, c.row)
    }

    /// Hex distance with horizontal wrap: the shortest of going around either way.
    pub fn distance(&self, a: Cell, b: Cell) -> i32 {
        let (aq, ar) = Self::to_axial(a);
        [-self.width, 0, self.width]
            .iter()
            .map(|shift| {
                let (bq, br) = Self::to_axial(Cell { col: b.col + shift, row: b.row });
                let dq = aq - bq;
                let dr = ar - br;
                (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
            })
            .min()
            .unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Biome {
    Ocean = 0,
    Plains = 1,
    Forest = 2,
    Hills = 3,
    Mountain = 4,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub elevation: i16,
    pub fertility: u8,
    pub biome: Biome,
    pub owner: u8, // 0 = nobody, 1..=civs
}

#[derive(Clone, Debug)]
pub struct Civ {
    pub id: u8,
    pub capital: Cell,
    pub food: i32,
    pub production: i32,
    pub population: i32,
    pub cohesion: i32,
}

#[derive(Clone)]
pub struct World {
    pub grid: Grid,
    pub tiles: Vec<Tile>,
    pub civs: Vec<Civ>,
    pub turn: u32,
    rng_events: Rng,
}

/// Integer value noise: per-cell random elevation smoothed by neighbour averaging.
fn generate_tiles(grid: Grid, seed: u64) -> Vec<Tile> {
    let mut rng = Rng::derive(seed, "elevation");
    let mut elev: Vec<i32> = (0..grid.len()).map(|_| rng.below(1000) as i32).collect();
    for _ in 0..4 {
        let prev = elev.clone();
        for i in 0..grid.len() {
            let n = grid.neighbors(grid.cell(i));
            let sum: i32 = n.iter().map(|c| prev[grid.index(*c)]).sum();
            elev[i] = (prev[i] * 2 + sum) / (2 + n.len() as i32);
        }
    }
    let mut fert = Rng::derive(seed, "fertility");
    elev.iter()
        .enumerate()
        .map(|(i, e)| {
            let row = grid.cell(i).row;
            let polar = row < 2 || row >= grid.height - 2;
            let biome = match *e {
                _ if polar => Biome::Mountain,
                e if e < 470 => Biome::Ocean,
                e if e < 520 => Biome::Plains,
                e if e < 560 => Biome::Forest,
                e if e < 600 => Biome::Hills,
                _ => Biome::Mountain,
            };
            Tile { elevation: *e as i16, fertility: fert.below(60) as u8 + 20, biome, owner: 0 }
        })
        .collect()
}

impl World {
    pub fn new(seed: u64, civs: u8, grid: Grid) -> Self {
        let mut tiles = generate_tiles(grid, seed);
        let mut rng = Rng::derive(seed, "starts");
        let land: Vec<usize> =
            (0..grid.len()).filter(|i| tiles[*i].biome == Biome::Plains).collect();
        let mut capitals: Vec<Cell> = Vec::new();
        let mut guard = 0;
        while capitals.len() < civs as usize && guard < 100_000 {
            guard += 1;
            let c = grid.cell(land[rng.below(land.len() as u32) as usize]);
            if capitals.iter().all(|o| grid.distance(*o, c) >= 8) {
                capitals.push(c);
            }
        }
        let civs: Vec<Civ> = capitals
            .iter()
            .enumerate()
            .map(|(i, c)| {
                tiles[grid.index(*c)].owner = i as u8 + 1;
                Civ { id: i as u8 + 1, capital: *c, food: 10, production: 0, population: 2, cohesion: 60 }
            })
            .collect();
        World { grid, tiles, civs, turn: 0, rng_events: Rng::derive(seed, "events") }
    }

    fn yield_of(t: &Tile) -> (i32, i32) {
        match t.biome {
            Biome::Ocean => (1, 0),
            Biome::Plains => (2 + t.fertility as i32 / 40, 1),
            Biome::Forest => (1, 2),
            Biome::Hills => (1, 2),
            Biome::Mountain => (0, 1),
        }
    }

    /// Toy `step`: pure integer rules, canonical iteration order (civ id, then tile index).
    pub fn step(&mut self) {
        self.turn += 1;
        let grid = self.grid;
        for ci in 0..self.civs.len() {
            let id = self.civs[ci].id;
            // Work the best owned tiles, one per population point, ties broken by index.
            let mut owned: Vec<(i32, usize)> = (0..grid.len())
                .filter(|i| self.tiles[*i].owner == id)
                .map(|i| {
                    let (f, p) = Self::yield_of(&self.tiles[i]);
                    (-(f * 2 + p), i)
                })
                .collect();
            owned.sort();
            let worked = owned.iter().take(self.civs[ci].population.max(1) as usize);
            let (mut food, mut prod) = (0, 0);
            for (_, i) in worked {
                let (f, p) = Self::yield_of(&self.tiles[*i]);
                food += f;
                prod += p;
            }
            let civ = &mut self.civs[ci];
            civ.food += food - civ.population;
            civ.production += prod;
            if civ.food > civ.population * 3 + 4 {
                civ.population += 1;
                civ.food = civ.population;
            } else if civ.food < 0 {
                civ.population = (civ.population - 1).max(1);
                civ.food = 0;
                civ.cohesion = (civ.cohesion - 2).max(0);
            }
            // Expand: claim the cheapest unowned neighbour of the territory.
            if civ.production >= 8 {
                let capital = civ.capital;
                let mut frontier: Vec<(i32, usize)> = Vec::new();
                for (_, i) in owned.iter() {
                    for n in grid.neighbors(grid.cell(*i)) {
                        let j = grid.index(n);
                        if self.tiles[j].owner == 0 && self.tiles[j].biome != Biome::Ocean {
                            frontier.push((grid.distance(capital, n), j));
                        }
                    }
                }
                frontier.sort();
                if let Some((_, j)) = frontier.first() {
                    self.tiles[*j].owner = id;
                    self.civs[ci].production -= 8;
                }
            }
        }
        // Event: a seeded drought on one random tile per turn (stands in for Entropy).
        let i = self.rng_events.below(grid.len() as u32) as usize;
        let t = &mut self.tiles[i];
        t.fertility = t.fertility.saturating_sub(5);
    }

    /// Canonical serialization -> FNV-1a. Field order is fixed; no maps, no floats.
    pub fn hash(&self) -> u64 {
        let mut buf: Vec<u8> = Vec::with_capacity(self.tiles.len() * 4 + 64);
        buf.extend_from_slice(&RNG_VERSION.to_le_bytes());
        buf.extend_from_slice(&self.turn.to_le_bytes());
        for t in &self.tiles {
            buf.extend_from_slice(&t.elevation.to_le_bytes());
            buf.push(t.fertility);
            buf.push(t.biome as u8);
            buf.push(t.owner);
        }
        for c in &self.civs {
            buf.push(c.id);
            for v in [c.capital.col, c.capital.row, c.food, c.production, c.population, c.cohesion] {
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
        fnv1a(&buf)
    }
}

/// Grid sized by GDD 02 §6: ~70 land tiles per civ at ~35% land.
pub fn grid_for(civs: u8) -> Grid {
    let total = (civs as i32 * 70 * 100 / 35).max(600);
    let width = ((total as f64).sqrt() as i32).max(20); // only used to size the map, not in rules
    Grid { width, height: (total + width - 1) / width }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_reproducible() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::new(7);
        for n in [1u32, 2, 3, 6, 1000, u32::MAX] {
            for _ in 0..200 {
                assert!(r.below(n) < n);
            }
        }
    }

    #[test]
    fn interior_cells_have_six_neighbours_and_wrap() {
        let g = Grid { width: 10, height: 8 };
        for i in 0..g.len() {
            let c = g.cell(i);
            let n = g.neighbors(c);
            if c.row > 0 && c.row < g.height - 1 {
                assert_eq!(n.len(), 6, "{c:?}");
            }
            for m in &n {
                assert!((0..g.width).contains(&m.col));
                assert_eq!(g.distance(c, *m), 1, "{c:?} -> {m:?}");
            }
        }
    }

    #[test]
    fn distance_is_symmetric_and_wraps() {
        let g = Grid { width: 12, height: 10 };
        let a = Cell { col: 0, row: 4 };
        let b = Cell { col: 11, row: 4 };
        assert_eq!(g.distance(a, b), 1, "east edge touches west edge");
        for i in 0..g.len() {
            for j in (0..g.len()).step_by(7) {
                let (x, y) = (g.cell(i), g.cell(j));
                assert_eq!(g.distance(x, y), g.distance(y, x));
                assert!(g.distance(x, y) <= g.width / 2 + g.height);
            }
        }
    }

    #[test]
    fn same_seed_same_hash_different_seed_differs() {
        let run = |seed| {
            let mut w = World::new(seed, 8, grid_for(8));
            for _ in 0..200 {
                w.step();
            }
            w.hash()
        };
        assert_eq!(run(1), run(1));
        assert_ne!(run(1), run(2));
    }
}
