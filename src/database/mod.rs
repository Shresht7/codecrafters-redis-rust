use tokio::fs;

// Library
use crate::parser::resp::Type;
use std::{collections::HashMap, time::Instant};

// Modules
mod opcode;
pub mod rdb;

/// Struct to hold the value and metadata of a database item.
#[derive(Clone, Debug)]
pub struct Item {
    /// The actual value of the item
    value: Type,
    /// The instant at which the database item was created
    created_at: Instant,
    /// The number of milliseconds since creation after which the item expires
    expires_at: Option<usize>,
}

/// Database struct to store key-value pairs.
#[derive(Clone)]
pub struct Database {
    /// The actual data store
    data: HashMap<Type, Item>,

    /// The directory where the database is stored
    pub dir: String,

    /// The name of the RDB file
    pub dbfilename: String,
}

/// Creates a new instance of the database.
pub fn new() -> Database {
    Database {
        data: HashMap::new(),
        dir: String::from(""),
        dbfilename: String::from(""),
    }
}

impl Database {
    /// Sets the value of a key in the database.
    pub fn set(&mut self, key: Type, value: Type, expires_at: Option<usize>) {
        self.data.insert(
            key,
            Item {
                value,
                created_at: Instant::now(),
                expires_at,
            },
        );
    }

    /// Gets the value of a key in the database.
    pub fn get(&self, key: &Type) -> Option<&Type> {
        let item = self.data.get(key)?;
        if item.expires_at.is_some() {
            if item.created_at.elapsed().as_millis() as usize >= item.expires_at? {
                return None;
            }
        }
        Some(&item.value)
    }

    // /// Removes a key from the database.
    // pub fn remove(&mut self, key: &Type) {
    //     self.data.remove(key);
    // }

    pub async fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let filepath = format!("{}/{}", self.dir, self.dbfilename);
        let contents = fs::read(filepath).await?;
        let rdb = rdb::parse(contents).await?;
        for ele in rdb.data {
            self.set(Type::BulkString(ele.0), Type::BulkString(ele.1), None);
        }
        Ok(())
    }

    pub fn keys(&self) -> Vec<&Type> {
        self.data.keys().collect()
    }
}
