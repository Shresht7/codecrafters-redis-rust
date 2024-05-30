// Library
use crate::{
    parser::resp,
    server::{
        connection::{Connection, Kind},
        Server,
    },
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ----
// PING
// ----

/// Handles the PING command.
/// The PING command simply returns a PONG response.
pub async fn command(
    args: &Vec<resp::Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate the response
    let response = resp::Type::SimpleString("PONG".into());

    // Send the response only if you are the master
    if connection.kind == Kind::Main {
        connection.write_all(&response.as_bytes()).await?;
    } else {
        // If you are a replica, update the replication offset
        let len = resp::array(args.clone()).as_bytes().len() as u64;
        let mut s = server.lock().await;
        println!(
            "PING(replica) {} + {} = {}",
            s.repl_offset,
            len,
            s.repl_offset + len
        );
        s.repl_offset += len;
    }

    Ok(())
}
