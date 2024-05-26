// Library
use crate::parser::resp;

// Commands
mod echo;
mod ping;

/// Handles the incoming command by parsing it and calling the appropriate command handler.
pub fn handle(cmd: Vec<resp::Type>) -> Result<String, Box<dyn std::error::Error>> {
    // Get command array from parsed data
    let array = match cmd.get(0) {
        Some(resp::Type::Array(array)) => array,
        _ => panic!("Invalid command"),
    };

    // Extract the command from the parsed data
    let command = match array.get(0) {
        Some(resp::Type::BulkString(command)) => command,
        err => panic!("Invalid command: {:?}", err),
        // _ => return Err(Box::new(parser::errors::ParserError::InvalidCommand)),
    };

    println!("Command: {:?}", command);
    println!("Arguments: {:?}", &array[1..]);

    // Handle the command
    match command.to_uppercase().as_str() {
        "PING" => ping::command(&array[1..]),
        "ECHO" => echo::command(&array[1..]),
        _ => Ok("-ERR unknown command\r\n".into()),
    }
}
