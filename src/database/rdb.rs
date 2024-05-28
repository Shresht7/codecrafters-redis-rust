// Library
use base64::{prelude::BASE64_STANDARD, Engine};

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
}
