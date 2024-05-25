// Library
use super::{errors::ParserError, reader, RESPData};

// The first byte of a null value
const FIRST_BYTE: u8 = b'_';

// ----------
// PARSE NULL
// ----------

/// Parses a `Null` value from the given input data.
///
/// A null value is represented by the underscore `_` character followed by a CRLF sequence.
/// A null value is used to represent the absence of a value in RESP2.
/// In RESP3, a null value is represented by the `Null` data type.
///
/// Example:
/// ```sh
/// _\r\n
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the underscore `_` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the first CRLF sequence
    let (start_pos, rest_pos) = bytes.find_crlf()?;

    // Check if the second byte is a CRLF sequence
    if start_pos != 1 {
        return Err(NullParserError::InvalidSecondByte(input[1]).into());
    }

    // Return the parsed null value and the remaining input
    Ok((RESPData::Null, &input[rest_pos..]))
}

// ------
// ERRORS
// ------

/// Errors that can occur while parsing a null value
#[derive(Debug)]
pub enum NullParserError {
    // The second byte does not start the CRLF sequence
    InvalidSecondByte(u8),
}

// Implement the `Display` trait for `NullParserError` type
impl std::fmt::Display for NullParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NullParserError::InvalidSecondByte(got) => {
                write!(
                    f,
                    "Invalid second byte. Should begin CRLF sequence but got {}",
                    *got as char
                )
            }
        }
    }
}

// Implement the `Error` trait for the `NullParserError` type
impl std::error::Error for NullParserError {}

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
    fn should_parse_null() {
        let input = b"_\r\n";
        let expected = (RESPData::Null, &b""[..]);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected.0),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"invalid";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_insufficient_data() {
        let input = b"_";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_invalid_input_underscore() {
        let input = b"X\r\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_invalid_input_crlf() {
        let input = b"_X\n";
        let result = parse(input);
        assert!(result.is_err());
    }
}
