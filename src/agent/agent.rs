use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::actions::{Action, ActionOutput};
use super::memory::{AgentMemory, EventOutcome};
use super::stimuli::Stimuli;
use crate::neural::{NeuralNet, NeuralNetwork};

/// 2D position vector
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn direction_to(&self, other: &Vec2) -> f32 {
        (other.y - self.y).atan2(other.x - self.x)
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2::new(self.x / len, self.y / len)
        } else {
            *self
        }
    }
}

/// An individual agent in the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier
    pub id: Uuid,
    /// Current position in world
    pub position: Vec2,
    /// Neural network for decision making
    pub network: NeuralNetwork,
    /// Current perceptual inputs
    #[serde(skip)]
    pub stimuli: Stimuli,
    /// Long-term memory
    pub memory: AgentMemory,
    /// Is agent alive
    pub alive: bool,
    /// Age in ticks
    pub age: u64,
    /// Generation (for evolutionary tracking)
    pub generation: u32,
    /// Parent IDs (if any)
    pub parents: Option<(Uuid, Uuid)>,
    /// Current health (0.0 to 1.0)
    pub health: f32,
    /// Metabolism rate (affects resource consumption)
    pub metabolism: f32,
    /// Movement speed multiplier
    pub speed: f32,
    /// Faction index (for visualization and grouping)
    pub faction: usize,
    /// Cached action output from last think() call
    #[serde(skip)]
    pub last_action: ActionOutput,
}

