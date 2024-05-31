// Library
use crate::{
    parser::{self, resp},
    server::{connection::Connection, Server},
};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Mutex},
    time::timeout,
};

// Commands
mod config;
mod echo;
mod get;
mod info;
mod keys;
mod ping;
mod psync;
mod replconf;
mod set;
mod type_cmd;
mod wait;
mod xadd;
mod xrange;

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
        "PING" => ping::command(cmd, conn, server).await?,

        "ECHO" => echo::command(&cmd[1..], conn).await?,

        "SET" => {
            set::command(cmd, conn, server).await?;
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

        "CONFIG" => config::command(&cmd, conn, server).await?,

        "KEYS" => keys::command(&cmd, conn, server).await?,

        "TYPE" => type_cmd::command(&cmd, conn, server).await?,

        "XADD" => xadd::command(&cmd, conn, server).await?,

        "XRANGE" => xrange::command(&cmd, conn, server).await?,

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
    let (addr, role, mut receiver) = {
        println!("receive locking ...");
        let server = server.lock().await;
        print!("locked 🔒");
        (
            server.addr.clone(),
            server.role.clone(),
            server.sender.subscribe(),
        )
    };

    loop {
        match receiver.recv().await {
            Ok(cmd) => {
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
                    command.to_uppercase() == "REPLCONF" && subcommand.to_uppercase() == "GETACK";

                println!("Received broadcast: {:?}", cmd);

                if is_wait_cmd {
                    let mut buf = [0; 512];
                    loop {
                        match conn.stream.try_read(&mut buf) {
                            Ok(0) => {
                                break;
                            }
                            Ok(_) => {
                                continue;
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                println!("Would block");
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error reading from socket: {}", e);
                                break;
                            }
                        }
                    }
                }

                println!("Forwarding broadcast to connection: {:?}", cmd);
                // Forward all broadcast messages to the connection
                conn.write_all(&cmd.as_bytes()).await?;

                if is_wait_cmd {
                    println!("Received REPLCONF ACK command");
                    let offset = match array.get(2) {
                        Some(resp::Type::BulkString(offset)) => offset,
                        _ => continue,
                    };

                    println!("offset: {:?}", offset);
                    let duration = Duration::from_millis(200);
                    let res = timeout(duration, conn.stream.readable()).await;

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

                    if bytes_read_vec.is_empty() {
                        let response = String::from_utf8_lossy(&bytes_read_vec);
                        println!("[{} - {}] Received: {:?}", addr, role, response);
                        continue;
                    }

                    println!("GOT BYTES: {:?}", bytes_read_vec);
                    let response = parser::parse(&bytes_read_vec)?;
                    println!("PARSED: {:?}", response);

                    let array = match response.get(0) {
                        Some(resp::Type::Array(array)) => array,
                        _ => {
                            println!("Invalid response: {:?}", response);
                            continue;
                        }
                    };

                    let offset = match array.get(2) {
                        Some(resp::Type::BulkString(offset)) => offset.parse::<u64>()?,
                        x => {
                            println!("Invalid offset: {:?}", x);
                            continue;
                        }
                    };

                    println!("[{} - {}] Received ACK with offset {}", addr, role, offset);

                    // Send the offset to the wait channel
                    {
                        println!("receive locking wait ...");
                        let wc = wait_channel.lock().await;
                        print!("locked 🔒");
                        wc.0.send(offset)
                            .await
                            .expect("Failed to send offset to wait channel");
                    }
                }
            }
            Err(e) => {
                println!("[{} - {}] Receiver Error: {:?}", addr, role, e);
                break;
            }
        }
    }

    Ok(())
}
