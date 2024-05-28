// Library
use crate::parser::resp::Type;
use tokio::{io::AsyncWriteExt, net::TcpStream};

/// Handles the ECHO command.
/// The ECHO command simply returns the argument provided to it.
pub async fn command(
    args: &[Type],
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the number of arguments
    if args.len() < 1 {
        let response =
            Type::SimpleError("ERR at least one argument is required for 'ECHO' command".into());
        stream.write_all(&response.as_bytes()).await?;
        return Ok(());
    }

    // Respond with the argument
    let response = if let Some(Type::BulkString(arg)) = args.get(0) {
        Type::BulkString(arg.clone())
    } else {
        Type::SimpleError("ERR invalid argument type for 'ECHO' command".into())
    };

    stream.write_all(&response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
