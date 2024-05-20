// Library
use super::{
    helpers::{self, CRLF},
    RESPData,
};

// --------------
// PARSE INTEGERS
// --------------

/// Parses an `Integer` from the given input data
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract data from the input
    let mut reader = helpers::read(input);

    // Find the position of the CRLF sequence in the input
    let end_pos = reader.find_crlf()?;

    // Extract the integer from the input up to the CRLF sequence and parse it as an i64
    let integer = reader.to(end_pos).parse::<i64>()?;

    // Return the parsed integer and the remaining input
    Ok((RESPData::Integer(integer), &input[end_pos + CRLF.len()..]))
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_integer() {
        let input = b"123\r\n";
        let expected = RESPData::Integer(123);
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_parse_negative_integer() {
        let input = b"-123\r\n";
        let expected = RESPData::Integer(-123);
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_parse_zero() {
        let input = b"0\r\n";
        let expected = RESPData::Integer(0);
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_not_parse_floats() {
        let input = b"123.45\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_return_the_remaining_input() {
        let input = b"123\r\nhello world";
        let expected = b"hello world";
        let (_, remaining) = parse(input).unwrap();
        assert_eq!(remaining, expected);
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"hello world\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_error_on_empty_input() {
        let input = b"\r\n";
        assert!(parse(input).is_err());
    }
}
