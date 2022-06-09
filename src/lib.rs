#![allow(clippy::len_without_is_empty)]
mod impls;
#[cfg(feature = "serde_json")]
mod serde_json;

pub use jayson_internal::DeserializeFromValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug)]
pub enum Value<V: IntoValue> {
    Null,
    Boolean(bool),
    Integer(u64),
    NegativeInteger(i64),
    Float(f64),
    String(String),
    Sequence(V::Sequence),
    Map(V::Map),
}

pub trait IntoValue: Sized {
    type Sequence: Sequence<Value = Self>;
    type Map: Map<Value = Self>;

    fn kind(&self) -> ValueKind;

    fn into_value(self) -> Value<Self>;
}

pub trait Sequence {
    type Value: IntoValue;
    type Iter: Iterator<Item = Self::Value>;

    fn len(&self) -> usize;
    fn into_iter(self) -> Self::Iter;
}

pub trait Map {
    type Value: IntoValue;
    type Iter: Iterator<Item = (String, Self::Value)>;

    fn len(&self) -> usize;
    fn remove(&mut self, key: &str) -> Option<Self::Value>;
    fn into_iter(self) -> Self::Iter;
}

pub trait DeserializeFromValue<E: DeserializeError>: Sized {
    fn deserialize_from_value<V: IntoValue>(value: Value<V>) -> Result<Self, E>;

    fn default() -> Option<Self> {
        None
    }
}

pub trait DeserializeError {
    fn incorrect_value_kind(accepted: &[ValueKind]) -> Self;
    fn missing_field(field: &str) -> Self;
    fn unexpected(msg: &str) -> Self;
}

#[derive(Debug)]
pub enum Error {
    IncorrectValueKind { accepted: Vec<ValueKind> },
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

    fn incorrect_value_kind(accepted: &[ValueKind]) -> Self {
        Self::IncorrectValueKind {
            accepted: accepted.to_vec(),
        }
    }
}
