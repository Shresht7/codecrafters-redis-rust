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

/// Handle the WAIT command.
/// The WAIT command blocks the client until the specified number of replicas for the specified key is reached,
/// or the timeout is reached. The command is used to wait for the completion of a write operation on a replica.
pub async fn command(
    args: &[resp::Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
    wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (role, master_repl_offset, addresses) = {
        let server = server.lock().await;
        (
            server.role.clone(),
            server.master_repl_offset,
            server.replicas.len(),
        )
    };

    // Check if the server is a master
    if !role.is_master() {
        let response = resp::Type::SimpleError("ERR This instance is not a master. WAIT command is only available on the master instance.".to_string());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Check if the number of arguments is correct
    if args.len() < 2 {
        let response =
            resp::Type::SimpleError("ERR wrong number of arguments for 'wait' command".to_string());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Extract number of replicas and timeout from the arguments
    let desired_replicas = match &args[0] {
        resp::Type::BulkString(replicas) => replicas.parse::<u32>()?,
        _ => {
            let response = resp::Type::SimpleError("ERR invalid number of replicas".to_string());
            connection.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };
    let desired_replicas = if desired_replicas as usize > addresses {
        addresses
    } else {
        desired_replicas as usize
    };
    let timeout = match &args[1] {
        resp::Type::BulkString(timeout) => timeout.parse::<u32>()?,
        _ => {
            let response = resp::Type::SimpleError("ERR invalid timeout".to_string());
            connection.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };
    let timeout = Instant::now() + Duration::from_millis(timeout as u64);

    println!(
        "WAIT replicas: {:?}, timeout: {:?}",
        desired_replicas, timeout
    );

    // Discard all the messages in the channel
    // while wc.1.try_recv().is_ok() {
    //     continue;
    // }

    println!("Locked wait_channel");

    // Counter to keep track of the number of replicas that have been synced
    let mut synced_replicas = 0;

    // If the master_repl_offset is 0, return the number of replicas
    if master_repl_offset == 0 {
        let response = resp::Type::Integer(addresses as i64);
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Flag to indicate if this is the first iteration
    let mut later_bytes = 0;
    let mut first_iteration = true;
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
            let s = server.lock().await;
            s.sender.send(command)?;
        }
        first_iteration = false; // Set the flag to false after the first iteration to avoid sending the REPLCONF GETACK command indefinitely

        // Sleep for 20 milliseconds
        tokio::time::sleep(Duration::from_millis(50)).await;

        {
            let mut wc = wait_channel.lock().await;
            // Await response from the replica
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
