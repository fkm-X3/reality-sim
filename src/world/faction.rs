use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A faction that agents can belong to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: Uuid,
    pub name: String,
    pub color: FactionColor,
    pub member_count: usize,
    pub total_fitness: f32,
    pub territory_center: (f32, f32),
}

/// RGB color for faction visualization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FactionColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FactionColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_rgb(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    #[cfg(feature = "gui")]
    pub fn to_egui_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.r, self.g, self.b)
    }
}

/// Predefined faction colors
pub const FACTION_COLORS: [FactionColor; 12] = [
    FactionColor { r: 231, g: 76, b: 60 },    // Red
    FactionColor { r: 46, g: 134, b: 193 },   // Blue
    FactionColor { r: 39, g: 174, b: 96 },    // Green
    FactionColor { r: 241, g: 196, b: 15 },   // Yellow
    FactionColor { r: 155, g: 89, b: 182 },   // Purple
    FactionColor { r: 230, g: 126, b: 34 },   // Orange
    FactionColor { r: 26, g: 188, b: 156 },   // Teal
    FactionColor { r: 236, g: 72, b: 153 },   // Pink
    FactionColor { r: 52, g: 73, b: 94 },     // Dark blue
    FactionColor { r: 149, g: 165, b: 166 },  // Gray
    FactionColor { r: 211, g: 84, b: 0 },     // Dark orange
    FactionColor { r: 22, g: 160, b: 133 },   // Dark teal
];

/// Faction name generator
const FACTION_NAMES: [&str; 12] = [
    "Crimson Tribe",
    "Azure Clan",
    "Emerald Band",
    "Golden Horde",
    "Violet Order",
    "Amber Guild",
    "Jade Collective",
    "Rose Alliance",
    "Obsidian Legion",
    "Silver Covenant",
    "Bronze Fellowship",
    "Sage Commune",
];

impl Faction {
    /// Create a new faction with a specific index
    pub fn new(index: usize, center: (f32, f32)) -> Self {
        let color_idx = index % FACTION_COLORS.len();
        Self {
            id: Uuid::new_v4(),
            name: FACTION_NAMES[color_idx].to_string(),
            color: FACTION_COLORS[color_idx],
            member_count: 0,
            total_fitness: 0.0,
            territory_center: center,
        }
    }

    /// Create factions distributed across the world
    pub fn create_factions(count: usize, world_width: f32, world_height: f32) -> Vec<Faction> {
        let mut rng = rand::thread_rng();
        let mut factions = Vec::with_capacity(count);

        // Distribute faction centers in a grid-like pattern
        let cols = (count as f32).sqrt().ceil() as usize;
        let rows = (count + cols - 1) / cols;

        let cell_width = world_width / cols as f32;
        let cell_height = world_height / rows as f32;

        for i in 0..count {
            let col = i % cols;
            let row = i / cols;

            // Add some randomness to center positions
            let center_x = (col as f32 + 0.5) * cell_width + rng.gen_range(-cell_width * 0.2..cell_width * 0.2);
            let center_y = (row as f32 + 0.5) * cell_height + rng.gen_range(-cell_height * 0.2..cell_height * 0.2);

            factions.push(Faction::new(i, (center_x.clamp(0.0, world_width), center_y.clamp(0.0, world_height))));
        }

        factions
    }

    /// Update faction statistics
    pub fn update_stats(&mut self, member_count: usize, total_fitness: f32) {
        self.member_count = member_count;
        self.total_fitness = total_fitness;
    }

    /// Get average fitness of faction members
    pub fn average_fitness(&self) -> f32 {
        if self.member_count > 0 {
            self.total_fitness / self.member_count as f32
        } else {
            0.0
        }
    }
}

/// Manager for all factions in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionManager {
    pub factions: Vec<Faction>,
}

impl FactionManager {
    /// Create a new faction manager with the specified number of factions
    pub fn new(faction_count: usize, world_width: f32, world_height: f32) -> Self {
        Self {
            factions: Faction::create_factions(faction_count, world_width, world_height),
        }
    }

    /// Assign an agent to the nearest faction based on position
    pub fn assign_faction(&self, position: (f32, f32)) -> usize {
        self.factions
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = Self::distance_squared(position, a.territory_center);
                let dist_b = Self::distance_squared(position, b.territory_center);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    /// Get faction color by index
    pub fn get_color(&self, faction_idx: usize) -> FactionColor {
        self.factions
            .get(faction_idx)
            .map(|f| f.color)
            .unwrap_or(FactionColor::new(128, 128, 128))
    }

    /// Get faction by index
    pub fn get(&self, idx: usize) -> Option<&Faction> {
        self.factions.get(idx)
    }

    /// Get mutable faction by index
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Faction> {
        self.factions.get_mut(idx)
    }

    fn distance_squared(a: (f32, f32), b: (f32, f32)) -> f32 {
        (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
    }
}

impl Default for FactionManager {
    fn default() -> Self {
        Self::new(6, 1000.0, 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faction_creation() {
        let factions = Faction::create_factions(4, 100.0, 100.0);
        assert_eq!(factions.len(), 4);
    }

    #[test]
    fn test_faction_assignment() {
        let manager = FactionManager::new(4, 100.0, 100.0);
        let faction = manager.assign_faction((10.0, 10.0));
        assert!(faction < 4);
    }
}
