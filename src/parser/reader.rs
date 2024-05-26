// Library
use super::errors::ParserError;

// ---------
// CONSTANTS
// ---------

/// The Carriage Return Line Feed (CRLF) sequence.
/// This is used as the terminator in the Redis Serialization Protocol (RESP)
pub const CRLF: &[u8] = b"\r\n";

// ------------
// BYTES READER
// ------------

/// A helper struct to read bytes from a byte slice
pub struct BytesReader<'a> {
    slice: &'a [u8],
    start_pos: usize,
    end_pos: usize,
}

/// Create a new `BytesReader` instance
pub fn read(input: &[u8]) -> BytesReader {
    BytesReader {
        slice: input,
        start_pos: 0,
        end_pos: input.len(),
    }
}

impl<'a> BytesReader<'a> {
    /// Check if the byte slice contains the given byte.
    ///
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let has_byte = bytes.contains(&b'h'); // => true
    /// ```
    pub fn contains(&mut self, byte: &u8) -> bool {
        self.slice.contains(byte)
    }

    /// Find the position of the first occurrence of the given byte in the byte slice.
    /// Respects the current start and end positions of the reader.
    /// If the byte is not found, return `None`.
    /// Otherwise, return the position of the byte.
    ///
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let pos = bytes.find(&b'w').unwrap(); // => 6
    /// ```
    pub fn find(&mut self, bytes: &[u8]) -> Option<usize> {
        let pos = self.slice[self.start_pos..self.end_pos]
            .windows(bytes.len())
            .position(|window| window == bytes)?;
        Some(pos)
    }

    /// Split the byte slice at the first occurrence of the given byte.
    /// Return the byte slices before and after the byte.
    /// If the byte is not found, return an error.
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let (first, rest) = bytes.split(b'w').unwrap(); // => (b"hello ", b"world")
    /// ```   
    pub fn split(
        &mut self,
        bytes: &[u8],
    ) -> Result<(BytesReader, BytesReader), Box<dyn std::error::Error>> {
        let position = self.find(bytes).unwrap();
        let (first, rest) = self.slice[self.start_pos..].split_at(position);
        let (_, rest) = rest.split_at(bytes.len());
        Ok((read(first), read(rest)))
    }

    /// Find the position of the first CRLF sequence in the byte slice.
    /// Respects the current start and end positions of the reader.
    /// If the CRLF sequence is not found, return an error.
    ///
    /// ```rs
    /// let input: &[u8] = b"hello world\r\n"; // Input byte slice
    /// let mut bytes = reader::read(input);   // Create a new BytesReader instance
    /// let pos = bytes.find_crlf().unwrap();  // => 11
    /// ```
    pub fn find_crlf(&mut self) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let start_pos = self
            .slice
            .windows(CRLF.len())
            .position(|window| window == CRLF)
            .ok_or(BytesReaderError::NonTerminating(self.slice.len()))?;
        let end_pos = start_pos + CRLF.len();
        Ok((start_pos, end_pos))
    }

    /// Split at the first CRLF sequence in the byte slice.
    /// Return the byte slices before and after the CRLF sequence.
    pub fn split_crlf(&mut self) -> Result<(BytesReader, BytesReader), Box<dyn std::error::Error>> {
        let (start, _) = self.find_crlf()?;
        let (first, rest) = self.slice.split_at(start);
        let (_, rest) = rest.split_at(CRLF.len());
        Ok((read(first), read(rest)))
    }

    /// Return the first byte in the byte slice
    /// If the byte slice is empty, return an error.
    ///
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let bytes = reader::read(input);   // Create a new BytesReader instance
    /// let first_byte = bytes.first().unwrap(); // => b'h'
    /// ```
    pub fn first(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.slice.get(0) {
            Some(b) => Ok(b.clone()),
            None => Err(Box::new(ParserError::EmptyInput)),
        }
    }

    /// Set the start and end positions of the reader to extract a byte slice.
    /// The start position is inclusive and the end position is exclusive.
    pub fn slice(&mut self, start: usize, end: usize) -> &mut Self {
        // Swap the start and end positions if the start position is greater than the end position
        if start > end {
            // Swap the start and end positions and re-call the slice method
            return self.slice(end, start);
        }

        // Set the start and end positions
        if start >= self.slice.len() {
            self.start_pos = self.end_pos;
        } else {
            self.start_pos = start;
        }
        if end > self.slice.len() {
            self.end_pos = self.slice.len();
        } else {
            self.end_pos = end;
        }

        self
    }

    /// Extract a byte slice from the current start position to the current end position.
    /// Reset the start and end positions of the reader.
    ///
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let slice = bytes.slice(1, 6).as_bytes(); // => b"ello "
    /// ```
    pub fn as_bytes(&mut self) -> &[u8] {
        let byte_slice = &self.slice[self.start_pos..self.end_pos];
        self.start_pos = 0;
        self.end_pos = 0;
        return byte_slice;
    }

    /// Return the byte slice as a string slice
    /// If the byte slice is not a valid UTF-8 sequence, return an error.
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let string = bytes.as_str().unwrap(); // => "hello world"
    /// ```
    pub fn as_str(&mut self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_bytes())
    }

    /// Return the byte slice as a String
    /// If the byte slice is not a valid UTF-8 sequence, return an error.
    /// ```rs
    /// let input: &[u8] = b"hello world"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let string = bytes.as_string().unwrap(); // => "hello world"
    /// ```
    pub fn as_string(&mut self) -> Result<String, std::str::Utf8Error> {
        Ok(self.as_str()?.to_string())
    }

    /// Parse the byte slice as a type that implements the `FromStr` trait
    /// If the byte slice is not a valid UTF-8 sequence, return an error.
    /// If the type cannot be parsed from the byte slice, return an error.
    /// ```rs
    /// let input: &[u8] = b":12345\r\n"; // Input byte slice
    /// let mut bytes = reader::read(input); // Create a new BytesReader instance
    /// let integer = bytes.slice(1, 6).parse::<i64>().unwrap(); // => 12345
    /// ```
    pub fn parse<T: std::str::FromStr>(&mut self) -> Result<T, Box<dyn std::error::Error>>
    where
        T::Err: std::error::Error + 'static,
    {
        Ok(self.as_str()?.parse::<T>()?)
    }
}

