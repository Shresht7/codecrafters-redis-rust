// Library
use crate::{
    parser::resp::{self, Type},
    server::{connection::Connection, Server},
};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};

/// Handle the WAIT command.
/// The WAIT command blocks the client until the specified number of replicas for the specified key is reached,
/// or the timeout is reached. The command is used to wait for the completion of a write operation on a replica.
pub async fn command(
    args: &[resp::Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (role, master_repl_offset, sender, mut receiver) = {
        let server = server.lock().await;
        (
            server.role.clone(),
            server.master_repl_offset,
            server.sender.clone(),
            server.sender.subscribe(),
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
        _ => 0,
    };
    let timeout = match &args[1] {
        resp::Type::BulkString(timeout) => timeout.parse::<u32>()?,
        _ => 1000,
    };
    let timeout = Instant::now() + Duration::from_secs(timeout as u64);

    println!(
        "WAIT replicas: {:?}, timeout: {:?}",
        desired_replicas, timeout
    );

    // Discard all the messages in the channel
    // while let Ok(_) = receiver.try_recv() {}

    // Counter to keep track of the number of replicas that have been synced
    let mut synced_replicas = 0;

    // Flag to indicate if this is the first iteration
    let first_iteration = true;
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
            let command = getack_command();
            sender.send(command)?;
        }

        // Sleep for 200 milliseconds
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Await response from the replica
        while let Ok(replica_response) = receiver.try_recv() {
            println!("Received response from replica: {:?}", replica_response);
            let offset = match replica_response {
                resp::Type::Array(response) => {
                    if response.len() == 3 {
                        match &response[2] {
                            resp::Type::BulkString(offset) => offset.parse::<u64>()?,
                            _ => 0,
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            };

            println!(
                "Received offset from replica: {}, master repl offset is {}",
                offset, master_repl_offset
            );
            // If the offset is greater than or equal to the master_repl_offset, increment the synced_replicas counter
            if offset >= (master_repl_offset as u64) {
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
    }

    Ok(())
}

fn getack_command() -> Type {
    Type::Array(vec![
        Type::BulkString("REPLCONF".to_string()),
        Type::BulkString("GETACK".to_string()),
        Type::BulkString("*".to_string()),
    ])
}