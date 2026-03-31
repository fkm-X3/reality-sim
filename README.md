# Reality Simulator

A Rust-based **Humanity Simulator** where thousands of agents perceive their environment, process stimuli through neural networks, take actions, and persist their experiences to JSON.

## Features

- **Neural Engine**: Custom lightweight neural network with configurable architectures
- **Neuroplasticity**: Hebbian, reward-modulated, or evolutionary weight adjustment
- **High Concurrency**: Optimized for 50,000+ agents using `rayon` and spatial partitioning
- **Era Progression**: Behavior modifiers change as time progresses (10,000 BCE → Modern)
- **JSON Persistence**: Save/load world state and agent memories
- **Configurable**: Starting year, climate, resources, agent count all adjustable
- **GUI Visualization**: Watch agents as colored dots, grouped by faction

## Quick Start

```bash
# Build the project
cargo build --release

# Run CLI with defaults (10,000 BCE, 1000 agents)
cargo run --release

# Custom configuration
cargo run --release -- --year -5000 --agents 5000 --climate 0.7 --resources 0.3

# Run GUI version
cargo run --release --bin reality-sim-gui
```

## GUI Features

The GUI (`reality-sim-gui`) provides real-time visualization:
- **World View**: Agents displayed as colored dots (colors = factions)
- **Control Panel**: Play/pause, speed control, show/hide grid and resources
- **Stats Panel**: Population, tick count, average fitnesscargo run --release --bin reality-sim-gui
- **Faction Panel**: View all factions with member counts
- **Agent Inspector**: Click an agent to see details (position, needs, memory)
- **Graphs**: Population and fitness history over time
- **Interactions**: Pan with drag, zoom with scroll, click to select agents

## Architecture

```
src/
├── neural/           # Neural network implementation
│   ├── activation.rs # ReLU, Sigmoid, Tanh, LeakyReLU
│   ├── layer.rs      # Dense layer with Xavier initialization
│   ├── network.rs    # NeuralNetwork trait & impl
│   └── plasticity.rs # Learning rules & evolutionary selection
├── agent/            # Individual agents
│   ├── stimuli.rs    # Perception inputs (hunger, thirst, proximity, etc.)
│   ├── actions.rs    # Output actions (move, gather, aggress, etc.)
│   ├── memory.rs     # Hybrid memory (latent vector + event log)
│   └── agent.rs      # Agent struct combining all components
├── world/            # Global state
│   ├── state.rs      # WorldState with configuration
│   ├── grid.rs       # Spatial partitioning for O(1) queries
│   ├── faction.rs    # Faction system (12 factions with colors)
│   └── resources.rs  # Food, water, shelter nodes
├── simulation/       # Main loop
│   ├── loop_runner.rs # Perception-Action cycle
│   ├── scheduler.rs  # Batch processing for parallelism
│   └── time.rs       # Time management & era progression
├── gui/              # GUI application (egui/eframe)
│   ├── main.rs       # GUI entry point
│   ├── app.rs        # Application state & update loop
│   ├── panels.rs     # UI panels (controls, stats, graphs)
│   └── renderer.rs   # World rendering (agents as colored dots)
└── persistence/      # JSON serialization
    ├── world_state.rs
    └── agent_memory.rs
```

## Neural Network

Each agent has a neural network with:
- **Inputs** (8): hunger, thirst, fear, nearest_agent_dist, nearest_resource_dist, era_factor, health, energy
- **Hidden layers**: 2 layers × 16 neurons (LeakyReLU)
- **Outputs** (8): move_direction, move_speed, gather, communicate, aggression, rest, reproduce, build

### Learning Rules

1. **Hebbian**: "Neurons that fire together, wire together"
2. **Reward-Modulated**: Weights adjusted by reward signal
3. **Evolutionary**: No online learning; selection + mutation

## Memory System

Hybrid approach balancing interpretability with scalability:

```json
{
  "agent_id": "uuid",
  "latent_state": [0.12, -0.34, ...],  // Compressed neural embedding
  "recent_events": [
    {"tick": 1000, "action": "Gather", "outcome": "Success", "reward": 0.5}
  ],
  "stats": {
    "survival_ticks": 50000,
    "offspring": 3
  }
}
```

## Era Modifiers

| Era | Years | Aggression | Cooperation | Technology |
|-----|-------|------------|-------------|------------|
| Prehistoric | 10,000 - 3,000 BCE | 1.5× | 0.5× | 0.0× |
| Ancient | 3,000 BCE - 500 CE | 1.2× | 0.8× | 0.2× |
| Medieval | 500 - 1500 CE | 1.0× | 1.0× | 0.5× |
| Modern | 1500 CE+ | 0.7× | 1.5× | 1.0× |

## Performance

Optimized for 50,000+ agents:
- **Spatial partitioning**: O(1) proximity queries via grid cells
- **Parallel processing**: `rayon` for batch neural network updates
- **Double buffering**: Lock-free simulation loop
- **Memory pools**: Pre-allocated agent memory to avoid churn

## CLI Options

```
reality-sim [OPTIONS]

Options:
  -y, --year <YEAR>         Starting year (negative for BCE) [default: -10000]
  -a, --agents <AGENTS>     Number of agents [default: 1000]
  -c, --climate <CLIMATE>   Climate severity (0.0-1.0) [default: 0.5]
  -r, --resources <RES>     Resource density (0.0-1.0) [default: 0.5]
  -t, --tps <TPS>           Ticks per second [default: 60]
  -s, --state-file <PATH>   Path to save/load world state
  -h, --help                Print help
```

## Example

```rust
use reality_sim::config::WorldConfig;
use reality_sim::simulation::SimulationRunner;

fn main() {
    let config = WorldConfig {
        starting_year: -10000,
        agent_count: 1000,
        climate_severity: 0.5,
        resource_density: 0.5,
        ..Default::default()
    };

    let mut sim = SimulationRunner::new(config);
    
    // Run for 10,000 ticks
    sim.run_ticks(10000);
    
    println!("Population: {}", sim.stats().current_population);
    println!("Deaths: {}", sim.stats().total_deaths);
}
```

## License

N/A