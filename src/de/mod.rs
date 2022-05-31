mod impls;

use crate::error::Error;
use alloc::boxed::Box;

/// Trait for data structures that can be deserialized from a JSON string.
///
/// [Refer to the module documentation for examples.][crate::de]
pub trait Jayson<E = Error>: Sized {
    /// The only correct implementation of this method is:
    ///
    /// ```rust
    /// # use jayson::make_place;
    /// # use jayson::de::{Jayson, Visitor, VisitorError};
    /// #
    /// # make_place!(Place);
    /// # struct S;
    /// # impl<E: VisitorError> Visitor<E> for Place<S> {}
    /// #
    /// # impl<E: VisitorError> Jayson<E> for S {
    /// fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
    ///     Place::new(out)
    /// }
    /// # }
    /// ```
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E>;

    // Not public API. This method is only intended for Option<T>, should not
    // need to be implemented outside of this crate.
    #[doc(hidden)]
    #[inline]
    fn default() -> Option<Self> {
        None
    }
}

pub trait VisitorError: 'static {
    fn unexpected(s: &str) -> Self;
    fn format_error(line: usize, pos: usize, msg: &str) -> Self;
    fn missing_field(field: &str) -> Self;
}

/// Trait that can write data into an output place.
///
/// [Refer to the module documentation for examples.][crate::de]
pub trait Visitor<E: VisitorError = Error> {
    fn null(&mut self) -> Result<(), E> {
        Err(E::unexpected("null"))
    }

    fn boolean(&mut self, b: bool) -> Result<(), E> {
        let _ = b;
        Err(E::unexpected("boolean"))
    }

    fn string(&mut self, s: &str) -> Result<(), E> {
        let _ = s;
        Err(E::unexpected("string"))
    }

    fn negative(&mut self, n: i64) -> Result<(), E> {
        let _ = n;
        Err(E::unexpected("negative integer"))
    }

    fn nonnegative(&mut self, n: u64) -> Result<(), E> {
        let _ = n;
        Err(E::unexpected("non negative integer"))
    }

    fn float(&mut self, n: f64) -> Result<(), E> {
        let _ = n;
        Err(E::unexpected("float"))
    }

    fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
        Err(E::unexpected("sequence"))
    }

    fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
        Err(E::unexpected("map"))
    }
}

/// Trait that can hand out places to write sequence elements.
///
/// [Refer to the module documentation for examples.][crate::de]
pub trait Seq<E> {
    fn element(&mut self) -> Result<&mut dyn Visitor<E>, E>;
    fn finish(&mut self) -> Result<(), E>;
}

/// Trait that can hand out places to write values of a map.
///
/// [Refer to the module documentation for examples.][crate::de]
pub trait Map<E> {
    fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<E>, E>;
    fn finish(&mut self) -> Result<(), E>;
}
