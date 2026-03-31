//! Basic simulation example
//!
//! Demonstrates running a small humanity simulation

use reality_sim::config::WorldConfig;
use reality_sim::simulation::SimulationRunner;

fn main() {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║     REALITY SIMULATOR - Basic Example               ║");
    println!("╚════════════════════════════════════════════════════╝\n");

    // Create world configuration
    let config = WorldConfig {
        starting_year: -10000,  // 10,000 BCE
        agent_count: 100,       // Start with 100 agents
        climate_severity: 0.6,  // Moderately harsh
        resource_density: 0.4,  // Somewhat scarce
        ticks_per_second: 60,
        world_width: 500.0,
        world_height: 500.0,
    };

    println!("Configuration:");
    println!("  Year: {} BCE", -config.starting_year);
    println!("  Agents: {}", config.agent_count);
    println!("  Climate: {:.0}% severity", config.climate_severity * 100.0);
    println!("  Resources: {:.0}% density", config.resource_density * 100.0);
    println!();

    // Create simulation
    let mut sim = SimulationRunner::new(config);

    println!("Initial state:");
    println!("  Population: {}", sim.stats().current_population);
    println!("  Resources: {}", sim.world.resources.len());
    println!();

    // Run simulation for 1000 ticks
    println!("Running simulation for 1000 ticks...");
    for i in 0..10 {
        sim.run_ticks(100);
        
        let stats = sim.stats();
        println!(
            "  Tick {:>4}: Pop={:>3}, Deaths={}, Avg Fitness={:.2}",
            (i + 1) * 100,
            stats.current_population,
            stats.total_deaths,
            stats.average_fitness
        );
    }

    println!();
    println!("Final state:");
    let final_stats = sim.stats();
    println!("  Population: {}", final_stats.current_population);
    println!("  Total spawned: {}", final_stats.total_agents_spawned);
    println!("  Total deaths: {}", final_stats.total_deaths);
    println!("  Max generation: {}", final_stats.max_generation);
    println!("  Average fitness: {:.2}", final_stats.average_fitness);

    // Find best agent
    if let Some(best) = sim.world.agents.iter()
        .map(|a| a.read())
        .max_by(|a, b| a.fitness().partial_cmp(&b.fitness()).unwrap())
    {
        println!();
        println!("Best agent:");
        println!("  ID: {}", best.id);
        println!("  Age: {} ticks", best.age);
        println!("  Generation: {}", best.generation);
        println!("  Fitness: {:.2}", best.fitness());
        println!("  Health: {:.0}%", best.health * 100.0);
    }

    println!();
    println!("Simulation complete!");
}
