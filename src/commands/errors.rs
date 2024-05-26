#[derive(Debug)]
pub enum CommandError {
    InvalidCommand,
    InvalidArgumentCount(usize, usize),
    InvalidArgument(String),
}

// Implement the `Display` trait for the `CommandError` type
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandError::InvalidCommand => write!(f, "Invalid command"),
            CommandError::InvalidArgumentCount(expected, actual) => {
                write!(
                    f,
                    "Invalid number of arguments. Expected {} but got {} ",
                    expected, actual
                )
            }
            CommandError::InvalidArgument(reason) => write!(f, "Invalid argument. {}", reason),
        }
    }
}

// Implement the `Error` trait for the `CommandError` type
impl std::error::Error for CommandError {}
