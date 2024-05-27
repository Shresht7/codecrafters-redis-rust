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

    // Respond with the server information
    let response: String = vec!["# Replication", role].join("\r\n");
    Type::BulkString(response)
}
