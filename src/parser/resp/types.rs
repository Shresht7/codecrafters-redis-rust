// Library
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

// ---------------------------------------
// REDIS SERIALIZATION PROTOCOL DATA TYPES
// ---------------------------------------

/// REDIS SERIALIZATION PROTOCOL
///
/// > Source: https://redis.io/topics/protocol
///
/// Redis Serialization Protocol (RESP) is the protocol used in Redis to
/// serialize the data exchanged between the server and the client.
///
/// RESP is a compromise between the following things:
/// - Simple and Fast to parse.
/// - Human readable and writable.
/// - Easy to implement.
///
/// RESP can serialize different data types including strings, integers, arrays, etc.
///
///
/// | RESP data type    | MPV    | Category   | First byte |
/// |-------------------|--------|------------|------------|
/// | Simple strings    | RESP2  | Simple     | `+`        |
/// | Simple Errors     | RESP2  | Simple     | `-`        |
/// | Integers          | RESP2  | Simple     | `:`        |
/// | Bulk strings      | RESP2  | Aggregate  | `$`        |
/// | Arrays            | RESP2  | Aggregate  | `*`        |
/// | Nulls             | RESP3  | Simple     | `_`        |
/// | Booleans          | RESP3  | Simple     | `#`        |
/// | Doubles           | RESP3  | Simple     | `,`        |
/// | Big numbers       | RESP3  | Simple     | `(`        |
/// | Bulk errors       | RESP3  | Aggregate  | `!`        |
/// | Verbatim strings  | RESP3  | Aggregate  | `=`        |
/// | Maps              | RESP3  | Aggregate  | `%`        |
/// | Sets              | RESP3  | Aggregate  | `~`        |
/// | Pushes            | RESP3  | Aggregate  | `>`        |
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Simple Strings are encoded with a leading `+` character followed by the string itself.
    /// The string is terminated by the CRLF sequence (it mustn't contain CRLF or LF characters).
    /// Simple Strings are meant for short, non-binary strings with minimal overhead.
    ///
    /// Example:
    /// ```sh
    /// +OK\r\n
    /// ```
    SimpleString(String),

    /// RESP has a specific data type for errors. _Simple Errors_ (or just _errors_) are similar to
    /// _Simple Strings_, but their first character is a minus `-` symbol.
    /// The difference between them is that clients **should** treat errors as **exceptions** and the
    /// string encoded in the error-type is the error message itself.
    ///
    /// Example:
    /// ```sh
    /// -ERR unknown command 'foobar'\r\n
    /// ```
    ///
    /// The first _uppercase_ word in the error is a generalized error category (called error prefix),
    /// which represents the kind of error returned. Note that this is a convention
    /// used by redis and not a formal part of the protocol.
    SimpleError(String),

    /// This type is a CRLF-terminated string that represents a signed, base-10, 64-bit integer.
    /// The first byte of the string is a colon `:` character, followed by the integer itself.
    /// The integer is terminated by the CRLF sequence.
    ///
    /// Example:
    /// ```sh
    /// :1000\r\n
    /// :-25\r\n
    /// :+25\r\n
    /// :0\r\n
    /// ```
    Integer(i64),

    /// A *Bulk String* represents a single binary string. The string can be of any size, but by default
    /// Redis limits it to 512MB.
    /// A bulk string is a sequence of bytes with a length of `N` bytes followed by `N` bytes of data.
    /// The length of the bulk string is encoded as a decimal number followed by a CRLF sequence.
    /// The bulk string is terminated by a CRLF sequence.
    ///
    /// If the length is -1, the bulk string is null.
    ///
    /// Example:
    /// ```sh
    /// $6\r\nfoobar\r\n # $<length>\r\n<data>\r\n
    /// $0\r\n\r\n # Empty string
    /// ```
    ///
    /// While RESP3 has a new data type for _Null_, RESP2 uses _Bulk Strings_ to represent null values.
    /// A _Bulk String_ with a length of `-1` represents a null value.
    ///
    /// Example:
    /// ```sh
    /// $-1\r\n
    /// ```
    BulkString(String),

    /// An **Array** is a sequence of RESP values. The first byte of the _Array_ is the asterisk `*` character,
    /// followed by the _number of elements_ in the array, and the CRLF sequence.
    /// Each element in the array is encoded according to the rules of the RESP protocol.
    /// The array is terminated by a CRLF sequence.
    ///
    /// An empty array is represented as `*0\r\n`.
    ///
    /// Example:
    /// ```sh
    /// *3\r\n:1\r\n:2\r\n:3\r\n => [1, 2, 3]
    /// ```
    ///
    /// Clients send commands to the Redis server as RESP arrays. Similarly
    /// some Redis commands that return a collection of elements use arrays as their replies.
    Array(Vec<Type>),

    /// A _Null_ value is a simple data type that represents a null value.
    /// This can be used in bulk strings, arrays, etc.
    /// The first byte of a _Null_ value is the underscore `_` character.
    /// A _Null_ value is terminated by the CRLF sequence.
    ///
    /// Example:
    /// ```sh
    /// _\r\n
    /// ```
    Null,

    /// A _Boolean_ value is a simple data type that represents a boolean value.
    /// A boolean value is represented by the hash `#` character
    /// followed by `t` or `f` for `true` or `false` respectively
    /// and is terminated by the CRLF sequence.
    ///
    /// Example:
    /// ```sh
    /// #t\r\n // true
    /// #f\r\n // false
    /// ```
    Boolean(bool),

    /// A *Double* value is a simple data type that represents a double-precision floating-point number.
    /// A double value is represented by the comma `,` character followed by the double value itself.
    /// The double value is terminated by the CRLF sequence.
    ///
    /// The doubles contain an optional sign, integer part, fractional part, and an optional exponent part.
    ///
    /// Example:
    /// ```sh
    /// ,3.14\r\n // 3.14
    /// ,-3.14e-2\r\n // -0.0314
    /// ```
    ///
    /// Because the fractional part is optional, integers can be encoded as doubles. In such cases,
    /// the Redis client should return native integer and double values.
    ///
    /// The positive infinity, negative infinity, and NaN are encoded as:
    /// ```sh
    /// ,inf\r\n // +inf
    /// ,-inf\r\n // -inf
    /// ,nan\r\n // NaN
    /// ```
    Double(f64),

    /// A *Big Number* is a simple data type that represents a big number.
    /// A big number is represented by the left parenthesis `(` character followed by the big number itself.
    /// The big number is terminated by the CRLF sequence.
    /// A big number is a signed, base-10, 64-bit integer.
    /// The big number can be positive or negative.
    /// The big number is used to represent large integers that can't be represented by the integer data type.
    ///
    /// Example:
    /// ```sh
    /// (1234567890\r\n // 1234567890
    /// (-1234567890\r\n // -1234567890
    /// ```
    BigNumber(i64),

    /// A *Bulk Error* is a data type that represents an error message.
    /// A bulk error is encoded as follows:
    /// - A prefix of `!`
    /// - One or more decimal for the error's length
    /// - CRLF terminator sequence
    /// - The error message
    /// - A final CRLF terminator sequence
    ///
    /// Example:
    /// ```sh
    /// !13\r\nError message\r\n => "Error message"
    /// ```
    ///
    /// As a convention the error begins with an uppercase word denoting the error type.
    BulkError(String),

    /// A *Verbatim String* is a data type similar to bulk string but with the addition of a hint about the data's encoding.
    /// A verbatim string is encoded as follows:
    /// - A prefix of `=`
    /// - One or more decimal for the string's length
    /// - CRLF terminator sequence
    /// - Exactly 3 bytes representing the data's encoding
    /// - The colon `:` character to separate the encoding from the data
    /// - The string data
    /// - A final CRLF terminator sequence
    ///
    /// Example:
    /// ```sh
    /// =6\r\nutf-8:foobar\r\n => "foobar"
    /// ```
    VerbatimString(String, String),

    /// A *Map* is a data type that represents a collection of key-value pairs.
    /// A map is encoded as follows:
    /// - A prefix of `%`
    /// - The number of key-value pairs in the map
    /// - CRLF terminator sequence
    /// - Each key-value pair is encoded according to the rules of the RESP protocol
    /// - A final CRLF terminator sequence
    ///
    /// Example:
    /// ```sh
    /// %2\r\n+key1\r\n:1\r\n+key2\r\n:2\r\n => {"key1": 1, "key2": 2}
    /// ```
    Map(HashMap<Type, Type>),

    /// A *Set* is a data type that represents a collection of unique elements.
    /// A set is encoded as follows:
    /// - A prefix of `~`
    /// - The number of elements in the set
    /// - CRLF terminator sequence
    /// - Each element in the set is encoded according to the rules of the RESP protocol
    /// - A final CRLF terminator sequence
    ///
    /// Example:
    /// ```sh
    /// ~3\r\n:1\r\n:2\r\n:3\r\n => {1, 2, 3}
    /// ```
    ///
    /// Sets are similar to arrays but with the distinction that sets contain unique elements.
    Set(HashSet<Type>),
    // TODO: Pushes
    /// RDB file format
    /// RDB files are the binary representation of the Redis database.
    /// The RDB file format is used for persistence and backups.
    RDBFile(Vec<u8>),
}

