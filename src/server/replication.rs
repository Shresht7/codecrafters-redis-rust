use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

// Library
use crate::{
    helpers,
    parser::resp::{array, bulk_string},
    server::connection,
};
use tokio::net::TcpStream;

use super::connection::Kind;

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
    /// The handshake is used to synchronize the replica server with the master server.
    /// The master server sends replication data to the replica server after the handshake is successful.
    /// The handshake consists of the following steps:
    /// 1. Send a PING command to the master server.
    /// 2. Send a REPLCONF listening-port <PORT> command to inform the master server of the port that the replica server is listening on.
    /// 3. Send a REPLCONF capa psync2 command to inform the master server that the replica server supports the PSYNC2 command.
    /// 4. Send a PSYNC <REPLID> <OFFSET> command to synchronize the replica server with the master server.
    /// The REPLID and OFFSET are used to identify the replication stream and the offset of the last received command.
    /// The REPLID is "?" if the replica server is syncing for the first time.
    /// The OFFSET is -1 if the replica server is syncing for the first time.
    /// The REPLID and OFFSET are used to resume replication from the last received command.
    /// Returns a connection to the master server if the handshake is successful.
    /// This connection is used to receive replication data from the master server.
    pub async fn send_handshake(
        &self,
        port: u16,
    ) -> Result<connection::Connection, Box<dyn std::error::Error>> {
        // Get the address of the replication master.
        // Return an error if the server is a master. Master servers cannot send handshakes.
        let addr = match self {
            Role::Replica(addr) => addr,
            _ => Err("This instance is a master server and cannot send a handshake")?,
        };

        // Connect to the replication master
        let stream = TcpStream::connect(&addr).await?;
        let (_, master_port) = helpers::split_host_and_port(addr.clone(), ":")?;
        let mut connection = connection::new(
            stream,
            SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), master_port.clone()),
            Kind::Replication,
        );

        // Send a PING
        send_ping(&mut connection).await?;

        // Send REPLCONF listening-port <PORT>
        send_replconf_listening_port(&mut connection, port).await?;

        // Send REPLCONF capa psync2
        send_replconf_capa_psync2(&mut connection).await?;

        // Send PSYNC <REPLID> <OFFSET>
        send_psync(&mut connection, "?", -1).await?;

        // Return the connection to the master server so that
        // we can re-use the same connection for replication.
        Ok(connection)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Role::Master => write!(f, "master"),
            Role::Replica(addr) => write!(f, "replica {}", addr),
        }
    }
}

// PING
// ----

/// Sends a PING command to the replication master server.
async fn send_ping(
    connection: &mut connection::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send a PING
    let response = array(vec![bulk_string("PING")]);
    connection.write_all(&response.as_bytes()).await?;
    connection.read().await?; // Read the PONG response (not used)
    Ok(())
}

// REPLCONF listening-port <PORT>
// ------------------------------

/// Sends a REPLCONF listening-port <PORT> command to the replication master server.
/// The command is used to inform the master server of the port that the replica server is listening on.
/// The master server uses this information to send replication data to the replica server.
async fn send_replconf_listening_port(
    connection: &mut connection::Connection,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send REPLCONF listening-port <PORT>
    let response = array(vec![
        bulk_string("REPLCONF"),
        bulk_string("listening-port"),
        bulk_string(port.to_string().as_str()),
    ]);
    connection.write_all(&response.as_bytes()).await?;
    connection.read().await?; // Read the OK response (not used)
    Ok(())
}

// REPLCONF capa psync2
// --------------------

// REPLCONF capa psync2 is sent as part of the handshake to inform the master server that the replica server supports the PSYNC2 command.
async fn send_replconf_capa_psync2(
    connection: &mut connection::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send REPLCONF capa psync2
    let response = array(vec![
        bulk_string("REPLCONF"),
        bulk_string("capa"),
        bulk_string("psync2"),
    ]);
    connection.write_all(&response.as_bytes()).await?;
    connection.read().await?; // Read the OK response (not used)
    Ok(())
}

// PSYNC
// -----

// PSYNC is used to synchronize the replica server with the master server.
async fn send_psync(
    connection: &mut connection::Connection,
    replid: &str,
    offset: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send PSYNC <REPLID> <OFFSET>
    let response = array(vec![
        bulk_string("PSYNC"),
        bulk_string(replid),
        bulk_string(offset.to_string().as_str()),
    ]);
    connection.write_all(&response.as_bytes()).await?;
    connection.read().await?; // Read the FULLRESYNC response (not used)
    Ok(())
}
