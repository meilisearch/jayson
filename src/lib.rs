/*!
Jayson is a crate for deserializing data, with the ability to return
custom, type-specific errors upon failure.

Unlike serde, Jayson does not parse the data in its serialization format itself,
but offload that work to other crates. Instead, it deserializes
the already-parsed serialized data into the final type. For example:

```ignore
// bytes of the serialized value
let s: &str = "{ "x": 7 }" ;
// parsed serialized data
let json: serde_json::Value = serde_json::from_str(s)?;
// finally deserialize with Jayson
let data = jayson::deserialize::<T, serde_json::Value, MyError>(json)?;
```

Thus, Jayson is a bit slower than crates that immediately deserialize a value while
parsing at the same time.

The main parts of Jayson are:
1. [`DeserializeFromValue<E>`] is the main trait for deserialization
2. [`IntoValue`] and [`Value`] describe the shape that the parsed serialized data must have
3. [`DeserializeError`] is the trait that all deserialization errors must conform to
4. [`MergeWithError<E>`] describes how to combine multiple errors together. It allows Jayson
to return multiple deserialization errors at once.
5. [`ValuePointerRef`] and [`ValuePointer`] point to locations within the value. They are
used to locate the origin of an error.
6. [`deserialize`] is the main function to use to deserialize a value
7. The [`DeserializeFromValue`](derive@DeserializeFromValue) derive proc macro

If the feature `serde` is activated, then an implementation of [`IntoValue`] is provided
for the type `serde_json::Value`. This allows using Jayson to deserialize from JSON.
*/

#![allow(clippy::len_without_is_empty)]
mod impls;
#[cfg(feature = "serde_json")]
mod serde_json;

/**
It is possible to derive the `DeserializeFromValue` trait for structs and enums with named fields.
The derive proc macro accept many arguments, explained below:

The basic usage is as follows:
```
use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
struct MyStruct {
    x: bool,
    y: u8,
}
```
This will implement `impl<E> DeserializeFromValue<E> MyStruct` for all `E: DeserializeError`.

To use it on enums, the attribute `tag` must be added:
```
use jayson::DeserializeFromValue;

#[derive(DeserializeFromValue)]
#[jayson(tag = "my_enum_tag")]
enum MyEnum {
    A,
    B { x: bool, y: u8 }
}
```
This will correctly deserialize the given enum for values of this shape:
```json
{
    "my_enum_tag": "A"
}
// or
{
    "my_enum_tag": "B",
    "x": true,
    "y": 1
}
```

It is possible to change the name of the keys corresponding to each field using the `rename` and `rename_all`
attributes:

```rust
use jayson::DeserializeFromValue;
#[derive(DeserializeFromValue)]
#[jayson(rename_all = camelCase)]
struct MyStruct {
    my_field: bool,
    #[jayson(rename = "goodbye_world")]
    hello_world: u8,
}
```
will parse the following:
```json
{
    "myField": 1,
    "goodbye_world": 2
}
```


*/
pub use jayson_internal::DeserializeFromValue;

use std::fmt::{Debug, Display};

/// A location within a [`Value`].
///
/// Conceptually, it is a list of choices that one has to make to go to a certain place within
/// the value. In practice, it is used to locate the origin of a deserialization error.
///
/// ## Example
/// ```
/// use jayson::ValuePointerRef;
///
/// let pointer = ValuePointerRef::Origin;
/// let pointer = pointer.push_key("a");
/// let pointer = pointer.push_index(2);
/// // now `pointer` points to "a".2
/// ```
///
/// A `ValuePointerRef` is an immutable data structure, so it is cheap to extend and to copy.
/// However, if you want to store it inside an owned type, you may want to convert it to a
/// [`ValuePointer`] instead using [`self.to_owned()`](ValuePointerRef::to_owned).
#[derive(Clone, Copy)]
pub enum ValuePointerRef<'a> {
    Origin,
    Key {
        key: &'a str,
        prev: &'a ValuePointerRef<'a>,
    },
    Index {
        index: usize,
        prev: &'a ValuePointerRef<'a>,
    },
}
impl<'a> Default for ValuePointerRef<'a> {
    fn default() -> Self {
        Self::Origin
    }
}
impl<'a> ValuePointerRef<'a> {
    /// Extend `self` such that it points to the next subvalue at the given `key`.
    #[must_use]
    pub fn push_key(&'a self, key: &'a str) -> Self {
        Self::Key { key, prev: self }
    }
    #[must_use]
    /// Extend `self` such that it points to the next subvalue at the given index.
    pub fn push_index(&'a self, index: usize) -> Self {
        Self::Index { index, prev: self }
    }
    /// Convert `self` to its owned version
    pub fn to_owned(&self) -> ValuePointer {
        let mut cur = self;
        let mut components = vec![];
        loop {
            match cur {
                ValuePointerRef::Origin => break,
                ValuePointerRef::Key { key, prev } => {
                    components.push(ValuePointerComponent::Key(key.to_string()));
                    cur = prev;
                }
                ValuePointerRef::Index { index, prev } => {
                    components.push(ValuePointerComponent::Index(*index));
                    cur = prev;
                }
            }
        }
        let components = components.into_iter().rev().collect();
        ValuePointer { path: components }
    }
}

