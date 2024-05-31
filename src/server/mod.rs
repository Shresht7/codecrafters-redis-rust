// Library
use crate::{config::Config, database, helpers, parser::resp::Type};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, Mutex},
};

// Modules
pub mod connection;
use connection::Kind;
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

    /// The number of bytes processed by the replica.
    pub repl_offset: u64,

    /// The list of replica servers connected to this master server.
    /// Stores the address of each replica server connected to this master server.
    pub replicas: Vec<SocketAddr>,

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
        repl_offset: 0,
        replicas: Vec::new(),
        sender: broadcast::channel(16).0,
    }
}

impl Server {
    /// Configures the server with the given configuration parameters.
    /// The server will set the replica-of address, directory, and dbfilename based on the configuration.
    /// Does NOT configure the port as it must be set when the server is instantiated.
    pub async fn configure(&mut self, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        // Set the replica-of address
        if let Some(addr) = config.replicaof {
            self.role = Role::Replica(addr);
        }

        // Set the directory
        if let Some(dir) = config.dir {
            self.db.dir = dir;
        }

        // Set the dbfilename
        if let Some(dbfilename) = config.dbfilename {
            self.db.dbfilename = dbfilename;
        }

        // Load the database
        self.db.load().await?;

        println!(
            "[{}] Server Configured: Role: {:?}, Directory: {}, DBFilename: {}",
            self.addr, self.role, self.db.dir, self.db.dbfilename
        );

        Ok(())
    }

    /// Runs the TCP server on the given address, listening for incoming connections.
    /// The server will handle each incoming connection in a separate thread.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clone the server instance and wrap it in an Arc<Mutex<Server>>
        // This will allows us to share the server instance across threads.
        let server = Arc::new(Mutex::new(self.clone()));
        let wait_channel = Arc::new(Mutex::new(mpsc::channel::<u64>(64)));

        // TODO: There seems to be a race condition here. There is a possibility
        // that the connection isn't established before the master server sends data.

        // If this server is a replica, connect to the master server
        if let Role::Replica(master_addr) = &self.role {
            self.handle_replication(master_addr, &server, &wait_channel)
                .await?;
        }

        // Handle the main connection
        self.handle_main_connections(server, &wait_channel).await?;

        Ok(())
    }

    /// Handles replication for the replica server.
    /// Connects to the master server at the given address and spawns a new thread to handle the connection.
    async fn handle_replication(
        &self,
        master_addr: &String,
        server: &Arc<Mutex<Server>>,
        wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[{}] Connecting to master server at {}...",
            self.addr, master_addr
        );
        // Send handshake and establish connection with the master server
        let mut connection = self.role.send_handshake(self.port).await?;
        println!("[{}] Connection Established to {}", self.addr, master_addr);

        // Clone the Arc<Mutex<Server>> instance
        let server = Arc::clone(server);
        let wait_channel = Arc::clone(wait_channel);

        // Handle the connection
        tokio::spawn(async move {
            println!("New replication connection from {}", connection.addr);
            connection
                .handle(&server, &wait_channel)
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
        wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Bind the server to the address and start listening for incoming connections
        let listener = TcpListener::bind(&self.addr).await?;
        println!("[{}] Server is listening on {}", self.addr, self.port);
        Ok(while let Ok((stream, addr)) = listener.accept().await {
            // Create a new Connection instance for the incoming connection
            let mut connection = connection::new(stream, addr, Kind::Main);

            // Clone the Arc<Mutex<Server>> instance
            let server = Arc::clone(&server);
            let wait_channel = Arc::clone(wait_channel);

            // ... and spawn a new thread for each incoming connection
            tokio::spawn(async move {
                println!("New main connection from {}", connection.addr);
                connection
                    .handle(&server, &wait_channel)
                    .await
                    .expect("Failed to handle connection");
            });
        })
    }
}
