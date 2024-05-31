// Library
use super::opcode;
use crate::helpers;
use std::collections::HashMap;

/// The magic bytes at the start of an RDB file
pub const MAGIC_BYTES: &[u8; 5] = b"REDIS";

/// Contents of an empty RDB file in base64 encoding
pub const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

/// Represents the contents of an RDB file
struct RDB {
    pub magic_string: String,
    pub version: String,
}

impl Default for RDB {
    fn default() -> Self {
        RDB {
            magic_string: String::new(),
            version: String::new(),
        }
    }
}

/// Parses the given RDB file data and returns the corresponding `RDB` struct
fn parse(data: Vec<u8>) -> Result<RDB, Box<dyn std::error::Error>> {
    let mut rdb = RDB::default();
    rdb.parse(data)?;
    Ok(rdb)
}

impl RDB {
    /// Parses the given RDB file data and updates the `RDB` struct
    fn parse(&mut self, data: Vec<u8>) -> Result<&mut Self, Box<dyn std::error::Error>> {
        // Check if the data starts with the correct magic string (the first 5 bytes)
        if !data.starts_with(MAGIC_BYTES) {
            return Err(format!("Invalid RDB file: Expected magic bytes {:?}", MAGIC_BYTES).into());
        }

        // Read the first five bytes as the magic string
        self.magic_string = String::from_utf8(data[0..4].to_vec())?;

        // Read the next four bytes for the version
        self.version = String::from_utf8(data[5..9].to_vec())?;

        Ok(self)
    }
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdb_default() {
        let bytes = helpers::base64_to_bytes(EMPTY_RDB);
        let rdb = parse(bytes).unwrap();
        assert_eq!(rdb.version, "0011");
    }
}
