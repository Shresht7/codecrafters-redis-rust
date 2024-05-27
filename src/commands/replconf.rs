// Library
use crate::{parser::resp::Type, server};
use std::sync::{Arc, Mutex};

/// Handles the REPLCONF command.
pub fn command(_args: &[Type], _server: &Arc<Mutex<server::Server>>) -> Type {
    Type::SimpleString("OK".into())
}
