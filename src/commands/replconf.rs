// Library
use crate::{
    parser::resp::Type,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Handles the REPLCONF command.
pub async fn command(
    args: &[Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the command has the correct number of arguments
    if args.len() < 2 {
        let response =
            Type::SimpleError("ERR wrong number of arguments for 'replconf' command".into());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Extract Sub-Command
    let subcommand = match args.get(0) {
        Some(Type::BulkString(subcommand)) => subcommand,
        x => {
            let response = Type::SimpleError(format!(
                "ERR Protocol error: expected bulk string but got '{:?}'",
                x
            ));
            connection.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };

    println!("REPLCONF: {:?}", subcommand);

    println!("REPLCONF: {:?}", args);

    // Handle the REPLCONF GETACK command
    match subcommand.to_uppercase().as_str() {
        "LISTENING-PORT" => {
            let response = Type::SimpleString("OK".into());
            connection.write_all(&response.as_bytes()).await?;
        }
        "CAPA" => {
            let response = Type::SimpleString("OK".into());
            connection.write_all(&response.as_bytes()).await?;
        }
        "GETACK" => get_ack(server, connection).await?,
        "ACK" => {
            println!("REPLCONF ACK: {:?}", args);
            let offset = match args.get(1) {
                Some(Type::BulkString(offset)) => offset.parse::<u64>().unwrap_or(0),
                _ => {
                    let wc = wait_channel.lock().await;
                    wc.0.send(0).await?;
                    connection
                        .write_all(&Type::SimpleString("OK".into()).as_bytes())
                        .await?;
                    return Ok(());
                }
            };
            let wc = wait_channel.lock().await;

            println!("REPLCONF ACK: Received ACK with offset {}", offset);
            wc.0.send(offset).await?;
            println!("REPLCONF ACK: Sent ACK with offset {}", offset);
            connection
                .write_all(&Type::SimpleString("OK".into()).as_bytes())
                .await?;
            return Ok(());
        }
        _ => {
            let ok = Type::SimpleString("OK".into());
            connection.write_all(&ok.as_bytes()).await?;
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
pub async fn get_ack(
    server: &Arc<Mutex<Server>>,
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the current replication offset from the server
    let (addr, offset, role) = {
        let server = server.lock().await;
        (server.addr.clone(), server.repl_offset, server.role.clone())
    };

    if role.is_master() {
        let response = Type::SimpleError("ERR This instance is a master".into());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    println!(
        "[{}] REPLCONF ACK: Sending ACK with offset {} to {}",
        addr, offset, connection.addr
    );

    // Send the REPLCONF ACK response
    let response = Type::Array(vec![
        Type::BulkString("REPLCONF".into()),
        Type::BulkString("ACK".into()),
        Type::BulkString(offset.to_string()),
    ]);

    let bytes = response.as_bytes();

    connection.write_all(&bytes).await?;
    println!("[{}] REPLCONF ACK: Sent ACK", addr);
    return Ok(());
}
