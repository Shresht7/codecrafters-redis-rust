// Library
use super::Type;
use crate::parser::{
    errors::ParserError,
    reader::{self, CRLF},
};

/// The first byte of the bulk error data type
const FIRST_BYTE: u8 = b'!';

// -----------------
// PARSE BULK ERRORS
// -----------------

/// Parses a `BulkError` from the given input data
///
/// A bulk error is encoded as follows:
/// - A prefix of `!`
/// - One or more decimal for the error's length
/// - CRLF terminator sequence
/// - The error message
/// - A final CRLF terminator sequence
///
/// Example:
/// ```sh
/// !13\r\nError message\r\n => "Error message"
/// ```
///
/// As a convention the error begins with an uppercase word denoting the error type.
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the exclamation mark `!` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the CRLF sequence in the input
    let (len_end_pos, data_start_pos) = bytes.find_crlf()?;

    // Extract the length of the error message
    let length = bytes.slice(1, len_end_pos).parse::<i64>()?;

    // Calculate the position of the end of the error message
    let error_end_pos = data_start_pos + length as usize;

    // Extract the error message
    let error_message = bytes
        .slice(data_start_pos, error_end_pos)
        .as_str()?
        .to_string();

    // Return the bulk error and the remaining input byte slice
    Ok((
        Type::BulkError(error_message),
        &input[error_end_pos + CRLF.len()..],
    ))
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to display errors in the test output
    fn show(err: Box<dyn std::error::Error>) {
        panic!("\u{001b}[31mERROR [{:?}]: {}\u{001b}[0m", err, err);
    }

    #[test]
    fn should_parse_bulk_error() {
        let input = b"!13\r\nError message\r\n";
        let expected = Type::BulkError("Error message".to_string());
        match parse(input) {
            Ok((data, _)) => assert_eq!(data, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_fail_to_parse_bulk_error() {
        let input = b"?13\r\nError message\r\n";
        let expected = ParserError::InvalidFirstByte(b'?', b'!');
        match parse(input) {
            Ok((data, _)) => panic!("Expected an error, got {:?}", data),
            Err(err) => assert_eq!(err.to_string(), expected.to_string()),
        }
    }
}
