// -----------
// REPLICATION
// -----------

// ROLE
// ----

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

// HANDSHAKE
// ---------
