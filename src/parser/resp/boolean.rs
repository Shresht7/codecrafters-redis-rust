// Library
use super::Type;
use crate::parser::{
    errors::ParserError,
    reader::{self, CRLF},
};

/// The first_byte of a boolean value
const FIRST_BYTE: u8 = b'#';

// --------------
// BOOLEAN PARSER
// --------------

/// Parses a boolean value from the input byte slice.
///
/// Booleans can be either `true` or `false`; and use the following encoding format:
/// - A prefix of `#` followed by `t` for `true` or `f` for `false`.
/// - CRLF terminator sequence at the end of the boolean.
///
/// Example:
/// ```sh
/// #t\r\n // true
/// #f\r\n // false
/// ```
pub fn parse(input: &[u8]) -> Result<(Type, &[u8]), Box<dyn std::error::Error>> {
    // Check if the input is long enough to contain the boolean value
    if input.len() < 4 {
        return Err(BooleanParserError::InsufficientData(input.len()).into());
    }
    
    // Create a reader to help extract information from the input byte slice
    let bytes = reader::read(input);

    // Check if the input starts with the hash `#` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Create a reader to extract information from the bytes
    let mut bytes = reader::read(input);

    // Find the position of the CRLF sequence
    let (crlf_pos, crlf_end_pos) = bytes.find_crlf()?;

    // Extract the boolean value
    let boolean = match input[1] {
        b't' => true,
        b'f' => false,
        _ => return Err(BooleanParserError::InvalidBooleanCharacter(input[1]).into())
    };

    // Check if the boolean value is followed by the CRLF sequence
    if !input[crlf_pos..crlf_end_pos].starts_with(CRLF) {
        return Err(
            BooleanParserError::InvalidTerminator(input[crlf_pos..crlf_end_pos].to_vec()).into()
        );
    }

    // Return the parsed boolean value and the remaining input
    Ok((
        Type::Boolean(boolean),
        &input[crlf_end_pos..], // Remaining bytes
    ))
}

// ------
// ERRORS
// ------

// Define the error types for the boolean parser
#[derive(Debug)]
pub enum BooleanParserError {
    InsufficientData(usize),
    InvalidBooleanCharacter(u8),
    InvalidTerminator(Vec<u8>),
}

// Implement the `Display` trait for the boolean error
impl std::fmt::Display for BooleanParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BooleanParserError::InsufficientData(len) => 
                write!(f, "Insufficient data. The input length is {} but it should contain at least 4 bytes to represent boolean values", len),
            BooleanParserError::InvalidBooleanCharacter(byte) => 
                write!(f, "Invalid boolean value. Expected 't' or 'f' but got {}", *byte as char),
            BooleanParserError::InvalidTerminator(terminator) => 
                write!(f, "Invalid terminator. Expected CRLF sequence at the end of the boolean value but got {:?}", terminator),
        }
    }
}

// Implement the `Error` trait for the boolean error
impl std::error::Error for BooleanParserError {}

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
    fn should_error_on_insufficient_data() {
        let input = b"#t";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_error_on_invalid_first_byte() {
        let input = b"invalid";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_parse_true_boolean() {
        let input = b"#t\r\n";
        let expected = (Type::Boolean(true), &b""[..]);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected.0),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_false_boolean() {
        let input = b"#f\r\n";
        let expected = (Type::Boolean(false), &b""[..]);
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, expected.0),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_invalid_input_boolean() {
        let input = b"#x\r\n";
        let result = parse(input);
        assert!(result.is_err());
    }
    
    #[test]
    fn should_error_on_invalid_input_crlf() {
        let input = b"#t\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn should_return_remaining_bytes() {
        let input = b"#t\r\n+OK\r\n";
        let expected = (Type::Boolean(true), &b"+OK\r\n"[..]);
        match parse(input) {
            Ok(res) => assert_eq!(res, expected),
            Err(err) => show(err),
        }
    }

}
