// Library
use crate::{
    parser::resp,
    server::{connection::Connection, Server},
};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Mutex},
    time::timeout,
};

// Commands
mod echo;
mod get;
mod info;
mod ping;
mod psync;
mod replconf;
mod set;
mod wait;

/// Handles the incoming command by parsing it and calling the appropriate command handler.
pub async fn handle(
    cmd: &Vec<resp::Type>,
    conn: &mut Connection,
    server: &Arc<Mutex<Server>>,
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
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

        "REPLCONF" => replconf::command(&cmd[1..], conn, server, wait_channel).await?,

        "PSYNC" => {
            psync::command(&cmd[1..], conn, server).await?;
            receive(server, conn, wait_channel).await?;
        }

        "WAIT" => wait::command(&cmd[1..], conn, server, wait_channel).await?,

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
    cmd: &Vec<resp::Type>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the server instance from the Arc<Mutex<Server>>
    let server = server.lock().await;
    let addr = server.addr.clone();
    let role = server.role.clone();

    // If there are no receivers, return early
    if server.sender.receiver_count() == 0 {
        return Ok(());
    }

    // Broadcast the value to all receivers
    println!(
        "[{} - {}] Broadcasting: {:?} to {} receivers",
        addr,
        role,
        cmd,
        server.sender.receiver_count()
    );
    server.sender.send(resp::Type::Array(cmd.clone()))?;
    Ok(())
}

/// Receive messages from the broadcast channel
async fn receive(
    server: &Arc<Mutex<Server>>,
    conn: &mut Connection,
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Acquire the server lock and create a receiver
    let server = server.lock().await;
    let addr = server.addr.clone();
    let role = server.role.clone();
    let mut receiver = server.sender.subscribe();

    // ! Drop the server lock. This needs to be done manually because the receiver
    // ! will be waiting for messages from the broadcast channel. If the lock is not
    // ! dropped, the server will be locked indefinitely.
    drop(server);

    Ok(while let Ok(cmd) = receiver.recv().await {
        let array = match &cmd {
            resp::Type::Array(array) => array,
            _ => continue,
        };

        let command = match array.get(0) {
            Some(resp::Type::BulkString(command)) => command,
            _ => continue,
        };

        let subcommand = match array.get(1) {
            Some(resp::Type::BulkString(subcommand)) => subcommand,
            _ => continue,
        };

        let is_wait_cmd =
            command.to_uppercase() == "REPLCONF" && subcommand.to_uppercase() == "ACK";

        println!("Received broadcast: {:?}", cmd);

        if !is_wait_cmd {
            println!("Forwarding broadcast to connection: {:?}", cmd);
            // Forward all broadcast messages to the connection
            conn.write_all(&cmd.as_bytes()).await?;
            continue;
        }

        if is_wait_cmd {
            println!("Received REPLCONF ACK command");
            let offset = match array.get(2) {
                Some(resp::Type::BulkString(offset)) => offset,
                _ => continue,
            };

            println!("offset: {:?}", offset);
            let duration = Duration::from_millis(200);
            let res = timeout(duration, conn.stream.readable())
                .await
                .expect("Timed out waiting for replica to respond");

            if res.is_err() {
                println!("[{} - {}] Replica did not respond", addr, role);
                continue;
            }

            let mut bytes_read_vec = Vec::new();
            let buf = &mut [0; 1024];
            match conn.stream.try_read(buf) {
                Ok(0) => {
                    println!("[{}] Connection closed", addr);
                    break;
                }
                Ok(n) => {
                    bytes_read_vec.extend_from_slice(&buf[..n]);
                }
                Err(e) => {
                    println!("[{} - {}] Error: {:?}", addr, role, e);
                }
            }

            if !bytes_read_vec.is_empty() {
                let response = String::from_utf8_lossy(&bytes_read_vec);
                println!("[{} - {}] Received: {:?}", addr, role, response);
                continue;
            }

            println!("GOT BYTES: {:?}", bytes_read_vec);

            println!("[{} - {}] Received ACK with offset {}", addr, role, offset);

            // Send the offset to the wait channel
            let offset = offset.parse::<u64>()?;
            let wc = wait_channel.lock().await;
            wc.0.send(offset)
                .await
                .expect("Failed to send offset to wait channel");
        }
    })
}
