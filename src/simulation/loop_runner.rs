use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::agent::Stimuli;
use crate::config::WorldConfig;
use crate::world::WorldState;

use super::scheduler::BatchScheduler;
use super::time::TimeManager;

/// Main simulation runner
pub struct SimulationRunner {
    pub world: WorldState,
    pub time_manager: TimeManager,
    pub scheduler: BatchScheduler,
    pub running: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
}

impl SimulationRunner {
    /// Create a new simulation
    pub fn new(config: WorldConfig) -> Self {
        let time_manager = TimeManager::new(config.ticks_per_second);
        let scheduler = BatchScheduler::new(rayon::current_num_threads());

        Self {
            world: WorldState::new(config),
            time_manager,
            scheduler,
            running: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Run a single simulation tick
    pub fn tick(&mut self) {
        let delta_time = 1.0 / self.world.config.ticks_per_second as f32;

        // Phase 1: Perception - Update stimuli for all agents
        self.update_perceptions();

        // Phase 2: Cognition - Process neural networks (parallel)
        self.process_decisions();

        // Phase 3: Action - Execute actions and interactions
        self.execute_actions(delta_time);

        // Phase 4: World update
        self.update_world(delta_time);

        // Phase 5: Learning - Apply rewards
        self.apply_learning();

        // Advance simulation time
        self.world.advance_tick();
        self.world.update_stats();
    }

    /// Update perception for all agents
    fn update_perceptions(&mut self) {
        let config = &self.world.config;
        let era_modifiers = config.get_era_modifiers(&self.world.era);
        let era_factor = match self.world.era {
            crate::config::Era::Prehistoric => 0.0,
            crate::config::Era::Ancient => 0.33,
            crate::config::Era::Medieval => 0.66,
            crate::config::Era::Modern => 1.0,
        };

        // Parallel perception update
        self.world.agents.par_iter().for_each(|agent_lock| {
            let mut agent = agent_lock.write();
            let pos = agent.position;

            // Find nearest agents and resources
            let nearby_agents = self.world.grid.query(pos, 50.0);
            let nearest_agent_dist = nearby_agents.iter()
                .filter(|&&id| id != agent.id)
                .map(|_| 1.0) // Simplified - would need actual distance
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(1.0);

            let stimuli = Stimuli {
                hunger: agent.stimuli.hunger,
                thirst: agent.stimuli.thirst,
                fear: agent.stimuli.fear * era_modifiers.aggression_factor,
                nearest_agent_distance: nearest_agent_dist,
                nearest_agent_direction: 0.0,
                nearest_resource_distance: 0.5, // Simplified
                nearest_resource_direction: 0.0,
                era_factor,
                health: agent.health,
                energy: agent.stimuli.energy,
                social_density: (nearby_agents.len() as f32 / 10.0).min(1.0),
                temperature_stress: config.climate_severity * 0.5,
            };

            agent.perceive(stimuli);
        });
    }

    /// Process neural network decisions in parallel
    fn process_decisions(&mut self) {
        self.world.agents.par_iter().for_each(|agent_lock| {
            let mut agent = agent_lock.write();
            if agent.alive {
                let _action = agent.think();
                // Action stored implicitly in agent state for execution phase
            }
        });
    }

    /// Execute agent actions
    fn execute_actions(&mut self, delta_time: f32) {
        let tick = self.world.tick;
        let world_width = self.world.config.world_width;
        let world_height = self.world.config.world_height;

        self.world.agents.par_iter().for_each(|agent_lock| {
            let mut agent = agent_lock.write();
            if !agent.alive {
                return;
            }

            // Compute action
            let action = agent.think();
            let reward = agent.act(&action, tick);

            // Boundary wrapping
            if agent.position.x < 0.0 {
                agent.position.x += world_width;
            }
            if agent.position.x >= world_width {
                agent.position.x -= world_width;
            }
            if agent.position.y < 0.0 {
                agent.position.y += world_height;
            }
            if agent.position.y >= world_height {
                agent.position.y -= world_height;
            }

            // Age and decay
            agent.tick(delta_time);

            // Remember action
            agent.remember(tick, action.primary_action(), reward > 0.0, reward);
        });

        // Update spatial grid
        self.world.update_grid();
    }

    /// Update world state (resources, etc.)
    fn update_world(&mut self, delta_time: f32) {
        // Regenerate resources
        for resource in &mut self.world.resources {
            resource.regenerate(delta_time);
        }

        // Remove depleted resources
        self.world.resources.retain(|r| !r.is_depleted());

        // Remove dead agents
        self.world.remove_dead_agents();

        // Spawn new resources occasionally
        if self.world.tick % 1000 == 0 {
            self.spawn_resources();
        }
    }

    /// Apply learning to agents
    fn apply_learning(&mut self) {
        self.world.agents.par_iter().for_each(|agent_lock| {
            let mut agent = agent_lock.write();
            if agent.alive {
                // Calculate reward based on state
                let mut reward = 0.0;

                // Reward for staying alive
                reward += 0.001;

                // Penalty for unmet needs
                reward -= agent.stimuli.hunger * 0.01;
                reward -= agent.stimuli.thirst * 0.01;

                // Reward for health
                reward += agent.health * 0.005;

                agent.learn(reward);
            }
        });
    }

    /// Spawn new resources
    fn spawn_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let count = (self.world.config.resource_density * 10.0) as usize;

        for _ in 0..count {
            let x = rand::Rng::gen_range(&mut rng, 0.0..self.world.config.world_width);
            let y = rand::Rng::gen_range(&mut rng, 0.0..self.world.config.world_height);
            self.world.resources.push(
                crate::world::ResourceNode::random(crate::agent::Vec2::new(x, y))
            );
        }
    }

    /// Run simulation for N ticks
    pub fn run_ticks(&mut self, count: u64) {
        for _ in 0..count {
            self.tick();
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> &crate::world::WorldStats {
        &self.world.stats
    }

    /// Start continuous running
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
    }

    /// Stop running
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Pause/unpause
    pub fn toggle_pause(&self) {
        let current = self.paused.load(Ordering::SeqCst);
        self.paused.store(!current, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let config = WorldConfig {
            agent_count: 10,
            ..Default::default()
        };
        let sim = SimulationRunner::new(config);
        assert_eq!(sim.world.agents.len(), 10);
    }

    #[test]
    fn test_simulation_tick() {
        let config = WorldConfig {
            agent_count: 10,
            ..Default::default()
        };
        let mut sim = SimulationRunner::new(config);
        
        let initial_tick = sim.world.tick;
        sim.tick();
        assert_eq!(sim.world.tick, initial_tick + 1);
    }

    #[test]
    fn test_run_multiple_ticks() {
        let config = WorldConfig {
            agent_count: 5,
            ..Default::default()
        };
        let mut sim = SimulationRunner::new(config);
        
        sim.run_ticks(100);
        assert_eq!(sim.world.tick, 100);
    }
}
