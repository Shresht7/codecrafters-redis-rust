// Library
use crate::{parser::resp, server::Server};
use std::sync::{Arc, Mutex};

/// Handles the PSYNC command
/// PSYNC is used to synchronize a replica with the master server.
/// The command takes two arguments: the replication ID and the replication offset.
/// The replica will use the replication ID to identify the master server.
/// The replica will use the replication offset to request new data from the master server.
pub fn command(_args: &[resp::Type], _server: &Arc<Mutex<Server>>) -> resp::Type {
    resp::Type::SimpleString("FULLRESYNC 0123456789 123\r\n".into())
}
