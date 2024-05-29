// Library
use crate::{
    parser::resp,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// Commands
mod echo;
mod get;
mod info;
mod ping;
mod psync;
mod replconf;
mod set;

/// Handles the incoming command by parsing it and calling the appropriate command handler.
pub async fn handle(
    cmd: Vec<resp::Type>,
    conn: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract the command from the parsed data
    let command = match cmd.get(0) {
        Some(resp::Type::BulkString(command)) => command,
        _ => {
            let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
            conn.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };

    // Handle the command
    match command.to_uppercase().as_str() {
        "PING" => ping::command(&cmd[1..], conn).await?,

        "ECHO" => echo::command(&cmd[1..], conn).await?,

        "SET" => {
            set::command(&cmd[1..], conn, server).await?;
            broadcast(server, cmd).await?;
        }

        "GET" => get::command(&cmd[1..], conn, server).await?,

        "INFO" => info::command(&cmd[1..], conn, server).await?,

        "REPLCONF" => replconf::command(&cmd[1..], conn, server).await?,

        "PSYNC" => psync::command(&cmd[1..], conn, server).await?,

        _ => {
            let response = resp::Type::SimpleError(format!("ERR unknown command: {:?}\r\n", cmd));
            conn.write_all(&response.as_bytes()).await?;
        }
    }

    Ok(())
}

// ----------------
// HELPER FUNCTIONS
// ----------------

/// Broadcast the value on the server's broadcast sender channel
async fn broadcast(
    server: &Arc<Mutex<Server>>,
    cmd: Vec<resp::Type>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the server instance from the Arc<Mutex<Server>>
    let server = server.lock().await;

    // If there are no receivers, return early
    if server.sender.receiver_count() == 0 {
        return Ok(());
    }

    // Broadcast the value to all receivers
    server.sender.send(resp::Type::Array(cmd))?;
    Ok(())
}