impl Eq for Type {
    fn assert_receiver_is_total_eq(&self) {
        // Implement this method to assert that the receiver is `Eq`
        // This method is called when you use the `assert_eq!` macro
        // to compare two `Type` values
        // You can use this method to perform additional checks
        // before comparing the two values
        // Modify the code below based on the structure of Type
        match self {
            Type::Map(map) => {
                // Check if the map is empty
                if map.is_empty() {
                    panic!("Map is empty");
                }
            }
            Type::Set(set) => {
                // Check if the set is empty
                if set.is_empty() {
                    panic!("Set is empty");
                }
            }
            _ => {}
        }
    }
}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Implement the hash function for Type
        // You can use any logic that uniquely identifies the Type
        // For example, if Type is an enum, you can hash the discriminant value
        // If Type is a struct, you can hash its fields
        // If Type is a primitive type, you can hash its value directly
        // Modify the code below based on the structure of Type
        match self {
            Type::Map(map) => {
                // Hash the map by hashing each key-value pair
                for (key, value) in map {
                    key.hash(state);
                    value.hash(state);
                }
            }
            Type::Set(set) => {
                // Hash the set by hashing each element
                for element in set {
                    element.hash(state);
                }
            }
            _ => {
                // For other types, hash the discriminant value
                std::mem::discriminant(self).hash(state);
            }
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::SimpleString(s) => write!(f, "+{}\r\n", s),

            Type::SimpleError(e) => write!(f, "-{}\r\n", e),

            Type::Integer(i) => write!(f, ":{}\r\n", i),

            Type::BulkString(s) => {
                if s == "" {
                    write!(f, "$-1\r\n")
                } else {
                    write!(f, "${}\r\n{}\r\n", s.len(), s)
                }
            }

            Type::Array(arr) => {
                write!(f, "*{}\r\n", arr.len())?;
                for elem in arr {
                    write!(f, "{}", elem)?;
                }
                Ok(())
            }

            Type::Null => write!(f, "$-1\r\n"),

            Type::Boolean(b) => write!(f, "#{}\r\n", if *b { 't' } else { 'f' }),

            Type::Double(d) => write!(f, ",{}\r\n", d),

            Type::BigNumber(n) => write!(f, "({}\r\n", n),

            Type::BulkError(e) => write!(f, "!{}\r\n", e),

            Type::VerbatimString(e, s) => write!(f, "={}\r\n{}:{}\r\n", s.len(), s, e),

            Type::Map(map) => {
                write!(f, "%{}\r\n", map.len())?;
                for (key, value) in map {
                    write!(f, "{}{}", key, value)?;
                }
                Ok(())
            }

            Type::Set(set) => {
                write!(f, "~{}\r\n", set.len())?;
                for elem in set {
                    write!(f, "{}", elem)?;
                }
                Ok(())
            }

            Type::RDBFile(data) => {
                let len = data.len();
                write!(f, "$({}\r\n{:?}", len, data)
            }
        }
    }
}

