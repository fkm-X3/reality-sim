use clap::Parser;
use reality_sim::WorldConfig;

#[derive(Parser, Debug)]
#[command(name = "reality-sim")]
#[command(about = "A Humanity Simulator with neural network agents")]
struct Args {
    /// Starting year for the simulation (negative for BCE)
    #[arg(short, long, default_value_t = -10000)]
    year: i32,

    /// Number of agents to simulate
    #[arg(short, long, default_value_t = 1000)]
    agents: usize,

    /// Climate severity (0.0 = mild, 1.0 = harsh)
    #[arg(short, long, default_value_t = 0.5)]
    climate: f32,

    /// Resource density (0.0 = scarce, 1.0 = abundant)
    #[arg(short, long, default_value_t = 0.5)]
    resources: f32,

    /// Ticks per second (simulation speed)
    #[arg(short, long, default_value_t = 60)]
    tps: u32,

    /// Path to save/load world state
    #[arg(short, long)]
    state_file: Option<String>,
}

fn main() {
    let args = Args::parse();

    let config = WorldConfig {
        starting_year: args.year,
        agent_count: args.agents,
        climate_severity: args.climate.clamp(0.0, 1.0),
        resource_density: args.resources.clamp(0.0, 1.0),
        ticks_per_second: args.tps,
        world_width: 1000.0,
        world_height: 1000.0,
    };

    println!("╔════════════════════════════════════════╗");
    println!("║       REALITY SIMULATOR v0.1.0         ║");
    println!("╠════════════════════════════════════════╣");
    println!("║ Year: {:>10} {:>18} ║", config.starting_year, if config.starting_year < 0 { "BCE" } else { "CE" });
    println!("║ Agents: {:>8}                        ║", config.agent_count);
    println!("║ Climate: {:>7.1}%                      ║", config.climate_severity * 100.0);
    println!("║ Resources: {:>5.1}%                      ║", config.resource_density * 100.0);
    println!("╚════════════════════════════════════════╝");

    // TODO: Initialize world state and run simulation loop
    println!("\nSimulation scaffolding ready. Implementation in progress...");
}
