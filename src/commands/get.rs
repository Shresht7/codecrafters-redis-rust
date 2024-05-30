// Library
use crate::{
    parser::resp::Type,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ---
// GET
// ---

/// Handles the GET command.
/// The GET command gets the value of a key in the database.
/// The command returns the value if the key exists.
/// The command returns an error if the number of arguments is invalid.
/// The command returns an error if the key does not exist.
pub async fn command(
    args: &[Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        return connection
            .write_error("ERR wrong number of arguments for 'GET' command. Usage GET key")
            .await;
    }

    // Extract the key from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        None => {
            return connection.write_error("ERR invalid key").await;
        }
    };

    // Get the value from the database
    let server = server.lock().await;
    let response = match server.db.get(key) {
        Some(value) => value.clone(),
        None => Type::BulkString("".into()),
    };

    // Respond with the value
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
