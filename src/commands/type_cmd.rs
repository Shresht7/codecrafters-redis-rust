// Library
use crate::{
    parser::resp::{self, Type},
    server::Server,
};

// ----
// TYPE
// ----

/// Handles the TYPE command.
/// Returns the type of the value stored at the specified key.
/// The TYPE command is used to determine the type of a value stored in the database.
/// The command takes a single argument, the key, and returns the type of the value stored at that key.
/// The command returns one of the following types:
/// - "string" for a string value
/// - "list" for a list value
/// - "set" for a set value
/// - "zset" for a sorted set value
/// - "hash" for a hash value
/// - "none" if the key does not exist
/// The command is used to determine the type of a value before performing operations on it.
pub async fn command(
    cmd: &Vec<resp::Type>,
    conn: &mut crate::server::connection::Connection,
    server: &std::sync::Arc<tokio::sync::Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract the key from the parsed data
    let key = match cmd.get(1) {
        Some(resp::Type::BulkString(key)) => key,
        _ => {
            let response = resp::Type::SimpleError("ERR invalid command\r\n".into());
            conn.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };
    let key = &Type::BulkString(key.clone());

    // Get the value from the server
    let s = server.lock().await;
    let value_type = s.db.get(key);

    // Convert the value type to a string
    let value_type_str = match value_type {
        Some(value) => match value {
            Type::SimpleString(_) => "string",
            Type::BulkString(_) => "string",
            Type::Stream(_) => "stream",
            // Type::Set(_) => "set",
            // Type::ZSet(_) => "zset",
            // Type::Hash(_) => "hash",
            _ => "none",
        },
        None => "none",
    };

    // Create the response
    let response = resp::Type::SimpleString(value_type_str.into());
    conn.write_all(&response.as_bytes()).await?;

    Ok(())
}
