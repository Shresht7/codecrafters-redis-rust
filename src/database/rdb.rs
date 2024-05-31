// Library
use crate::helpers;
use std::collections::HashMap;

/// The magic bytes at the start of an RDB file
pub const MAGIC_BYTES: &[u8; 5] = b"REDIS";

/// Contents of an empty RDB file in base64 encoding
pub const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

/// Represents the contents of an RDB file
struct RDB {
    pub version: String,
    pub header: HashMap<String, String>,
    pub data: Vec<u8>,
    // pub checksum: String,
}

fn parse(data: Vec<u8>) -> Result<RDB, Box<dyn std::error::Error>> {
    // Check if the data starts with the correct magic string (the first 5 bytes)
    if !data.starts_with(MAGIC_BYTES) {
        return Err(format!("Invalid RDB file: Expected magic bytes {:?}", MAGIC_BYTES).into());
    }

    // Read the next four bytes for the version
    let version = String::from_utf8(data[5..9].to_vec())?;

    // Read the rest of the data
    let data = String::from_utf8_lossy(&data).as_bytes().to_vec();

    Ok(RDB {
        version,
        header: HashMap::new(),
        data,
    })
}

impl RDB {}

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
        println!("Headers: {:?}", rdb.header);
        println!("Data: {:?}", String::from_utf8_lossy(&rdb.data));
    }
}
