// Library
use crate::{
    parser::resp::{self, Type},
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ------
// CONFIG
// ------

/// Handles the CONFIG command.
/// The CONFIG command is used to read and write configuration parameters.
pub async fn command(
    args: &Vec<Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 3 {
        return connection
            .write_error("ERR wrong number of arguments for 'CONFIG' command")
            .await;
    }

    // Extract the subcommand from the arguments
    let subcommand = match args.get(1) {
        Some(Type::BulkString(subcommand)) => subcommand,
        _ => {
            return connection.write_error("ERR invalid subcommand").await;
        }
    };

    // Handle the subcommand
    match subcommand.to_string().to_uppercase().as_str() {
        "GET" => get(args, connection, server).await?,
        // "SET" => set(args, connection, server).await?,
        x => {
            return connection
                .write_error(format!("ERR unknown subcommand '{}'", x))
                .await;
        }
    }

    Ok(())
}

// ---
// GET
// ---

/// Handles the CONFIG GET subcommand.
/// The CONFIG GET subcommand is used to read configuration parameters.
/// The subcommand is in the format `CONFIG GET 'key'`.
async fn get(
    args: &Vec<Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 3 {
        return connection
            .write_error("ERR wrong number of arguments for 'CONFIG GET' command")
            .await;
    }

    // Extract the key from the arguments
    let key = match args.get(2) {
        Some(Type::BulkString(str)) => str,
        _ => {
            return connection.write_error("ERR invalid key").await;
        }
    };

    // Get the value of the key
    let value = get_config_value(key, server).await?;

    // Write the value to the client
    let response = resp::array(vec![resp::bulk_string(&key), resp::bulk_string(&value)]);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}

/// Gets the value of the configuration parameter with the given key.
async fn get_config_value(
    key: &String,
    server: &Arc<Mutex<Server>>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Acquire the server lock
    let s = server.lock().await;

    // Get the value of the key
    let value = match key.to_string().to_uppercase().as_str() {
        "DIR" => s.db.dir.clone(),

        "DBFILENAME" => s.db.dbfilename.clone(),

        _ => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "ERR invalid key",
            )));
        }
    };

    // Return the value
    Ok(value)
}
