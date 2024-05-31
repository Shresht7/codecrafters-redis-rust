// Library
use base64::{prelude::BASE64_STANDARD, Engine};

/// The magic bytes at the start of an RDB file
pub const MAGIC_BYTES: &[u8; 5] = b"REDIS";

/// Contents of an empty RDB file in base64 encoding
pub const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

/// Convert a base64 encoded string to a byte vector
pub fn base64_to_bytes(base64: &str) -> Vec<u8> {
    BASE64_STANDARD.decode(base64).unwrap()
}

/// Convert a byte vector to a base64 encoded string
// pub fn bytes_to_base64(bytes: &[u8]) -> String {
//     BASE64_STANDARD.encode(bytes)
// }

/// Represents the contents of an RDB file
struct RDB {
    pub version: String,
    // pub header: String,
    pub data: Vec<u8>,
    // pub checksum: String,
}

// The default implementation for RDB struct
impl Default for RDB {
    fn default() -> Self {
        let version = "0011".to_string();
        let data = base64_to_bytes(EMPTY_RDB);
        // let header = "redis-version:7.2.0\nredis-bits:64\n";
        // let checksum = "a2b3c4d5e6f7".to_string();

        RDB {
            version,
            // header,
            data,
            // checksum,
        }
    }
}

impl RDB {
    fn parse(&mut self, data: Vec<u8>) -> Result<&Self, Box<dyn std::error::Error>> {
        // Check if the data starts with the correct magic string (the first 5 bytes)
        if !data.starts_with(MAGIC_BYTES) {
            return Err(format!("Invalid RDB file: Expected magic bytes {:?}", MAGIC_BYTES).into());
        }

        // Read the next four bytes for the version
        self.version = String::from_utf8(data[5..9].to_vec())?;

        // Read the rest of the data
        self.data = String::from_utf8_lossy(&data).as_bytes().to_vec();

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
    fn test_base64_to_bytes() {
        let base64 = "SGVsbG8gV29ybGQ=";
        let bytes = base64_to_bytes(base64);
        assert_eq!(bytes, b"Hello World");
    }

    // #[test]
    // fn test_bytes_to_base64() {
    //     let bytes = b"Hello World";
    //     let base64 = bytes_to_base64(bytes);
    //     assert_eq!(base64, "SGVsbG8gV29ybGQ=");
    // }

    #[test]
    fn test_rdb_default() {
        let mut rdb = RDB::default();
        let bytes = base64_to_bytes(EMPTY_RDB);
        match rdb.parse(bytes) {
            Ok(_) => {
                assert_eq!(rdb.version, "0011");
                println!("{:?}", String::from_utf8_lossy(&rdb.data));
            }
            Err(e) => {
                panic!("Error parsing RDB: {:?}", e);
            }
        }
    }
}
