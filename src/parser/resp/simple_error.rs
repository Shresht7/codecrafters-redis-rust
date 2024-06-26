// Library
use super::Type;
use crate::parser::{errors::ParserError, reader};

/// The first byte of a simple error
const FIRST_BYTE: u8 = b'-';

// -------------------
// PARSE SIMPLE ERRORS
// -------------------

/// Parses a `SimpleError` from the given input data
///
/// A simple error is encoded as follows:
/// - A prefix of `-` followed by the error message
/// - CRLF terminator sequence
///
/// Example:
/// ```sh
/// -Error message\r\n => "Error message"
/// ```
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the minus `-` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the CRLF sequence in the input
    let (end_pos, rest_pos) = bytes.find_crlf()?;

    // Extract the error message from the input up to the CRLF sequence
    let error_message = bytes.slice(1, end_pos).as_string()?;

    // Return the parsed error message and the remaining input
    Ok((Type::SimpleError(error_message), &input[rest_pos..]))
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
    fn should_parse_simple_error() {
        let input = b"-Error message\r\n";
        let expected = Type::SimpleError("Error message".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_empty_simple_error() {
        let input = b"-\r\n";
        let expected = Type::SimpleError("".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_simple_error_with_special_characters() {
        let input = b"-Error message with special characters: !@#$%^&*()\r\n";
        let expected =
            Type::SimpleError("Error message with special characters: !@#$%^&*()".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_not_parse_simple_error_without_crlf() {
        let input = b"-Error message";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_return_the_remaining_input() {
        let input = b"-Error message\r\nRemaining input";
        let expected = b"Remaining input";
        match parse(input) {
            Ok((_, actual)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"-Error message\r\nRemaining input";
        assert!(parse(input).is_ok());
    }

    #[test]
    fn should_error_on_empty_input() {
        let input = b"-\r\n";
        assert!(parse(input).is_ok());
    }

    #[test]
    fn should_error_on_invalid_utf8() {
        let input = b"-Error message\xF0\x28\x8C\xBC\r\n";
        assert!(parse(input).is_err());
    }
}
