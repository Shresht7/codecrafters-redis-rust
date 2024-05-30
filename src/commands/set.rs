// Library
use super::resp::Type;
use crate::{
    parser::resp,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ---
// SET
// ---

/// Handles the SET command.
/// The SET command sets the value of a key in the database.
/// If the key already exists, the value is overwritten.
/// The command returns OK if the value was set successfully.
/// The command returns an error if the number of arguments is invalid.
pub async fn command(
    args: &Vec<resp::Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine the length of the command bytes
    let cmd_bytes_len = resp::array(args.clone()).as_bytes().len() as u64;

    // Get the role of the server
    let role = {
        let server = server.lock().await;
        server.role.clone()
    };

    // Check the number of arguments
    if args.len() < 3 {
        if role.is_master() {
            connection
                .write_error("ERR wrong number of arguments for 'SET' command")
                .await?;
        }
        return Ok(());
    }

    // Extract the key and value from the arguments
    let key = match args.get(1) {
        Some(key) => key,
        _ => {
            if role.is_master() {
                connection.write_error("ERR invalid key").await?;
            }
            return Ok(());
        }
    };
    let value = match args.get(2) {
        Some(value) => value,
        _ => {
            if role.is_master() {
                connection.write_error("ERR invalid value").await?;
            }
            return Ok(());
        }
    };
    // Extract the expiration time from the arguments if it exists, and parse it as a u64, otherwise set it to None
    let expiry = match args.get(3) {
        Some(Type::BulkString(expiry)) => Some(expiry.parse::<usize>()?),
        _ => None,
    };

    // Set the value in the database
    let mut s = server.lock().await;
    s.db.set(key.clone(), value.clone(), expiry);

    if role.is_master() {
        // If the server is a master, increment the master replication offset
        println!(
            "SET(master) {} + {}",
            s.master_repl_offset, cmd_bytes_len as u64
        );
        connection.write_ok().await?;
        s.master_repl_offset += cmd_bytes_len;
    } else {
        // If the server is a replica, increment the replica replication offset
        println!("SET(replica) {} + {}", s.repl_offset, cmd_bytes_len as u64);
        s.repl_offset += cmd_bytes_len;
    }

    Ok(())
}