impl Agent {
    /// Create a new agent at the specified position
    pub fn new(position: Vec2) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            position,
            network: NeuralNetwork::new_agent_network(),
            stimuli: Stimuli::default(),
            memory: AgentMemory::new(id),
            alive: true,
            age: 0,
            generation: 0,
            parents: None,
            health: 1.0,
            metabolism: 1.0,
            speed: 1.0,
            faction: 0,
            last_action: ActionOutput::default(),
        }
    }

    /// Create a new agent with randomized initial state
    pub fn new_randomized(position: Vec2) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let id = Uuid::new_v4();
        
        let mut stimuli = Stimuli::default();
        stimuli.hunger = rng.gen_range(0.0..0.3);
        stimuli.thirst = rng.gen_range(0.0..0.3);
        stimuli.energy = rng.gen_range(0.7..1.0);
        
        Self {
            id,
            position,
            network: NeuralNetwork::new_agent_network(),
            stimuli,
            memory: AgentMemory::new(id),
            alive: true,
            age: 0,
            generation: 0,
            parents: None,
            health: rng.gen_range(0.9..1.0),
            metabolism: rng.gen_range(0.8..1.2), // ±20% variation
            speed: rng.gen_range(0.9..1.1),
            faction: 0,
            last_action: ActionOutput::default(),
        }
    }

    /// Create a new agent with faction assignment and randomized state
    pub fn new_with_faction(position: Vec2, faction: usize) -> Self {
        let mut agent = Self::new_randomized(position);
        agent.faction = faction;
        agent
    }

    /// Create agent from parent(s) with inherited traits
    pub fn from_parents(parent_a: &Agent, parent_b: &Agent, position: Vec2) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let id = Uuid::new_v4();
        
        // Randomize initial needs for offspring too
        let mut stimuli = Stimuli::default();
        stimuli.hunger = rng.gen_range(0.0..0.2);
        stimuli.thirst = rng.gen_range(0.0..0.2);
        stimuli.energy = rng.gen_range(0.8..1.0);
        
        let mut child = Self {
            id,
            position,
            network: parent_a.network.crossover(&parent_b.network, 0.5),
            stimuli,
            memory: AgentMemory::new(id),
            alive: true,
            age: 0,
            generation: parent_a.generation.max(parent_b.generation) + 1,
            parents: Some((parent_a.id, parent_b.id)),
            health: 1.0,
            metabolism: (parent_a.metabolism + parent_b.metabolism) / 2.0 * rng.gen_range(0.9..1.1),
            speed: (parent_a.speed + parent_b.speed) / 2.0 * rng.gen_range(0.9..1.1),
            faction: parent_a.faction, // Inherit faction from first parent
            last_action: ActionOutput::default(),
        };

        // Apply mutation
        child.network.mutate(0.05);
        child
    }

    /// Update stimuli from environment
    pub fn perceive(&mut self, stimuli: Stimuli) {
        self.stimuli = stimuli;
        self.stimuli.health = self.health;
    }

    /// Process stimuli and decide on action (caches result in last_action)
    pub fn think(&mut self) -> ActionOutput {
        let inputs = self.stimuli.to_input_vector();
        let outputs = self.network.forward(&inputs);
        self.last_action = ActionOutput::from_nn_output(&outputs);
        self.last_action.clone()
    }

    /// Execute an action and get reward
    pub fn act(&mut self, action: &ActionOutput, _tick: u64) -> f32 {
        let primary = action.primary_action();
        let mut reward = 0.0;

        match primary {
            Action::Move => {
                let (dx, dy) = action.movement_vector();
                self.position.x += dx * self.speed;
                self.position.y += dy * self.speed;
                reward = -0.01; // Small cost for movement
            }
            Action::Rest => {
                self.health = (self.health + 0.01).min(1.0);
                self.stimuli.energy = (self.stimuli.energy + 0.02).min(1.0);
                reward = 0.05;
            }
            Action::Gather => {
                // Reward determined externally by world in process_gathering()
                reward = 0.0;
            }
            Action::Communicate => {
                // Handled externally by world in process_communication()
                reward = 0.01; // Small social reward
            }
            Action::Aggress => {
                reward = -0.1; // Aggression has cost
            }
            Action::Flee => {
                let (dx, dy) = action.movement_vector();
                self.position.x += dx * self.speed * 1.5;
                self.position.y += dy * self.speed * 1.5;
                reward = -0.02;
            }
            Action::Reproduce => {
                // Handled externally by world in process_reproduction()
                reward = 0.0;
            }
            Action::Build => {
                // Building requires energy and is era-dependent
                // Era factor > 0.5 means Medieval or Modern era
                if self.stimuli.era_factor > 0.5 && self.stimuli.energy > 0.3 {
                    self.stimuli.energy -= 0.1;
                    reward = 0.1; // Building is productive
                } else {
                    reward = -0.05; // Failed build attempt
                }
            }
        }

        reward
    }

    /// Apply learning based on reward
    pub fn learn(&mut self, reward: f32) {
        self.network.adjust_weights(reward);
    }

    /// Record action outcome to memory
    pub fn remember(&mut self, tick: u64, action: Action, success: bool, reward: f32) {
        let outcome = if reward > 0.0 {
            EventOutcome::Success
        } else if reward < 0.0 {
            EventOutcome::Failure
        } else {
            EventOutcome::Neutral
        };
        self.memory.record_event(tick, action, outcome, reward);
    }

    /// Update agent state each tick
    pub fn tick(&mut self, delta_time: f32) {
        self.age += 1;
        self.memory.tick();
        self.stimuli.decay_needs(delta_time, self.metabolism);

        // Health decay from unmet needs
        if self.stimuli.hunger > 0.8 {
            self.health -= 0.001 * delta_time;
        }
        if self.stimuli.thirst > 0.8 {
            self.health -= 0.002 * delta_time;
        }

        // Death check
        if self.health <= 0.0 || self.stimuli.is_critical() {
            self.alive = false;
        }
    }

    /// Get fitness score for evolution
    pub fn fitness(&self) -> f32 {
        self.memory.fitness()
    }

    /// Export agent state to latent memory
    pub fn export_knowledge(&self) -> Vec<f32> {
        self.network.export_latent()
    }

    /// Import knowledge from latent memory
    pub fn import_knowledge(&mut self, latent: &[f32]) {
        self.network.import_latent(latent);
        self.memory.update_latent(latent.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(Vec2::new(10.0, 20.0));
        assert!(agent.alive);
        assert_eq!(agent.position.x, 10.0);
        assert_eq!(agent.position.y, 20.0);
        assert_eq!(agent.generation, 0);
    }

    #[test]
    fn test_agent_think() {
        let mut agent = Agent::new(Vec2::default());
        agent.stimuli.hunger = 0.8;
        let action = agent.think();
        // Should return some action output
        assert!(action.move_speed >= 0.0);
    }

    #[test]
    fn test_agent_offspring() {
        let parent_a = Agent::new(Vec2::new(0.0, 0.0));
        let parent_b = Agent::new(Vec2::new(10.0, 10.0));
        let child = Agent::from_parents(&parent_a, &parent_b, Vec2::new(5.0, 5.0));

        assert_eq!(child.generation, 1);
        assert!(child.parents.is_some());
    }

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance(&b) - 5.0).abs() < 0.001);
    }
}
