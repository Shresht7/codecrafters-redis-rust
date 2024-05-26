// Library
use super::errors::CommandError;
use crate::{database::Database, parser::resp};

/// Handles the GET command.
/// The GET command gets the value of a key in the database.
/// The command returns the value if the key exists.
/// The command returns an error if the number of arguments is invalid.
/// The command returns an error if the key does not exist.
pub fn command(args: &[resp::Type], db: &Database) -> Result<String, Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() != 1 {
        return Err(Box::new(CommandError::InvalidArgumentCount(1, args.len())));
    }

    // Extract the key from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        _ => Err(Box::new(CommandError::InvalidArgument(
            "Invalid key".into(),
        )))?,
    };

    // Get the value from the database
    match db.get(key) {
        Some(value) => Ok(format!("+{:?}\r\n", value)),
        None => Ok("$-1\r\n".into()),
    }
}
