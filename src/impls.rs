use crate::{DeserializeError, DeserializeFromValue, Map, Sequence, Value, ValueKind};
use std::{
    collections::{BTreeMap, HashMap},
    convert::TryFrom,
    hash::Hash,
    str::FromStr,
};

impl<T> Sequence for Vec<T>
where
    T: Value,
{
    type Value = T;
    type Iter = <Self as IntoIterator>::IntoIter;

    fn len(&self) -> usize {
        self.len()
    }

    fn into_iter(self) -> Self::Iter {
        <Self as IntoIterator>::into_iter(self)
    }
}

impl<E: DeserializeError> DeserializeFromValue<E> for () {
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        if value.is_null() {
            Ok(())
        } else {
            Err(E::incorrect_value_kind(value.kind(), &[ValueKind::Null]))
        }
    }
}

impl<E: DeserializeError> DeserializeFromValue<E> for bool {
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        value
            .as_boolean()
            .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::Boolean]))
    }
}

macro_rules! deserialize_impl_integer {
    ($t:ty) => {
        impl<E: DeserializeError> DeserializeFromValue<E> for $t {
            fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
                let kind = value.kind();
                value
                    .as_integer()
                    .and_then(|x| <$t>::try_from(x).ok())
                    .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::Integer]))
            }
        }
    };
}
deserialize_impl_integer!(u8);
deserialize_impl_integer!(u16);
deserialize_impl_integer!(u32);
deserialize_impl_integer!(u64);
deserialize_impl_integer!(usize);

macro_rules! deserialize_impl_negative_integer {
    ($t:ty) => {
        impl<E: DeserializeError> DeserializeFromValue<E> for $t {
            fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
                let kind = value.kind();
                match kind {
                    ValueKind::Integer => {
                        let x = value.as_integer().unwrap();
                        return <$t>::try_from(x).map_err(|_| E::unexpected("todo"));
                    }
                    ValueKind::NegativeInteger => {
                        let x = value.as_negative_integer().unwrap();
                        return <$t>::try_from(x).map_err(|_| E::unexpected("todo"));
                    }
                    _ => {
                        return Err(E::incorrect_value_kind(
                            kind,
                            &[ValueKind::Integer, ValueKind::NegativeInteger],
                        ))
                    }
                };
            }
        }
    };
}

deserialize_impl_negative_integer!(i8);
deserialize_impl_negative_integer!(i16);
deserialize_impl_negative_integer!(i32);
deserialize_impl_negative_integer!(i64);
deserialize_impl_negative_integer!(isize);

macro_rules! deserialize_impl_float {
    ($t:ty) => {
        impl<E: DeserializeError> DeserializeFromValue<E> for $t {
            fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
                let kind = value.kind();
                match kind {
                    ValueKind::Integer => {
                        let x = value.as_integer().unwrap();
                        return Ok(x as $t);
                    }
                    ValueKind::NegativeInteger => {
                        let x = value.as_negative_integer().unwrap();
                        return Ok(x as $t);
                    }
                    ValueKind::Float => {
                        let x = value.as_float().unwrap();
                        return Ok(x as $t);
                    }
                    _ => {
                        return Err(E::incorrect_value_kind(
                            kind,
                            &[
                                ValueKind::Float,
                                ValueKind::Integer,
                                ValueKind::NegativeInteger,
                            ],
                        ))
                    }
                };
            }
        }
    };
}
deserialize_impl_float!(f32);
deserialize_impl_float!(f64);

impl<E: DeserializeError> DeserializeFromValue<E> for String {
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        value
            .as_string()
            .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::String]))
    }
}

impl<T, E: DeserializeError> DeserializeFromValue<E> for Vec<T>
where
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        if let Some(seq) = value.as_sequence() {
            let mut result = Vec::with_capacity(seq.len());
            for x in seq.into_iter() {
                let x = T::deserialize_from_value(x)?;
                result.push(x);
            }
            Ok(result)
        } else {
            Err(E::incorrect_value_kind(kind, &[ValueKind::Sequence]))
        }
    }
}

impl<T, E: DeserializeError> DeserializeFromValue<E> for Option<T>
where
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        match value.kind() {
            ValueKind::Null => Ok(None),
            _ => T::deserialize_from_value(value).map(Some),
        }
    }
    fn default() -> Option<Self> {
        Some(None)
    }
}

impl<T, E: DeserializeError> DeserializeFromValue<E> for Box<T>
where
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        T::deserialize_from_value(value).map(Box::new)
    }
}

impl<Key, T, E: DeserializeError> DeserializeFromValue<E> for HashMap<Key, T>
where
    Key: FromStr + Hash + Eq,
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        let map = value
            .as_map()
            .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::Map]))?;

        let mut res = HashMap::with_capacity(map.len());
        for (key, value) in map.into_iter() {
            let key = Key::from_str(&key).map_err(|_| E::unexpected("todo"))?;
            let value = T::deserialize_from_value(value)?;
            res.insert(key, value);
        }
        Ok(res)
    }
}

impl<Key, T, E: DeserializeError> DeserializeFromValue<E> for BTreeMap<Key, T>
where
    Key: FromStr + Ord,
    T: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        let map = value
            .as_map()
            .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::Map]))?;

        let mut res = BTreeMap::new();
        for (key, value) in map.into_iter() {
            let key = Key::from_str(&key).map_err(|_| E::unexpected("todo"))?;
            let value = T::deserialize_from_value(value)?;
            res.insert(key, value);
        }
        Ok(res)
    }
}

impl<A, B, E: DeserializeError> DeserializeFromValue<E> for (A, B)
where
    A: DeserializeFromValue<E>,
    B: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        let seq = value
            .as_sequence()
            .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::Sequence]))?;

        let len = seq.len();
        if len < 2 {
            return Err(E::unexpected("todo"));
        }
        if len > 2 {
            return Err(E::unexpected("todo"));
        }
        let mut iter = seq.into_iter();

        let a = A::deserialize_from_value(iter.next().unwrap())?;
        let b = B::deserialize_from_value(iter.next().unwrap())?;

        Ok((a, b))
    }
}

impl<A, B, C, E: DeserializeError> DeserializeFromValue<E> for (A, B, C)
where
    A: DeserializeFromValue<E>,
    B: DeserializeFromValue<E>,
    C: DeserializeFromValue<E>,
{
    fn deserialize_from_value<V: Value>(value: V) -> Result<Self, E> {
        let kind = value.kind();
        let seq = value
            .as_sequence()
            .ok_or_else(|| E::incorrect_value_kind(kind, &[ValueKind::Sequence]))?;

        let len = seq.len();
        if len < 3 {
            return Err(E::unexpected("todo"));
        }
        if len > 3 {
            return Err(E::unexpected("todo"));
        }
        let mut iter = seq.into_iter();

        let a = A::deserialize_from_value(iter.next().unwrap())?;
        let b = B::deserialize_from_value(iter.next().unwrap())?;
        let c = C::deserialize_from_value(iter.next().unwrap())?;

        Ok((a, b, c))
    }
}