/// Part of a [`ValuePointer`]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValuePointerComponent {
    Key(String),
    Index(usize),
}

/// The owned version of a [`ValuePointerRef`].
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValuePointer {
    pub path: Vec<ValuePointerComponent>,
}
impl Display for ValuePointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for component in self.path.iter().rev() {
            match component {
                ValuePointerComponent::Index(i) => {
                    write!(f, ".{i}")?;
                }
                ValuePointerComponent::Key(s) => {
                    write!(f, ".{s}")?;
                }
            }
        }
        Ok(())
    }
}

/// Equivalent to [`Value`] but without the associated data.
#[derive(Clone, Copy, PartialEq, Eq)]
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
impl Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Null => write!(f, "Null"),
            ValueKind::Boolean => write!(f, "Boolean"),
            ValueKind::Integer => write!(f, "Integer"),
            ValueKind::NegativeInteger => write!(f, "NegativeInteger"),
            ValueKind::Float => write!(f, "Float"),
            ValueKind::String => write!(f, "String"),
            ValueKind::Sequence => write!(f, "Sequence"),
            ValueKind::Map => write!(f, "Map"),
        }
    }
}
impl Debug for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

/// `Value<V>` is a view into the parsed serialization data (of type `V`) that
/// is readable by Jayson.
///
/// It is an enum with a variant for each possible value kind. The content of the variants
/// is either a simple value, such as `bool` or `String`, or an abstract [`Sequence`] or
/// [`Map`], which are views into the rest of the serialized data.
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
impl<V: IntoValue> Value<V> {
    pub fn kind(&self) -> ValueKind {
        match self {
            Value::Null => ValueKind::Null,
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Integer(_) => ValueKind::Integer,
            Value::NegativeInteger(_) => ValueKind::NegativeInteger,
            Value::Float(_) => ValueKind::Float,
            Value::String(_) => ValueKind::String,
            Value::Sequence(_) => ValueKind::Sequence,
            Value::Map(_) => ValueKind::Map,
        }
    }
}

/// A trait for a value that can be deserialized via [`DeserializeFromValue`].
pub trait IntoValue: Sized {
    type Sequence: Sequence<Value = Self>;
    type Map: Map<Value = Self>;

    fn kind(&self) -> ValueKind;
    fn into_value(self) -> Value<Self>;
}

/// A sequence of values conforming to [`IntoValue`].
pub trait Sequence {
    type Value: IntoValue;
    type Iter: Iterator<Item = Self::Value>;

    fn len(&self) -> usize;
    fn into_iter(self) -> Self::Iter;
}

/// A keyed map of values conforming to [`IntoValue`].
pub trait Map {
    type Value: IntoValue;
    type Iter: Iterator<Item = (String, Self::Value)>;

    fn len(&self) -> usize;
    fn remove(&mut self, key: &str) -> Option<Self::Value>;
    fn into_iter(self) -> Self::Iter;
}

/// A trait for types that can be deserialized from a [`Value`]. The generic type
/// parameter `E` is the custom error that is returned when deserialization fails.
pub trait DeserializeFromValue<E: DeserializeError>: Sized {
    /// Attempts to deserialize `Self` from the given value. Note that this method is an
    /// implementation detail. You probably want to use the [`deserialize`] function directly instead.
    fn deserialize_from_value<V: IntoValue>(
        value: Value<V>,
        location: ValuePointerRef,
    ) -> Result<Self, E>;
    /// The value of `Self`, if any, when deserializing from a non-existent value.
    fn default() -> Option<Self> {
        None
    }
}

/// Deserialize the given value.
///
/// This function has three generic arguments, two of which can often be inferred.
/// 1. `Ret` is the type we want to deserialize to. For example: `MyStruct`
/// 2. `Val` is the type of the value given as argument. For example: `serde_json::Value`
/// 3. `E` is the error type we want to get when deserialization fails. For example: `MyError`
pub fn deserialize<Ret, Val, E>(value: Val) -> Result<Ret, E>
where
    Ret: DeserializeFromValue<E>,
    Val: IntoValue,
    E: DeserializeError,
{
    Ret::deserialize_from_value(value.into_value(), ValuePointerRef::Origin)
}

