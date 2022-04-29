use crate::de::{Deserialize, Map, Seq, Visitor, VisitorError};
use crate::json::{Array, Number, Object};
use crate::Place;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use core::mem;
use core::str;

/// Any valid JSON value.
///
/// This type has a non-recursive drop implementation so it is safe to build
/// arbitrarily deeply nested instances.
///
/// ```rust
/// use miniserde::json::{Array, Value};
///
/// let mut value = Value::Null;
#[cfg_attr(not(miri), doc = "for _ in 0..100000 {")]
#[cfg_attr(miri, doc = "for _ in 0..40 {")]
///     let mut array = Array::new();
///     array.push(value);
///     value = Value::Array(array);
/// }
/// // no stack overflow when `value` goes out of scope
/// ```
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Array),
    Object(Object),
}

impl Default for Value {
    /// The default value is null.
    fn default() -> Self {
        Value::Null
    }
}

impl<E: VisitorError> Deserialize<E> for Value {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError> Visitor<E> for Place<Value> {
            fn null(&mut self) -> Result<(), E> {
                self.out = Some(Value::Null);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<(), E> {
                self.out = Some(Value::Bool(b));
                Ok(())
            }

            fn string(&mut self, s: &str) -> Result<(), E> {
                self.out = Some(Value::String(s.to_owned()));
                Ok(())
            }

            fn negative(&mut self, n: i64) -> Result<(), E> {
                self.out = Some(Value::Number(Number::I64(n)));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                self.out = Some(Value::Number(Number::U64(n)));
                Ok(())
            }

            fn float(&mut self, n: f64) -> Result<(), E> {
                self.out = Some(Value::Number(Number::F64(n)));
                Ok(())
            }

            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                Ok(Box::new(ArrayBuilder {
                    out: &mut self.out,
                    array: Array::new(),
                    element: None,
                }))
            }

            fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
                Ok(Box::new(ObjectBuilder {
                    out: &mut self.out,
                    object: Object::new(),
                    key: None,
                    value: None,
                }))
            }
        }

        struct ArrayBuilder<'a> {
            out: &'a mut Option<Value>,
            array: Array,
            element: Option<Value>,
        }

        impl<'a> ArrayBuilder<'a> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.array.push(e);
                }
            }
        }

        impl<'a, E: VisitorError> Seq<E> for ArrayBuilder<'a> {
            fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<(), E> {
                self.shift();
                *self.out = Some(Value::Array(mem::replace(&mut self.array, Array::new())));
                Ok(())
            }
        }

        struct ObjectBuilder<'a> {
            out: &'a mut Option<Value>,
            object: Object,
            key: Option<String>,
            value: Option<Value>,
        }

        impl<'a> ObjectBuilder<'a> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.object.insert(k, v);
                }
            }
        }

        impl<'a, E: VisitorError> Map<E> for ObjectBuilder<'a> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<E>, E> {
                self.shift();
                self.key = Some(k.to_owned());
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<(), E> {
                self.shift();
                *self.out = Some(Value::Object(mem::replace(&mut self.object, Object::new())));
                Ok(())
            }
        }

        Place::new(out)
    }
}
