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
    let mut args = args.iter();

    // Error if the first argument is not `streams`
    let subcommand = match args.next() {
        Some(Type::BulkString(subcommand)) => subcommand,
        _ => {
            return connection.write_error("ERR invalid subcommand").await;
        }
    };
    let blocking_duration = if subcommand.to_uppercase() == "BLOCK" {
        match args.next() {
            Some(Type::BulkString(duration)) => duration.parse::<u64>().ok(),
            _ => {
                return connection.write_error("ERR invalid duration").await;
            }
        }
    } else {
        None
    };

    // If blocking, wait for the specified duration
    if let Some(duration) = blocking_duration {
        tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
    }

    // Note: Assume the happy path and ignore the case where the number of streams is not even

    // Extract the streams and IDs from the arguments
    let streams = &args.clone().take_while(|arg| match arg {
        Type::BulkString(arg) => !arg.contains('-'),
        _ => false,
    });
    let ids: &Vec<&Type> = &args
        .clone()
        .take_while(|arg| match arg {
            Type::BulkString(arg) => arg.contains('-'),
            _ => false,
        })
        .collect();

    // The collection of entries of all the streams
    let mut entries_of_entries = Vec::new();

    for (stream, id) in streams.clone().zip(ids) {
        println!("Stream: {:?}, ID: {:?}", stream, id);
        let stream = match stream {
            Type::BulkString(stream) => stream,
            _ => {
                return connection.write_error("ERR invalid stream name").await;
            }
        };

        let id = match id {
            Type::BulkString(id) => StreamID::from_id(&id),
            _ => {
                return connection.write_error("ERR invalid ID").await;
            }
        };

        let key = Type::BulkString(stream.clone());
        let entries = match xread(server, &key, connection, id).await {
            Ok(value) => value,
            Err(value) => return value,
        };

        entries_of_entries.push(entries);
    }

    // Write the entries to the client
    let response = Type::Array(entries_of_entries);

    // println!("Response: {:?}", response);

    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}

async fn xread(
    server: &Arc<Mutex<Server>>,
    key: &Type,
    connection: &mut Connection,
    id: StreamID,
) -> Result<Type, Result<(), Box<dyn std::error::Error>>> {
    let s = server.lock().await;

    let stream = match s.db.get(key) {
        Some(Type::Stream(stream)) => stream,
        _ => {
            return Err(connection.write_error("ERR no such stream").await);
        }
    };

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

    Ok(Type::Array(vec![key.clone(), Type::Array(entries)]))
}
