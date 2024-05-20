// ---------
// CONSTANTS
// ---------

/// The Carriage Return Line Feed (CRLF) sequence
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
        end_pos: 0,
    }
}

impl<'a> BytesReader<'a> {
    /// Find the position of the first CRLF sequence in the byte slice.
    /// Respects the current start and end positions of the reader.
    /// If the CRLF sequence is not found, return an error.
    pub fn find_crlf(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        self.slice
            .windows(CRLF.len())
            .position(|window| window == CRLF)
            .ok_or("Invalid input. Expecting a CRLF sequence".into())
    }

    /// Set the start position of the reader
    pub fn from(&mut self, pos: usize) -> &mut Self {
        self.start_pos = pos;
        self
    }

    /// Set the end position of the reader
    pub fn to(&mut self, pos: usize) -> &mut Self {
        self.end_pos = pos;
        self
    }

    /// Extract a byte slice from the current start position to the current end position.
    /// Reset the start and end positions of the reader.
    pub fn as_bytes(&mut self) -> &[u8] {
        let byte_slice = &self.slice[self.start_pos..self.end_pos];
        self.start_pos = 0;
        self.end_pos = 0;
        return byte_slice;
    }

    /// Return the byte slice as a string slice
    pub fn as_str(&mut self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_bytes())
    }

    /// Return the byte slice as a String
    pub fn as_string(&mut self) -> Result<String, std::str::Utf8Error> {
        Ok(self.as_str()?.to_string())
    }

    /// Parse the byte slice as a type that implements the `FromStr` trait
    pub fn parse<T: std::str::FromStr>(&mut self) -> Result<T, Box<dyn std::error::Error>>
    where
        T::Err: std::error::Error + 'static,
    {
        Ok(self.as_str()?.parse::<T>()?)
    }
}
