// Library
use super::errors::CommandError;
use super::resp::Type;
use crate::{database::Database, parser::resp};

/// Handles the SET command.
/// The SET command sets the value of a key in the database.
/// If the key already exists, the value is overwritten.
/// The command returns OK if the value was set successfully.
/// The command returns an error if the number of arguments is invalid.
pub fn command(args: &[resp::Type], db: &mut Database) -> Type {
    // Check the number of arguments
    if args.len() < 2 {
        return Type::SimpleError(
            format!(
                "ERR wrong number of arguments for 'SET' command. Expected {} but got {}",
                2,
                args.len()
            )
            .into(),
        );
    }

    // Extract the key and value from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        _ => return Type::SimpleError("ERR invalid key".into()),
    };

    let value = match args.get(1) {
        Some(value) => value,
        _ => return Type::SimpleError("ERR invalid value".into()),
    };

    // Set the value in the database
    db.set(key.clone(), value.clone());

    // Respond with OK
    Type::SimpleString("OK".into())
}
