// Library
use super::{
    reader::{self, CRLF},
    RESPData,
};

// -------------------
// PARSE SIMPLE STRING
// -------------------

/// Parses a `SimpleString` from the given input data
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Find the position of the CRLF sequence in the input
    let end_pos = bytes.find_crlf()?;

    // Extract the simple string from the input up to the CRLF sequence
    let simple_string = bytes.to(end_pos).as_string()?;

    // Return the parsed simple string and the remaining input
    Ok((
        RESPData::SimpleString(simple_string),
        &input[end_pos + CRLF.len()..],
    ))
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_simple_string() {
        let input = b"hello world\r\n";
        let expected = RESPData::SimpleString("hello world".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"hello world";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_parse_empty_input() {
        let input = b"\r\n";
        let expected = RESPData::SimpleString("".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_return_the_remaining_input() {
        let input = b"hello world\r\nextra data";
        let expected = RESPData::SimpleString("hello world".to_string());
        let (actual, remaining) = parse(input).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(remaining, b"extra data");
    }
}
