use napi::bindgen_prelude::*;
use napi_derive::napi;
use napi::Result; // Ensure this is napi::Result, not napi::bindgen_prelude::Result
use std::collections::HashMap;
use std::sync::Mutex;
use serde_json::Value as JsonValue; // Alias for convenience


#[napi(js_name = "DataStore")] // This will expose the Rust struct as a JS class named "DataStore"
pub struct DataStore {
    // Each DataStore instance will have its own private database.
    // The Mutex guards access to this specific instance's HashMap.
    db: Mutex<HashMap<String, JsonValue>>,
}

#[napi]
impl DataStore {
    #[napi(constructor)] // This makes it callable as `new DataStore()` from JS
    pub fn new() -> Self {
        DataStore {
            db: Mutex::new(HashMap::new()),
        }
    }

    /// Sets a key-value pair in this DataStore instance.
    /// The value can be any valid JSON structure (object, array, string, number, boolean, null).
    #[napi]
    pub fn set_item(&self, key: String, value: JsonValue) -> Result<()> {
        let mut db_guard = self.db.lock().unwrap(); // Lock this instance's db
        db_guard.insert(key, value);
        Ok(())
    }

    /// Gets a value by key from this DataStore instance.
    /// Returns the value as a JSON structure.
    #[napi]
    pub fn get_item(&self, key: String) -> Result<Option<JsonValue>> {
        let db_guard = self.db.lock().unwrap(); // Lock this instance's db
        // .cloned() is important here as get returns a reference,
        // and we need an owned JsonValue to return.
        Ok(db_guard.get(&key).cloned())
    }

    /// Removes an item by key from this DataStore instance.
    #[napi]
    pub fn remove_item(&self, key: String) -> Result<Option<JsonValue>> {
        let mut db_guard = self.db.lock().unwrap();
        Ok(db_guard.remove(&key))
    }

    /// Clears all items from this DataStore instance.
    #[napi]
    pub fn clear(&self) -> Result<()> {
        let mut db_guard = self.db.lock().unwrap();
        db_guard.clear();
        Ok(())
    }

    /// Returns the number of items in this DataStore instance.
    #[napi]
    pub fn size(&self) -> Result<u32> {
        let db_guard = self.db.lock().unwrap();
        Ok(db_guard.len() as u32)
    }
}