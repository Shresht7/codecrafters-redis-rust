// Library
use crate::{parser::resp::Type, server};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

/// Handles the INFO command.
/// The INFO command returns information and statistics about the server.
pub async fn command(
    args: &[Type],
    stream: &mut TcpStream,
    server: &Arc<Mutex<server::Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        let response =
            Type::SimpleError("ERR at least one argument is required for 'INFO' command".into());
        stream.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Get server instance from the Server
    println!("[info.rs::fn command] Acquiring lock");
    let server = server.lock().await;

    // Get the role of the server
    let role = match server.role {
        server::Role::Master => "role:master",
        server::Role::Replica(_) => "role:slave",
    };

    // Get Master Replication ID and Offset
    let master_replid = server.master_replid.clone();
    let master_repl_offset = server.master_repl_offset;

    // Respond with the server information
    let response: String = vec![
        "# Replication".to_string(),
        role.to_string(),
        format!("master_replid:{}", master_replid),
        format!("master_repl_offset:{}", master_repl_offset),
    ]
    .join("\r\n");

    let response = Type::BulkString(response);
    stream.write_all(&response.as_bytes()).await?;
    Ok(())
}
