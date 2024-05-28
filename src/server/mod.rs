// Library
use crate::{database, helpers, parser::resp::Type};
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{broadcast, Mutex},
};

// Modules
pub mod conn;
pub mod replication;
use replication::Role;

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

    /// The broadcast sender is used to send the server instance to each thread.
    /// This allows each thread to access the server instance and share data across threads.
    pub sender: broadcast::Sender<Type>,
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
        sender: broadcast::channel(16).0,
    }
}

impl Server {
    /// Runs the TCP server on the given address, listening for incoming connections.
    /// The server will handle each incoming connection in a separate thread.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clone the server instance and wrap it in an Arc<Mutex<Server>>
        // This will allows us to share the server instance across threads.
        let server = Arc::new(Mutex::new(self.clone()));

        // If this server is a replica, connect to the master server
        if let Role::Replica(master_addr) = &self.role {
            self.handle_replication(master_addr, &server).await?;
        }

        // Handle the main connection
        self.handle_main_connections(server).await?;

        Ok(())
    }

    /// Handles replication for the replica server.
    /// Connects to the master server at the given address and spawns a new thread to handle the connection.
    async fn handle_replication(
        &self,
        master_addr: &String,
        server: &Arc<Mutex<Server>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[{}] Connecting to master server at {}",
            self.addr, master_addr
        );
        // Send handshake and establish connection with the master server
        let mut connection = self.role.send_handshake(self.port).await?;

        // Clone the Arc<Mutex<Server>> instance
        let server = Arc::clone(server);

        // Handle the connection
        tokio::spawn(async move {
            connection
                .handle(&server)
                .await
                .expect("Failed to handle connection");
        });
        Ok(())
    }

    /// Handles the main logic for the server.
    /// Listens for incoming connections on the server's address and spawns a new thread to handle each connection.
    async fn handle_main_connections(
        &self,
        server: Arc<Mutex<Server>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        Ok(while let Ok((stream, _)) = listener.accept().await {
            // Create a new Connection instance for the incoming connection
            let mut connection = conn::new(stream);

            // Clone the Arc<Mutex<Server>> instance
            let server = Arc::clone(&server);

            // ... and spawn a new thread for each incoming connection
            tokio::spawn(async move {
                connection
                    .handle(&server)
                    .await
                    .expect("Failed to handle connection");
            });
        })
    }

    /// Sets the server to act as a replica of the given address.
    /// The server will act as a replica and connect to the master server at the given address.
    /// The server will send a handshake to the master server to establish the connection.
    /// The server will start receiving data from the master server.
    pub fn replicaof(&mut self, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        self.role = Role::Replica(addr);
        Ok(())
    }
}
