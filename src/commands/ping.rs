use std::sync::Arc;

use tokio::sync::Mutex;

// Library
use crate::{
    parser::resp,
    server::{
        connection::{Connection, Kind},
        Server,
    },
};

/// Handles the PING command.
/// The PING command simply returns a PONG response.
pub async fn command(
    args: &Vec<resp::Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = resp::Type::SimpleString("PONG".into());

    let len = resp::array(args.clone()).as_bytes().len() as u64;

    // Send the response only if you are the master
    if connection.kind == Kind::Main {
        connection.write_all(&response.as_bytes()).await?;
    } else {
        let mut s = server.lock().await;
        println!("PING(replica) {} + {}", s.repl_offset, len as u64);
        s.repl_offset += len;
    }

    Ok(())
}
