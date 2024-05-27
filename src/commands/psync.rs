// Library
use crate::{parser::resp, server::Server};
use std::sync::{Arc, Mutex};

/// Handles the PSYNC command
/// PSYNC is used to synchronize a replica with the master server.
/// The command takes two arguments: the replication ID and the replication offset.
/// The replica will use the replication ID to identify the master server.
/// The replica will use the replication offset to request new data from the master server.
pub fn command(args: &[resp::Type], server: &Arc<Mutex<Server>>) -> resp::Type {
    // Check the number of arguments
    if args.len() < 2 {
        return resp::Type::SimpleError("ERR wrong number of arguments for 'PSYNC' command".into());
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

    // FULLRESYNC
    // if repl_id == "?" && repl_offset == -1 {
    let repl_id = server.lock().unwrap().master_replid.clone();
    let repl_offset = server.lock().unwrap().master_repl_offset;

    // Send a full synchronization request to the master server
    resp::Type::SimpleString(format!("FULLRESYNC {} {}", repl_id, repl_offset))
    // }
}
