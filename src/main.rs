// Library
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// Modules
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
    let cmd_args = CommandLineArguments::parse(args);

    // Determine the address using the port variable
    let port = cmd_args.port.unwrap_or(6379); // Default port is 6379
    let addr = format!("127.0.0.1:{}", port);

    // Run the server on the given address and port
    run_server(&addr).await.unwrap();
}

// ----------
// TCP SERVER
// ----------

/// Runs the TCP server on the given address, listening for incoming connections.
/// The server will handle each incoming connection in a separate thread.
async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a TCPListener and bind it to the given address and port
    // Note: 6379 is the default port that Redis uses (You may have to stop any running Redis instances)
    let listener = TcpListener::bind(addr).await?;

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

/// Handles the incoming connection stream by reading the incoming data,
/// parsing it, and writing a response back to the stream.
async fn handle_connection(
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

// ------------
// COMMAND LINE
// ------------

struct CommandLineArguments {
    port: Option<u16>,
}

impl CommandLineArguments {
    fn parse(args: Vec<String>) -> Self {
        // Declare the variables to store the command-line arguments
        let mut port = None;

        // Extract the port variable from the command-line arguments
        for i in 0..args.len() {
            match args[i].as_str() {
                "-p" | "--port" => {
                    if i + 1 < args.len() {
                        port = match args[i + 1].parse::<u16>() {
                            Ok(p) => Some(p),
                            Err(_) => {
                                eprintln!("Invalid port number: {}", args[i + 1]);
                                None
                            }
                        };
                    } else {
                        eprintln!("Port number not provided");
                        port = None;
                    };
                }
                _ => {}
            }
        }

        // Return the parsed command-line arguments
        Self { port }
    }
}
