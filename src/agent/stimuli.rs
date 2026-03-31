use serde::{Deserialize, Serialize};

/// Perceptual inputs from the environment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stimuli {
    /// Hunger level (0.0 = full, 1.0 = starving)
    pub hunger: f32,
    /// Thirst level (0.0 = hydrated, 1.0 = dehydrated)
    pub thirst: f32,
    /// Fear/threat level (0.0 = safe, 1.0 = extreme danger)
    pub fear: f32,
    /// Proximity to nearest agent (normalized, 0.0 = adjacent, 1.0 = far)
    pub nearest_agent_distance: f32,
    /// Direction to nearest agent (radians, 0 to 2π)
    pub nearest_agent_direction: f32,
    /// Proximity to nearest resource (normalized)
    pub nearest_resource_distance: f32,
    /// Direction to nearest resource (radians)
    pub nearest_resource_direction: f32,
    /// Era factor (0.0 = prehistoric, 1.0 = modern)
    pub era_factor: f32,
    /// Current health (0.0 = dying, 1.0 = perfect)
    pub health: f32,
    /// Energy level (0.0 = exhausted, 1.0 = full energy)
    pub energy: f32,
    /// Social proximity (how many agents nearby, normalized)
    pub social_density: f32,
    /// Temperature discomfort (0.0 = comfortable, 1.0 = extreme)
    pub temperature_stress: f32,
}

impl Stimuli {
    /// Convert stimuli to neural network input vector
    pub fn to_input_vector(&self) -> Vec<f32> {
        vec![
            self.hunger,
            self.thirst,
            self.fear,
            self.nearest_agent_distance,
            self.nearest_resource_distance,
            self.era_factor,
            self.health,
            self.energy,
        ]
    }

    /// Create stimuli from input vector
    pub fn from_input_vector(inputs: &[f32]) -> Self {
        Self {
            hunger: inputs.get(0).copied().unwrap_or(0.0),
            thirst: inputs.get(1).copied().unwrap_or(0.0),
            fear: inputs.get(2).copied().unwrap_or(0.0),
            nearest_agent_distance: inputs.get(3).copied().unwrap_or(1.0),
            nearest_resource_distance: inputs.get(4).copied().unwrap_or(1.0),
            era_factor: inputs.get(5).copied().unwrap_or(0.0),
            health: inputs.get(6).copied().unwrap_or(1.0),
            energy: inputs.get(7).copied().unwrap_or(1.0),
            ..Default::default()
        }
    }

    /// Update hunger over time
    pub fn decay_needs(&mut self, delta_time: f32, metabolism_rate: f32) {
        self.hunger = (self.hunger + delta_time * metabolism_rate * 0.01).min(1.0);
        self.thirst = (self.thirst + delta_time * metabolism_rate * 0.015).min(1.0);
        self.energy = (self.energy - delta_time * metabolism_rate * 0.005).max(0.0);
    }

    /// Check if agent is in critical condition
    pub fn is_critical(&self) -> bool {
        self.hunger > 0.9 || self.thirst > 0.9 || self.health < 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_vector() {
        let stimuli = Stimuli {
            hunger: 0.5,
            thirst: 0.3,
            fear: 0.1,
            ..Default::default()
        };
        let vec = stimuli.to_input_vector();
        assert_eq!(vec.len(), 8);
        assert_eq!(vec[0], 0.5);
        assert_eq!(vec[1], 0.3);
    }

    #[test]
    fn test_needs_decay() {
        let mut stimuli = Stimuli::default();
        stimuli.decay_needs(1.0, 1.0);
        assert!(stimuli.hunger > 0.0);
        assert!(stimuli.thirst > 0.0);
    }
}
