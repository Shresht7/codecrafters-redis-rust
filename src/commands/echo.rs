// Library
use crate::{parser::resp::Type, server::connection::Connection};

/// Handles the ECHO command.
/// The ECHO command simply returns the argument provided to it.
pub async fn command(
    args: &[Type],
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        let response =
            Type::SimpleError("ERR at least one argument is required for 'ECHO' command".into());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Respond with the argument
    let response = if let Some(Type::BulkString(arg)) = args.get(0) {
        Type::BulkString(arg.clone())
    } else {
        Type::SimpleError("ERR invalid argument type for 'ECHO' command".into())
    };

    connection.write_all(&response.as_bytes()).await?;
    Ok(())
}