impl Type {
    pub fn as_bytes(&self) -> Vec<u8> {
        match &self {
            Type::SimpleString(data) => vec![b'+']
                .into_iter()
                .chain(data.as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),
            Type::SimpleError(data) => vec![b'-']
                .into_iter()
                .chain(data.as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),
            Type::Integer(data) => vec![b':']
                .into_iter()
                .chain(data.to_string().as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),
            Type::BulkString(data) => {
                let mut bytes = vec![b'$']
                    .into_iter()
                    .chain(data.len().to_string().as_bytes().to_vec())
                    .chain(vec![b'\r', b'\n'])
                    .collect::<Vec<u8>>();
                bytes.extend(data.as_bytes());
                bytes.extend(vec![b'\r', b'\n']);
                bytes
            }
            Type::Array(data) => {
                let mut bytes = vec![b'*']
                    .into_iter()
                    .chain(data.len().to_string().as_bytes().to_vec())
                    .chain(vec![b'\r', b'\n'])
                    .collect::<Vec<u8>>();
                for item in data {
                    bytes.extend(item.as_bytes());
                }
                bytes
            }
            Type::Null => vec![b'_', b'\r', b'\n'],

            Type::Boolean(data) => vec![b'#']
                .into_iter()
                .chain(data.to_string().as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),

            Type::Double(data) => vec![b',']
                .into_iter()
                .chain(data.to_string().as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),

            Type::BigNumber(data) => vec![b'(']
                .into_iter()
                .chain(data.to_string().as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),

            Type::BulkError(data) => vec![b'!']
                .into_iter()
                .chain(data.len().to_string().as_bytes().to_vec())
                .chain(vec![b'\r', b'\n'])
                .collect(),

            Type::VerbatimString(encoding, data) => {
                let mut bytes = vec![b'=']
                    .into_iter()
                    .chain(data.len().to_string().as_bytes().to_vec())
                    .chain(vec![b'\r', b'\n'])
                    .collect::<Vec<u8>>();
                bytes.extend(encoding.as_bytes());
                bytes.extend(vec![b':']);
                bytes.extend(data.as_bytes());
                bytes.extend(vec![b'\r', b'\n']);
                bytes
            }

            Type::Map(data) => {
                let mut bytes = vec![b'%']
                    .into_iter()
                    .chain(data.len().to_string().as_bytes().to_vec())
                    .chain(vec![b'\r', b'\n'])
                    .collect::<Vec<u8>>();
                for (key, value) in data {
                    bytes.extend(key.as_bytes());
                    bytes.extend(value.as_bytes());
                }
                bytes
            }

            Type::Set(data) => {
                let mut bytes = vec![b'~']
                    .into_iter()
                    .chain(data.len().to_string().as_bytes().to_vec())
                    .chain(vec![b'\r', b'\n'])
                    .collect::<Vec<u8>>();
                for item in data {
                    bytes.extend(item.as_bytes());
                }
                bytes
            }

            Type::RDBFile(data) => {
                let mut bytes = vec![b'$']
                    .into_iter()
                    .chain(data.len().to_string().as_bytes().to_vec())
                    .chain(vec![b'\r', b'\n'])
                    .collect::<Vec<u8>>();
                bytes.extend(data);
                bytes
            }
        }
    }
}
