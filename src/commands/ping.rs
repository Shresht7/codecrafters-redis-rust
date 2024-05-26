// Library
use crate::parser::resp;

/// Handles the PING command
pub fn command(_args: &[resp::Type]) -> Result<String, Box<dyn std::error::Error>> {
    Ok("+PONG\r\n".into())
}
