use std::sync::Arc;

use tokio::sync::Mutex;

// Library
use crate::{
    parser::resp::{self, Type},
    server::{connection::Connection, Server},
};

// ----
// KEYS
// ----

/// Handles the KEYS command.
/// The KEYS command is used to return all keys matching a given pattern.
/// The command is in the format `KEYS 'pattern'`.
/// The pattern can contain the `*` and `?` wildcards.
pub async fn command(
    args: &Vec<Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() != 2 {
        return connection
            .write_error("ERR wrong number of arguments for 'KEYS' command")
            .await;
    }

    // // Extract the pattern from the arguments
    // let pattern = match args.get(1) {
    //     Some(Type::BulkString(pattern)) => pattern,
    //     _ => {
    //         return connection.write_error("ERR invalid pattern").await;
    //     }
    // };

    // Get the server lock
    let server = server.lock().await;

    // Get the keys that match the pattern
    let keys = server.db.keys();

    // Write the keys to the connection
    let response = resp::array(keys);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
