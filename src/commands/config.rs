use std::sync::Arc;

use tokio::sync::Mutex;

// Library
use crate::{
    parser::resp::Type,
    server::{connection::Connection, Server},
};

// ------
// CONFIG
// ------

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
        Some(subcommand) => subcommand,
        _ => {
            return connection.write_error("ERR invalid subcommand").await;
        }
    };

    // Handle the subcommand
    match subcommand.to_string().to_uppercase().as_str() {
        "GET" => get(args, connection, server).await?,
        // "SET" => set(args, connection, server).await?,
        _ => {
            return connection.write_error("ERR unsupported subcommand").await;
        }
    }

    Ok(())
}

// ---
// GET
// ---

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

    let value = get_config_value(key, server).await?;

    // Write the value to the client
    let response = Type::BulkString(value);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}

async fn get_config_value(
    key: &String,
    server: &Arc<Mutex<Server>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let s = server.lock().await;
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

    Ok(value)
}
