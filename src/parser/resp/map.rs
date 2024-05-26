// Library
use super::Type;
use crate::parser::{_parse, errors::ParserError, reader};
use std::collections::HashMap;

/// The first byte of a map value.
const FIRST_BYTE: u8 = b'%';

// ---------
// PARSE MAP
// ---------

/// Parses a RESP map from the given input data.
///
/// Maps use the following encoding format:
/// - A prefix of `%` followed by the number of key-value pairs in the map.
/// - Each key-value pair is encoded according to the rules of the RESP protocol.
/// - CRLF terminator sequence at the end of the map.
///
/// Example:
/// ```sh
/// %2\r\n+key1\r\n:1\r\n+key2\r\n:2\r\n => {"key1": 1, "key2": 2}
/// ```
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the percent `%` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(ParserError::InvalidFirstByte(first_byte, FIRST_BYTE).into());
    }

    // Find the position of the first CRLF sequence and the start of the map data
    let (len_end_pos, data_start_pos) = bytes.find_crlf()?;

    // Extract the "length" of the map
    let length = bytes.slice(1, len_end_pos).parse::<i64>()?;

    // If the length is -1, the map is null
    if length == -1 {
        return Ok((Type::Null, &input[data_start_pos..]));
    }

    // If the length is 0, the map is empty
    if length <= 0 {
        return Ok((Type::Map(HashMap::new()), &input[data_start_pos..]));
    }

    // Parse the key-value pairs of the map
    let mut map = HashMap::new();
    let mut remaining = &input[data_start_pos..];
    for _ in 0..length {
        let (key, rest) = _parse(remaining)?;
        let (value, rest) = _parse(rest)?;
        map.insert(key, value);
        remaining = rest;
    }

    // Return the parsed map
    Ok((Type::Map(map), remaining))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_map() {
        let input = b"%2\r\n+key1\r\n:1\r\n+key2\r\n:2\r\n";
        let (map, remaining) = parse(input).unwrap();

        assert_eq!(
            map,
            Type::Map(
                vec![
                    (Type::SimpleString("key1".to_string()), Type::Integer(1)),
                    (Type::SimpleString("key2".to_string()), Type::Integer(2)),
                ]
                .into_iter()
                .collect()
            )
        );
        assert_eq!(remaining, b"");
    }

    #[test]
    fn should_parse_map_with_keys_as_other_types() {
        let input = b"%2\r\n:1\r\n+key2\r\n+test\r\n:2\r\n";
        let (map, remaining) = parse(input).unwrap();

        assert_eq!(
            map,
            Type::Map(
                vec![
                    (Type::Integer(1), Type::SimpleString("key2".to_string())),
                    (Type::SimpleString("test".to_string()), Type::Integer(2)),
                ]
                .into_iter()
                .collect()
            )
        );
        assert_eq!(remaining, b"");
    }

    #[test]
    fn test_parse_map_null() {
        let input = b"%-1\r\n";
        let (map, remaining) = parse(input).unwrap();

        assert_eq!(map, Type::Null);
        assert_eq!(remaining, b"");
    }

    #[test]
    fn test_parse_map_empty() {
        let input = b"%0\r\n";
        let (map, remaining) = parse(input).unwrap();

        assert_eq!(map, Type::Map(HashMap::new()));
        assert_eq!(remaining, b"");
    }
}
