// Library
use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::SystemTime;
use tokio::io::AsyncReadExt;

/// The magic bytes at the start of an RDB file
pub const MAGIC_BYTES: &[u8; 5] = b"REDIS";

/// Contents of an empty RDB file in base64 encoding
pub const EMPTY_RDB: &str = "UkVESVMwMDEx+glyZWRpcy12ZXIFNy4yLjD6CnJlZGlzLWJpdHPAQPoFY3RpbWXCbQi8ZfoIdXNlZC1tZW3CsMQQAPoIYW9mLWJhc2XAAP/wbjv+wP9aog==";

/// Represents the contents of an RDB file
pub struct RDB {
    pub magic_string: String,
    pub version: String,
    pub data: HashMap<String, (String, Option<u128>)>,
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
        let mut cursor = Cursor::new(&data);

        // Check if the data starts with the correct magic string (the first 5 bytes)
        if !data.starts_with(MAGIC_BYTES) {
            return Err(format!("Invalid RDB file: Expected magic bytes {:?}", MAGIC_BYTES).into());
        }

        // Read the first five bytes as the magic string
        let mut buf = [0; 5];
        cursor
            .read_exact(&mut buf)
            .await
            .expect("Failed to read magic string");
        self.magic_string = String::from_utf8(buf.to_vec())?;

        // Read the next four bytes for the version
        let mut buf = [0; 4];
        cursor
            .read_exact(&mut buf)
            .await
            .expect("Failed to read version");
        self.version = String::from_utf8(buf.to_vec())?;

        // Read the rest of the data
        loop {
            let next_byte = cursor.read_u8().await.expect("Failed to read opcode byte");
            // println!("Opcode Byte: {}", next_byte);
            match next_byte {
                0xFA => self
                    .parse_aux(&mut cursor)
                    .await
                    .expect("Failed to parse aux"),
                0xFB => {
                    self.parse_resize_db(&mut cursor)
                        .await
                        .expect("Failed to parse resize db");
                    break;
                }
                0xFE => self
                    .parse_select_db(&mut cursor)
                    .await
                    .expect("Failed to parse select db"),
                0xFF => break, // End of the RDB file
                _ => {
                    panic!("Invalid opcode: {}", next_byte);
                }
            }
        }

        Ok(self)
    }

    async fn parse_aux(
        &self,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Key
        read_encoded_string(cursor)
            .await
            .expect("Failed to read aux key");
        // Value
        read_encoded_string(cursor)
            .await
            .expect("Failed to read aux value");
        Ok(())
    }

    async fn parse_resize_db(
        &mut self,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // We essentially skip over these
        let database_hash_table_size = read_int(cursor).await?;
        let _expiry_hash_table_size = read_int(cursor).await?;
        self.parse_hash_table(database_hash_table_size, cursor)
            .await?;
        Ok(())
    }

    async fn parse_hash_table(
        &mut self,
        size: u32,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Iterate over the hash table for the given size
        for _ in 0..size {
            let value_type = cursor.read_u8().await?;

            let expiry: Option<u128>;
            // println!("ValueType {:b}", value_type);
            match value_type {
                0xFC => {
                    let val = cursor.read_u64_le().await? as u128;
                    expiry = Some(val);
                    cursor.read_u8().await?;
                }
                0xFD => {
                    let val = cursor.read_u32_le().await? as u128;
                    expiry = Some(val * 1000);
                    cursor.read_u8().await?;
                }
                0xFF => break,
                _ => expiry = None,
            }

            let key = read_encoded_string(cursor).await?;
            let value = read_encoded_string(cursor).await?;

            println!(
                "\u{001b}[31mKey: {:?}, Value: {:?}, Expiry: {:?} (vs {})\u{001b}[0m",
                key,
                value,
                expiry,
                get_time()
            );

            // If the key is already expired, skip it
            if !expiry.is_none() && expiry.unwrap() < get_time() {
                continue;
            }

            // Insert the key-value pair into the data
            self.data.insert(key, (value, expiry));
        }

        Ok(())
    }
    async fn parse_select_db(
        &self,
        cursor: &mut Cursor<&Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // DB NUMBER
        cursor.read_u8().await?; // We essentially skip over this
        Ok(())
    }
}

fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get time")
        .as_millis()
}

// -------
// HELPERS
// -------

async fn read_int(cursor: &mut Cursor<&Vec<u8>>) -> Result<u32, Box<dyn std::error::Error>> {
    let n = read_length_encoding(cursor).await?;
    return Ok(n.0);
}

