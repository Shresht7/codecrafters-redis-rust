// Library
use super::{errors::ParserError, reader, RESPData};

/// The first byte of a double value
const FIRST_BYTE: u8 = b',';

// -------------
// DOUBLE PARSER
// -------------

/// Parses a double value from the input byte slice.
///
/// Doubles are encoded as a sequence of bytes representing the double value.
/// A double encodes a double-precision floating-point number.
///
/// The doubles are encoded as the following:
/// - A prefix of `,`
/// - An optional (+) or (-) sign
/// - One or more decimal digits for the integer part
/// - An optional dot `.` character
/// - Zero or more decimal digits for the fractional part
/// - An optional exponent part, which consists of the letter `e` or `E`, an optional (+) or (-) sign, and one or more decimal digits
/// - CRLF terminator sequence at the end of the double
///
/// Example:
/// ```sh
/// ,3.14\r\n // 3.14
/// ,-3.14e-2\r\n // -0.0314
/// ```
///
/// Because the fractional part is optional, integers can be encoded as doubles:
/// ```sh
/// ,3\r\n // 3
/// ```
/// In such cases, the Redis client should return native integer and double values.
///
/// The positive infinity, negative infinity and NaN are encoded as:
/// ```sh
/// ,inf\r\n // +inf
/// ,-inf\r\n // -inf
/// ,nan\r\n // NaN
/// ```
pub fn parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Create a reader to help extract information from the input byte slice
    let mut bytes = reader::read(input);

    // Check if the input starts with the comma `,` character
    let first_byte = bytes.first()?;
    if first_byte != FIRST_BYTE {
        return Err(Box::new(ParserError::InvalidFirstByte(
            first_byte, FIRST_BYTE,
        )));
    }

    // Find the position of the CRLF sequence
    let (crlf_pos, rest_pos) = bytes.find_crlf()?;

    // // Handle the special cases for positive infinity, negative infinity and NaN
    match bytes.slice(1, crlf_pos).as_str()? {
        "inf" => return Ok((RESPData::Double(f64::INFINITY), &input[rest_pos..])),
        "-inf" => return Ok((RESPData::Double(f64::NEG_INFINITY), &input[rest_pos..])),
        "nan" => return Ok((RESPData::Double(f64::NAN), &input[rest_pos..])),
        _ => (),
    }

    // Check if the number is an integer
    if !bytes.contains(&b'.') {
        // Parse the double value so that it handles the exponents
        let double = bytes.slice(1, crlf_pos).parse::<f64>()?;
        // Convert the double to an integer
        let integer = double as i64;
        return Ok((RESPData::Integer(integer), &input[rest_pos..]));
    }

    // Parse the double value
    let double = bytes.slice(1, crlf_pos).parse::<f64>()?;

    // Return the double value and the remaining bytes
    Ok((RESPData::Double(double), &input[rest_pos..]))
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
    fn test_parse_double() {
        let input = b",3.14\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Double(3.14)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_negative_double() {
        let input = b",-3.14\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Double(-3.14)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_double_with_exponent() {
        let input = b",3.14e2\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Double(314.0)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_negative_double_with_exponent() {
        let input = b",-3.14e2\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Double(-314.0)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_integer_as_double() {
        let input = b",3\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Integer(3)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_integer_with_exponent() {
        let input = b",3e2\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Integer(300)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_positive_infinity() {
        let input = b",inf\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Double(f64::INFINITY)),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_negative_infinity() {
        let input = b",-inf\r\n";
        match parse(input) {
            Ok((actual, _)) => assert_eq!(actual, RESPData::Double(f64::NEG_INFINITY)),
            Err(err) => show(err),
        }
    }

    // Helper function to check if a double value is NaN
    impl RESPData {
        pub fn is_nan(&self) -> bool {
            match self {
                RESPData::Double(value) => value.is_nan(),
                _ => false,
            }
        }
    }

    #[test]
    fn test_parse_nan() {
        let input = b",nan\r\n";
        match parse(input) {
            Ok((actual, _)) => assert!(actual.is_nan()),
            Err(err) => show(err),
        }
    }

    #[test]
    fn test_parse_insufficient_data() {
        let input = b",3.14";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_first_byte() {
        let input = b"invalid";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_double() {
        let input = b",3.14e";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_exponent() {
        let input = b",3.14e-";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_exponent_value() {
        let input = b",3.14e-x\r\n";
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_crlf() {
        let input = b",3.14\n";
        let result = parse(input);
        assert!(result.is_err());
    }
}
