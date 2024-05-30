// Library
use crate::{
    parser::resp::Type,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

// --------
// REPLCONF
// --------

/// Handles the REPLCONF command.
pub async fn command(
    args: &[Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the command has the correct number of arguments
    if args.len() < 2 {
        return connection
            .write_error("ERR wrong number of arguments for 'REPLCONF' command. Usage REPLCONF <subcommand> <arg>")
            .await;
    }

    // Extract Sub-Command
    let subcommand = match args.get(0) {
        Some(Type::BulkString(subcommand)) => subcommand,
        x => {
            return connection
                .write_error(format!("ERR invalid subcommand {:?}", x))
                .await
        }
    };

    // Handle the REPLCONF subcommands
    match subcommand.to_uppercase().as_str() {
        "LISTENING-PORT" => connection.write_ok().await?,

        "CAPA" => connection.write_ok().await?,

        "GETACK" => get_ack(server, connection).await?,

        "ACK" => ack(args, wait_channel, connection).await?,

        _ => connection.write_ok().await?,
    }

    Ok(())
}

// ------------
// SUB-COMMANDS
// ------------

// GET ACK
// -------

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
    // Get the server address, replication offset, and role from the server instance
    let (addr, offset, role) = {
        let server = server.lock().await;
        (server.addr.clone(), server.repl_offset, server.role.clone())
    };

    // If the server is a master, return an error
    if role.is_master() {
        return connection
            .write_error("ERR GETACK can only be called on a replica")
            .await;
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

    // Update the replication offset of the replica
    {
        let mut server = server.lock().await;
        // TODO: Fix this. Hardcoded value for testing purposes (37 bytes for REPLCONF GETACK *)
        server.repl_offset += 37;
    }

    return Ok(());
}

// ACK
// ---

/// Handles the REPLCONF ACK subcommand.
/// When a replica receives a `REPLCONF GETACK *` command from the master, it responds with a `REPLCONF ACK <replication_offset>` command.
/// The `<replication_offset>` is the number of bytes of commands processed by the replica. It starts at 0
/// and is incremented for every command processed by the replica.
/// The master waits for the ACK response from the replica before sending more commands.
async fn ack(
    args: &[Type],
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract the offset from the arguments
    let offset = match args.get(1) {
        Some(Type::BulkString(offset)) => offset,
        _ => {
            return connection.write_error("ERR invalid offset").await;
        }
    };
    // Parse the offset as a u64
    let offset = match offset.parse::<u64>() {
        Ok(offset) => offset,
        Err(x) => {
            return connection
                .write_error(format!("ERR failed to parse offset as u64: {:?}", x))
                .await;
        }
    };

    // Send the offset to the master
    let wc = wait_channel.lock().await;
    println!("REPLCONF ACK: Received ACK with offset {}", offset);
    wc.0.send(offset).await?;
    println!("REPLCONF ACK: Sent ACK with offset {}", offset);

    connection.write_ok().await?;
    Ok(())
}
