// Library
use crate::{parser::resp::Type, server::Server};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

/// Handles the GET command.
/// The GET command gets the value of a key in the database.
/// The command returns the value if the key exists.
/// The command returns an error if the number of arguments is invalid.
/// The command returns an error if the key does not exist.
pub async fn command(
    args: &[Type],
    stream: &mut TcpStream,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        let response =
            Type::SimpleError("ERR at least one argument is required for 'GET' command".into());
        stream.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Extract the key from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        _ => {
            return {
                let response = Type::SimpleError("ERR invalid key".into());
                stream.write_all(&response.as_bytes()).await?;
                Ok(())
            }
        }
    };

    // Get database instance from the Server
    println!("[get.rs::fn command] Acquiring lock");
    let server = server.lock().await;

    // Get the value from the database
    let response = match server.db.get(key) {
        Some(value) => value.clone(),
        None => Type::BulkString("".into()),
    };

    // Respond with the value
    stream.write_all(&response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
