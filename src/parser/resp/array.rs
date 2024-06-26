// Library
use super::Type;
use crate::parser::{_parse, errors::ParserError, reader};

/// The first byte of an array value.
const FIRST_BYTE: u8 = b'*';

// -----------
// PARSE ARRAY
// -----------

/// Parses a RESP array from the given input data.
///
/// Arrays use the following encoding format:
/// - A prefix of `*` followed by the number of elements in the array.
/// - Each element in the array is encoded according to the rules of the RESP protocol.
/// - CRLF terminator sequence at the end of the array.
///
/// Example:
/// ```sh
/// *3\r\n:1\r\n:2\r\n:3\r\n => [1, 2, 3]
/// ```
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Check if the input is long enough to contain the array value
    if input.len() < 4 {
        return Err(ArrayParserError::InsufficientData(input.len()).into());
    }

    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the asterisk `*` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the first CRLF sequence and the start of the array data
    let (len_end_pos, data_start_pos) = bytes.find_crlf()?;

    // Extract the "length" of the array
    let length = bytes.slice(1, len_end_pos).parse::<i64>()?;

    // If the length is -1, the array is null
    if length == -1 {
        return Ok((Type::Null, &input[data_start_pos..]));
    }

    // If the length is 0, the array is empty
    if length <= 0 {
        return Ok((
            Type::Array(Vec::new()),
            &input[data_start_pos..], // Remaining bytes
        ));
    }

    // Parse the elements of the array
    let mut elements = Vec::new();
    let mut remaining = &input[data_start_pos..];
    // Iterate for the length of the array
    for _ in 0..length {
        let (element, rest) = _parse(remaining)?;
        elements.push(element);
        remaining = rest;
    }

    // Return the parsed array and the remaining input
    Ok((
        Type::Array(elements),
        remaining, // Remaining bytes
    ))
}

// ------
// ERRORS
// ------

/// Errors that can occur during array parsing.
/// These errors are returned as boxed trait objects.
/// This allows the caller to handle errors without knowing the exact type.
#[derive(Debug)]
pub enum ArrayParserError {
    InsufficientData(usize),
}

// Implement the `Display` trait for the array parser error
impl std::fmt::Display for ArrayParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArrayParserError::InsufficientData(len) => write!(
                f,
                "Insufficient data. The input length is {} but it should contain at least 4 bytes to represent array values",
                len
            ),
        }
    }
}

// Implement the `Error` trait for the array parser error
impl std::error::Error for ArrayParserError {}

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
    fn should_parse_array() {
        let input = b"*3\r\n:1\r\n:2\r\n:3\r\n";
        let expected = vec![Type::Integer(1), Type::Integer(2), Type::Integer(3)];
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, Type::Array(expected)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_bulk_string_array() {
        let input = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let expected = vec![
            Type::BulkString("hello".to_string()),
            Type::BulkString("world".to_string()),
        ];
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, Type::Array(expected)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_empty_array() {
        let input = b"*0\r\n";
        let expected = vec![];
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, Type::Array(expected)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_null_array() {
        let input = b"*-1\r\n";
        let expected = Type::Null;
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_not_parse_invalid_length() {
        let input = b"*3\r\n:1\r\n:2\r\n";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_insufficient_data() {
        let input = b"*3\r\n:1\r\n:2";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_missing_data() {
        let input = b"*3\r\n:1\r\n";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_missing_crlf() {
        let input = b"*3\r\n:1\r\n:2\r\n:3";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_invalid_element() {
        let input = b"*3\r\n:1\r\n:2\r\nabc\r\n";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_parse_mixed_data_types() {
        let input = b"*3\r\n:1\r\n+OK\r\n$6\r\nfoobar\r\n";
        let expected = vec![
            Type::Integer(1),
            Type::SimpleString("OK".to_string()),
            Type::BulkString("foobar".to_string()),
        ];
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, Type::Array(expected)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_support_nesting() {
        let input = b"*3\r\n:-23\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n$5\r\nhello\r\n-world\r\n";
        let expected = vec![
            Type::Integer(-23),
            Type::Array(vec![Type::Integer(1), Type::Integer(2), Type::Integer(3)]),
            Type::Array(vec![
                Type::BulkString("hello".to_string()),
                Type::SimpleError("world".to_string()),
            ]),
        ];
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, Type::Array(expected)),
            Err(err) => show(err),
        }
    }
}
