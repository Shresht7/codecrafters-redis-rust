// Library
use crate::{parser::resp::Type, server::Server};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

/// Handles the REPLCONF command.
pub async fn command(
    args: &[Type],
    stream: &mut TcpStream,
    _server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the command has the correct number of arguments
    if args.len() != 2 {
        let response =
            Type::SimpleError("ERR wrong number of arguments for 'replconf' command".into());
        stream.write_all(&response.as_bytes()).await?;
        stream.flush().await?;
        return Ok(());
    }

    // Handle the REPLCONF GETACK command
    match args[0].to_string().to_uppercase().as_str() {
        "GETACK" => get_ack(stream).await?,
        _ => {
            let response =
                Type::SimpleError("ERR unknown subcommand for 'replconf' command".into());
            stream.write_all(&response.as_bytes()).await?;
            stream.flush().await?;
        }
    }

    Ok(())
}

// ------------
// SUB-COMMANDS
// ------------

/// Handles the REPLCONF GETACK subcommand.
/// When a master requires an acknowledgement from a replica, it sends a `REPLCONF GETACK *` to the replica.
/// This is sent over the replication connection (the connection that was established with the handshake).
/// The replica responds with a `REPLCONF ACK <replication_offset>` response.
/// The `<replication_offset>` is the number of bytes of commands processed by the replica. It starts at 0
/// and is incremented for every command processed by the replica.
pub async fn get_ack(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let response = Type::Array(vec![
        Type::BulkString("REPLCONF".into()),
        Type::BulkString("ACK".into()),
        Type::Integer(0),
    ]);
    stream.write_all(&response.as_bytes()).await?;
    stream.flush().await?;
    return Ok(());
}
