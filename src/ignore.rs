use crate::de::{Map, Seq, Visitor, VisitorError};
use alloc::boxed::Box;

impl<E: VisitorError> dyn Visitor<E> {
    pub fn ignore() -> &'static mut dyn Visitor<E> {
        static mut IGNORE: Ignore = Ignore;
        unsafe { &mut IGNORE }
        //
        // The following may be needed if stacked borrows gets more selective
        // about the above in the future:
        //
        //     unsafe { &mut *ptr::addr_of_mut!(IGNORE) }
        //
        // Conceptually we have an array of type [Ignore; ∞] in a static, which
        // is zero sized, and each caller of `fn ignore` gets a unique one of
        // them, as if by `&mut *ptr::addr_of_mut!(IGNORE[i++])` for some
        // appropriately synchronized i.
    }
}

pub(crate) struct Ignore;

impl<E: VisitorError + 'static> Visitor<E> for Ignore {
    fn null(&mut self) -> Result<(), E> {
        Ok(())
    }

    fn boolean(&mut self, _b: bool) -> Result<(), E> {
        Ok(())
    }

    fn string(&mut self, _s: &str) -> Result<(), E> {
        Ok(())
    }

    fn negative(&mut self, _n: i64) -> Result<(), E> {
        Ok(())
    }

    fn nonnegative(&mut self, _n: u64) -> Result<(), E> {
        Ok(())
    }

    fn float(&mut self, _n: f64) -> Result<(), E> {
        Ok(())
    }

    fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
        Ok(Box::new(Ignore))
    }

    fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
        Ok(Box::new(Ignore))
    }
}

impl<E: VisitorError + 'static> Seq<E> for Ignore {
    fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
        Ok(<dyn Visitor<E>>::ignore())
    }

    fn finish(&mut self) -> Result<(), E> {
        Ok(())
    }
}

impl<E: VisitorError + 'static> Map<E> for Ignore {
    fn key(&mut self, _k: &str) -> Result<&mut dyn Visitor<E>, E> {
        Ok(<dyn Visitor<E>>::ignore())
    }

    fn finish(&mut self) -> Result<(), E> {
        Ok(())
    }
}
