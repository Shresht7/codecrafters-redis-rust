// Library
use super::errors::CommandError;
use crate::parser::resp;

/// Handles the ECHO command.
/// The ECHO command simply returns the argument provided to it.
pub fn command(args: &[resp::Type]) -> Result<String, Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        return Err(Box::new(CommandError::InvalidArgumentCount(1, args.len())));
    }

    // Respond with the argument
    if let Some(resp::Type::BulkString(arg)) = args.get(0) {
        Ok(format!("${}\r\n{}\r\n", arg.len(), arg))
    } else {
        Err(Box::new(CommandError::InvalidArgument(format!(
            "Expected BulkString but got: {:?}",
            args.get(0)
        ))))
    }
}
