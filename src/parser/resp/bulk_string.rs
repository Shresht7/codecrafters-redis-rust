// Library
use super::Type;
use crate::parser::{
    errors::ParserError,
    reader::{self, CRLF},
};

/// The first byte of a bulk string value.
const FIRST_BYTE: u8 = b'$';

// ------------------
// PARSE BULK STRINGS
// ------------------

/// Parses a `BulkString` from the given input data.
///
/// A bulk string is encoded as follows:
/// - A prefix of `$` followed by the length of the bulk string.
/// - The bulk string data
/// - CRLF terminator sequence
///
/// Example:
/// ```sh
/// 6\r\nfoobar\r\n => "foobar"
/// ```
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Check if the input is long enough to contain the bulk string
    if input.len() < 4 {
        return Err(BulkStringParserError::InsufficientData(input.len()).into());
    }

    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the dollar `$` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the first CRLF sequence and the start of the bulk string data
    let (len_end_pos, data_start_pos) = bytes.find_crlf()?;

    // Extract the "length" of the bulk string
    let length = bytes.slice(1, len_end_pos).parse::<i64>()?;

    // Check if the bulk string is null
    if length == -1 {
        return Ok((
            Type::Null,
            &input[data_start_pos..], // Remaining bytes
        ));
    }

    // Check if there is enough data to parse the bulk string
    if data_start_pos + length as usize > input.len() {
        return Err(BulkStringParserError::InvalidLength(length as usize, input.len()).into());
    }

    // Calculate the position of the end of the bulk string data
    let data_end_pos = data_start_pos + length as usize;

    // Extract the bulk string from the input and convert it to a String
    let bulk_string = bytes.slice(data_start_pos, data_end_pos).as_bytes();

    // Check if the data begins with REDIS0011
    if bulk_string.starts_with(b"REDIS0011") {
        return Ok((
            Type::RDBFile(bulk_string.to_vec()),
            &input[data_end_pos + CRLF.len()..], // Remaining bytes
        ));
    }

    // Convert the bulk string to a String
    let bulk_string = std::str::from_utf8(bulk_string)?.to_string();

    // Return the parsed bulk string and the remaining input
    Ok((
        Type::BulkString(bulk_string),
        &input[data_end_pos + CRLF.len()..], // Remaining bytes
    ))
}

// ------
// ERRORS
// ------

/// Errors that can occur while parsing a bulk string
#[derive(Debug)]
pub enum BulkStringParserError {
    InsufficientData(usize),
    InvalidLength(usize, usize),
}

// Implement the `Display` trait for `BulkStringParserError`
impl std::fmt::Display for BulkStringParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkStringParserError::InsufficientData(len) => {
                write!(f, "Invalid input. Insufficient data: {}", len)
            }
            BulkStringParserError::InvalidLength(expected, actual) => {
                write!(
                    f,
                    "Invalid input. Expected a bulk string of length {} but got {}",
                    expected, actual
                )
            }
        }
    }
}

// Implement the `Error` trait for `BulkStringParserError`
impl std::error::Error for BulkStringParserError {}

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
    fn should_parse_bulk_string() {
        let input = b"$6\r\nfoobar\r\n";
        let expected = Type::BulkString("foobar".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(error) => show(error),
        }
    }

    #[test]
    fn should_parse_empty_bulk_string() {
        let input = b"$0\r\n\r\n";
        let expected = Type::BulkString("".to_string());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(error) => show(error),
        }
    }

    #[test]
    fn should_parse_null_bulk_string() {
        let input = b"$-1\r\n";
        let expected = Type::Null;
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(error) => show(error),
        }
    }

    #[test]
    fn should_not_parse_invalid_length() {
        let input = b"$abc\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_insufficient_data() {
        let input = b"$6\r\nfoo\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_missing_data() {
        let input = b"$3\r\nfo";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_missing_crlf() {
        let input = b"$3\nfoo\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_input_without_crlf() {
        let input = b"$foobar";
        assert!(parse(input).is_err());
    }
}
