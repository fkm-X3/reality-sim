use serde::{Deserialize, Serialize};

/// Global configuration for the simulation world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Starting year (negative values represent BCE)
    pub starting_year: i32,
    /// Number of agents to spawn
    pub agent_count: usize,
    /// Climate severity: 0.0 (mild) to 1.0 (harsh)
    pub climate_severity: f32,
    /// Resource density: 0.0 (scarce) to 1.0 (abundant)
    pub resource_density: f32,
    /// Simulation ticks per second
    pub ticks_per_second: u32,
    /// World width in units
    pub world_width: f32,
    /// World height in units
    pub world_height: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            starting_year: -10000, // 10,000 BCE
            agent_count: 1000,
            climate_severity: 0.5,
            resource_density: 0.5,
            ticks_per_second: 60,
            world_width: 1000.0,
            world_height: 1000.0,
        }
    }
}

impl WorldConfig {
    /// Returns the current era based on the year
    pub fn get_era(&self, current_year: i32) -> Era {
        match current_year {
            y if y < -3000 => Era::Prehistoric,
            y if y < 500 => Era::Ancient,
            y if y < 1500 => Era::Medieval,
            _ => Era::Modern,
        }
    }

    /// Returns era-specific behavior modifiers
    pub fn get_era_modifiers(&self, era: &Era) -> EraModifiers {
        match era {
            Era::Prehistoric => EraModifiers {
                aggression_factor: 1.5,
                cooperation_factor: 0.5,
                technology_factor: 0.0,
                tribal_bonus: 1.5,
            },
            Era::Ancient => EraModifiers {
                aggression_factor: 1.2,
                cooperation_factor: 0.8,
                technology_factor: 0.2,
                tribal_bonus: 1.2,
            },
            Era::Medieval => EraModifiers {
                aggression_factor: 1.0,
                cooperation_factor: 1.0,
                technology_factor: 0.5,
                tribal_bonus: 0.8,
            },
            Era::Modern => EraModifiers {
                aggression_factor: 0.7,
                cooperation_factor: 1.5,
                technology_factor: 1.0,
                tribal_bonus: 0.5,
            },
        }
    }
}

/// Historical eras that affect agent behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Era {
    Prehistoric, // 10,000 BCE - 3,000 BCE
    Ancient,     // 3,000 BCE - 500 CE
    Medieval,    // 500 CE - 1500 CE
    Modern,      // 1500 CE - present
}

/// Behavior modifiers based on historical era
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EraModifiers {
    pub aggression_factor: f32,
    pub cooperation_factor: f32,
    pub technology_factor: f32,
    pub tribal_bonus: f32,
}
