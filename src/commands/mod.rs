// Library
use crate::{parser::resp, server::Server};
use std::sync::{Arc, Mutex};

// Commands
mod echo;
mod get;
mod info;
mod ping;
mod set;

/// Handles the incoming command by parsing it and calling the appropriate command handler.
pub fn handle(cmd: Vec<resp::Type>, server: &Arc<Mutex<Server>>) -> resp::Type {
    // Get command array from parsed data
    let array = match cmd.get(0) {
        Some(resp::Type::Array(array)) => array,
        _ => return resp::Type::SimpleError("ERR unknown command\r\n".into()),
    };

    // Extract the command from the parsed data
    let command = match array.get(0) {
        Some(resp::Type::BulkString(command)) => command,
        _ => return resp::Type::SimpleError("ERR unknown command\r\n".into()),
    };

    // Handle the command
    match command.to_uppercase().as_str() {
        "PING" => ping::command(&array[1..]),
        "ECHO" => echo::command(&array[1..]),
        "SET" => set::command(&array[1..], &server),
        "GET" => get::command(&array[1..], &server),
        "INFO" => info::command(&array[1..], &server),
        _ => resp::Type::SimpleError("ERR unknown command\r\n".into()),
    }
}
