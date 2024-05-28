// Library
use crate::parser::resp;
use tokio::{io::AsyncWriteExt, net::TcpStream};

/// Handles the PING command.
/// The PING command simply returns a PONG response.
pub async fn command(
    _args: &[resp::Type],
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = resp::Type::SimpleString("PONG".into());
    stream.write_all(&response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
