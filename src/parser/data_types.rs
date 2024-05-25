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
#[derive(Debug, PartialEq)]
pub enum RESPData {
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
    Array(Vec<RESPData>),

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
}
