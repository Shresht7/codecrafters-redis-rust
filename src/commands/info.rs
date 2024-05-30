// Library
use crate::{
    parser::resp::Type,
    server::{connection::Connection, replication::Role, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ----
// INFO
// ----

/// Handles the INFO command.
/// The INFO command returns information and statistics about the server.
pub async fn command(
    args: &[Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        return connection
            .write_error("ERR wrong number of arguments for 'INFO' command")
            .await;
    }

    // Lock the server instance
    let server = server.lock().await;

    // Get the role of the server
    let role = match server.role {
        Role::Master => "role:master",
        Role::Replica(_) => "role:slave",
    };

    // Get Master Replication ID and Offset
    let master_replid = server.master_replid.clone();
    let master_repl_offset = server.master_repl_offset;

    // Generate the response
    let response: String = vec![
        "# Replication".to_string(),
        role.to_string(),
        format!("master_replid:{}", master_replid),
        format!("master_repl_offset:{}", master_repl_offset),
    ]
    .join("\r\n");

    // Respond with the server information
    let response = Type::BulkString(response);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
