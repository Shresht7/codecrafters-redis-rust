// Library
use super::errors::CommandError;
use crate::{database::Database, parser::resp};

/// Handles the SET command.
/// The SET command sets the value of a key in the database.
/// If the key already exists, the value is overwritten.
/// The command returns OK if the value was set successfully.
/// The command returns an error if the number of arguments is invalid.
pub fn command(
    args: &[resp::Type],
    db: &mut Database,
) -> Result<String, Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 2 {
        return Err(Box::new(CommandError::InvalidArgumentCount(2, args.len())));
    }

    // Extract the key and value from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        _ => Err(Box::new(CommandError::InvalidArgument(
            "Invalid key".into(),
        )))?,
    };

    let value = match args.get(1) {
        Some(value) => value,
        _ => Err(Box::new(CommandError::InvalidArgument(
            "Invalid value".into(),
        )))?,
    };

    // Set the value in the database
    db.set(key.clone(), value.clone());

    // Respond with OK
    Ok("+OK\r\n".into())
}
