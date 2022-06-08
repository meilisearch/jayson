mod impls;
#[cfg(feature = "serde_json")]
mod serde_json;

pub use jayson_internal::DeserializeFromValue;

use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Null,
    Boolean,
    Integer,
    NegativeInteger,
    Float,
    String,
    Sequence,
    Map,
}

pub trait Value: Sized {
    type Sequence: Sequence<Value = Self>;
    type Map: Map<Value = Self>;

    fn kind(&self) -> ValueKind;

    fn is_null(&self) -> bool;
    fn as_boolean(self) -> Option<bool>;
    fn as_integer(self) -> Option<u64>;
    fn as_negative_integer(self) -> Option<i64>;
    fn as_float(self) -> Option<f64>;
    fn as_string(self) -> Option<String>;
    fn as_sequence(self) -> Option<Self::Sequence>;
    fn as_map(self) -> Option<Self::Map>;
}

pub trait Sequence {
    type Value: Value;
    type Iter: Iterator<Item = Self::Value>;

    fn len(&self) -> usize;
    fn into_iter(self) -> Self::Iter;
}

pub trait Map {
    type Value: Value;
    type Iter: Iterator<Item = (String, Self::Value)>;

    fn len(&self) -> usize;

    fn remove(&mut self, key: &str) -> Option<Self::Value>;

    fn into_iter(self) -> Self::Iter;
}

pub trait DeserializeFromValue<E: DeserializeError>: Sized {
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E>;

    fn default() -> Option<Self> {
        None
    }
}

pub trait DeserializeError {
    fn incorrect_value_kind(actual: ValueKind, accepted: &[ValueKind]) -> Self;
    fn missing_field(field: &str) -> Self;
    fn unexpected(msg: &str) -> Self;
}

#[derive(Debug)]
pub enum Error {
    IncorrectValueKind {
        actual: ValueKind,
        accepted: Vec<ValueKind>,
    },
    Unexpected(String),
    MissingField(String),
}
impl DeserializeError for Error {
    fn unexpected(s: &str) -> Self {
        Self::Unexpected(s.to_owned())
    }

    fn missing_field(field: &str) -> Self {
        Self::MissingField(field.to_owned())
    }

    fn incorrect_value_kind(actual: ValueKind, accepted: &[ValueKind]) -> Self {
        Self::IncorrectValueKind {
            actual,
            accepted: accepted.into_iter().copied().collect(),
        }
    }
}
