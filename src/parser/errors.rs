// -------------
// PARSER ERRORS
// -------------

/// Errors that can occur while parsing a RESP data type
#[derive(Debug)]
pub enum ParserError {
    /// The input byte slice is empty
    EmptyInput,
    /// The first byte of the input data is invalid
    InvalidFirstByte(u8, u8), // Actual, Expected
}

// Implement the `Display` trait for the `ParserError` type
impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParserError::EmptyInput => write!(f, "The byte slice is empty!"),

            ParserError::InvalidFirstByte(actual, expected) => {
                write!(
                    f,
                    "Invalid first byte. expected '{}', found '{}'",
                    *actual as char, *expected as char
                )
            }
        }
    }
}

// Implement the `Error` trait for the `ParserError` type
impl std::error::Error for ParserError {}
