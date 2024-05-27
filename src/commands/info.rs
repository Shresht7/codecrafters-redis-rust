// Library
use crate::parser::resp::Type;

/// Handles the INFO command.
/// The INFO command returns information and statistics about the server.
pub fn command(args: &[Type]) -> Type {
    // Check the number of arguments
    if args.len() < 1 {
        return Type::SimpleError("ERR wrong number of arguments for 'INFO' command".into());
    }

    // Respond with the server information
    let response: String =
        vec!["# Replication".to_string(), "role:master".to_string()].join("\r\n");
    Type::BulkString(response)
}
