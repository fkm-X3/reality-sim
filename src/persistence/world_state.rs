use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::world::WorldState;

/// Save world state to JSON file
pub fn save_world_state(world: &mut WorldState, path: &Path) -> std::io::Result<()> {
    // Prepare agent data for serialization
    world.prepare_for_save();

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, world)?;
    Ok(())
}

/// Load world state from JSON file
pub fn load_world_state(path: &Path) -> std::io::Result<WorldState> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut world: WorldState = serde_json::from_reader(reader)?;
    
    // Restore runtime state
    world.restore_from_load();
    
    Ok(world)
}

/// Save world state with compression (for large simulations)
pub fn save_world_state_compressed(world: &mut WorldState, path: &Path) -> std::io::Result<()> {
    world.prepare_for_save();
    
    let json = serde_json::to_string(world)?;
    std::fs::write(path, json)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldConfig;
    use tempfile::tempdir;

    #[test]
    fn test_save_load_world() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("world.json");

        let config = WorldConfig {
            agent_count: 5,
            ..Default::default()
        };
        let mut world = WorldState::new(config);
        
        // Save
        save_world_state(&mut world, &file_path).unwrap();
        
        // Load
        let loaded = load_world_state(&file_path).unwrap();
        
        assert_eq!(loaded.agents.len(), 5);
        assert_eq!(loaded.config.starting_year, world.config.starting_year);
    }
}
