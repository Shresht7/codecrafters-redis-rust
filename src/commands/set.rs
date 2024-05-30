// Library
use super::resp::Type;
use crate::{
    parser::resp,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    // Get database instance from the Server
    let mut server = server.lock().await;

    // Get the role of the server
    let role = server.role.clone();

    let len = resp::array(args.clone()).as_bytes().len() as u64;

    // Check the number of arguments
    if args.len() < 3 {
        if role.is_master() {
            let response = Type::SimpleError(
                format!(
                    "ERR wrong number of arguments for 'SET' command. Expected {} but got {}",
                    3,
                    args.len()
                )
                .into(),
            );
            connection.write_all(&response.as_bytes()).await?;
        }
        return Ok(());
    }

    // Extract the key and value from the arguments
    let key = match args.get(1) {
        Some(key) => key,
        _ => {
            if role.is_master() {
                let response = Type::SimpleError("ERR invalid key".into());
                connection.write_all(&response.as_bytes()).await?;
            }
            return Ok(());
        }
    };
    let value = match args.get(2) {
        Some(value) => value,
        _ => {
            if role.is_master() {
                let response = Type::SimpleError("ERR invalid value".into());
                connection.write_all(&response.as_bytes()).await?;
            }
            return Ok(());
        }
    };

    if args.len() == 3 {
        // Set the value in the database
        server.db.set(key.clone(), value.clone(), None);

        // Respond with OK
        if role.is_master() {
            let response = Type::SimpleString("OK".into());
            connection.write_all(&response.as_bytes()).await?;
        }
        return Ok(());
    }

    // Extract the expiration time from the arguments
    let milliseconds = match args.get(3).unwrap().to_string().to_uppercase().as_str() {
        "PX" => match args.get(4) {
            Some(Type::BulkString(time)) => match time.parse::<usize>() {
                Ok(time) => Some(time),
                _ => {
                    let response = Type::SimpleError("ERR invalid time".into());
                    connection.write_all(&response.as_bytes()).await?;
                    return Ok(());
                }
            },
            _ => {
                if role.is_master() {
                    let response = Type::SimpleError("ERR invalid time".into());
                    connection.write_all(&response.as_bytes()).await?;
                }
                return Ok(());
            }
        },
        _ => Some(7),
    };

    // Set the value in the database
    server.db.set(key.clone(), value.clone(), milliseconds);

    // Respond with OK
    if role.is_master() {
        println!("SET(master) {} + {}", server.master_repl_offset, len as u64);
        let response = Type::SimpleString("OK".into());
        connection.write_all(&response.as_bytes()).await?;
        server.master_repl_offset += len;
    } else {
        println!("SET(replica) {} + {}", server.repl_offset, len as u64);
        server.repl_offset += len;
    }

    drop(server);

    Ok(())
}
