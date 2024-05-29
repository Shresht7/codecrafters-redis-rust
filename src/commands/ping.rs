// Library
use crate::{
    parser::resp,
    server::connection::{Connection, Kind},
};

/// Handles the PING command.
/// The PING command simply returns a PONG response.
pub async fn command(
    _args: &[resp::Type],
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = resp::Type::Array(vec![resp::Type::SimpleString("PONG".into())]);

    // Send the response only if you are the master
    if connection.kind == Kind::Main {
        connection.write_all(&response.as_bytes()).await?;
    }

    Ok(())
}
