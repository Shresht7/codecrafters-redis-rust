// Library
use super::resp::Type;
use crate::{parser::resp, server::Server};
use std::ops::DerefMut;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::MutexGuard};

/// Handles the SET command.
/// The SET command sets the value of a key in the database.
/// If the key already exists, the value is overwritten.
/// The command returns OK if the value was set successfully.
/// The command returns an error if the number of arguments is invalid.
pub async fn command<'a>(
    args: &[resp::Type],
    stream: &mut TcpStream,
    server: &mut MutexGuard<'a, Server>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 2 {
        let response = Type::SimpleError(
            format!(
                "ERR wrong number of arguments for 'SET' command. Expected {} but got {}",
                2,
                args.len()
            )
            .into(),
        );
        stream.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Extract the key and value from the arguments
    let key = match args.get(0) {
        Some(key) => key,
        _ => {
            let response = Type::SimpleError("ERR invalid key".into());
            stream.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };

    let value = match args.get(1) {
        Some(value) => value,
        _ => {
            let response = Type::SimpleError("ERR invalid value".into());
            stream.write_all(&response.as_bytes()).await?;
            return Ok(());
        }
    };

    // Get database instance from the Server
    let server = server.deref_mut();

    if args.len() == 2 {
        // Set the value in the database
        server.db.set(key.clone(), value.clone(), None);

        // Respond with OK
        let response = Type::SimpleString("OK".into());
        stream.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Extract the expiration time from the arguments
    let milliseconds = match args.get(2).unwrap().to_string().to_uppercase().as_str() {
        "PX" => match args.get(3) {
            Some(Type::BulkString(time)) => match time.parse::<usize>() {
                Ok(time) => Some(time),
                _ => {
                    let response = Type::SimpleError("ERR invalid time".into());
                    stream.write_all(&response.as_bytes()).await?;
                    return Ok(());
                }
            },
            _ => {
                let response = Type::SimpleError("ERR invalid time".into());
                stream.write_all(&response.as_bytes()).await?;
                return Ok(());
            }
        },
        _ => Some(7),
    };

    // Set the value in the database
    server.db.set(key.clone(), value.clone(), milliseconds);

    // Respond with OK
    let respone = Type::SimpleString("OK".into());
    stream.write_all(&respone.as_bytes()).await?;
    Ok(())
}
