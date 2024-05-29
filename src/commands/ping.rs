// Library
use crate::{parser::resp, server::connection::Connection};

/// Handles the PING command.
/// The PING command simply returns a PONG response.
pub async fn command(
    _args: &[resp::Type],
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = resp::Type::Array(vec![resp::Type::SimpleString("PONG".into())]);
    connection.write_all(&response.as_bytes()).await?;
    Ok(())
}
