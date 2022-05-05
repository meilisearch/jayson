pub use alloc::borrow::Cow;
pub use alloc::boxed::Box;
pub use alloc::string::String;
pub use core::option::Option::{self, None, Some};
pub use core::result::Result::{self, Err, Ok};

use crate::de::{Visitor, VisitorError};
use crate::json::{Number, Value};

pub use self::help::Str as str;
pub use self::help::Usize as usize;

mod help {
    pub type Str = str;
    pub type Usize = usize;
}

pub fn apply_object_to_visitor<E: VisitorError>(
    v: &mut dyn Visitor<E>,
    val: Value,
) -> Result<(), E> {
    match val {
        Value::Null => v.null()?,
        Value::Bool(b) => v.boolean(b)?,
        Value::Number(Number::U64(n)) => v.nonnegative(n)?,
        Value::Number(Number::I64(n)) => v.negative(n)?,
        Value::Number(Number::F64(n)) => v.float(n)?,
        Value::String(ref s) => v.string(s)?,
        Value::Array(a) => {
            let mut s = v.seq()?;
            for val in a {
                let v = s.element()?;
                apply_object_to_visitor(v, val)?;
            }

            s.finish()?;
        }
        Value::Object(o) => {
            let mut m = v.map()?;
            for (key, val) in o {
                let v = m.key(&key)?;
                apply_object_to_visitor(v, val)?;
            }

            m.finish()?;
        }
    }

    Ok(())
}