async fn read_length_encoding(
    cursor: &mut Cursor<&Vec<u8>>,
) -> Result<(u32, bool), Box<dyn std::error::Error>> {
    let byte = cursor.read_u8().await?; // Read the first byte
    let two_most_significant_bits = (byte & 0xC0) >> 6; // Get the two most significant bits of the byte

    let mut is_encoded = false;
    let length: u32;
    match two_most_significant_bits {
        0x0 => length = (byte & 0x3F) as u32, // The next 6 bits are the length
        0x02 => {
            // Discard the 6 bits, the next 32 bits (4 bytes) are the length
            length = cursor.read_u32_le().await?;
        }
        0x01 => {
            // Read one additional byte, the combined 14 bits are the length
            let next_byte = cursor.read_u8().await?;
            let other_len = (((byte & 0x3F) << 8) | next_byte) as u32;
            length = other_len;
        }
        _ => {
            is_encoded = true;
            match byte & 0x3F {
                0x00 => length = 1,
                0x01 => length = 2,
                0x02 => length = 4,
                _ => {
                    panic!(
                        "not supported special length encoding {}: {}",
                        (byte & 0xC0) >> 6,
                        byte & 0x3F
                    )
                }
            }
        }
    };

    Ok((length, is_encoded))
}

async fn read_encoded_string(
    cursor: &mut Cursor<&Vec<u8>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let length = read_length_encoding(cursor)
        .await
        .expect("Failed to read length");
    // println!("Length: {:?}", length);
    let str = match length {
        (len, false) => {
            // Not encoded, read the string as is
            let mut buf = vec![0u8; len as usize];
            cursor
                .read_exact(&mut buf)
                .await
                .expect("Failed to read string");
            String::from_utf8_lossy(&buf).to_string()
        }
        (len, true) => {
            // Encoded, read the string as base64
            let mut buf = vec![0u8; len as usize];
            cursor
                .read_exact(&mut buf)
                .await
                .expect("Failed to read string");

            let res = match len {
                1 => buf[0] as i8 as i32,
                2 => LittleEndian::read_i16(&buf) as i32,
                4 => LittleEndian::read_i32(&buf),
                _ => panic!("Invalid length for encoded string: {}", len),
            };

            res.to_string()
        }
    };

    println!("Len: {:?}, String: {:?}", length, str);

    Ok(str)
}

// -----
// TESTS
// -----

// #[cfg(test)]
// mod tests {
//     use super::*;

// #[tokio::test]
// async fn test_rdb_default() {
//     let bytes = helpers::base64_to_bytes(EMPTY_RDB);
//     let rdb = parse(bytes).await.unwrap();
//     assert_eq!(rdb.version, "0011");
// }

// #[tokio::test]
// async fn test_rdb() {
//     let bytes = [
//         82, 69, 68, 73, 83, 48, 48, 48, 51, 250, 9, 114, 101, 100, 105, 115, 45, 118, 101, 114,
//         5, 55, 46, 50, 46, 48, 250, 10, 114, 101, 100, 105, 115, 45, 98, 105, 116, 115, 192,
//         64, 254, 0, 251, 3, 3, 252, 0, 156, 239, 18, 126, 1, 0, 0, 0, 9, 98, 108, 117, 101, 98,
//         101, 114, 114, 121, 4, 112, 101, 97, 114, 252, 0, 12, 40, 138, 199, 1, 0, 0, 0, 4, 112,
//         101, 97, 114, 9, 112, 105, 110, 101, 97, 112, 112, 108, 101, 252, 0, 12, 40, 138, 199,
//         1, 0, 0, 0, 5, 103, 114, 97, 112, 101, 9, 98, 108, 117, 101, 98, 101, 114, 114, 121,
//         255, 76, 205, 60, 203, 238, 60, 229, 217, 10,
//     ];
//     let rdb = parse(bytes.to_vec()).await.unwrap();
//     assert_eq!(rdb.version, "0003");
//     assert_eq!(rdb.data.len(), 1); // Only one key-value pair for now
// }
// }

// TEST CONTENTS
// [82, 69, 68, 73, 83, 48, 48, 48, 51, 250, 9, 114, 101, 100, 105, 115, 45, 118, 101, 114, 5, 55, 46, 50, 46, 48, 250, 10, 114, 101, 100, 105, 115, 45, 98, 105, 116, 115, 192, 64, 254, 0, 251, 1, 0, 0, 4, 112, 101, 97, 114, 5, 97, 112, 112, 108, 101, 255, 98, 13, 59, 53, 179, 65, 228, 176, 10]
