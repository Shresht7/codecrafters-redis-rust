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
    println!("[{}] Server Initialized", server.addr);

    // Configure the server
    server
        .configure(config)
        .await
        .expect("Failed to configure the server");

    // Start the server
    println!("[{}] Server Starting...", server.addr);
    if let Err(e) = server.run().await {
        eprintln!("[ERROR]: {}", e);
        return; // Exit the program
    }
}
