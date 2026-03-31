use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::agent::{Action, Agent, Stimuli, Vec2};
use crate::config::WorldConfig;
use crate::world::{ResourceType, WorldState};

use super::scheduler::BatchScheduler;
use super::time::TimeManager;

/// Request for gathering resources (collected during parallel phase)
struct GatherRequest {
    agent_id: Uuid,
    position: Vec2,
    intensity: f32,
}

/// Main simulation runner
pub struct SimulationRunner {
    pub world: WorldState,
    pub time_manager: TimeManager,
    pub scheduler: BatchScheduler,
    pub running: Arc<AtomicBool>,
    pub paused: Arc<AtomicBool>,
    /// Reproduction cooldowns (agent_id -> tick when can reproduce again)
    reproduction_cooldowns: std::collections::HashMap<Uuid, u64>,
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
            reproduction_cooldowns: std::collections::HashMap::new(),
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

        // Phase 4: Gathering - Process resource gathering
        self.process_gathering();

        // Phase 5: Reproduction - Create offspring
        self.process_reproduction();

        // Phase 6: Communication - Knowledge sharing
        self.process_communication();

        // Phase 7: Building - Create shelter structures
        self.process_building();

        // Phase 8: World update
        self.update_world(delta_time);

        // Phase 9: Learning - Apply rewards
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

