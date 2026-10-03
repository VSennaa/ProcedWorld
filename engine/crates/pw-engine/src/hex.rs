//! Pointy-top hexagonal geometry on a horizontally wrapped cylinder.

use crate::ids::TileIndex;

/// Odd-r offset coordinate used by dense map storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
    pub col: i32,
    pub row: i32,
}

/// Axial coordinate, with cube `s` derived as `-q-r`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

impl Axial {
    pub const fn s(self) -> i32 {
        -self.q - self.r
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapError {
    InvalidDimensions,
    InvalidCell(Cell),
    InvalidTileIndex(TileIndex),
}

/// Finite pointy-top grid with horizontal wrap and closed northern/southern poles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Result<Self, MapError> {
        if width == 0
            || height == 0
            || width > i32::MAX as u32
            || height > i32::MAX as u32
            || width.checked_mul(height).is_none()
        {
            return Err(MapError::InvalidDimensions);
        }
        Ok(Self { width, height })
    }

    pub fn len(self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    pub fn normalize(self, cell: Cell) -> Result<Cell, MapError> {
        if !(0..self.height as i32).contains(&cell.row) {
            return Err(MapError::InvalidCell(cell));
        }
        Ok(Cell { col: cell.col.rem_euclid(self.width as i32), row: cell.row })
    }

    pub fn tile_index(self, cell: Cell) -> Result<TileIndex, MapError> {
        let cell = self.normalize(cell)?;
        Ok(TileIndex((cell.row as u32) * self.width + cell.col as u32))
    }

    pub fn cell(self, index: TileIndex) -> Result<Cell, MapError> {
        if index.0 >= self.width * self.height {
            return Err(MapError::InvalidTileIndex(index));
        }
        Ok(Cell { col: (index.0 % self.width) as i32, row: (index.0 / self.width) as i32 })
    }

    pub fn to_axial(self, cell: Cell) -> Result<Axial, MapError> {
        let cell = self.normalize(cell)?;
        Ok(Axial { q: cell.col - (cell.row - (cell.row & 1)) / 2, r: cell.row })
    }

    pub fn from_axial(self, axial: Axial) -> Result<Cell, MapError> {
        let col = axial.q + (axial.r - (axial.r & 1)) / 2;
        self.normalize(Cell { col, row: axial.r })
    }

    /// Neighbours follow canonical E, NE, NW, W, SW, SE order. Invalid polar
    /// crossings are omitted.
    pub fn neighbors(self, cell: Cell) -> Result<Vec<Cell>, MapError> {
        let cell = self.normalize(cell)?;
        let deltas = if cell.row & 1 == 1 {
            [(1, 0), (1, -1), (0, -1), (-1, 0), (0, 1), (1, 1)]
        } else {
            [(1, 0), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)]
        };
        let mut result = Vec::with_capacity(6);
        for (dc, dr) in deltas {
            if let Ok(neighbor) = self.normalize(Cell { col: cell.col + dc, row: cell.row + dr }) {
                if neighbor != cell && !result.contains(&neighbor) {
                    result.push(neighbor);
                }
            }
        }
        Ok(result)
    }

    pub fn distance(self, left: Cell, right: Cell) -> Result<u32, MapError> {
        let left = self.to_axial(left)?;
        let right = self.to_axial(right)?;
        let mut shortest = i64::MAX;
        for shift in [-i64::from(self.width), 0, i64::from(self.width)] {
            let dq = i64::from(left.q) - (i64::from(right.q) + shift);
            let dr = i64::from(left.r) - i64::from(right.r);
            let distance = (dq.abs() + dr.abs() + (dq + dr).abs()) / 2;
            shortest = shortest.min(distance);
        }
        Ok(shortest as u32)
    }

    /// All cells exactly `radius` steps away, in canonical distance/index order.
    pub fn ring(self, center: Cell, radius: u32) -> Result<Vec<Cell>, MapError> {
        self.collect_radius(center, radius, false)
    }

    /// All cells at most `radius` steps away, in canonical distance/index order.
    pub fn area(self, center: Cell, radius: u32) -> Result<Vec<Cell>, MapError> {
        self.collect_radius(center, radius, true)
    }

