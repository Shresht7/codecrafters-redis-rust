// Library
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};

// Modules
use crate::{
    commands, database, helpers,
    parser::{
        self,
        resp::{array, bulk_string, Type},
    },
};

// ----------
// TCP SERVER
// ----------

/// Struct to hold information about the Server and its configuration
#[derive(Clone)]
pub struct Server {
    /// The host to listen on
    // host: &'static str,

    /// The port to listen on (default is 6379)
    port: u16,

    /// The full address (host:port) to listen on
    pub addr: String,

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
#[derive(Debug, Clone)]
pub enum Role {
    Master,
    Replica(String),
}

impl Role {
    /// Returns true if the server is a master
    pub fn is_master(&self) -> bool {
        match self {
            Role::Master => true,
            _ => false,
        }
    }

    // /// Returns true if the server is a replica
    // pub fn is_replica(&self) -> bool {
    //     match self {
    //         Role::Replica(_) => true,
    //         _ => false,
    //     }
    // }
}

/// Creates a new Server instance with the given host and port
pub fn new(host: &'static str, port: u16) -> Server {
    Server {
        // host,
        port,
        addr: format!("{}:{}", host, port),
        role: Role::Master,
        db: database::new(),
        master_replid: helpers::generate_id(40),
        master_repl_offset: 0,
    }
}

/// The size of the buffer to read incoming data
const BUFFER_SIZE: usize = 1024;

impl Server {
    /// Runs the TCP server on the given address, listening for incoming connections.
    /// The server will handle each incoming connection in a separate thread.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create a TCPListener and bind it to the given address and port
        // Note: 6379 is the default port that Redis uses (You may have to stop any running Redis instances)
        let listener = TcpListener::bind(&self.addr).await?;

        // Clone the server instance and wrap it in an Arc<Mutex<Server>>
        // This will allows us to share the server instance across threads.
        let server = Arc::new(Mutex::new(self.clone()));

        // Create a broadcast channel to send the server instance to each thread
        let sender: broadcast::Sender<Type> = broadcast::channel(16).0;
        let sender = Arc::new(sender);

        // If this server is a replica, connect to the master server
        if let Role::Replica(addr) = &self.role {
            println!("Connecting to master server at {}", addr);
            let mut stream = self.send_handshake(addr).await?;

            // Clone the Arc<Mutex<Server>> instance
            let server = Arc::clone(&server);
            // Clone the broadcast sender instance
            let sender = Arc::clone(&sender);

            // Spawn a new thread to handle the replication connection
            tokio::spawn(async move {
                handle_connection(&server, &mut stream, sender)
                    .await
                    .expect("Failed to handle connection".into())
            });
        }

        // Listen for incoming connections and handle them
        while let Ok((mut stream, _)) = listener.accept().await {
            // Clone the Arc<Mutex<Server>> instance
            let server = Arc::clone(&server);
            // Clone the broadcast sender instance
            let sender = Arc::clone(&sender);

            // ... and spawn a new thread for each incoming connection
            tokio::spawn(async move {
                handle_connection(&server, &mut stream, sender)
                    .await
                    .expect("Failed to handle connection".into())
            });
        }

        Ok(())
    }

    /// Sets the server to act as a replica of the given address
    pub fn replicaof(&mut self, addr: &String) {
        self.role = Role::Replica(addr.clone());
    }

    /// Sends a handshake to the replication master server at the given address.
    /// The handshake includes a PING command, REPLCONF listening-port, and REPLCONF capa psync2.
    pub async fn send_handshake(
        &self,
        addr: &String,
    ) -> Result<TcpStream, Box<dyn std::error::Error>> {
        // Connect to the replication master
        let mut stream = TcpStream::connect(addr).await?;

        // Send a PING
        let response = array(vec![bulk_string("PING")]);
        stream.write_all(&response.as_bytes()).await?;
        stream.flush().await?;
        stream.read(&mut [0; BUFFER_SIZE]).await?; // Read the PONG response (not used)

        // Send REPLCONF listening-port <PORT>
        let response = array(vec![
            bulk_string("REPLCONF"),
            bulk_string("listening-port"),
            bulk_string(self.port.to_string().as_str()),
        ]);
        stream.write_all(&response.as_bytes()).await?;
        stream.flush().await?;
        stream.read(&mut [0; BUFFER_SIZE]).await?; // Read the OK response (not used)

        // Send REPLCONF capa psync2
        let response = array(vec![
            bulk_string("REPLCONF"),
            bulk_string("capa"),
            bulk_string("psync2"),
        ]);
        stream.write_all(&response.as_bytes()).await?;
        stream.flush().await?;
        stream.read(&mut [0; BUFFER_SIZE]).await?; // Read the OK response (not used)

        // Send PSYNC <REPLID> <OFFSET>
        let response = array(vec![
            bulk_string("PSYNC"),
            bulk_string("?"),
            bulk_string("-1"),
        ]);
        stream.write_all(&response.as_bytes()).await?;
        stream.flush().await?;
        stream.read(&mut [0; BUFFER_SIZE]).await?; // Read the FULLRESYNC response (not used)

        Ok(stream)
    }
}

/// Handles the incoming connection stream by reading the incoming data,
/// parsing it, and writing a response back to the stream.
async fn handle_connection(
    server: &Arc<Mutex<Server>>,
    stream: &mut tokio::net::TcpStream,
    sender: Arc<broadcast::Sender<Type>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Loop as long as requests are being made
    loop {
        // Read the incoming data from the stream
        let mut buffer = [0; BUFFER_SIZE];
        let bytes_read = stream.read(&mut buffer).await.expect("Failed to read data");

        // If no bytes were read, the client has closed the connection
        if bytes_read == 0 {
            break;
        }

        // Print the incoming data
        let cmd = parser::parse(&buffer[..bytes_read]).expect("Failed to parse data");

        println!("Incoming Request: {:?}", cmd);

        // Handle the parsed data and get a response
        commands::handle(cmd, stream, server, &sender)
            .await
            .expect("Failed to handle command");
    }

    Ok(())
}
