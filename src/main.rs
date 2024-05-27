// Library
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// Modules
mod cli;
mod commands;
mod database;
mod parser;

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
    let server = Server::new("127.0.0.1", port);
    if let Err(e) = server.run().await {
        eprintln!("Error: {}", e);
    }
}

// ----------
// TCP SERVER
// ----------

/// Struct to hold the server configuration
struct Server {
    /// The full address to listen on
    addr: String,
}

impl Server {
    /// Creates a new Server instance with the given host and port
    pub fn new(host: &str, port: u16) -> Self {
        Server {
            addr: format!("{}:{}", host, port),
        }
    }

    /// Runs the TCP server on the given address, listening for incoming connections.
    /// The server will handle each incoming connection in a separate thread.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create a TCPListener and bind it to the given address and port
        // Note: 6379 is the default port that Redis uses (You may have to stop any running Redis instances)
        let listener = TcpListener::bind(&self.addr).await?;

        // Listen for incoming connections and handle them
        loop {
            // Accept an incoming connection ...
            let (mut stream, _) = listener.accept().await?;
            // ... and spawn a new thread for each incoming connection
            tokio::spawn(async move {
                handle_connection(&mut stream).await.unwrap();
            });
        }
    }
}

/// Handles the incoming connection stream by reading the incoming data,
/// parsing it, and writing a response back to the stream.
pub async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    // Database
    let mut db = database::Database::new();

    /// The size of the buffer to read incoming data
    const BUFFER_SIZE: usize = 1024;

    // Loop as long as requests are being made
    loop {
        // Read the incoming data from the stream
        let mut buffer = [0; BUFFER_SIZE];
        let bytes_read = stream.read(&mut buffer).await?;

        // If no bytes were read, the client has closed the connection
        if bytes_read == 0 {
            break;
        }

        // Print the incoming data
        let cmd = parser::parse(&buffer[..bytes_read])?;

        // Handle the parsed data and get a response
        let response = commands::handle(cmd, &mut db)?;

        // Write a response back to the stream
        stream.write_all(response.as_bytes()).await?;

        // Flush the stream to ensure the response is sent
        stream.flush().await?;
    }

    Ok(())
}
