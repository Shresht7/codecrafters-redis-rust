// Library
use crate::{parser::resp, server::Server};
use std::sync::Arc;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{broadcast::Sender, Mutex},
};

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
    stream: &mut TcpStream,
    server: &Arc<Mutex<Server>>,
    sender: &Arc<Sender<resp::Type>>,
) -> Result<(), Box<dyn std::error::Error>> {
    for cmd in &cmds {
        if let resp::Type::Array(array) = cmd {
            // Extract the command from the parsed data
            let command = match array.get(0) {
                Some(resp::Type::BulkString(command)) => command,
                _ => {
                    let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
                    stream.write_all(&response.as_bytes()).await?;
                    return Ok(());
                }
            };

            println!("Received command: {:?}", command);

            // Handle the command
            match command.to_uppercase().as_str() {
                "PING" => ping::command(&array[1..], stream).await?,
                "ECHO" => echo::command(&array[1..], stream).await?,
                "SET" => {
                    println!("Sender count: {:?}", sender.receiver_count());
                    if sender.receiver_count() > 0 {
                        sender
                            .send(cmds[0].clone())
                            .expect("Failed to send message");
                    }
                    set::command(&array[1..], stream, server).await?
                }
                "GET" => get::command(&array[1..], stream, server).await?,
                "INFO" => info::command(&array[1..], stream, server).await?,
                "REPLCONF" => replconf::command(&array[1..], stream, server).await?,
                "PSYNC" => psync::command(&array[1..], stream, server, sender).await?,
                _ => {
                    let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
                    stream.write_all(&response.as_bytes()).await?;
                }
            }
        }
    }
    Ok(())
}
