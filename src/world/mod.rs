//! World module - global state and environment

mod faction;
mod grid;
mod resources;
mod state;

pub use faction::{Faction, FactionColor, FactionManager, FACTION_COLORS};
pub use grid::SpatialGrid;
pub use resources::{Resource, ResourceNode, ResourceType};
pub use state::{WorldState, WorldStats};
