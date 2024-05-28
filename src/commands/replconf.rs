// Library
use crate::{parser::resp::Type, server::Server};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::MutexGuard};

/// Handles the REPLCONF command.
pub async fn command<'a>(
    _args: &[Type],
    stream: &mut TcpStream,
    _server: &mut MutexGuard<'a, Server>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = Type::SimpleString("OK".into());
    stream.write_all(&response.as_bytes()).await?;
    Ok(())
}
