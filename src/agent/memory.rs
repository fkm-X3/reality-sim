use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Action;

/// A single memory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub tick: u64,
    pub action: Action,
    pub outcome: EventOutcome,
    pub reward: f32,
}

/// Outcome of an event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventOutcome {
    Success,
    Failure,
    Neutral,
}

/// Agent memory system - hybrid approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemory {
    /// Unique agent identifier
    pub agent_id: Uuid,
    /// Latent state vector (compressed neural embedding)
    pub latent_state: Vec<f32>,
    /// Recent events (sliding window)
    pub recent_events: Vec<MemoryEvent>,
    /// Maximum events to keep
    #[serde(default = "default_max_events")]
    pub max_events: usize,
    /// Lifetime statistics
    pub stats: MemoryStats,
}

fn default_max_events() -> usize {
    100
}

/// Lifetime statistics for an agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    pub survival_ticks: u64,
    pub resources_gathered: u64,
    pub interactions: u64,
    pub offspring: u32,
    pub successful_actions: u64,
    pub failed_actions: u64,
    pub total_reward: f32,
}

impl AgentMemory {
    /// Create a new empty memory
    pub fn new(agent_id: Uuid) -> Self {
        Self {
            agent_id,
            latent_state: Vec::new(),
            recent_events: Vec::with_capacity(100),
            max_events: 100,
            stats: MemoryStats::default(),
        }
    }

    /// Record a new event
    pub fn record_event(&mut self, tick: u64, action: Action, outcome: EventOutcome, reward: f32) {
        let event = MemoryEvent {
            tick,
            action,
            outcome,
            reward,
        };

        self.recent_events.push(event);

        // Sliding window
        if self.recent_events.len() > self.max_events {
            self.recent_events.remove(0);
        }

        // Update stats
        self.stats.total_reward += reward;
        match outcome {
            EventOutcome::Success => self.stats.successful_actions += 1,
            EventOutcome::Failure => self.stats.failed_actions += 1,
            EventOutcome::Neutral => {}
        }

        if matches!(action, Action::Gather) && outcome == EventOutcome::Success {
            self.stats.resources_gathered += 1;
        }
        if matches!(action, Action::Communicate | Action::Aggress | Action::Reproduce) {
            self.stats.interactions += 1;
        }
    }

    /// Tick survival counter
    pub fn tick(&mut self) {
        self.stats.survival_ticks += 1;
    }

    /// Update latent state from neural network
    pub fn update_latent(&mut self, latent: Vec<f32>) {
        self.latent_state = latent;
    }

    /// Get success rate for a specific action type
    pub fn action_success_rate(&self, action: Action) -> f32 {
        let (success, total) = self.recent_events.iter()
            .filter(|e| e.action == action)
            .fold((0, 0), |(s, t), e| {
                (s + (e.outcome == EventOutcome::Success) as u32, t + 1)
            });

        if total == 0 {
            0.5 // Unknown, assume neutral
        } else {
            success as f32 / total as f32
        }
    }

    /// Calculate fitness score for evolutionary selection
    pub fn fitness(&self) -> f32 {
        let survival_score = (self.stats.survival_ticks as f32).sqrt();
        let resource_score = self.stats.resources_gathered as f32 * 0.5;
        let offspring_score = self.stats.offspring as f32 * 10.0;
        let efficiency = if self.stats.successful_actions + self.stats.failed_actions > 0 {
            self.stats.successful_actions as f32
                / (self.stats.successful_actions + self.stats.failed_actions) as f32
        } else {
            0.5
        };

        survival_score + resource_score + offspring_score + efficiency * 5.0
    }

    /// Compress memory for persistence (keep important events)
    pub fn compress(&mut self) {
        if self.recent_events.len() <= self.max_events / 2 {
            return;
        }

        // Keep events with high absolute reward
        self.recent_events.sort_by(|a, b| {
            b.reward.abs().partial_cmp(&a.reward.abs()).unwrap()
        });
        self.recent_events.truncate(self.max_events / 2);

        // Re-sort by tick
        self.recent_events.sort_by_key(|e| e.tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let id = Uuid::new_v4();
        let memory = AgentMemory::new(id);
        assert_eq!(memory.agent_id, id);
        assert!(memory.recent_events.is_empty());
    }

    #[test]
    fn test_event_recording() {
        let mut memory = AgentMemory::new(Uuid::new_v4());
        memory.record_event(1, Action::Gather, EventOutcome::Success, 1.0);
        memory.record_event(2, Action::Gather, EventOutcome::Failure, -0.5);

        assert_eq!(memory.recent_events.len(), 2);
        assert_eq!(memory.stats.successful_actions, 1);
        assert_eq!(memory.stats.failed_actions, 1);
        assert_eq!(memory.stats.resources_gathered, 1);
    }

    #[test]
    fn test_sliding_window() {
        let mut memory = AgentMemory::new(Uuid::new_v4());
        memory.max_events = 5;

        for i in 0..10 {
            memory.record_event(i, Action::Rest, EventOutcome::Neutral, 0.0);
        }

        assert_eq!(memory.recent_events.len(), 5);
        assert_eq!(memory.recent_events[0].tick, 5);
    }

    #[test]
    fn test_success_rate() {
        let mut memory = AgentMemory::new(Uuid::new_v4());
        memory.record_event(1, Action::Gather, EventOutcome::Success, 1.0);
        memory.record_event(2, Action::Gather, EventOutcome::Success, 1.0);
        memory.record_event(3, Action::Gather, EventOutcome::Failure, -0.5);

        let rate = memory.action_success_rate(Action::Gather);
        assert!((rate - 0.666).abs() < 0.01);
    }
}
