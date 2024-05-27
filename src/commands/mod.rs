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
pub fn handle(cmd: Vec<resp::Type>, server: &Arc<Mutex<Server>>) -> String {
    // Get command array from parsed data
    let array = match cmd.get(0) {
        Some(resp::Type::Array(array)) => array,
        _ => return "-ERR unknown command\r\n".into(),
    };

    // Extract the command from the parsed data
    let command = match array.get(0) {
        Some(resp::Type::BulkString(command)) => command,
        _ => return "-ERR unknown command\r\n".into(),
    };

    // Handle the command
    match command.to_uppercase().as_str() {
        "PING" => ping::command(&array[1..]),
        "ECHO" => echo::command(&array[1..]).to_string(),
        "SET" => set::command(&array[1..], &server).to_string(),
        "GET" => get::command(&array[1..], &server).to_string(),
        "INFO" => info::command(&array[1..], &server).to_string(),
        _ => "-ERR unknown command\r\n".into(),
    }
}
