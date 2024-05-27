// Library
use crate::parser::resp;

/// Handles the PING command.
/// The PING command simply returns a PONG response.
pub fn command(_args: &[resp::Type]) -> String {
    "+PONG\r\n".into()
}
