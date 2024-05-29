// Library
use crate::{
    parser::resp,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle the WAIT command.
/// The WAIT command blocks the client until the specified number of replicas for the specified key is reached,
/// or the timeout is reached. The command is used to wait for the completion of a write operation on a replica.
pub async fn command(
    args: &[resp::Type],
    connection: &mut Connection,
    _server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the number of arguments is correct
    if args.len() < 2 {
        let response =
            resp::Type::SimpleError("ERR wrong number of arguments for 'wait' command".to_string());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Extract number of replicas and timeout from the arguments
    let replicas = match &args[0] {
        resp::Type::BulkString(replicas) => Some(replicas.parse::<u32>()?),
        _ => None,
    };
    let timeout = match &args[1] {
        resp::Type::BulkString(timeout) => Some(timeout.parse::<u32>()?),
        _ => None,
    };

    println!("WAIT replicas: {:?}, timeout: {:?}", replicas, timeout);

    Ok(())
}
