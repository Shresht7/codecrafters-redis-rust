// Library
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// Modules
use crate::{commands, database, helpers, parser};

// ----------
// TCP SERVER
// ----------

/// Struct to hold information about the Server and its configuration
#[derive(Clone)]
pub struct Server {
    /// The full address (host:port) to listen on
    addr: String,

    /// The database instance to store data
    pub db: database::Database,

    /// The role of the server (master or replica)
    pub role: Role,

    /// The master replication ID is used to identify the master server in a replication setup.
    /// The master server will generate a new ID every time it starts.
    /// The replica server will use this ID to identify the master server.
    pub master_replid: String,

    /// The replication offset is used to keep track of the last byte read from the master server.
    /// The replica server will use this offset to request new data from the master server.
    /// The master server will use this offset to send new data to the replica server.
    /// The offset is set to 0 when the server starts.
    /// The offset is incremented every time new data is read from the master server.
    pub master_repl_offset: u64,
}

/// Enum to represent the role of the server.
/// The server can be either a master or a replica.
#[derive(Clone)]
pub enum Role {
    Master,
    Replica(String),
}

/// Creates a new Server instance with the given host and port
pub fn new(host: &str, port: u16) -> Server {
    Server {
        addr: format!("{}:{}", host, port),
        role: Role::Master,
        db: database::Database::new(),
        master_replid: helpers::generate_id(40),
        master_repl_offset: 0,
    }
}

impl Server {
    /// Runs the TCP server on the given address, listening for incoming connections.
    /// The server will handle each incoming connection in a separate thread.
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create a TCPListener and bind it to the given address and port
        // Note: 6379 is the default port that Redis uses (You may have to stop any running Redis instances)
        let listener = TcpListener::bind(&self.addr).await?;

        // Listen for incoming connections and handle them
        loop {
            // Accept an incoming connection ...
            let (mut stream, _) = listener.accept().await?;

            // Clone the server instance and wrap it in an Arc<Mutex<Server>>
            // This allows us to share the server instance across threads.
            let server = Arc::new(Mutex::new(self.clone()));

            // ... and spawn a new thread for each incoming connection
            tokio::spawn(async move {
                handle_connection(&server, &mut stream).await.unwrap();
            });
        }
    }

    /// Sets the server to act as a replica of the given address
    pub fn replicaof(&mut self, addr: String) {
        self.role = Role::Replica(addr);
    }
}

/// Handles the incoming connection stream by reading the incoming data,
/// parsing it, and writing a response back to the stream.
async fn handle_connection(
    server: &Arc<Mutex<Server>>,
    stream: &mut tokio::net::TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
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

        println!("Incoming Request: {:?}", cmd);

        // Handle the parsed data and get a response
        let response = commands::handle(cmd, server)?;

        println!("Outgoing Response: {:?}", response);

        // Write a response back to the stream
        stream.write_all(response.as_bytes()).await?;

        // Flush the stream to ensure the response is sent
        stream.flush().await?;
    }

    Ok(())
}
