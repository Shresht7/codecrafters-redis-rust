// Library
use super::{helpers, RESPData, CRLF};

// ------------------
// PARSE BULK STRINGS
// ------------------

/// Parses a `BulkString` from the given input data
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Find the position of the first CRLF sequence in the input
    let len_end_pos = helpers::find_crlf(input)?;

    // Extract the length of the bulk string
    let length = String::from_utf8(input[..len_end_pos].to_vec())?.parse::<i64>()?;

    // Check if the bulk string is null
    if length == -1 {
        return Ok((
            RESPData::BulkString(String::new()),
            &input[len_end_pos + CRLF.len()..],
        ));
    }

    // Check if the input has enough elements
    if input.len() < len_end_pos + CRLF.len() + length as usize {
        return Err("Invalid input. Expecting more data".into());
    }

    // Find the position of the next CRLF sequence in the input
    let data_end_pos = len_end_pos + CRLF.len() + length as usize;

    // Extract the bulk string from the input and convert it to a String
    let bulk_string = String::from_utf8(input[len_end_pos + CRLF.len()..data_end_pos].to_vec())?;

    // Return the parsed bulk string and the remaining input
    Ok((
        RESPData::BulkString(bulk_string),
        &input[data_end_pos + CRLF.len()..],
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
        let expected = RESPData::BulkString("".to_string());
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
