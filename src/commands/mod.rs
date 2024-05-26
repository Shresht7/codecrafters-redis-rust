// Library
use crate::{database::Database, parser::resp};

// Commands
mod echo;
mod get;
mod ping;
mod set;

mod errors;
use errors::CommandError;

/// Handles the incoming command by parsing it and calling the appropriate command handler.
pub fn handle(
    cmd: Vec<resp::Type>,
    db: &mut Database,
) -> Result<String, Box<dyn std::error::Error>> {
    // Get command array from parsed data
    let array = match cmd.get(0) {
        Some(resp::Type::Array(array)) => array,
        _ => Err(Box::new(CommandError::InvalidCommand))?,
    };

    // Extract the command from the parsed data
    let command = match array.get(0) {
        Some(resp::Type::BulkString(command)) => command,
        _ => Err(Box::new(CommandError::InvalidCommand))?,
    };

    println!("Command: {:?}", command);
    println!("Arguments: {:?}", &array[1..]);

    // Handle the command
    match command.to_uppercase().as_str() {
        "PING" => ping::command(&array[1..]),
        "ECHO" => Ok(echo::command(&array[1..]).to_string()),
        "SET" => Ok(set::command(&array[1..], db).to_string()),
        "GET" => Ok(get::command(&array[1..], db).to_string()),
        _ => Ok("-ERR unknown command\r\n".into()),
    }
}
