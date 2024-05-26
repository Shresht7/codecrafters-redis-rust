// Library
use super::Type;
use crate::parser::{
    errors::ParserError,
    reader::{self, CRLF},
};

/// The first byte of a verbatim string value.
const FIRST_BYTE: u8 = b'=';

// ---------------------
// PARSE VERBATIM STRING
// ---------------------

/// Parses a `VerbatimString` from the given input data.
///
/// A verbatim string is encoded as follows:
/// - A prefix of `=`
/// - The length of the verbatim string
/// - The CRLF terminator sequence
/// - Exactly 3 bytes representing the data's encoding
/// - The colon `:` character to separate the encoding from the data
/// - The verbatim string data
/// - The final CRLF terminator sequence
///
/// Example:
/// ```sh
/// =6\r\nutf-8:foobar\r\n => "foobar"
/// ```
///
/// TODO: Add URL to the specification
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the equals `=` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(ParserError::InvalidFirstByte(first_byte, FIRST_BYTE).into());
    }

    // Find the position of the first CRLF sequence and the start of the verbatim string data
    let (len_end_pos, data_start_pos) = bytes.find_crlf()?;

    // Parse the length of the verbatim string
    let length = bytes.slice(1, len_end_pos).parse::<i64>()?;

    // Calculate the total length of the verbatim string
    // data_start_pos = (length of the prefix + length of the CRLF terminator sequence)
    // 3 bytes for the encoding
    // 1 byte for the colon separator
    // `length` bytes for the verbatim string data
    // 2 bytes for the CRLF terminator sequence
    let total_length = data_start_pos + 3 + 1 + length as usize + CRLF.len();

    // Check if there is enough data to parse the verbatim string
    if input.len() < total_length {
        return Err(VerbatimStringParserError::InvalidLength(total_length, input.len()).into());
    }

    // Extract the verbatim string data
    let data = bytes.slice(data_start_pos, data_start_pos + length as usize);

    // Extract the encoding and the verbatim string data
    let (mut encoding_part, mut verbatim_string_part) = data
        .split(b":")
        .map_err(|_| VerbatimStringParserError::MissingEncodingSeparator)?;

    // Only take the length for verbatim string data
    let verbatim_string = verbatim_string_part.slice(0, length as usize);

    // Return the verbatim string and the remaining input
    Ok((
        Type::VerbatimString(encoding_part.as_string()?, verbatim_string.as_string()?),
        &input[total_length..], // Remaining bytes
    ))
}

// ------
// ERRORS
// ------

/// Errors that can occur while parsing a bulk string
#[derive(Debug)]
pub enum VerbatimStringParserError {
    InvalidLength(usize, usize),
    MissingEncodingSeparator,
}

// Implement the `Display` trait for `BulkStringParserError`
impl std::fmt::Display for VerbatimStringParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerbatimStringParserError::InvalidLength(expected, actual) => {
                write!(
                    f,
                    "Invalid input. Expected a bulk string of length {} but got {}",
                    expected, actual
                )
            }
            VerbatimStringParserError::MissingEncodingSeparator => {
                write!(f, "Invalid input. Missing encoding separator `:`")
            }
        }
    }
}

// Implement the `Error` trait for `BulkStringParserError`
impl std::error::Error for VerbatimStringParserError {}

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
    fn test_parse() {
        let input = b"=6\r\nutf-8:foobar";
        match parse(input) {
            Ok((Type::VerbatimString(encoding, verbatim_string), remaining)) => {
                assert_eq!(encoding, "utf-8");
                assert_eq!(verbatim_string, "foobar");
                assert_eq!(remaining, b"");
            }
            Err(err) => show(err),
            _ => panic!("Unexpected Type"),
        }
    }

    #[test]
    fn test_parse_invalid_first_byte() {
        let input = b"6\r\nutf-8:foobar";
        assert!(parse(input).is_err())
    }

    #[test]
    fn test_parse_invalid_length() {
        let input = b"=6\r\nutf-8:foo";
        assert!(parse(input).is_err())
    }

    #[test]
    fn test_parse_missing_encoding_separator() {
        let input = b"=6\r\nutf-8foobar";
        assert!(parse(input).is_err())
    }

    #[test]
    fn test_parse_remaining() {
        let input = b"=6\r\nutf-8:foobar\r\nremaining";
        match parse(input) {
            Ok((Type::VerbatimString(encoding, verbatim_string), remaining)) => {
                assert_eq!(encoding, "utf-8");
                assert_eq!(verbatim_string, "foobar");
                assert_eq!(remaining, b"\r\nremaining");
            }
            Err(err) => show(err),
            _ => panic!("Unexpected Type"),
        }
    }
}
