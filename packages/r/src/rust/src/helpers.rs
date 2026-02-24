//! R <-> Rust type conversion utilities

use extendr_api::prelude::*;
use serde_json::Value;

/// Convert a serde_json::Value to an R object
pub fn json_to_robj(value: &Value) -> Robj {
    match value {
        Value::Null => ().into(),
        Value::Bool(b) => b.into_robj(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                (i as i32).into_robj()
            } else if let Some(f) = n.as_f64() {
                f.into_robj()
            } else {
                ().into()
            }
        }
        Value::String(s) => s.as_str().into_robj(),
        Value::Array(arr) => {
            let items: Vec<Robj> = arr.iter().map(json_to_robj).collect();
            List::from_values(items).into_robj()
        }
        Value::Object(map) => {
            let names: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            let values: Vec<Robj> = map.values().map(json_to_robj).collect();
            let list = List::from_names_and_values(names, values).unwrap();
            list.into_robj()
        }
    }
}

/// Convert an R object to serde_json::Value
pub fn robj_to_json(robj: &Robj) -> Value {
    if robj.is_null() {
        Value::Null
    } else if let Some(b) = robj.as_bool() {
        Value::Bool(b)
    } else if let Some(i) = robj.as_integer() {
        Value::Number(serde_json::Number::from(i))
    } else if let Some(f) = robj.as_real() {
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null)
    } else if let Some(s) = robj.as_str() {
        Value::String(s.to_string())
    } else if robj.is_list() {
        if let Ok(list) = List::try_from(robj.clone()) {
            let names = list.names();
            if let Some(names) = names {
                let mut map = serde_json::Map::new();
                for (name, value) in names.zip(list.values()) {
                    map.insert(name.to_string(), robj_to_json(&value));
                }
                Value::Object(map)
            } else {
                let arr: Vec<Value> = list.values().map(|v| robj_to_json(&v)).collect();
                Value::Array(arr)
            }
        } else {
            Value::Null
        }
    } else {
        Value::Null
    }
}
