// Library
use super::resp::Type;
use crate::{parser::resp, server};
use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

/// Handles the SET command.
/// The SET command sets the value of a key in the database.
/// If the key already exists, the value is overwritten.
/// The command returns OK if the value was set successfully.
/// The command returns an error if the number of arguments is invalid.
pub fn command(args: &[resp::Type], server: &Arc<Mutex<server::Server>>) -> Type {
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

    // Get database instance from the Server
    let mut server = server.lock().unwrap();
    let server = server.deref_mut();

    if args.len() == 2 {
        // Set the value in the database
        server.db.set(key.clone(), value.clone(), None);

        // Respond with OK
        return Type::SimpleString("OK".into());
    }

    // Extract the expiration time from the arguments
    let milliseconds = match args.get(2).unwrap().to_string().to_uppercase().as_str() {
        "PX" => match args.get(3) {
            Some(Type::BulkString(time)) => match time.parse::<usize>() {
                Ok(time) => Some(time),
                _ => return Type::SimpleError("ERR invalid time".into()),
            },
            _ => return Type::SimpleError("ERR invalid time".into()),
        },
        _ => Some(7),
    };

    // Set the value in the database
    server.db.set(key.clone(), value.clone(), milliseconds);

    // Respond with OK
    Type::SimpleString("OK".into())
}
