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
    cmds: Vec<resp::Type>,
    conn: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Iterate over the parsed commands
    // There can be multiple commands in a single request
    for cmd in &cmds {
        // All commands are in the shape of a RESP Array
        if let resp::Type::Array(array) = cmd {
            // Extract the command from the parsed data
            let command = match array.get(0) {
                Some(resp::Type::BulkString(command)) => command,
                _ => {
                    let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
                    conn.write_all(&response.as_bytes()).await?;
                    return Ok(());
                }
            };

            // Handle the command
            match command.to_uppercase().as_str() {
                "PING" => ping::command(&array[1..], conn).await?,

                "ECHO" => echo::command(&array[1..], conn).await?,

                "SET" => {
                    set::command(&array[1..], conn, server).await?;
                    broadcast(server, cmds[0].clone()).await?;
                }

                "GET" => get::command(&array[1..], conn, server).await?,

                "INFO" => info::command(&array[1..], conn, server).await?,

                "REPLCONF" => replconf::command(&array[1..], conn, server).await?,

                "PSYNC" => psync::command(&array[1..], conn, server).await?,

                _ => {
                    let response =
                        resp::Type::SimpleError(format!("ERR unknown command: {:?}\r\n", cmd));
                    conn.write_all(&response.as_bytes()).await?;
                }
            }
        } else {
            // If the command is not an array, ignore it for now
            let response = resp::Type::SimpleString("OK".into());
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
    value: resp::Type,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the server instance from the Arc<Mutex<Server>>
    let server = server.lock().await;

    // If there are no receivers, return early
    if server.sender.receiver_count() == 0 {
        return Ok(());
    }

    // Broadcast the value to all receivers
    server.sender.send(value)?;
    Ok(())
}
