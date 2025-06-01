use napi::JsUnknown;
use napi::{JsBoolean, JsObject, bindgen_prelude::*};
use napi_derive::napi;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Mutex;

#[napi]
pub struct DataStore {
    db: Mutex<HashMap<String, JsonValue>>,
}

#[napi(object)]
#[derive(Debug, Serialize)]
pub struct QueryResultItem {
    pub key: String,
    pub value: JsonValue,
}

#[napi(object)]
#[derive(Debug, Serialize)]
pub struct FindResults {
    pub documents: Vec<JsonValue>,
    pub total_count: u32,
}

#[napi]
impl DataStore {
    #[napi(constructor)]
    pub fn new() -> Self {
        DataStore {
            db: Mutex::new(HashMap::new()),
        }
    }

    #[napi]
    pub fn insert(&self, documents: Vec<JsonValue>) -> Result<()> {
        let mut db_guard = self.db.lock().unwrap();

        for document in documents {
            if let Some(key) = document.get("id").and_then(|v| v.as_str()) {
                if db_guard.contains_key(key) {
                    return Err(Error::new(
                        Status::GenericFailure,
                        format!("Document with key '{}' already exists.", key),
                    ));
                }
                db_guard.insert(key.to_string(), document);
            } else {
                let key = uuid::Uuid::new_v4().to_string();
                db_guard.insert(key, document);
            }
        }
        Ok(())
    }

    #[napi(ts_args_type = "query: { where?: any}")]
    pub fn delete(&self, env: Env, query: JsObject) -> Result<()> {
        let where_clause: Option<JsonValue> =
            if let Ok(js_value) = query.get_named_property::<JsUnknown>("where") {
                let value_type = js_value.get_type()?;
                if value_type == ValueType::Undefined || value_type == ValueType::Null {
                    None
                } else {
                    Some(env.from_js_value(js_value)?)
                }
            } else {
                None
            };

        let mut keys_to_delete: Vec<String> = Vec::new();
        {
            let db_guard = self.db.lock().unwrap();
            if let Some(ref wc) = where_clause {
                for (key, value) in db_guard.iter() {
                    if matches_where_clause(value, wc) {
                        keys_to_delete.push(key.clone());
                    }
                }
            }
        }

        if keys_to_delete.is_empty() {
            return Ok(());
        }

        let mut db_guard = self.db.lock().unwrap();
        for key in keys_to_delete {
            db_guard.remove(&key);
        }
        Ok(())
    }

    #[napi(ts_args_type = "query: { where?: any, sort?: string, page?: number, size?: number }")] // Add #[napi] here
    pub fn find_documents(&self, env: Env, query: JsObject) -> Result<FindResults> {
        let db_guard = self.db.lock().unwrap();

        let where_clause: Option<JsonValue> =
            if let Ok(js_value) = query.get_named_property::<JsUnknown>("where") {
                let value_type = js_value.get_type()?;
                if value_type == ValueType::Undefined || value_type == ValueType::Null {
                    None
                } else {
                    Some(env.from_js_value(js_value)?)
                }
            } else {
                None
            };

        let sort_field: Option<String> = query.get_named_property("sort")?;
        let page: Option<u32> = query.get_named_property("page")?;
        let size: Option<u32> = query.get_named_property("size")?;

        let mut intermediate_results: Vec<QueryResultItem> = Vec::new();

        for (key, value) in db_guard.iter() {
            if let Some(ref wc) = where_clause {
                if matches_where_clause(value, wc) {
                    intermediate_results.push(QueryResultItem {
                        key: key.clone(),
                        value: value.clone(),
                    });
                }
            } else {
                intermediate_results.push(QueryResultItem {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
        }

        let total_count = intermediate_results.len() as u32;

        if let Some(sort_field_name) = sort_field {
            intermediate_results.sort_by(|a, b| {
                let a_val = a.value.get(&sort_field_name).unwrap_or(&JsonValue::Null);
                let b_val = b.value.get(&sort_field_name).unwrap_or(&JsonValue::Null);
                a_val.to_string().cmp(&b_val.to_string())
            });
        }

        let mut paginated_values: Vec<JsonValue> = intermediate_results
            .into_iter()
            .map(|item| item.value)
            .collect();

        if let (Some(p), Some(s)) = (page, size) {
            if s > 0 {
                let start = (p * s) as usize;
                paginated_values = paginated_values
                    .into_iter()
                    .skip(start)
                    .take(s as usize)
                    .collect();
            }
        }

        Ok(FindResults {
            documents: paginated_values,
            total_count,
        })
    }

    #[napi]
    pub fn clear(&self) -> Result<()> {
        let mut db_guard = self.db.lock().unwrap();
        db_guard.clear();
        Ok(())
    }

    #[napi]
    pub fn size(&self) -> Result<u32> {
        let db_guard = self.db.lock().unwrap();
        Ok(db_guard.len() as u32)
    }
}

fn matches_where_clause(document: &JsonValue, where_clause: &JsonValue) -> bool {
    if let Some(where_obj) = where_clause.as_object() {
        if where_obj.is_empty() {
            // If where clause is an empty object, match all
            return true;
        }
        for (key, value) in where_obj {
            if document.get(key) != Some(value) {
                return false;
            }
        }
        true
    } else {
        // If where_clause is not an object (e.g. null, string, array), it doesn't match
        false
    }
}
