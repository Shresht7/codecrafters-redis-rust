// Library
mod errors;
mod reader;
pub mod resp;

/// Parses the given input data and returns the corresponding `RESPData` and the remaining input
fn _parse(input: &[u8]) -> Result<(resp::Type, &[u8]), Box<dyn std::error::Error>> {
    // Extract the first byte from the input, which indicates the data type
    let first_byte = input.first().ok_or("Empty input")?;

    // Match on the first_byte to determine the data type and parse the input accordingly
    match first_byte {
        b'+' => resp::simple_string::parse(&input),
        b'-' => resp::simple_error::parse(&input),
        b':' => resp::integer::parse(&input),
        b'$' => resp::bulk_string::parse(&input),
        b'*' => resp::array::parse(&input),
        b'_' => resp::null::parse(&input),
        b'#' => resp::boolean::parse(&input),
        b',' => resp::double::parse(&input),
        b'(' => resp::big_number::parse(&input),
        b'!' => resp::bulk_error::parse(&input),
        b'=' => resp::verbatim_string::parse(&input),
        b'%' => resp::map::parse(&input),
        b'~' => resp::set::parse(&input),
        _ => Err(format!("Invalid first byte in {}", String::from_utf8_lossy(input)).into()),
    }
}

// -----
// PARSE
// -----

/// Parses the given input data and returns the corresponding `RESPData`
pub fn parse(input: &[u8]) -> Result<Vec<resp::Type>, Box<dyn std::error::Error>> {
    // The parsed data
    let mut data = Vec::new();

    // The remaining input data that has not been parsed yet
    let mut remaining = input;

    // Parse the input data until there is no more data to parse
    while !remaining.is_empty() {
        // Parse the next data element and update the remaining input
        let (element, rest) = _parse(remaining)?;
        data.push(element);
        remaining = rest;
    }

    // Return the parsed data
    Ok(data)
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
    fn should_parse_simple_string() {
        let input = b"+hello world\r\n";
        let expected = vec![resp::Type::SimpleString("hello world".to_string())];
        match parse(input) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_empty_input() {
        let input = b"\r\n";
        assert!(parse(input).is_err());
    }

    #[test]
    fn should_parse_multiple_elements() {
        let input = b"+hello world\r\n:-123\r\n";
        let expected = vec![
            resp::Type::SimpleString("hello world".to_string()),
            resp::Type::Integer(-123),
        ];
        match parse(input) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    #[ignore]
    fn should_parse_echo_command() {
        let input = ["*2\r\n", "$4\r\n", "ECHO\r\n", "$9\r\n", "pineapple\r\n"];
        let expected = vec![
            resp::Type::Integer(2),
            resp::Type::SimpleString("ECHO".to_string()),
            resp::Type::SimpleString("pineapple".to_string()),
        ];
        match parse(input.concat().as_bytes()) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_null() {
        let input = b"_\r\n";
        let expected = vec![resp::Type::Null];
        match parse(input) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_null_array() {
        let input = b"*-1\r\n";
        let expected = vec![resp::Type::Null];
        match parse(input) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_null_bulk_string() {
        let input = b"$-1\r\n";
        let expected = vec![resp::Type::Null];
        match parse(input) {
            Ok(actual) => assert_eq!(actual, expected),
            Err(err) => show(err),
        }
    }
}
