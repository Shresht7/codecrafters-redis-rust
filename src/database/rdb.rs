// Library
use crate::helpers;
use crate::parser::reader;
use std::collections::HashMap;

/// The magic bytes at the start of an RDB file
pub const MAGIC_BYTES: &[u8; 5] = b"REDIS";

/// Contents of an empty RDB file in base64 encoding
pub const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

/// Represents the contents of an RDB file
pub struct RDB {
    pub magic_string: String,
    pub version: String,
    pub data: HashMap<String, String>,
}

impl Default for RDB {
    fn default() -> Self {
        RDB {
            magic_string: String::new(),
            version: String::new(),
            data: HashMap::new(),
        }
    }
}

/// Parses the given RDB file data and returns the corresponding `RDB` struct
pub async fn parse(data: Vec<u8>) -> Result<RDB, Box<dyn std::error::Error>> {
    let mut rdb = RDB::default();
    rdb.parse(data).await?;
    Ok(rdb)
}

impl RDB {
    /// Parses the given RDB file data and updates the `RDB` struct
    async fn parse(&mut self, data: Vec<u8>) -> Result<&mut Self, Box<dyn std::error::Error>> {
        // Check if the data starts with the correct magic string (the first 5 bytes)
        if !data.starts_with(MAGIC_BYTES) {
            return Err(format!("Invalid RDB file: Expected magic bytes {:?}", MAGIC_BYTES).into());
        }

        // Read the first five bytes as the magic string
        self.magic_string = String::from_utf8(data[0..4].to_vec())?;

        // Read the next four bytes for the version
        self.version = String::from_utf8(data[5..9].to_vec())?;

        println!("{:?}", data);

        // Create a cursor to read the remaining data
        let mut bytes = reader::read(&data[9..]);

        // TODO: THIS IS A COMPLETE HACK. NEED TO FIX THIS
        // Read until the double zero !HACK
        let (_, mut rest) = bytes
            .split(&[0, 0])
            .expect("Failed to find double zero. THIS IS A HACK");
        let mut b = rest.as_bytes();

        loop {
            println!("{:?}", b);
            if b[0] == 0xFF {
                // End of the data
                break;
            }
            let key_length = b[0] as usize;
            let key = String::from_utf8(b[1..=key_length].to_vec())?;

            let value_len = (&b[key_length + 1..key_length + 2])[0] as usize;
            let value = String::from_utf8(b[key_length + 2..key_length + 2 + value_len].to_vec())?;
            println!("Key: {}, Value: {}", key, value);

            // Insert the key-value pair into the data hashmap
            self.data.insert(key, value);

            // Update the remaining data
            b = &b[key_length + 2 + value_len..];
        }

        Ok(self)
    }
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rdb_default() {
        let bytes = helpers::base64_to_bytes(EMPTY_RDB);
        let rdb = parse(bytes).await.unwrap();
        assert_eq!(rdb.version, "0011");
    }

    #[tokio::test]
    async fn test_rdb() {
        let bytes = [
            82, 69, 68, 73, 83, 48, 48, 48, 51, 250, 9, 114, 101, 100, 105, 115, 45, 118, 101, 114,
            5, 55, 46, 50, 46, 48, 250, 10, 114, 101, 100, 105, 115, 45, 98, 105, 116, 115, 192,
            64, 254, 0, 251, 3, 3, 252, 0, 156, 239, 18, 126, 1, 0, 0, 0, 9, 98, 108, 117, 101, 98,
            101, 114, 114, 121, 4, 112, 101, 97, 114, 252, 0, 12, 40, 138, 199, 1, 0, 0, 0, 4, 112,
            101, 97, 114, 9, 112, 105, 110, 101, 97, 112, 112, 108, 101, 252, 0, 12, 40, 138, 199,
            1, 0, 0, 0, 5, 103, 114, 97, 112, 101, 9, 98, 108, 117, 101, 98, 101, 114, 114, 121,
            255, 76, 205, 60, 203, 238, 60, 229, 217, 10,
        ];
        let rdb = parse(bytes.to_vec()).await.unwrap();
        assert_eq!(rdb.version, "0003");
        assert_eq!(rdb.data.len(), 1); // Only one key-value pair for now
        assert_eq!(rdb.data.get("blueberry").unwrap(), "pear");
    }
}

// TEST CONTENTS
// [82, 69, 68, 73, 83, 48, 48, 48, 51, 250, 9, 114, 101, 100, 105, 115, 45, 118, 101, 114, 5, 55, 46, 50, 46, 48, 250, 10, 114, 101, 100, 105, 115, 45, 98, 105, 116, 115, 192, 64, 254, 0, 251, 1, 0, 0, 4, 112, 101, 97, 114, 5, 97, 112, 112, 108, 101, 255, 98, 13, 59, 53, 179, 65, 228, 176, 10]
