// Library
mod array;
mod bulk_string;
mod data_types;
mod integer;
mod reader;
mod simple_error;
mod simple_string;

// Use statements
use data_types::RESPData;

/// Parses the given input data and returns the corresponding `RESPData` and the remaining input
fn _parse(input: &[u8]) -> Result<(RESPData, &[u8]), Box<dyn std::error::Error>> {
    // Extract the first byte from the input, which indicates the data type
    let first_byte = input.first().ok_or("Empty input")?;

    // Match on the first_byte to determine the data type and parse the input accordingly
    match first_byte {
        b'+' => simple_string::parse(&input[1..]),
        b'-' => simple_error::parse(&input[1..]),
        b':' => integer::parse(&input[1..]),
        b'$' => bulk_string::parse(&input[1..]),
        b'*' => array::parse(&input[1..]),
        _ => Err("Invalid data type".into()),
    }
}

// -----
// PARSE
// -----

/// Parses the given input data and returns the corresponding `RESPData`
pub fn parse(input: &[u8]) -> Result<Vec<RESPData>, Box<dyn std::error::Error>> {
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

    #[test]
    fn should_parse_simple_string() {
        let input = b"+hello world\r\n";
        let expected = vec![RESPData::SimpleString("hello world".to_string())];
        let actual = parse(input).unwrap();
        assert_eq!(actual, expected);
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
            RESPData::SimpleString("hello world".to_string()),
            RESPData::Integer(-123),
        ];
        let actual = parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore]
    fn should_parse_echo_command() {
        let input = ["*2\r\n", "$4\r\n", "ECHO\r\n", "$9\r\n", "pineapple\r\n"];
        let expected = vec![
            RESPData::Integer(2),
            RESPData::SimpleString("ECHO".to_string()),
            RESPData::SimpleString("pineapple".to_string()),
        ];
        let actual = parse(input.concat().as_bytes()).unwrap();
        assert_eq!(actual, expected);
    }
}
