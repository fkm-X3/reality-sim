use serde::{Deserialize, Serialize};

/// Discrete action types an agent can take
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Stay in place, recover energy
    Rest,
    /// Move in a direction
    Move,
    /// Gather nearby resources (food, water, materials)
    Gather,
    /// Attempt communication with nearby agent
    Communicate,
    /// Aggressive action toward nearby agent
    Aggress,
    /// Flee from threat
    Flee,
    /// Attempt reproduction (requires another willing agent)
    Reproduce,
    /// Build/craft (modern era)
    Build,
}

/// Output from neural network, converted to action + parameters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionOutput {
    /// Movement direction (radians, 0 to 2π)
    pub move_direction: f32,
    /// Movement speed (0.0 to 1.0)
    pub move_speed: f32,
    /// Gather intensity (0.0 to 1.0)
    pub gather_intensity: f32,
    /// Communication signal (arbitrary value for other agents)
    pub communication_signal: f32,
    /// Aggression level (0.0 to 1.0)
    pub aggression: f32,
    /// Rest desire (0.0 to 1.0)
    pub rest_desire: f32,
    /// Reproduction desire (0.0 to 1.0)
    pub reproduction_desire: f32,
    /// Build/craft desire (0.0 to 1.0)
    pub build_desire: f32,
}

impl ActionOutput {
    /// Create from neural network output vector
    pub fn from_nn_output(outputs: &[f32]) -> Self {
        Self {
            move_direction: outputs.get(0).copied().unwrap_or(0.0) * std::f32::consts::PI,
            move_speed: Self::normalize(outputs.get(1).copied().unwrap_or(0.0)),
            gather_intensity: Self::normalize(outputs.get(2).copied().unwrap_or(0.0)),
            communication_signal: outputs.get(3).copied().unwrap_or(0.0),
            aggression: Self::normalize(outputs.get(4).copied().unwrap_or(0.0)),
            rest_desire: Self::normalize(outputs.get(5).copied().unwrap_or(0.0)),
            reproduction_desire: Self::normalize(outputs.get(6).copied().unwrap_or(0.0)),
            build_desire: Self::normalize(outputs.get(7).copied().unwrap_or(0.0)),
        }
    }

    /// Normalize tanh output (-1 to 1) to (0 to 1)
    #[inline]
    fn normalize(x: f32) -> f32 {
        (x + 1.0) / 2.0
    }

    /// Get the primary action based on highest desire
    pub fn primary_action(&self) -> Action {
        let actions = [
            (self.rest_desire, Action::Rest),
            (self.move_speed, Action::Move),
            (self.gather_intensity, Action::Gather),
            (self.communication_signal.abs(), Action::Communicate),
            (self.aggression, Action::Aggress),
            (self.reproduction_desire, Action::Reproduce),
            (self.build_desire, Action::Build),
        ];

        actions
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, action)| *action)
            .unwrap_or(Action::Rest)
    }

    /// Get movement vector (x, y) from direction and speed
    pub fn movement_vector(&self) -> (f32, f32) {
        let x = self.move_direction.cos() * self.move_speed;
        let y = self.move_direction.sin() * self.move_speed;
        (x, y)
    }

    /// Check if agent wants to be aggressive
    pub fn is_aggressive(&self) -> bool {
        self.aggression > 0.6
    }

    /// Check if agent wants to flee
    pub fn wants_to_flee(&self) -> bool {
        self.move_speed > 0.7 && self.aggression < 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_nn_output() {
        let outputs = vec![0.0, 0.5, 0.3, -0.2, 0.0, 0.8, 0.1, 0.0];
        let action = ActionOutput::from_nn_output(&outputs);
        assert!(action.move_speed > 0.0);
        assert!(action.rest_desire > 0.0);
    }

    #[test]
    fn test_primary_action() {
        let mut action = ActionOutput::default();
        action.rest_desire = 0.9;
        assert_eq!(action.primary_action(), Action::Rest);

        action.gather_intensity = 0.95;
        assert_eq!(action.primary_action(), Action::Gather);
    }

    #[test]
    fn test_movement_vector() {
        let action = ActionOutput {
            move_direction: 0.0,
            move_speed: 1.0,
            ..Default::default()
        };
        let (x, y) = action.movement_vector();
        assert!((x - 1.0).abs() < 0.001);
        assert!(y.abs() < 0.001);
    }
}
