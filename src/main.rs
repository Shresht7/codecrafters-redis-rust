// Modules
mod commands;
mod config;
mod database;
mod helpers;
mod parser;
mod server;

// ----
// MAIN
// ----

#[tokio::main]
async fn main() {
    // Parse the configuration parameters from the command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let config = config::from_command_line(args).expect("Failed to parse command-line arguments");

    // Instantiate the server with the address
    let mut server = server::new("127.0.0.1", config.port);
    println!("Server listening on: {}", server.addr);

    // If the replica-of address is set, the server will act as a replica
    if let Some(replicaof) = config.replicaof {
        println!("Replica of: {}", replicaof);
        server.replicaof(&replicaof);
    }

    // Start the server
    if let Err(e) = server.run().await {
        eprintln!("[ERROR]: {}", e);
        return; // Exit the program
    }
}
