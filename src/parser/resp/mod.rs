// Modules
pub(crate) mod array;
pub(crate) mod big_number;
pub(crate) mod boolean;
pub(crate) mod bulk_error;
pub(crate) mod bulk_string;
pub(crate) mod double;
pub(crate) mod integer;
pub(crate) mod map;
pub(crate) mod null;
pub(crate) mod set;
pub(crate) mod simple_error;
pub(crate) mod simple_string;
pub(crate) mod verbatim_string;

// Exports
pub(crate) mod types;
pub use types::Type;

// ----------------
// HELPER FUNCTIONS
// ----------------

/// Creates a new RESP array with the given elements
pub fn array(elements: Vec<Type>) -> Type {
    Type::Array(elements)
}

/// Creates a new RESP bulk string with the given value
pub fn bulk_string(value: &str) -> Type {
    Type::BulkString(value.into())
}
