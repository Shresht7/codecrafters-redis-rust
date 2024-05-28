// Modules
mod cli;
mod commands;
mod database;
mod helpers;
mod parser;
mod server;

// ----
// MAIN
// ----

#[tokio::main]
async fn main() {
    // Parse the command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut cli = cli::CommandLineArguments::default();
    if let Err(e) = cli.parse(args) {
        eprintln!("Failed to parse the command line arguments!\n{}", e);
        return;
    }

    // Determine the address using the port variable
    let port = cli.port; // Default port is 6379

    // Instantiate the server with the address
    let mut server = server::new("127.0.0.1", port);

    // If the replica-of address is set, the server will act as a replica
    if let Some(replicaof) = cli.replicaof {
        println!("Replicating from: {}", replicaof);
        server.replicaof(&replicaof);
    }

    // Start the server
    if let Err(e) = server.run().await {
        eprintln!("Error: {}", e);
        return;
    }
}
