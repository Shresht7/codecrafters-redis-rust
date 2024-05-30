// Library
use crate::{
    parser::resp::{self, Type},
    server::{connection::Connection, Server},
};
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Mutex},
    time::Instant,
};

// ----
// WAIT
// ----

/// Handle the WAIT command.
/// The WAIT command blocks the client until the specified number of replicas for the specified key is reached,
/// or the timeout is reached. The command is used to wait for the completion of a write operation on a replica.
pub async fn command(
    args: &[resp::Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the role, master_repl_offset, and number of replicas from the server
    let (role, master_repl_offset, addresses) = {
        let server = server.lock().await;
        (
            server.role.clone(),
            server.master_repl_offset.clone(),
            server.replicas.len(),
        )
    };

    // Check if the server is a master
    if !role.is_master() {
        return connection
            .write_error("ERR WAIT command is only valid on master")
            .await;
    }

    // Check if the number of arguments is correct
    if args.len() < 2 {
        return connection
            .write_error("ERR wrong number of arguments for 'WAIT' command. Usage: WAIT <num_replicas> <timeout>")
            .await;
    }

    // Extract the desired number of replicas from the arguments
    let desired_replicas = match &args[0] {
        resp::Type::BulkString(replicas) => replicas.parse::<usize>()?,
        x => {
            return connection
                .write_error(format!("ERR invalid number of replicas {:?}", x))
                .await;
        }
    };
    // If the desired number of replicas is greater than the number of replicas, set the desired number of replicas to the number of replicas
    let desired_replicas = if desired_replicas > addresses {
        addresses
    } else {
        desired_replicas as usize
    };

    // Extract the timeout from the arguments
    let timeout = match &args[1] {
        resp::Type::BulkString(timeout) => timeout.parse::<u64>()?,
        x => {
            return connection
                .write_error(format!("ERR invalid timeout {:?}", x))
                .await;
        }
    };
    // Calculate the timeout
    let timeout = Instant::now() + Duration::from_millis(timeout);

    let mut synced_replicas = 0;

    // Counter to keep track of the number of bytes to send later
    let mut later_bytes = 0;

    // Counter to keep track of the number of replicas that have been synced
    synced_replicas = if master_repl_offset == 0 {
        // If the master_repl_offset is 0, return the number of replicas
        addresses
    } else {
        // Flag to indicate if this is the first iteration
        let mut first_iteration = true;
        // Loop until the timeout is reached or the number of synced replicas reaches the desired number
        while Instant::now() < timeout {
            // If the number of synced replicas reaches the desired number, break the loop
            if synced_replicas >= desired_replicas {
                println!(
                    "Number of synced replicas reached the desired number: {}/{}",
                    synced_replicas, desired_replicas
                );
                break;
            }

            // If this is the first iteration, send the REPLCONF GETACK command
            if first_iteration {
                let command = Type::Array(vec![
                    Type::BulkString("REPLCONF".to_string()),
                    Type::BulkString("GETACK".to_string()),
                    Type::BulkString("*".to_string()),
                ]);
                later_bytes += command.as_bytes().len();
                println!("Sending REPLCONF GETACK * command");
                {
                    let s = server.lock().await;
                    s.sender.send(command)?;
                }
            }
            first_iteration = false; // Set the flag to false after the first iteration to avoid sending the REPLCONF GETACK command indefinitely

            // Sleep for a few milliseconds
            tokio::time::sleep(Duration::from_millis(50)).await;

            {
                // Await response from the replica
                let mut wc = wait_channel.lock().await;
                loop {
                    match wc.1.try_recv() {
                        Ok(offset) => {
                            println!(
                                "Received offset from replica: {}, master repl offset is {}",
                                offset, master_repl_offset
                            );
                            // If the offset is greater than or equal to the master_repl_offset, increment the synced_replicas counter
                            if offset >= master_repl_offset {
                                println!("Replica is synced");
                                synced_replicas += 1;
                            }
                            // If the number of synced replicas reaches the desired number, break the loop
                            if synced_replicas >= desired_replicas {
                                println!(
                                    "Number of synced replicas reached the desired number: {}/{}",
                                    synced_replicas, desired_replicas
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("No response from replica. Error: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
        synced_replicas
    };

    println!("Number of synced replicas: {}", synced_replicas);

    // Send the response to the client
    let response = resp::Type::Integer(synced_replicas as i64);
    connection.write_all(&response.as_bytes()).await?;

    // Add the bytes that were sent later to the master_repl_offset
    {
        let mut s = server.lock().await;
        s.master_repl_offset += later_bytes as u64;
    }

    Ok(())
}
