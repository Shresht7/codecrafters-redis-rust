// Library
use super::{
    reader::{self, CRLF},
    RESPData,
};

// -----------
// PARSE ARRAY
// -----------

/// Clients send commands to the Redis server as RESP arrays. Similarly
/// some Redis commands that return a collection of elements use arrays as their replies.
/// Arrays use the following encoding format:
/// - A prefix of `*` followed by the number of elements in the array.
/// - Each element in the array is encoded according to the rules of the RESP protocol.
/// - CRLF terminator sequence at the end of the array.
///
/// Example:
/// ```sh
/// *3\r\n:1\r\n:2\r\n:3\r\n => [1, 2, 3]
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Find the position of the first CRLF sequence
    let len_end_pos = bytes.find_crlf()?;

    // Extract the "length" of the array
    let length = bytes.to(len_end_pos).parse::<i64>()?;

    // Calculate the position of the start of the array data
    let data_start_pos = len_end_pos + CRLF.len();

    // Check if the array is empty or null
    if length <= 0 {
        return Ok((
            RESPData::Array(Vec::new()),
            &input[data_start_pos..], // Remaining bytes
        ));
    }

    // Check if there is enough data to parse the array
    if data_start_pos >= input.len() {
        return Err("Invalid input. Insufficient data".into());
    }

    // Parse the elements of the array
    let mut elements = Vec::new();
    let mut remaining = &input[data_start_pos..];
    // Iterate for the length of the array
    for _ in 0..length {
        let (element, rest) = super::_parse(remaining)?;
        elements.push(element);
        remaining = rest;
    }

    // Return the parsed array and the remaining input
    Ok((
        RESPData::Array(elements),
        remaining, // Remaining bytes
    ))
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_array() {
        let input = b"3\r\n:1\r\n:2\r\n:3\r\n";
        let expected = vec![
            RESPData::Integer(1),
            RESPData::Integer(2),
            RESPData::Integer(3),
        ];
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, RESPData::Array(expected));
    }

    #[test]
    fn should_parse_bulk_string_array() {
        let input = b"2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let expected = vec![
            RESPData::BulkString("hello".to_string()),
            RESPData::BulkString("world".to_string()),
        ];
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, RESPData::Array(expected));
    }

    #[test]
    fn should_parse_empty_array() {
        let input = b"0\r\n";
        let expected = vec![];
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, RESPData::Array(expected));
    }

    #[test]
    fn should_parse_null_array() {
        let input = b"-1\r\n";
        let expected = vec![];
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, RESPData::Array(expected));
    }

    #[test]
    fn should_not_parse_invalid_length() {
        let input = b"3\r\n:1\r\n:2\r\n";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_insufficient_data() {
        let input = b"3\r\n:1\r\n:2";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_missing_data() {
        let input = b"3\r\n:1\r\n";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_missing_crlf() {
        let input = b"3\r\n:1\r\n:2\r\n:3";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_not_parse_invalid_element() {
        let input = b"3\r\n:1\r\n:2\r\nabc\r\n";
        let actual = parse(input);
        assert!(actual.is_err());
    }

    #[test]
    fn should_parse_mixed_data_types() {
        let input = b"3\r\n:1\r\n+OK\r\n$6\r\nfoobar\r\n";
        let expected = vec![
            RESPData::Integer(1),
            RESPData::SimpleString("OK".to_string()),
            RESPData::BulkString("foobar".to_string()),
        ];
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, RESPData::Array(expected));
    }

    #[test]
    fn should_support_nesting() {
        let input = b"3\r\n:-23\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n$5\r\nhello\r\n-world\r\n";
        let expected = vec![
            RESPData::Integer(-23),
            RESPData::Array(vec![
                RESPData::Integer(1),
                RESPData::Integer(2),
                RESPData::Integer(3),
            ]),
            RESPData::Array(vec![
                RESPData::BulkString("hello".to_string()),
                RESPData::SimpleError("world".to_string()),
            ]),
        ];
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, RESPData::Array(expected));
    }
}
