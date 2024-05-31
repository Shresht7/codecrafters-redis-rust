// Library
use crate::{
    parser::resp::Type,
    server::{connection::Connection, Server},
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

// ----
// XADD
// ----

/// Handles the XADD command.
/// The XADD command is used to append a new entry to a stream.
/// The command is in the format `XADD 'stream' 'id' 'field1' 'value1' 'field2' 'value2' ...`.
/// The command returns the ID of the new entry.
/// If the stream does not exist, it is created.
pub async fn command(
    args: &Vec<Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 3 || args.len() % 2 == 0 {
        return connection
            .write_error("ERR wrong number of arguments for 'XADD' command")
            .await;
    }

    // Extract the stream name and ID from the arguments
    let name = match args.get(1) {
        Some(stream) => stream,
        _ => {
            return connection.write_error("ERR invalid stream name").await;
        }
    };
    let id = match args.get(2) {
        Some(Type::BulkString(id)) => id,
        _ => {
            return connection.write_error("ERR invalid ID").await;
        }
    };

    // Extract the field-value pairs from the arguments
    let mut fields = HashMap::new();
    for i in (3..args.len()).step_by(2) {
        let field = match args.get(i) {
            Some(Type::BulkString(field)) => field,
            _ => {
                return connection.write_error("ERR invalid field").await;
            }
        };
        let value = match args.get(i + 1) {
            Some(Type::BulkString(value)) => value,
            _ => {
                return connection.write_error("ERR invalid value").await;
            }
        };
        fields.insert(field.to_string(), value.to_string());
    }

    // Get the current stream
    let mut s = server.lock().await;
    let item = s.db.get(name);
    let mut stream = match item {
        Some(Type::Stream(stream)) => stream.clone(),
        _ => Vec::new(), // Create a new stream
    };

    // Check that the ID is larger than the last entry
    let last_entry = match stream.last() {
        Some(entry) => Some(entry.clone()),
        None => None,
    };

    // Split the id into its parts
    let (milliseconds, sequence) = parse_id(id, last_entry.clone());
    println!("Stream ID {}: {}-{}", id, milliseconds, sequence);

    // Check if the ID is valid
    if milliseconds == 0 && sequence == 0 {
        return connection
            .write_error("ERR The ID specified in XADD must be greater than 0-0")
            .await;
    }

    // Check if the ID is greater than the last entry
    if let Some(last_entry) = last_entry {
        let (last_milliseconds, last_sequence) = simple_parse_id(&last_entry.0.clone());
        if milliseconds < last_milliseconds
            || (milliseconds == last_milliseconds && sequence <= last_sequence)
        {
            return connection
                .write_error(
                    "ERR The ID specified in XADD is equal or smaller than the target stream top item",
                )
                .await;
        }
    }

    // Append the entry to the stream
    stream.push((id.to_string(), fields));

    // Update the database
    s.db.set(name.clone(), Type::Stream(stream), None);

    // Update the ID format
    let id = format!("{}-{}", milliseconds, sequence);
    println!("Stream ID: {}", id);

    // Write the ID of the new entry
    let response = Type::BulkString(id.to_string());
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}

fn simple_parse_id(id: &str) -> (u64, u64) {
    let (milliseconds, sequence) = match id.split_once("-") {
        Some((milliseconds, sequence)) => {
            let milliseconds = milliseconds.parse::<u64>().unwrap_or(0);
            let sequence = sequence.parse::<u64>().unwrap_or(0);
            (milliseconds, sequence)
        }
        None => (0, 0),
    };
    (milliseconds, sequence)
}

fn parse_id(id: &str, last_entry: Option<(String, HashMap<String, String>)>) -> (u64, u64) {
    let (milliseconds, sequence) = match id.split_once("-") {
        Some((milliseconds, sequence)) => {
            let milliseconds = parse_milliseconds(milliseconds);
            let sequence = parse_sequence(sequence, milliseconds, last_entry);
            (milliseconds, sequence)
        }
        None => (0, 0),
    };
    (milliseconds, sequence)
}

fn parse_milliseconds(milliseconds: &str) -> u64 {
    match milliseconds {
        _ => milliseconds.parse::<u64>().unwrap_or(0),
    }
}

fn parse_sequence(
    sequence: &str,
    milliseconds: u64,
    last_entry: Option<(String, HashMap<String, String>)>,
) -> u64 {
    match sequence {
        "*" => {
            if let Some((last_id, _)) = last_entry {
                let (last_milliseconds, last_sequence) = simple_parse_id(&last_id);
                if milliseconds == last_milliseconds {
                    last_sequence + 1
                } else {
                    0
                }
            } else {
                1
            }
        }
        _ => sequence.parse::<u64>().unwrap_or(0),
    }
}
