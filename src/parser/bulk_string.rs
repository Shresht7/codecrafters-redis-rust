// Library
use super::{
    reader::{self, CRLF},
    RESPData,
};

// ------------------
// PARSE BULK STRINGS
// ------------------

/// Parses a `BulkString` from the given input data.
/// A bulk string is a sequence of bytes with a length of `N` bytes followed by `N` bytes of data.
/// The length of the bulk string is encoded as a decimal number followed by a CRLF sequence.
/// If the length is -1, the bulk string is null.
/// The bulk string is terminated by a CRLF sequence.
///
/// Example:
/// ```sh
/// 6\r\nfoobar\r\n => "foobar"
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Find the position of the first CRLF sequence
    let len_end_pos = bytes.find_crlf()?;

    // Extract the "length" of the bulk string
    let length = bytes.to(len_end_pos).parse::<i64>()?;

    // Calculate the position of the start of the bulk string data
    let data_start_pos = len_end_pos + CRLF.len();

    // Check if the bulk string is null
    if length == -1 {
        return Ok((
            RESPData::Null,
            &input[data_start_pos..], // Remaining bytes
        ));
    }

    // Check if there is enough data to parse the bulk string
    if data_start_pos + length as usize > input.len() {
        return Err("Invalid input. Insufficient data".into());
    }

    // Calculate the position of the end of the bulk string data
    let data_end_pos = data_start_pos + length as usize;

    // Extract the bulk string from the input and convert it to a String
    let bulk_string = bytes.from(data_start_pos).to(data_end_pos).as_string()?;

    // Return the parsed bulk string and the remaining input
    Ok((
        RESPData::BulkString(bulk_string),
        &input[data_end_pos + CRLF.len()..], // Remaining bytes
    ))
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_bulk_string() {
        let input = b"6\r\nfoobar\r\n";
        let expected = RESPData::BulkString("foobar".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_parse_empty_bulk_string() {
        let input = b"0\r\n\r\n";
        let expected = RESPData::BulkString("".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_parse_null_bulk_string() {
        let input = b"-1\r\n";
        let expected = RESPData::Null;
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_not_parse_invalid_length() {
        let input = b"abc\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_insufficient_data() {
        let input = b"6\r\nfoo\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_missing_data() {
        let input = b"3\r\nfo";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_missing_crlf() {
        let input = b"3\nfoo\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_not_parse_input_without_crlf() {
        let input = b"foobar";
        assert!(parse(input)
            .is_err_and(|e| e.to_string() == "Invalid input. Expecting a CRLF sequence"));
    }
}
