mod impls;

use crate::error::Error;
use alloc::boxed::Box;

/// Trait for data structures that can be deserialized from a JSON string.
///
/// [Refer to the module documentation for examples.][crate::de]
pub trait Deserialize<E = Error>: Sized {
    /// The only correct implementation of this method is:
    ///
    /// ```rust
    /// # use miniserde::make_place;
    /// # use miniserde::de::{Deserialize, Visitor};
    /// #
    /// # make_place!(Place);
    /// # struct S;
    /// # impl Visitor for Place<S> {}
    /// #
    /// # impl Deserialize for S {
    /// fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
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
    fn unexpected() -> Self;
    fn format_error(line: usize, pos: usize, msg: &str) -> Self;
}

/// Trait that can write data into an output place.
///
/// [Refer to the module documentation for examples.][crate::de]
pub trait Visitor<E: VisitorError = Error> {
    fn null(&mut self) -> Result<(), E> {
        Err(E::unexpected())
    }

    fn boolean(&mut self, b: bool) -> Result<(), E> {
        let _ = b;
        Err(E::unexpected())
    }

    fn string(&mut self, s: &str) -> Result<(), E> {
        let _ = s;
        Err(E::unexpected())
    }

    fn negative(&mut self, n: i64) -> Result<(), E> {
        let _ = n;
        Err(E::unexpected())
    }

    fn nonnegative(&mut self, n: u64) -> Result<(), E> {
        let _ = n;
        Err(E::unexpected())
    }

    fn float(&mut self, n: f64) -> Result<(), E> {
        let _ = n;
        Err(E::unexpected())
    }

    fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
        Err(E::unexpected())
    }

    fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
        Err(E::unexpected())
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
