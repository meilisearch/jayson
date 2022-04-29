//! JSON data format.
//!
//! [See the crate level doc](../index.html#example) for an example of
//! serializing and deserializing JSON.

mod de;
pub use self::de::from_str;

mod value;
pub use self::value::Value;

mod number;
pub use self::number::Number;

mod array;
pub use self::array::Array;

mod object;
pub use self::object::Object;

mod drop;
