// Library
use crate::{parser::resp::Type, server::Server};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

/// Handles the REPLCONF command.
pub async fn command(
    _args: &[Type],
    stream: &mut TcpStream,
    _server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = Type::SimpleString("OK".into());
    stream.write_all(&response.as_bytes()).await?;
    Ok(())
}
