use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::Vec2;

/// Types of resources in the world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Water,
    Shelter,
    Material,
}

/// A resource that can be gathered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub amount: f32,
    pub quality: f32,
}

impl Resource {
    pub fn food(amount: f32) -> Self {
        Self {
            resource_type: ResourceType::Food,
            amount,
            quality: 1.0,
        }
    }

    pub fn water(amount: f32) -> Self {
        Self {
            resource_type: ResourceType::Water,
            amount,
            quality: 1.0,
        }
    }
}

/// A resource node in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: Uuid,
    pub position: Vec2,
    pub resource_type: ResourceType,
    /// Current amount available
    pub amount: f32,
    /// Maximum amount (for regeneration)
    pub max_amount: f32,
    /// Regeneration rate per tick
    pub regen_rate: f32,
    /// Is this a permanent resource (water source, etc.)
    pub permanent: bool,
}

impl ResourceNode {
    /// Create a new resource node
    pub fn new(position: Vec2, resource_type: ResourceType, amount: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            resource_type,
            amount,
            max_amount: amount,
            regen_rate: 0.001,
            permanent: matches!(resource_type, ResourceType::Water),
        }
    }

    /// Create a random resource node
    pub fn random(position: Vec2) -> Self {
        let mut rng = rand::thread_rng();
        let resource_type = match rng.gen_range(0..4) {
            0 => ResourceType::Food,
            1 => ResourceType::Water,
            2 => ResourceType::Shelter,
            _ => ResourceType::Material,
        };
        let amount = rng.gen_range(10.0..100.0);
        
        Self::new(position, resource_type, amount)
    }

    /// Gather resources from this node
    pub fn gather(&mut self, amount: f32) -> Resource {
        let gathered = amount.min(self.amount);
        self.amount -= gathered;
        
        Resource {
            resource_type: self.resource_type,
            amount: gathered,
            quality: 1.0,
        }
    }

    /// Regenerate resources over time
    pub fn regenerate(&mut self, delta_time: f32) {
        if self.amount < self.max_amount {
            self.amount = (self.amount + self.regen_rate * delta_time).min(self.max_amount);
        }
    }

    /// Check if depleted (non-permanent resources)
    pub fn is_depleted(&self) -> bool {
        !self.permanent && self.amount <= 0.0
    }

    /// Check if this resource satisfies hunger
    pub fn satisfies_hunger(&self) -> bool {
        self.resource_type == ResourceType::Food && self.amount > 0.0
    }

    /// Check if this resource satisfies thirst
    pub fn satisfies_thirst(&self) -> bool {
        self.resource_type == ResourceType::Water && self.amount > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let node = ResourceNode::new(
            Vec2::new(10.0, 20.0),
            ResourceType::Food,
            100.0,
        );
        assert_eq!(node.amount, 100.0);
        assert!(!node.permanent);
    }

    #[test]
    fn test_gather() {
        let mut node = ResourceNode::new(
            Vec2::default(),
            ResourceType::Food,
            50.0,
        );
        
        let gathered = node.gather(20.0);
        assert_eq!(gathered.amount, 20.0);
        assert_eq!(node.amount, 30.0);
    }

    #[test]
    fn test_gather_exceeds_available() {
        let mut node = ResourceNode::new(
            Vec2::default(),
            ResourceType::Food,
            10.0,
        );
        
        let gathered = node.gather(50.0);
        assert_eq!(gathered.amount, 10.0);
        assert_eq!(node.amount, 0.0);
    }

    #[test]
    fn test_regeneration() {
        let mut node = ResourceNode::new(
            Vec2::default(),
            ResourceType::Food,
            100.0,
        );
        node.amount = 50.0;
        
        node.regenerate(100.0);
        assert!(node.amount > 50.0);
        assert!(node.amount <= 100.0);
    }

    #[test]
    fn test_water_is_permanent() {
        let node = ResourceNode::new(
            Vec2::default(),
            ResourceType::Water,
            100.0,
        );
        assert!(node.permanent);
    }
}