// ------
// ERRORS
// ------

/// Errors that can occur while reading bytes from a byte slice
#[derive(Debug)]
pub enum BytesReaderError {
    /// The CRLF sequence was not found in the byte slice
    NonTerminating(usize),
}

// Implement the `Display` trait for the `BytesReaderError` type
impl std::fmt::Display for BytesReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BytesReaderError::NonTerminating(len) => {
                write!(
                    f,
                    "Non-terminating byte slice: Expected CRLF sequence at position {}",
                    len
                )
            }
        }
    }
}

// Implement the `Error` trait for the `BytesReaderError` type
impl std::error::Error for BytesReaderError {}

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
    fn should_contain_byte() {
        let input = b"hello world";
        let mut bytes = read(input);
        assert!(bytes.contains(&b'h'));
    }

    #[test]
    fn should_not_contain_byte() {
        let input = b"hello world";
        let mut bytes = read(input);
        assert!(!bytes.contains(&b'z'));
    }

    #[test]
    fn should_find_byte() {
        let input = b"hello world";
        let mut bytes = read(input);
        match bytes.find(b"w") {
            Some(pos) => assert_eq!(pos, 6),
            None => panic!("Byte {:?} not found in {:?}", b'w', input,),
        }
    }

    #[test]
    fn should_not_find_byte() {
        let input = b"hello world";
        let mut bytes = read(input);
        let pos = bytes.find(b"z");
        assert!(pos.is_none());
    }

    #[test]
    fn should_split_bytes() {
        let input = b"hello world";
        let mut bytes = read(input);
        match bytes.split(b" ") {
            Ok((mut first, mut rest)) => {
                let first = first.as_bytes();
                let rest = rest.as_bytes();
                assert_eq!(first, b"hello");
                assert_eq!(rest, b"world");
            }
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_find_crlf() {
        let input = b"hello world\r\n";
        let mut bytes = read(input);
        match bytes.find_crlf() {
            Ok((start, end)) => {
                assert_eq!(start, 11);
                assert_eq!(end, 13);
            }
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_error_on_missing_crlf() {
        let input = b"hello world";
        let mut bytes = read(input);
        let result = bytes.find_crlf();
        assert!(result.is_err());
    }

    #[test]
    fn should_split_crlf() {
        let input = b"hello\r\nworld";
        let mut bytes = read(input);
        match bytes.split_crlf() {
            Ok((mut first, mut rest)) => {
                assert_eq!(first.as_bytes(), b"hello");
                assert_eq!(rest.as_bytes(), b"world");
            }
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_get_the_first_byte() {
        let input = b"hello world";
        let bytes = read(input);
        match bytes.first() {
            Ok(byte) => assert_eq!(byte, b'h'),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_slice_bytes() {
        let input = b"hello world";
        let mut bytes = read(input);
        let slice = bytes.slice(1, 6).as_bytes();
        assert_eq!(slice, b"ello ");
    }

    #[test]
    fn should_parse_integer() {
        let input = b":12345\r\n";
        let mut bytes = read(input);
        match bytes.slice(1, 6).parse::<i64>() {
            Ok(integer) => assert_eq!(integer, 12345),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_double() {
        let input = b",3.14\r\n";
        let mut bytes = read(input);
        match bytes.slice(1, 5).parse::<f64>() {
            Ok(double) => assert_eq!(double, 3.14),
            Err(err) => show(err),
        }
    }

    #[test]
    fn should_parse_string() {
        let input = b"+hello world\r\n";
        let mut bytes = read(input);
        match bytes.slice(1, 12).as_string() {
            Ok(string) => assert_eq!(string, "hello world"),
            Err(err) => panic!("Error: {}", err),
        }
    }
}
