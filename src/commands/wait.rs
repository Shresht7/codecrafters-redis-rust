// Library
use crate::{parser::resp, server::connection::Connection};

pub async fn command(
    _args: &[resp::Type],
    connection: &mut Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = resp::Type::Integer(0);
    connection.write_all(&response.as_bytes()).await?;
    Ok(())
}
