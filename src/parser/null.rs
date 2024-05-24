// Library
use super::{reader::CRLF, RESPData};

// ----------
// PARSE NULL
// ----------

/// Parses a `Null` value from the given input data.
/// A null value is represented by the underscore `_` character followed by a CRLF sequence.
/// A null value is used to represent the absence of a value in RESP2.
/// In RESP3, a null value is represented by the `Null` data type.
///
/// Example:
/// ```sh
/// _\r\n
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Find the position of the first CRLF sequence
    let end_pos = CRLF.len() + 1;

    // Check if the input is long enough to contain a null value
    if input.len() < end_pos {
        return Err("Invalid input. Insufficient data".into());
    }

    // Check if the first byte is an underscore
    if input[0] != b'_' {
        return Err("Invalid input. Expected underscore".into());
    }

    // Check if the second byte is a CRLF sequence
    if &input[1..end_pos] != CRLF {
        return Err("Invalid input. Expected CRLF".into());
    }

    // Return the parsed null value and the remaining input
    Ok((RESPData::Null, &input[end_pos..]))
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_null() {
        let input = b"_\r\n";
        let expected = (RESPData::Null, &b""[..]);
        let result = parse(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"invalid";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_insufficient_data() {
        let input = b"_";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_invalid_input_underscore() {
        let input = b"X\r\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_invalid_input_crlf() {
        let input = b"_X\n";
        let result = parse(input);
        assert!(result.is_err());
    }
}
