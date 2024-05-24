// Library
use super::errors::ParserError;
use super::{reader, RESPData};

/// The first byte of a simple string
const FIRST_BYTE: u8 = b'+';

// -------------------
// PARSE SIMPLE STRING
// -------------------

/// Parses a `SimpleString` from the given input data
///
/// A simple string is encoded as follows:
/// - A prefix of `+` followed by the string data
/// - CRLF terminator sequence
///
/// Example:
/// ```sh
/// +hello world\r\n => "hello world"
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the plus `+` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the CRLF sequence in the byte slice
    let (end_pos, rest_pos) = bytes.find_crlf()?;

    // Extract the simple string from the first byte up to the CRLF sequence
    let simple_string = bytes.slice(1, end_pos).as_string()?;

    // Return the parsed simple string and the remaining input
    Ok((RESPData::SimpleString(simple_string), &input[rest_pos..]))
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
    fn should_parse_simple_string() {
        let input = b"+hello world\r\n";
        let expected = RESPData::SimpleString("hello world".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_empty_string() {
        let input = b"+\r\n";
        let expected = RESPData::SimpleString("".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_return_the_remaining_input() {
        let input = b"+hello world\r\nextra data";
        let expected = RESPData::SimpleString("hello world".to_string());
        match parse(input) {
            Ok((actual, remaining)) => {
                assert_eq!(actual, expected);
                assert_eq!(remaining, b"extra data");
            }
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_invalid_first_byte() {
        let input = b"invalid\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_error_on_non_terminating_input() {
        let input = b"+hello world";
        assert!(parse(input).is_err());
    }
}
