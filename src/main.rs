use threebody_sim::{Simulator, parse_ini_file};
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Get config file path from command line argument
    let config_file = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from("./data/ic.ini")
    };

    // Get output file from command line or use default
    let output_file = if args.len() > 2 {
        args[2].clone()
    } else {
        String::from("./data/results.csv")
    };

    println!("Loading configuration from: {}", config_file);
    
    // Parse the configuration file
    let config = parse_ini_file(&config_file)?;
    
    println!("Loaded {} bodies", config.bodies.len());
    for (idx, body) in config.bodies.iter().enumerate() {
        println!(
            "  Body {}: mass={:.4e}, pos=[{:.4e}, {:.4e}, {:.4e}], vel=[{:.4e}, {:.4e}, {:.4e}]",
            idx,
            body.mass,
            body.position[0], body.position[1], body.position[2],
            body.velocity[0], body.velocity[1], body.velocity[2]
        );
    }

    // Create simulator with the configuration
    println!("\nCreating simulator with time step: {:.2} s, {} steps", 
             config.time_step, config.num_steps);
    let mut sim = Simulator::with_output(config.bodies, config.time_step, &output_file)?;

    println!("\nInitial state:");
    sim.print_positions();
    println!("Initial total energy: {:.6e} J", sim.total_energy());

    // Run the simulation
    println!("\nRunning simulation for {} steps...", config.num_steps);
    sim.run(config.num_steps);

    println!("\nFinal state:");
    sim.print_positions();
    println!("Final total energy: {:.6e} J", sim.total_energy());
    
    println!("Simulation time elapsed: {:.2} days", sim.time() / 86400.0);
    println!("\nResults saved to: {}", output_file);

    Ok(())
}
