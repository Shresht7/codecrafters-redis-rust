// Library
use super::{errors::ParserError, reader, RESPData};

/// The first byte of a integer
const FIRST_BYTE: u8 = b':';

// --------------
// PARSE INTEGERS
// --------------

/// Parses an `Integer` from the given input data
///
/// An integer is encoded as follows:
/// - A prefix of `:` followed by the numerical data
/// - CRLF terminator sequence
///
/// Example:
/// ```sh
/// :5\r\n => 5
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the colon `:` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the CRLF sequence in the input
    let (end_pos, rest_start_pos) = bytes.find_crlf()?;

    // Extract the integer from the input up to the CRLF sequence and parse it as an i64
    let integer = bytes.slice(1, end_pos).parse::<i64>()?;

    // Return the parsed integer and the remaining input
    Ok((RESPData::Integer(integer), &input[rest_start_pos..]))
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
    fn should_parse_integer() {
        let input = b":123\r\n";
        let expected = RESPData::Integer(123);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_negative_integer() {
        let input = b":-123\r\n";
        let expected = RESPData::Integer(-123);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_zero() {
        let input = b":0\r\n";
        let expected = RESPData::Integer(0);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_not_parse_floats() {
        let input = b":123.45\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_return_the_remaining_input() {
        let input = b":123\r\nhello world";
        let expected = b"hello world";
        match parse(input) {
            Ok((_, actual)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"hello world\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_error_on_empty_input() {
        let input = b"\r\n";
        assert!(parse(input).is_err());
    }
}
