// Library
use crate::{
    database,
    parser::resp,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handles the PSYNC command
/// PSYNC is used to synchronize a replica with the master server.
/// The command takes two arguments: the replication ID and the replication offset.
/// The replica will use the replication ID to identify the master server.
/// The replica will use the replication offset to request new data from the master server.
pub async fn command(
    args: &[resp::Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 2 {
        let response =
            resp::Type::SimpleError("ERR wrong number of arguments for 'PSYNC' command".into());
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Get server instance from the Server
    let server = server.lock().await;

    // Get the replication ID and offset from the arguments
    // let repl_id = match &args[0] {
    //     resp::Type::BulkString(id) => id.clone(),
    //     _ => return resp::Type::SimpleError("ERR invalid replication ID".into()),
    // };
    // let repl_offset = match &args[1] {
    //     resp::Type::BulkString(offset) => match offset.parse::<i32>() {
    //         Ok(offset) => offset,
    //         Err(_) => return resp::Type::SimpleError("ERR invalid replication offset".into()),
    //     },
    //     _ => return resp::Type::SimpleError("ERR invalid replication offset".into()),
    // };

    // FULLRESYNC
    // if repl_id == "?" && repl_offset == -1 {
    let repl_id = server.master_replid.clone();
    let repl_offset = server.master_repl_offset;

    // Read Empty RDB File
    let rdb = database::rdb::EMPTY_RDB;
    let rdb_bytes = database::rdb::base64_to_bytes(rdb);

    // Send a full synchronization request to the master server
    let response = resp::Type::SimpleString(format!("FULLRESYNC {} {}", repl_id, repl_offset));
    connection.write_all(&response.as_bytes()).await?;

    let response = resp::Type::RDBFile(rdb_bytes);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
    // }
}