    /// Process resource gathering for all agents
    fn process_gathering(&mut self) {
        const GATHER_RADIUS: f32 = 30.0;
        const GATHER_AMOUNT: f32 = 5.0;

        // Collect gather requests from agents
        let gather_requests: Vec<GatherRequest> = self.world.agents.par_iter()
            .filter_map(|agent_lock| {
                let agent = agent_lock.read();
                if !agent.alive {
                    return None;
                }
                
                // Check if agent wants to gather (using cached action)
                if agent.last_action.primary_action() == Action::Gather {
                    Some(GatherRequest {
                        agent_id: agent.id,
                        position: agent.position,
                        intensity: agent.last_action.gather_intensity,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Process each gather request (sequential to avoid resource conflicts)
        for request in gather_requests {
            // Find nearest resource
            let mut best_resource_idx: Option<usize> = None;
            let mut best_dist = f32::MAX;

            for (idx, resource) in self.world.resources.iter().enumerate() {
                if resource.amount <= 0.0 {
                    continue;
                }
                let dist = request.position.distance(&resource.position);
                if dist < GATHER_RADIUS && dist < best_dist {
                    best_dist = dist;
                    best_resource_idx = Some(idx);
                }
            }

            if let Some(resource_idx) = best_resource_idx {
                let gather_amount = GATHER_AMOUNT * request.intensity;
                let resource = &mut self.world.resources[resource_idx];
                let gathered = resource.gather(gather_amount);

                // Apply gathered resource to agent
                if let Some(agent_lock) = self.world.agents.iter()
                    .find(|a| a.read().id == request.agent_id)
                {
                    let mut agent = agent_lock.write();
                    match gathered.resource_type {
                        ResourceType::Food => {
                            agent.stimuli.hunger = (agent.stimuli.hunger - gathered.amount * 0.02).max(0.0);
                            agent.health = (agent.health + gathered.amount * 0.005).min(1.0);
                        }
                        ResourceType::Water => {
                            agent.stimuli.thirst = (agent.stimuli.thirst - gathered.amount * 0.03).max(0.0);
                        }
                        ResourceType::Shelter => {
                            // Shelter provides protection, reduces fear
                            agent.stimuli.fear = (agent.stimuli.fear - 0.1).max(0.0);
                        }
                        ResourceType::Material => {
                            // Materials are for building (handled elsewhere)
                            agent.stimuli.energy = (agent.stimuli.energy + 0.05).min(1.0);
                        }
                    }
                    self.world.stats.total_resources_gathered += 1;
                }
            }
        }
    }

    /// Process reproduction between willing agents
    fn process_reproduction(&mut self) {
        const REPRODUCTION_RADIUS: f32 = 20.0;
        const REPRODUCTION_THRESHOLD: f32 = 0.6;
        const REPRODUCTION_COOLDOWN: u64 = 500; // Ticks between reproductions

        let tick = self.world.tick;

        // Clean up old cooldowns
        self.reproduction_cooldowns.retain(|_, cooldown_tick| *cooldown_tick > tick);

        // Find agents wanting to reproduce
        let mut reproduction_candidates: Vec<(Uuid, Vec2, f32)> = Vec::new();
        
        for agent_lock in &self.world.agents {
            let agent = agent_lock.read();
            if !agent.alive || agent.health < 0.5 {
                continue;
            }
            
            // Check cooldown
            if self.reproduction_cooldowns.contains_key(&agent.id) {
                continue;
            }

            // Use cached action
            if agent.last_action.reproduction_desire > REPRODUCTION_THRESHOLD {
                reproduction_candidates.push((agent.id, agent.position, agent.last_action.reproduction_desire));
            }
        }

        // Match pairs for reproduction
        let mut used_agents: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
        let mut new_agents: Vec<Agent> = Vec::new();

        for i in 0..reproduction_candidates.len() {
            let (id_a, pos_a, _) = reproduction_candidates[i];
            if used_agents.contains(&id_a) {
                continue;
            }

            // Find nearest compatible partner
            for j in (i + 1)..reproduction_candidates.len() {
                let (id_b, pos_b, _) = reproduction_candidates[j];
                if used_agents.contains(&id_b) {
                    continue;
                }

                let dist = pos_a.distance(&pos_b);
                if dist < REPRODUCTION_RADIUS {
                    // Create offspring
                    let parent_a = self.world.agents.iter()
                        .find(|a| a.read().id == id_a)
                        .map(|a| a.read().clone());
                    let parent_b = self.world.agents.iter()
                        .find(|a| a.read().id == id_b)
                        .map(|a| a.read().clone());

                    if let (Some(pa), Some(pb)) = (parent_a, parent_b) {
                        let child_pos = Vec2::new(
                            (pos_a.x + pos_b.x) / 2.0,
                            (pos_a.y + pos_b.y) / 2.0,
                        );
                        let child = Agent::from_parents(&pa, &pb, child_pos);
                        new_agents.push(child);

                        // Mark parents as used and on cooldown
                        used_agents.insert(id_a);
                        used_agents.insert(id_b);
                        self.reproduction_cooldowns.insert(id_a, tick + REPRODUCTION_COOLDOWN);
                        self.reproduction_cooldowns.insert(id_b, tick + REPRODUCTION_COOLDOWN);

                        self.world.stats.total_births += 1;
                        break;
                    }
                }
            }
        }

        // Add new agents to world
        for child in new_agents {
            self.world.add_agent(child);
        }
    }

    /// Process communication between nearby agents
    fn process_communication(&mut self) {
        const COMMUNICATION_RADIUS: f32 = 40.0;
        const COMMUNICATION_THRESHOLD: f32 = 0.5;

        // Collect communication requests
        let mut communicators: Vec<(Uuid, Vec2, f32)> = Vec::new();
        
        for agent_lock in &self.world.agents {
            let agent = agent_lock.read();
            if !agent.alive {
                continue;
            }

            // Use cached action
            if agent.last_action.primary_action() == Action::Communicate 
                && agent.last_action.communication_signal.abs() > COMMUNICATION_THRESHOLD 
            {
                communicators.push((agent.id, agent.position, agent.last_action.communication_signal));
            }
        }

        // Process communication pairs
        for i in 0..communicators.len() {
            let (id_a, pos_a, signal_a) = communicators[i];

            for j in (i + 1)..communicators.len() {
                let (id_b, pos_b, signal_b) = communicators[j];

                let dist = pos_a.distance(&pos_b);
                if dist < COMMUNICATION_RADIUS {
                    // Similar signals = cooperative communication
                    let signal_similarity = 1.0 - (signal_a - signal_b).abs() / 2.0;
                    
                    if signal_similarity > 0.5 {
                        // Share knowledge between agents
                        let knowledge_a = self.world.agents.iter()
                            .find(|a| a.read().id == id_a)
                            .map(|a| a.read().export_knowledge());
                        let knowledge_b = self.world.agents.iter()
                            .find(|a| a.read().id == id_b)
                            .map(|a| a.read().export_knowledge());

                        if let (Some(ka), Some(kb)) = (knowledge_a, knowledge_b) {
                            // Blend knowledge
                            let blend_factor = signal_similarity * 0.1; // Small influence
                            
                            if let Some(agent_a) = self.world.agents.iter()
                                .find(|a| a.read().id == id_a)
                            {
                                let blended: Vec<f32> = ka.iter().zip(kb.iter())
                                    .map(|(a, b)| a * (1.0 - blend_factor) + b * blend_factor)
                                    .collect();
                                agent_a.write().import_knowledge(&blended);
                            }
                            
                            if let Some(agent_b) = self.world.agents.iter()
                                .find(|a| a.read().id == id_b)
                            {
                                let blended: Vec<f32> = kb.iter().zip(ka.iter())
                                    .map(|(b, a)| b * (1.0 - blend_factor) + a * blend_factor)
                                    .collect();
                                agent_b.write().import_knowledge(&blended);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Process building actions - create shelter structures
    fn process_building(&mut self) {
        const BUILD_THRESHOLD: f32 = 0.6;
        const BUILD_ENERGY_COST: f32 = 0.2;
        const SHELTER_AMOUNT: f32 = 50.0;

        // Only allow building in Medieval+ eras
        let era_factor = match self.world.era {
            crate::config::Era::Prehistoric => 0.0,
            crate::config::Era::Ancient => 0.33,
            crate::config::Era::Medieval => 0.66,
            crate::config::Era::Modern => 1.0,
        };

        if era_factor < 0.5 {
            return; // Building not available in early eras
        }

        // Collect build requests
        let mut build_requests: Vec<(Uuid, Vec2)> = Vec::new();

        for agent_lock in &self.world.agents {
            let agent = agent_lock.read();
            if !agent.alive {
                continue;
            }

            // Check if agent wants to build and has energy
            if agent.last_action.primary_action() == Action::Build 
                && agent.last_action.build_desire > BUILD_THRESHOLD
                && agent.stimuli.energy > BUILD_ENERGY_COST
            {
                build_requests.push((agent.id, agent.position));
            }
        }

        // Process build requests (limit to prevent spam)
        let max_builds_per_tick = 3;
        for (idx, (agent_id, position)) in build_requests.into_iter().enumerate() {
            if idx >= max_builds_per_tick {
                break;
            }

            // Deduct energy from builder
            if let Some(agent_lock) = self.world.agents.iter()
                .find(|a| a.read().id == agent_id)
            {
                let mut agent = agent_lock.write();
                agent.stimuli.energy -= BUILD_ENERGY_COST;
            }

            // Create shelter at agent's position
            let shelter = crate::world::ResourceNode::shelter(position, SHELTER_AMOUNT * era_factor);
            self.world.resources.push(shelter);
        }
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
