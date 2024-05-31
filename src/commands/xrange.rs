// Library
use crate::{
    parser::resp::{stream::StreamID, Type},
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

// ------
// XRANGE
// ------

/// Handles the XRANGE command.
/// The XRANGE command is used to get a range of entries from a stream.
/// The command is in the format `XRANGE 'stream' 'start' 'end'`.
/// Both the start and end values are inclusive.
/// The command returns an array of entries.
pub async fn command(
    args: &Vec<Type>,
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 3 || args.len() > 5 {
        return connection
            .write_error("ERR wrong number of arguments for 'XRANGE' command")
            .await;
    }

    // Extract the stream name and the range from the arguments
    let name = match args.get(1) {
        Some(stream) => stream,
        _ => {
            return connection.write_error("ERR invalid stream name").await;
        }
    };
    let start = match args.get(2) {
        Some(Type::BulkString(start)) => start,
        _ => {
            return connection.write_error("ERR invalid start").await;
        }
    };
    let start = StreamID::from_id(&start);
    let end = match args.get(3) {
        Some(Type::BulkString(end)) => end,
        _ => {
            return connection.write_error("ERR invalid end").await;
        }
    };
    let end = StreamID::from_id(&end);

    // Lock the server
    let s = server.lock().await;

    // Get the stream
    let stream = match s.db.get(name) {
        Some(Type::Stream(stream)) => stream,
        _ => {
            return connection.write_error("ERR no such stream").await;
        }
    };

    let res: Vec<Type> = stream
        .iter()
        .filter_map(|entry| {
            let id = entry.0.clone();
            if (id.milliseconds >= start.milliseconds && id.sequence >= start.sequence)
                && (id.milliseconds <= end.milliseconds && id.sequence <= end.sequence)
            {
                let fields = entry
                    .1
                    .iter()
                    .flat_map(|(k, v)| {
                        vec![
                            Type::BulkString(k.to_string()),
                            Type::BulkString(v.to_string()),
                        ]
                    })
                    .collect();

                Some(Type::Array(vec![
                    Type::BulkString(id.to_string()),
                    Type::Array(fields),
                ]))
            } else {
                None
            }
        })
        .collect();

    println!("{:?}", res);

    // Write the response
    let response = Type::Array(res);
    connection.write_all(&response.as_bytes()).await?;

    Ok(())
}
