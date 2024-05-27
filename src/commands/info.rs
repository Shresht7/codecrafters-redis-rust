// Library
use crate::{parser::resp::Type, server};
use std::sync::{Arc, Mutex};

/// Handles the INFO command.
/// The INFO command returns information and statistics about the server.
pub fn command(args: &[Type], server: &Arc<Mutex<server::Server>>) -> Type {
    // Check the number of arguments
    if args.len() < 1 {
        return Type::SimpleError("ERR wrong number of arguments for 'INFO' command".into());
    }

    // Get the role of the server
    let role = match server.lock().unwrap().role {
        server::Role::Master => "role:master",
        server::Role::Replica(_) => "role:slave",
    };

    // Get Master Replication ID and Offset
    let master_replid = server.lock().unwrap().master_replid.clone();
    let master_repl_offset = server.lock().unwrap().master_repl_offset;

    // Respond with the server information
    let response: String = vec![
        "# Replication".to_string(),
        role.to_string(),
        format!("master_replid:{}", master_replid),
        format!("master_repl_offset:{}", master_repl_offset),
    ]
    .join("\r\n");
    Type::BulkString(response)
}
