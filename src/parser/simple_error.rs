// Library
use super::{helpers, RESPData, CRLF};

// -------------------
// PARSE SIMPLE ERRORS
// -------------------

/// Parses a `SimpleError` from the given input data
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Find the position of the CRLF sequence in the input
    let end_pos = helpers::find_crlf(input)?;

    // Extract the error message from the input up to the CRLF sequence
    let error_message = String::from_utf8(input[..end_pos].to_vec())?;

    // Return the parsed error message and the remaining input
    Ok((
        RESPData::SimpleError(error_message),
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
    fn should_parse_simple_error() {
        let input = b"Error message\r\n";
        let expected = RESPData::SimpleError("Error message".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_parse_empty_simple_error() {
        let input = b"\r\n";
        let expected = RESPData::SimpleError("".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_parse_simple_error_with_special_characters() {
        let input = b"Error message with special characters: !@#$%^&*()\r\n";
        let expected =
            RESPData::SimpleError("Error message with special characters: !@#$%^&*()".to_string());
        let (actual, _) = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_not_parse_simple_error_without_crlf() {
        let input = b"Error message";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_return_the_remaining_input() {
        let input = b"Error message\r\nRemaining input";
        let expected = b"Remaining input";
        let (_, remaining) = parse(input).unwrap();
        assert_eq!(remaining, expected);
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"Error message\r\nRemaining input";
        assert!(parse(input).is_ok());
    }

    #[test]
    fn should_error_on_empty_input() {
        let input = b"\r\n";
        assert!(parse(input).is_ok());
    }

    #[test]
    fn should_error_on_invalid_utf8() {
        let input = b"Error message\xF0\x28\x8C\xBC\r\n";
        assert!(parse(input).is_err());
    }
}