/// A trait which describes how to combine two errors together.
pub trait MergeWithError<T>: Sized {
    /// Merge two errors together.
    ///
    /// ## Arguments:
    /// - `self_`: the existing error, if any
    /// - `other`: the new error
    /// - `merge_location`: the location where the merging happens.
    ///
    /// ## Return value
    /// It should return the merged error inside a `Result`.
    ///
    /// The variant of the returned result should be `Ok(e)` to signal that the deserialization
    /// should continue (to accumulate more errors), or `Err(e)` to stop the deserialization immediately.
    ///
    /// Note that in both cases, the deserialization should eventually fail.
    ///
    /// ## Example
    /// Imagine you have the following json:
    /// ```json
    /// {
    ///    "w": true,
    ///    "x" : { "y": 1 }
    /// }
    /// ```
    /// It may be that deserializing the first field, `w`, fails with error `suberror: E`. This is the
    /// first deserialization error we encounter, so the current error value is `None`. The function `Self::merge`
    /// is called as follows:
    /// ```ignore
    /// // let mut error = None;
    /// // let mut location : ValuePointerRef::Origin;
    /// error = Some(Self::merge(error, suberror, location.push_key("w"))?);
    /// // if the returned value was Err(e), then we returned early from the deserialize method
    /// // otherwise, `error` is now set
    /// ```
    /// Later on, we encounter a new suberror originating from `x.y`. The `merge` function is called again:
    /// ```ignore
    /// // let mut error = Some(..);
    /// // let mut location : ValuePointerRef::Origin;
    /// error = Some(Self::merge(error, suberror, location.push_key("x"))?);
    /// // if the returned value was Err(e), then we returned early from the deserialize method
    /// // otherwise, `error` is now the result of its merging with suberror.
    /// ```
    /// Note that even though the suberror originated at `x.y`, the `merge_location` argument was `x`
    /// because that is where the merge happened.
    fn merge(self_: Option<Self>, other: T, merge_location: ValuePointerRef) -> Result<Self, Self>;
}

/// A trait for errors returned by [`deserialize_from_value`](DeserializeFromValue::deserialize_from_value).
pub trait DeserializeError: Sized + MergeWithError<Self> {
    /// Return the origin of the error, if it can be found
    fn location(&self) -> Option<ValuePointer>;
    /// Create a new error due to an unexpected value kind.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn incorrect_value_kind(
        self_: Option<Self>,
        actual: ValueKind,
        accepted: &[ValueKind],
        location: ValuePointerRef,
    ) -> Result<Self, Self>;
    /// Create a new error due to a missing key.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn missing_field(
        self_: Option<Self>,
        field: &str,
        location: ValuePointerRef,
    ) -> Result<Self, Self>;
    /// Create a new error due to finding an unknown key.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn unknown_key(
        self_: Option<Self>,
        key: &str,
        accepted: &[&str],
        location: ValuePointerRef,
    ) -> Result<Self, Self>;
    /// Create a new error with the custom message.
    ///
    /// Return `Ok` to continue deserializing or `Err` to fail early.
    fn unexpected(self_: Option<Self>, msg: &str, location: ValuePointerRef) -> Result<Self, Self>;
}

/// Used by the derive proc macro. Do not use.
#[doc(hidden)]
pub enum FieldState<T> {
    Missing,
    Err,
    Some(T),
}
impl<T> From<Option<T>> for FieldState<T> {
    fn from(x: Option<T>) -> Self {
        match x {
            Some(x) => FieldState::Some(x),
            None => FieldState::Missing,
        }
    }
}
impl<T> FieldState<T> {
    pub fn is_missing(&self) -> bool {
        matches!(self, FieldState::Missing)
    }
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            FieldState::Some(x) => x,
            _ => panic!("Unwrapping an empty field state"),
        }
    }
    pub fn map<U>(self, f: impl Fn(T) -> U) -> FieldState<U> {
        match self {
            FieldState::Some(x) => FieldState::Some(f(x)),
            FieldState::Missing => FieldState::Missing,
            FieldState::Err => FieldState::Err,
        }
    }
}

/// Used by the derive proc macro. Do not use.
#[doc(hidden)]
pub fn take_result_content<T>(r: Result<T, T>) -> T {
    match r {
        Ok(x) => x,
        Err(x) => x,
    }
}
