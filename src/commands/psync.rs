// Library
use crate::{
    database,
    parser::resp,
    server::{connection::Connection, Server},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

/// Handles the PSYNC command
/// The PSYNC command is used to synchronize a replica server with a master server.
/// The command is used by the replica to request a full synchronization from the master.
/// The master sends an RDB file to the replica, which is used to synchronize the replica server.
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

    // Lock the server instance
    let mut server = server.lock().await;
    let role = server.role.clone();

    // Check if the server is a master
    if !role.is_master() {
        let response = resp::Type::SimpleError(
            "ERR This instance is not a master. PSYNC command is only available on the master instance."
                .into(),
        );
        connection.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Send a full synchronization request to the replica
    let repl_id = server.master_replid.clone();
    let master_repl_offset = server.master_repl_offset;
    let response =
        resp::Type::SimpleString(format!("FULLRESYNC {} {}", repl_id, master_repl_offset));
    connection.write_all(&response.as_bytes()).await?;

    // Add the replica to the list of replicas
    server.replicas.push(connection.addr.clone());

    let duration = Duration::from_millis(500);
    tokio::time::sleep(duration).await;

    // Send an empty RDB file to the replica
    let rdb = database::rdb::EMPTY_RDB;
    let rdb_bytes = database::rdb::base64_to_bytes(rdb);
    let response = resp::Type::RDBFile(rdb_bytes);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
