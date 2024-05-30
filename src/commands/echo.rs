// Library
use crate::{parser::resp::Type, server::connection::Connection};

// ----
// ECHO
// ----

/// Handles the ECHO command.
/// The ECHO command simply returns the argument provided to it.
pub async fn command(
    args: &[Type],
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        return connection
            .write_error("ERR wrong number of arguments for 'ECHO' command. Usage ECHO message")
            .await;
    }

    // Respond with the argument
    let response = match args.get(0) {
        Some(Type::BulkString(arg)) => Type::BulkString(arg.clone()),
        _ => Type::SimpleError("ERR invalid argument type for 'ECHO' command".into()),
    };
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
