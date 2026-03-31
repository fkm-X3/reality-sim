use serde::{Deserialize, Serialize};

/// Learning rule for neuroplasticity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningRule {
    /// Hebbian learning: "neurons that fire together, wire together"
    Hebbian,
    /// Reward-modulated learning: weights adjusted based on reward signal
    RewardModulated,
    /// Evolutionary: no online learning, weights change through selection/mutation
    Evolutionary,
}

/// Configuration for neuroplasticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticityConfig {
    pub rule: LearningRule,
    pub learning_rate: f32,
    pub weight_decay: f32,
    pub momentum: f32,
    pub clip_range: (f32, f32),
}

impl Default for PlasticityConfig {
    fn default() -> Self {
        Self {
            rule: LearningRule::RewardModulated,
            learning_rate: 0.01,
            weight_decay: 0.0001,
            momentum: 0.9,
            clip_range: (-5.0, 5.0),
        }
    }
}

impl PlasticityConfig {
    /// Create config for Hebbian learning
    pub fn hebbian() -> Self {
        Self {
            rule: LearningRule::Hebbian,
            learning_rate: 0.001,
            ..Default::default()
        }
    }

    /// Create config for evolutionary (no online learning)
    pub fn evolutionary() -> Self {
        Self {
            rule: LearningRule::Evolutionary,
            learning_rate: 0.0,
            ..Default::default()
        }
    }

    /// Create config for reward-modulated learning
    pub fn reward_modulated(learning_rate: f32) -> Self {
        Self {
            rule: LearningRule::RewardModulated,
            learning_rate,
            ..Default::default()
        }
    }
}

/// Applies fitness-based selection for evolutionary approach
pub struct EvolutionarySelector {
    pub mutation_rate: f32,
    pub elite_fraction: f32,
    pub tournament_size: usize,
}

impl Default for EvolutionarySelector {
    fn default() -> Self {
        Self {
            mutation_rate: 0.1,
            elite_fraction: 0.1,
            tournament_size: 3,
        }
    }
}

impl EvolutionarySelector {
    /// Select parents for next generation based on fitness
    pub fn select_parents(&self, fitness_scores: &[f32], count: usize) -> Vec<usize> {
        let mut selected = Vec::with_capacity(count);
        let mut rng = rand::thread_rng();

        // Elitism: keep top performers
        let mut sorted_indices: Vec<usize> = (0..fitness_scores.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            fitness_scores[b].partial_cmp(&fitness_scores[a]).unwrap()
        });

        let elite_count = (fitness_scores.len() as f32 * self.elite_fraction) as usize;
        selected.extend_from_slice(&sorted_indices[..elite_count.min(count)]);

        // Tournament selection for the rest
        while selected.len() < count {
            let mut best_idx = rand::Rng::gen_range(&mut rng, 0..fitness_scores.len());
            let mut best_fitness = fitness_scores[best_idx];

            for _ in 1..self.tournament_size {
                let idx = rand::Rng::gen_range(&mut rng, 0..fitness_scores.len());
                if fitness_scores[idx] > best_fitness {
                    best_idx = idx;
                    best_fitness = fitness_scores[idx];
                }
            }

            selected.push(best_idx);
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PlasticityConfig::default();
        assert_eq!(config.rule, LearningRule::RewardModulated);
        assert!(config.learning_rate > 0.0);
    }

    #[test]
    fn test_selector() {
        let selector = EvolutionarySelector::default();
        let fitness = vec![1.0, 5.0, 3.0, 2.0, 4.0];
        let parents = selector.select_parents(&fitness, 3);
        assert_eq!(parents.len(), 3);
        // Best fitness (5.0 at index 1) should be in elite
        assert!(parents.contains(&1));
    }
}
