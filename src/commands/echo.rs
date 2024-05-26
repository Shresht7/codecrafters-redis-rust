// Library
use crate::parser::resp;

/// Handles the ECHO command.
/// The ECHO command simply returns the argument provided to it.
pub fn command(args: &[resp::Type]) -> Result<String, Box<dyn std::error::Error>> {
    // Get the first argument of the ECHO command
    let arg = match args.get(0) {
        Some(resp::Type::BulkString(arg)) => arg,
        _ => panic!("Invalid argument"),
        // _ => return Err(Box::new(errors::ParserError::InvalidCommand)),
    };

    // Respond with the argument
    Ok(format!("${}\r\n{}\r\n", arg.len(), arg))
}
