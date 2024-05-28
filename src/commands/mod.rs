use tokio::{io::AsyncWriteExt, net::TcpStream, sync::MutexGuard};

// Library
use crate::{parser::resp, server::Server};

// Commands
mod echo;
mod get;
mod info;
mod ping;
mod psync;
mod replconf;
mod set;

/// Handles the incoming command by parsing it and calling the appropriate command handler.
pub async fn handle<'a>(
    cmd: Vec<resp::Type>,
    stream: &mut TcpStream,
    server: &mut MutexGuard<'a, Server>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get command array from parsed data
    let array = match cmd.get(0) {
        Some(resp::Type::Array(array)) => array,
        _ => {
            let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
            stream.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };

    // Extract the command from the parsed data
    let command = match array.get(0) {
        Some(resp::Type::BulkString(command)) => command,
        _ => {
            let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
            stream.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };

    // Handle the command
    match command.to_uppercase().as_str() {
        "PING" => ping::command(&array[1..], stream).await,
        "ECHO" => echo::command(&array[1..], stream).await,
        "SET" => set::command(&array[1..], stream, server).await,
        "GET" => get::command(&array[1..], stream, server).await,
        "INFO" => info::command(&array[1..], stream, server).await,
        "REPLCONF" => replconf::command(&array[1..], stream, server).await,
        "PSYNC" => psync::command(&array[1..], stream, server).await,
        _ => {
            let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
            stream.write_all(&response.as_bytes()).await?;
            Ok(())
        }
    }
}
