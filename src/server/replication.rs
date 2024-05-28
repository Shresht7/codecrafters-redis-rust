// Library
use crate::{
    parser::resp::{array, bulk_string},
    server::conn,
};
use tokio::net::TcpStream;

// -----------
// REPLICATION
// -----------

// ROLE
// ----

/// Enum to represent the role of the server.
/// The server can be either a master or a replica.
#[derive(Debug, Clone)]
pub enum Role {
    /// The server is a master
    Master,
    /// Stores the address of the replication master server
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

// HANDSHAKE
// ---------

impl Role {
    /// Sends a handshake to the replication master server at the given address.
    /// The handshake includes a PING command, REPLCONF listening-port, and REPLCONF capa psync2.
    pub async fn send_handshake(
        &self,
        port: u16,
    ) -> Result<conn::Connection, Box<dyn std::error::Error>> {
        // Get the address of the replication master.
        // Return an error if the server is a master. Master servers cannot send handshakes.
        let addr = match self {
            Role::Replica(addr) => addr,
            _ => Err("This instance is a master server and cannot send a handshake")?,
        };

        // Connect to the replication master
        let stream = TcpStream::connect(addr).await?;
        let mut connection = conn::new(stream);

        // Send a PING
        let response = array(vec![bulk_string("PING")]);
        connection.write_all(&response.as_bytes()).await?;
        connection.read().await?; // Read the PONG response (not used)

        // Send REPLCONF listening-port <PORT>
        let response = array(vec![
            bulk_string("REPLCONF"),
            bulk_string("listening-port"),
            bulk_string(port.to_string().as_str()),
        ]);
        connection.write_all(&response.as_bytes()).await?;
        connection.read().await?; // Read the OK response (not used)

        // Send REPLCONF capa psync2
        let response = array(vec![
            bulk_string("REPLCONF"),
            bulk_string("capa"),
            bulk_string("psync2"),
        ]);
        connection.write_all(&response.as_bytes()).await?;
        connection.read().await?; // Read the OK response (not used)

        // Send PSYNC <REPLID> <OFFSET>
        let response = array(vec![
            bulk_string("PSYNC"),
            bulk_string("?"),
            bulk_string("-1"),
        ]);
        connection.write_all(&response.as_bytes()).await?;
        connection.read().await?; // Read the FULLRESYNC response (not used)

        Ok(connection)
    }
}
