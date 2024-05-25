// Library
use super::{errors::ParserError, reader, RESPData};

/// The first byte of a big number
const FIRST_BYTE: u8 = b'(';

// -----------------
// BIG NUMBER PARSER
// -----------------

/// Parse a big number from the input byte slice
///
/// The big number is encoded as:
/// - The left parenthesis `(` character
/// - An optional plus `+` or minus `-` sign
/// - One or more decimal digits for the integer part
/// - The CRLF terminator sequence at the end of the big number
///
/// Example:
/// ```sh
/// (1234567890\r\n // 1234567890
/// (-1234567890\r\n // -1234567890
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the left parenthesis `(` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the CRLF sequence
    let (crlf_pos, rest_pos) = bytes.find_crlf()?;

    // Parse the big number
    let big_number = bytes.slice(1, crlf_pos).as_str()?.parse::<i64>()?;

    // Return the big number and the remaining input byte slice
    Ok((RESPData::BigNumber(big_number), &input[rest_pos..]))
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
    fn should_parse_big_number() {
        let input = b"(1234567890\r\n";
        let expected = RESPData::BigNumber(1234567890);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_negative_big_number() {
        let input = b"(-1234567890\r\n";
        let expected = RESPData::BigNumber(-1234567890);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_invalid_first_byte() {
        let input = b"(1234567890";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_invalid_input() {
        let input = b"1234567890\r\n";
        let result = parse(input);
        assert!(result.is_err());
    }
}
