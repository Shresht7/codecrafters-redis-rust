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
}
