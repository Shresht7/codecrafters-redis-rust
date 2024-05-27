// Modules
mod cli;
mod commands;
mod database;
mod parser;
mod server;

// ----
// MAIN
// ----

#[tokio::main]
async fn main() {
    // Parse the command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut cli = cli::CommandLineArguments { port: 6379 };
    cli.parse(args);

    // Determine the address using the port variable
    let port = cli.port; // Default port is 6379

    // Run the server on the given address and port
    let server = server::new("127.0.0.1", port);
    if let Err(e) = server.run().await {
        eprintln!("Error: {}", e);
    }
}
