// Library
use crate::parser::resp::Type;
use std::collections::HashMap;

pub struct Database {
    data: HashMap<Type, Type>,
}

impl Database {
    /// Creates a new instance of the database.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Sets the value of a key in the database.
    pub fn set(&mut self, key: Type, value: Type) {
        self.data.insert(key, value);
    }

    /// Gets the value of a key in the database.
    pub fn get(&self, key: &Type) -> Option<&Type> {
        self.data.get(key)
    }

    /// Removes a key from the database.
    pub fn remove(&mut self, key: &Type) {
        self.data.remove(key);
    }
}
