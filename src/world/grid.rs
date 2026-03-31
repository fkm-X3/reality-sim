use std::collections::HashMap;
use uuid::Uuid;

use crate::agent::Vec2;

/// Spatial partitioning grid for efficient proximity queries
#[derive(Debug, Clone)]
pub struct SpatialGrid {
    pub width: f32,
    pub height: f32,
    pub cell_size: f32,
    pub cols: usize,
    pub rows: usize,
    /// Cell -> Agent IDs
    cells: Vec<Vec<Uuid>>,
    /// Agent ID -> Cell index
    agent_cells: HashMap<Uuid, usize>,
}

impl SpatialGrid {
    /// Create a new spatial grid
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cols = (width / cell_size).ceil() as usize;
        let rows = (height / cell_size).ceil() as usize;
        let cells = vec![Vec::new(); cols * rows];

        Self {
            width,
            height,
            cell_size,
            cols,
            rows,
            cells,
            agent_cells: HashMap::new(),
        }
    }

    /// Get cell index for a position
    #[inline]
    pub fn cell_index(&self, pos: Vec2) -> usize {
        let col = ((pos.x / self.cell_size).floor() as usize).min(self.cols - 1);
        let row = ((pos.y / self.cell_size).floor() as usize).min(self.rows - 1);
        row * self.cols + col
    }

    /// Insert an agent into the grid
    pub fn insert(&mut self, id: Uuid, pos: Vec2) {
        let cell = self.cell_index(pos);
        self.cells[cell].push(id);
        self.agent_cells.insert(id, cell);
    }

    /// Remove an agent from the grid
    pub fn remove(&mut self, id: Uuid) {
        if let Some(cell) = self.agent_cells.remove(&id) {
            self.cells[cell].retain(|&agent_id| agent_id != id);
        }
    }

    /// Update an agent's position
    pub fn update(&mut self, id: Uuid, new_pos: Vec2) {
        let new_cell = self.cell_index(new_pos);
        
        if let Some(&old_cell) = self.agent_cells.get(&id) {
            if old_cell != new_cell {
                self.cells[old_cell].retain(|&agent_id| agent_id != id);
                self.cells[new_cell].push(id);
                self.agent_cells.insert(id, new_cell);
            }
        }
    }

    /// Query agents within radius of a position
    pub fn query(&self, pos: Vec2, radius: f32) -> Vec<Uuid> {
        let mut results = Vec::new();
        
        // Calculate cell range to check
        let min_col = ((pos.x - radius) / self.cell_size).floor().max(0.0) as usize;
        let max_col = ((pos.x + radius) / self.cell_size).ceil().min(self.cols as f32) as usize;
        let min_row = ((pos.y - radius) / self.cell_size).floor().max(0.0) as usize;
        let max_row = ((pos.y + radius) / self.cell_size).ceil().min(self.rows as f32) as usize;

        for row in min_row..max_row {
            for col in min_col..max_col {
                let cell = row * self.cols + col;
                results.extend_from_slice(&self.cells[cell]);
            }
        }

        results
    }

    /// Get all agents in a specific cell
    pub fn get_cell(&self, col: usize, row: usize) -> &[Uuid] {
        let cell = row * self.cols + col;
        if cell < self.cells.len() {
            &self.cells[cell]
        } else {
            &[]
        }
    }

    /// Get number of agents in grid
    pub fn agent_count(&self) -> usize {
        self.agent_cells.len()
    }

    /// Clear all agents from grid
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
        self.agent_cells.clear();
    }

    /// Get cell ranges for parallel processing
    pub fn get_cell_ranges(&self, chunk_count: usize) -> Vec<(usize, usize)> {
        let total_cells = self.cells.len();
        let chunk_size = (total_cells + chunk_count - 1) / chunk_count;
        
        (0..chunk_count)
            .map(|i| {
                let start = i * chunk_size;
                let end = ((i + 1) * chunk_size).min(total_cells);
                (start, end)
            })
            .filter(|(start, end)| start < end)
            .collect()
    }
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new(1000.0, 1000.0, 50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let grid = SpatialGrid::new(100.0, 100.0, 10.0);
        assert_eq!(grid.cols, 10);
        assert_eq!(grid.rows, 10);
        assert_eq!(grid.cells.len(), 100);
    }

    #[test]
    fn test_insert_and_query() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 10.0);
        let id = Uuid::new_v4();
        
        grid.insert(id, Vec2::new(25.0, 25.0));
        
        let nearby = grid.query(Vec2::new(25.0, 25.0), 5.0);
        assert!(nearby.contains(&id));
    }

    #[test]
    fn test_update_position() {
        let mut grid = SpatialGrid::new(100.0, 100.0, 10.0);
        let id = Uuid::new_v4();
        
        grid.insert(id, Vec2::new(5.0, 5.0));
        grid.update(id, Vec2::new(95.0, 95.0));
        
        // Should not be found at old position
        let nearby_old = grid.query(Vec2::new(5.0, 5.0), 5.0);
        assert!(!nearby_old.contains(&id));
        
        // Should be found at new position
        let nearby_new = grid.query(Vec2::new(95.0, 95.0), 5.0);
        assert!(nearby_new.contains(&id));
    }
}
