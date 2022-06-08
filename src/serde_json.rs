use crate::{Map, Value, ValueKind};
use serde_json::{Map as JMap, Value as JValue};

impl Map for JMap<String, JValue> {
    type Value = JValue;
    type Iter = <Self as IntoIterator>::IntoIter;

    fn len(&self) -> usize {
        self.len()
    }

    fn remove(&mut self, key: &str) -> Option<Self::Value> {
        JMap::remove(self, key)
    }

    fn into_iter(self) -> Self::Iter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl Value for JValue {
    type Sequence = Vec<JValue>;
    type Map = JMap<String, JValue>;

    fn kind(&self) -> ValueKind {
        match self {
            JValue::Null => ValueKind::Null,
            JValue::Bool(_) => ValueKind::Boolean,
            JValue::Number(n) => {
                if n.is_u64() {
                    ValueKind::Integer
                } else if n.is_i64() {
                    ValueKind::NegativeInteger
                } else if n.is_f64() {
                    ValueKind::Float
                } else {
                    panic!();
                }
            }
            JValue::String(_) => ValueKind::String,
            JValue::Array(_) => ValueKind::Sequence,
            JValue::Object(_) => ValueKind::Map,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, JValue::Null)
    }

    fn as_boolean(self) -> Option<bool> {
        self.as_bool()
    }

    fn as_integer(self) -> Option<u64> {
        self.as_u64()
    }

    fn as_negative_integer(self) -> Option<i64> {
        self.as_i64()
    }

    fn as_float(self) -> Option<f64> {
        self.as_f64()
    }

    fn as_string(self) -> Option<String> {
        match self {
            JValue::String(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(self) -> Option<Self::Sequence> {
        match self {
            JValue::Array(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_map(self) -> Option<Self::Map> {
        match self {
            JValue::Object(map) => Some(map),
            _ => None,
        }
    }
}
