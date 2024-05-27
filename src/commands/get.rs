// Library
use crate::{parser::resp::Type, server};
use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

/// Handles the GET command.
/// The GET command gets the value of a key in the database.
/// The command returns the value if the key exists.
/// The command returns an error if the number of arguments is invalid.
/// The command returns an error if the key does not exist.
pub fn command(args: &[Type], server: &Arc<Mutex<server::Server>>) -> Type {
    // Check the number of arguments
    if args.len() < 1 {
        return Type::SimpleError(
            format!(
                "ERR wrong number of arguments for 'GET' command. Expected at least 1 but got {}",
                args.len()
            )
            .into(),
        );
    }

    // Extract the key from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        _ => return Type::SimpleError("ERR invalid key".into()),
    };

    // Get database instance from the Server
    let mut server = server.lock().unwrap();
    let server = server.deref_mut();

    // Get the value from the database
    match server.db.get(key) {
        Some(value) => value.clone(),
        None => Type::BulkString("".into()),
    }
}
