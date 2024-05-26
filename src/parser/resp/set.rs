use std::collections::HashSet;

// Library
use super::Type;
use crate::parser::{_parse, errors::ParserError, reader};

/// The first byte of a set value.
const FIRST_BYTE: u8 = b'~';

// ---------
// PARSE SET
// ---------

/// Parses a RESP set from the given input data.
///
/// Sets use the following encoding format:
/// - A prefix of `~` followed by the number of elements in the set.
/// - Each element in the set is encoded according to the rules of the RESP protocol.
/// - CRLF terminator sequence at the end of the set.
///
/// Example:
/// ```sh
/// ~3\r\n:1\r\n:2\r\n:3\r\n => {1, 2, 3}
/// ```
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the tilde `~` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the first CRLF sequence and the start of the set data
    let (len_end_pos, data_start_pos) = bytes.find_crlf()?;

    // Extract the "length" of the set
    let length = bytes.slice(1, len_end_pos).parse::<i64>()?;

    // If the length is 0, the set is empty
    if length <= 0 {
        return Ok((
            Type::Set(HashSet::new()),
            &input[data_start_pos..], // Remaining bytes
        ));
    }

    // Parse the elements of the set
    let mut elements = HashSet::new();
    let mut remaining = &input[data_start_pos..];
    for _ in 0..length {
        let (element, rest) = _parse(remaining)?;
        elements.insert(element);
        remaining = rest;
    }

    // Return the parsed set
    Ok((Type::Set(elements), &input[data_start_pos..]))
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
    fn should_parse_set() {
        let input = b"~3\r\n:1\r\n:2\r\n:3\r\n";
        let expected = Type::Set(HashSet::from([
            Type::Integer(1),
            Type::Integer(2),
            Type::Integer(3),
        ]));
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_empty_set() {
        let input = b"~0\r\n";
        let expected = Type::Set(HashSet::new());
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_only_collect_unique_values() {
        let input = b"~3\r\n:1\r\n:2\r\n:1\r\n";
        let expected = Type::Set(HashSet::from([Type::Integer(1), Type::Integer(2)]));
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_not_parse_invalid_first_byte() {
        let input = b"*3\r\n:1\r\n:2\r\n:3\r\n";
        assert!(parse(input).is_err())
    }

    #[test]
    fn should_not_parse_invalid_length() {
        let input = b"~3\r\n:1\r\n:2\r\n";
        assert!(parse(input).is_err())
    }
}