    fn collect_radius(self, center: Cell, radius: u32, include_inner: bool) -> Result<Vec<Cell>, MapError> {
        let center = self.normalize(center)?;
        let mut cells = Vec::new();
        for raw_index in 0..(self.width * self.height) {
            let cell = self.cell(TileIndex(raw_index))?;
            let distance = self.distance(center, cell)?;
            if distance == radius || (include_inner && distance <= radius) {
                cells.push((distance, raw_index, cell));
            }
        }
        cells.sort_unstable_by_key(|(distance, index, _)| (*distance, *index));
        Ok(cells.into_iter().map(|(_, _, cell)| cell).collect())
    }

    /// Returns an inclusive, deterministic shortest path. Ties use the lowest
    /// tile index, so no floating-point interpolation is needed.
    pub fn line(self, start: Cell, end: Cell) -> Result<Vec<Cell>, MapError> {
        let start = self.normalize(start)?;
        let end = self.normalize(end)?;
        let mut path = vec![start];
        let mut current = start;
        while current != end {
            let remaining = self.distance(current, end)?;
            let next = self
                .neighbors(current)?
                .into_iter()
                .filter(|candidate| self.distance(*candidate, end).ok() == Some(remaining - 1))
                .min_by_key(|candidate| self.tile_index(*candidate).expect("neighbours are valid"))
                .expect("a valid finite hex grid always has a descending shortest-path neighbour");
            current = next;
            path.push(current);
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Grid {
        Grid::new(12, 10).unwrap()
    }

    #[test]
    fn interior_cells_have_six_neighbors_and_neighbors_are_adjacent() {
        let grid = grid();
        for raw_index in 0..grid.len() as u32 {
            let cell = grid.cell(TileIndex(raw_index)).unwrap();
            let neighbors = grid.neighbors(cell).unwrap();
            if cell.row > 0 && cell.row < grid.height as i32 - 1 {
                assert_eq!(neighbors.len(), 6, "{cell:?}");
            }
            for neighbor in neighbors {
                assert_eq!(grid.distance(cell, neighbor), Ok(1), "{cell:?} -> {neighbor:?}");
            }
        }
    }

    #[test]
    fn distance_is_symmetric_and_wraps() {
        let grid = grid();
        let left = Cell { col: 0, row: 4 };
        let right = Cell { col: 11, row: 4 };
        assert_eq!(grid.distance(left, right), Ok(1));
        for first in 0..grid.len() as u32 {
            for second in (0..grid.len() as u32).step_by(7) {
                let a = grid.cell(TileIndex(first)).unwrap();
                let b = grid.cell(TileIndex(second)).unwrap();
                assert_eq!(grid.distance(a, b), grid.distance(b, a));
            }
        }
    }

    #[test]
    fn ring_and_area_have_expected_sizes_away_from_edges() {
        let grid = Grid::new(20, 20).unwrap();
        let center = Cell { col: 10, row: 10 };
        assert_eq!(grid.ring(center, 0).unwrap(), vec![center]);
        assert_eq!(grid.ring(center, 3).unwrap().len(), 18);
        assert_eq!(grid.area(center, 3).unwrap().len(), 37);
    }

    #[test]
    fn ring_and_area_are_unique_and_respect_closed_poles() {
        let grid = Grid::new(7, 4).unwrap();
        let center = Cell { col: 0, row: 0 };
        let ring = grid.ring(center, 2).unwrap();
        let area = grid.area(center, 10).unwrap();
        assert!(ring.iter().all(|cell| grid.distance(center, *cell) == Ok(2)));
        assert_eq!(area.len(), grid.len());
        for cell in &area {
            assert!((0..grid.height as i32).contains(&cell.row));
        }
    }

    #[test]
    fn axial_round_trip_and_line_are_deterministic() {
        let grid = grid();
        let start = Cell { col: 0, row: 4 };
        let end = Cell { col: 7, row: 7 };
        assert_eq!(grid.from_axial(grid.to_axial(start).unwrap()), Ok(start));
        let line = grid.line(start, end).unwrap();
        assert_eq!(line.first(), Some(&start));
        assert_eq!(line.last(), Some(&end));
        for pair in line.windows(2) {
            assert_eq!(grid.distance(pair[0], pair[1]), Ok(1));
        }
    }
}
