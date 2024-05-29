// Library
use crate::{
    parser::resp,
    server::{connection::Connection, Server},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn command(
    _args: &[resp::Type],
    connection: &mut Connection,
    server: &Arc<Mutex<Server>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = server.lock().await;
    let replicas = server.replicas.clone();
    let response = resp::Type::Integer(replicas.len() as i64);
    connection.write_all(&response.as_bytes()).await?;
    Ok(())
}
