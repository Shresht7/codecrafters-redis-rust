// Library
use crate::{
    parser::resp,
    server::{conn::Connection, Server},
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
    for cmd in &cmds {
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

            println!("Received command: {:?}", command);

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
                    let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
                    conn.write_all(&response.as_bytes()).await?;
                }
            }
        }
    }
    Ok(())
}

/// Broadcast the value on the server's broadcast sender channel
async fn broadcast(
    server: &Arc<Mutex<Server>>,
    value: resp::Type,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = server.lock().await;
    server.sender.send(value)?;
    Ok(())
}
