use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::agent::{Agent, Vec2};
use crate::config::{Era, WorldConfig};

use super::faction::FactionManager;
use super::grid::SpatialGrid;
use super::resources::ResourceNode;

/// Global world state
#[derive(Serialize, Deserialize)]
pub struct WorldState {
    /// Current simulation tick
    pub tick: u64,
    /// Current simulation year
    pub current_year: i32,
    /// World configuration
    pub config: WorldConfig,
    /// Current era
    pub era: Era,
    /// All agents (thread-safe access)
    #[serde(skip)]
    pub agents: Vec<Arc<RwLock<Agent>>>,
    /// Agent data for serialization
    #[serde(rename = "agents")]
    pub agents_data: Vec<Agent>,
    /// Resource nodes
    pub resources: Vec<ResourceNode>,
    /// Spatial partitioning grid
    #[serde(skip)]
    pub grid: SpatialGrid,
    /// Faction manager
    pub factions: FactionManager,
    /// Global statistics
    pub stats: WorldStats,
}

/// Global simulation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldStats {
    pub total_agents_spawned: u64,
    pub total_deaths: u64,
    pub total_births: u64,
    pub total_resources_gathered: u64,
    pub current_population: usize,
    pub average_fitness: f32,
    pub max_generation: u32,
}

impl WorldState {
    /// Create a new world from configuration
    pub fn new(config: WorldConfig) -> Self {
        Self::with_factions(config, 6) // Default 6 factions
    }

    /// Create a new world with specified number of factions
    pub fn with_factions(config: WorldConfig, faction_count: usize) -> Self {
        let era = config.get_era(config.starting_year);
        let grid = SpatialGrid::new(
            config.world_width,
            config.world_height,
            50.0, // Cell size
        );
        let factions = FactionManager::new(faction_count, config.world_width, config.world_height);

        let mut world = Self {
            tick: 0,
            current_year: config.starting_year,
            config,
            era,
            agents: Vec::new(),
            agents_data: Vec::new(),
            resources: Vec::new(),
            grid,
            factions,
            stats: WorldStats::default(),
        };

        world.spawn_initial_agents();
        world.spawn_initial_resources();
        world
    }

    /// Spawn initial agents with faction assignments
    fn spawn_initial_agents(&mut self) {
        let mut rng = rand::thread_rng();
        
        for _ in 0..self.config.agent_count {
            let x = rand::Rng::gen_range(&mut rng, 0.0..self.config.world_width);
            let y = rand::Rng::gen_range(&mut rng, 0.0..self.config.world_height);
            let faction = self.factions.assign_faction((x, y));
            let agent = Agent::new_with_faction(Vec2::new(x, y), faction);
            
            self.grid.insert(agent.id, agent.position);
            self.agents.push(Arc::new(RwLock::new(agent)));
            self.stats.total_agents_spawned += 1;
        }

        self.stats.current_population = self.agents.len();
    }

    /// Spawn initial resources based on density
    fn spawn_initial_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let resource_count = (self.config.world_width * self.config.world_height 
            * self.config.resource_density * 0.001) as usize;

        for _ in 0..resource_count {
            let x = rand::Rng::gen_range(&mut rng, 0.0..self.config.world_width);
            let y = rand::Rng::gen_range(&mut rng, 0.0..self.config.world_height);
            self.resources.push(ResourceNode::random(Vec2::new(x, y)));
        }
    }

    /// Get agents near a position
    pub fn agents_near(&self, pos: Vec2, radius: f32) -> Vec<Uuid> {
        self.grid.query(pos, radius)
    }

    /// Add a new agent to the world
    pub fn add_agent(&mut self, agent: Agent) {
        self.grid.insert(agent.id, agent.position);
        self.agents.push(Arc::new(RwLock::new(agent)));
        self.stats.total_agents_spawned += 1;
        self.stats.current_population = self.agents.len();
    }

    /// Remove dead agents
    pub fn remove_dead_agents(&mut self) {
        let initial_count = self.agents.len();
        
        self.agents.retain(|agent_lock| {
            let agent = agent_lock.read();
            if !agent.alive {
                self.grid.remove(agent.id);
                false
            } else {
                true
            }
        });

        let deaths = initial_count - self.agents.len();
        self.stats.total_deaths += deaths as u64;
        self.stats.current_population = self.agents.len();
    }

    /// Advance time by one tick
    pub fn advance_tick(&mut self) {
        self.tick += 1;

        // Year progression (configurable ticks per year)
        let ticks_per_year = 365 * self.config.ticks_per_second as u64;
        if self.tick % ticks_per_year == 0 {
            self.current_year += 1;
            self.era = self.config.get_era(self.current_year);
        }
    }

    /// Update grid positions for all agents
    pub fn update_grid(&mut self) {
        for agent_lock in &self.agents {
            let agent = agent_lock.read();
            self.grid.update(agent.id, agent.position);
        }
    }

    /// Calculate and update statistics
    pub fn update_stats(&mut self) {
        self.stats.current_population = self.agents.len();

        if !self.agents.is_empty() {
            let total_fitness: f32 = self.agents.iter()
                .map(|a| a.read().fitness())
                .sum();
            self.stats.average_fitness = total_fitness / self.agents.len() as f32;

            self.stats.max_generation = self.agents.iter()
                .map(|a| a.read().generation)
                .max()
                .unwrap_or(0);
        }
    }

    /// Prepare for serialization
    pub fn prepare_for_save(&mut self) {
        self.agents_data = self.agents.iter()
            .map(|a| a.read().clone())
            .collect();
    }

    /// Restore from serialization
    pub fn restore_from_load(&mut self) {
        self.agents.clear();
        self.grid = SpatialGrid::new(
            self.config.world_width,
            self.config.world_height,
            50.0,
        );

        for agent in self.agents_data.drain(..) {
            self.grid.insert(agent.id, agent.position);
            self.agents.push(Arc::new(RwLock::new(agent)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let config = WorldConfig {
            agent_count: 10,
            ..Default::default()
        };
        let world = WorldState::new(config);
        
        assert_eq!(world.agents.len(), 10);
        assert_eq!(world.tick, 0);
        assert!(!world.resources.is_empty());
    }

    #[test]
    fn test_agent_removal() {
        let config = WorldConfig {
            agent_count: 5,
            ..Default::default()
        };
        let mut world = WorldState::new(config);
        
        // Kill first agent
        {
            let mut agent = world.agents[0].write();
            agent.alive = false;
        }
        
        world.remove_dead_agents();
        assert_eq!(world.agents.len(), 4);
        assert_eq!(world.stats.total_deaths, 1);
    }
}
