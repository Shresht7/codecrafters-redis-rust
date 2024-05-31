use std::sync::Arc;

use tokio::sync::Mutex;

// Library
use crate::{
    parser::resp::{stream::StreamID, Type},
    server::{connection::Connection, Server},
};

// -----
// XREAD
// -----

/// Handles the XREAD command.
/// The XREAD command is used to read data from one or more streams.
pub async fn command(
    args: &Vec<Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 4 {
        return connection
            .write_error("ERR wrong number of arguments for 'XREAD' command")
            .await;
    }

    // Error if the first argument is not `streams`
    let subcommand = match args.get(1) {
        Some(Type::BulkString(subcommand)) => subcommand,
        _ => {
            return connection.write_error("ERR invalid subcommand").await;
        }
    };
    if subcommand.to_uppercase() != "STREAMS" {
        return connection.write_error("ERR invalid subcommand").await;
    }

    // Get the key from the second argument
    let key = match args.get(2) {
        Some(key) => key,
        _ => {
            return connection.write_error("ERR invalid key").await;
        }
    };

    // Get the streamID from the third argument
    let id = match args.get(3) {
        Some(Type::BulkString(id)) => StreamID::from_id(&id),
        _ => {
            return connection.write_error("ERR invalid stream ID").await;
        }
    };

    // Lock the server
    let s = server.lock().await;

    // Get the stream
    let stream = match s.db.get(key) {
        Some(Type::Stream(stream)) => stream,
        _ => {
            return connection.write_error("ERR stream not found").await;
        }
    };

    // Get the entries from the stream starting from the given ID
    let entries = stream
        .iter()
        .filter_map(|entry| {
            if entry.0.milliseconds >= id.milliseconds && entry.0.sequence >= id.sequence {
                Some(entry)
            } else {
                None
            }
        })
        .flat_map(|entry| {
            let id = entry.0.clone();
            let fields = entry
                .1
                .iter()
                .flat_map(|(k, v)| vec![Type::BulkString(k.clone()), Type::BulkString(v.clone())])
                .collect::<Vec<_>>();
            vec![Type::Array(vec![
                Type::BulkString(id.to_string()),
                Type::Array(fields),
            ])]
        })
        .collect::<Vec<_>>();

    // Write the entries to the client
    let response = Type::Array(vec![Type::Array(vec![key.clone(), Type::Array(entries)])]);

    // println!("Response: {:?}", response);

    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
