use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use crate::agent::AgentMemory;

/// Memory export format for persistence
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MemoryExport {
    pub version: u32,
    pub exported_at: String,
    pub memories: Vec<AgentMemory>,
}

impl MemoryExport {
    pub fn new(memories: Vec<AgentMemory>) -> Self {
        Self {
            version: 1,
            exported_at: chrono_lite_timestamp(),
            memories,
        }
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

/// Save agent memories to JSON file
pub fn save_agent_memories(memories: &[AgentMemory], path: &Path) -> std::io::Result<()> {
    let export = MemoryExport::new(memories.to_vec());
    
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &export)?;
    Ok(())
}

/// Load agent memories from JSON file
pub fn load_agent_memories(path: &Path) -> std::io::Result<Vec<AgentMemory>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let export: MemoryExport = serde_json::from_reader(reader)?;
    Ok(export.memories)
}

/// Save a single agent's memory
pub fn save_single_memory(memory: &AgentMemory, path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, memory)?;
    Ok(())
}

/// Load a single agent's memory
pub fn load_single_memory(path: &Path) -> std::io::Result<AgentMemory> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let memory: AgentMemory = serde_json::from_reader(reader)?;
    Ok(memory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use tempfile::tempdir;

    #[test]
    fn test_save_load_memories() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("memories.json");

        let memories = vec![
            AgentMemory::new(Uuid::new_v4()),
            AgentMemory::new(Uuid::new_v4()),
        ];
        
        // Save
        save_agent_memories(&memories, &file_path).unwrap();
        
        // Load
        let loaded = load_agent_memories(&file_path).unwrap();
        
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_save_load_single_memory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("memory.json");

        let mut memory = AgentMemory::new(Uuid::new_v4());
        memory.stats.survival_ticks = 1000;
        
        // Save
        save_single_memory(&memory, &file_path).unwrap();
        
        // Load
        let loaded = load_single_memory(&file_path).unwrap();
        
        assert_eq!(loaded.agent_id, memory.agent_id);
        assert_eq!(loaded.stats.survival_ticks, 1000);
    }
}
