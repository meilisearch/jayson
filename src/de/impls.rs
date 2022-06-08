use crate::de::{Jayson, Map, Seq, Visitor};
use crate::ignore::Ignore;
use crate::ptr::NonuniqueBox;
use crate::Place;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::{self, ManuallyDrop};
use core::str::{self, FromStr};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{BuildHasher, Hash};

use super::VisitorError;

impl<E: VisitorError> Jayson<E> for () {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError> Visitor<E> for Place<()> {
            fn null(&mut self) -> Result<(), E> {
                self.out = Some(());
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<E: VisitorError> Jayson<E> for bool {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError> Visitor<E> for Place<bool> {
            fn boolean(&mut self, b: bool) -> Result<(), E> {
                self.out = Some(b);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<E: VisitorError> Jayson<E> for String {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError> Visitor<E> for Place<String> {
            fn string(&mut self, s: &str) -> Result<(), E> {
                self.out = Some(s.to_owned());
                Ok(())
            }
        }
        Place::new(out)
    }
}

macro_rules! signed {
    ($ty:ident) => {
        impl<E> Jayson<E> for $ty
        where
            E: VisitorError,
        {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
                impl<E: VisitorError> Visitor<E> for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<(), E> {
                        if n >= $ty::min_value() as i64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(E::unexpected("error"))
                        }
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(E::unexpected("error"))
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
signed!(i8);
signed!(i16);
signed!(i32);
signed!(i64);
signed!(isize);

macro_rules! unsigned {
    ($ty:ident) => {
        impl<E> Jayson<E> for $ty
        where
            E: VisitorError,
        {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
                impl<E: VisitorError> Visitor<E> for Place<$ty> {
                    fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(E::unexpected("value overflow"))
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
unsigned!(u8);
unsigned!(u16);
unsigned!(u32);
unsigned!(u64);
unsigned!(usize);

macro_rules! float {
    ($ty:ident) => {
        impl<E: VisitorError> Jayson<E> for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
                impl<E: VisitorError> Visitor<E> for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<(), E> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }

                    fn float(&mut self, n: f64) -> Result<(), E> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }
                }
                Place::new(out)
            }
        }
    };
}
float!(f32);
float!(f64);

impl<E: VisitorError, T: Jayson<E>> Jayson<E> for Box<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError, T: Jayson<E>> Visitor<E> for Place<Box<T>> {
            fn null(&mut self) -> Result<(), E> {
                let mut out = None;
                Jayson::begin(&mut out).null()?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<(), E> {
                let mut out = None;
                Jayson::begin(&mut out).boolean(b)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn string(&mut self, s: &str) -> Result<(), E> {
                let mut out = None;
                Jayson::begin(&mut out).string(s)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn negative(&mut self, n: i64) -> Result<(), E> {
                let mut out = None;
                Jayson::begin(&mut out).negative(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                let mut out = None;
                Jayson::begin(&mut out).nonnegative(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn float(&mut self, n: f64) -> Result<(), E> {
                let mut out = None;
                Jayson::begin(&mut out).float(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                let mut value = NonuniqueBox::new(None);
                let ptr = unsafe { extend_lifetime!(&mut *value as &mut Option<T>) };
                Ok(Box::new(BoxSeq {
                    out: &mut self.out,
                    value,
                    seq: ManuallyDrop::new(Jayson::begin(ptr).seq()?),
                }))
            }

            fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
                let mut value = NonuniqueBox::new(None);
                let ptr = unsafe { extend_lifetime!(&mut *value as &mut Option<T>) };
                Ok(Box::new(BoxMap {
                    out: &mut self.out,
                    value,
                    map: ManuallyDrop::new(Jayson::begin(ptr).map()?),
                }))
            }
        }

        struct BoxSeq<'a, E, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: NonuniqueBox<Option<T>>,
            // May borrow from self.value, so must drop first.
            seq: ManuallyDrop<Box<dyn Seq<E> + 'a>>,
        }

        impl<'a, E, T: 'a> Drop for BoxSeq<'a, E, T> {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.seq) }
            }
        }

        impl<'a, E: VisitorError, T: Jayson<E>> Seq<E> for BoxSeq<'a, E, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                self.seq.element()
            }

            fn finish(&mut self) -> Result<(), E> {
                self.seq.finish()?;
                *self.seq = Box::new(Ignore);
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        struct BoxMap<'a, E, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: NonuniqueBox<Option<T>>,
            // May borrow from self.value, so must drop first.
            map: ManuallyDrop<Box<dyn Map<E> + 'a>>,
        }

        impl<'a, E, T: 'a> Drop for BoxMap<'a, E, T> {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.map) }
            }
        }

        impl<'a, E: VisitorError, T: Jayson<E>> Map<E> for BoxMap<'a, E, T> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<E>, E> {
                self.map.key(k)
            }

            fn finish(&mut self) -> Result<(), E> {
                self.map.finish()?;
                *self.map = Box::new(Ignore);
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, T: Jayson<E>> Jayson<E> for Option<T> {
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError, T: Jayson<E>> Visitor<E> for Place<Option<T>> {
            fn null(&mut self) -> Result<(), E> {
                self.out = Some(None);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<(), E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).boolean(b)
            }

            fn string(&mut self, s: &str) -> Result<(), E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).string(s)
            }

            fn negative(&mut self, n: i64) -> Result<(), E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).negative(n)
            }

            fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).nonnegative(n)
            }

            fn float(&mut self, n: f64) -> Result<(), E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).float(n)
            }

            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).seq()
            }

            fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
                self.out = Some(None);
                Jayson::begin(self.out.as_mut().unwrap()).map()
            }
        }

        Place::new(out)
    }
}

impl<E, A, B> Jayson<E> for (A, B)
where
    E: VisitorError,
    A: Jayson<E>,
    B: Jayson<E>,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError, A: Jayson<E>, B: Jayson<E>> Visitor<E> for Place<(A, B)> {
            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                Ok(Box::new(TupleBuilder {
                    out: &mut self.out,
                    tuple: (None, None),
                }))
            }
        }

        struct TupleBuilder<'a, A: 'a, B: 'a> {
            out: &'a mut Option<(A, B)>,
            tuple: (Option<A>, Option<B>),
        }

        impl<'a, E, A, B> Seq<E> for TupleBuilder<'a, A, B>
        where
            E: VisitorError,
            A: Jayson<E>,
            B: Jayson<E>,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                if self.tuple.0.is_none() {
                    Ok(Jayson::begin(&mut self.tuple.0))
                } else if self.tuple.1.is_none() {
                    Ok(Jayson::begin(&mut self.tuple.1))
                } else {
                    Err(E::unexpected("tuple has more than 2 items."))
                }
            }

            fn finish(&mut self) -> Result<(), E> {
                if let (Some(a), Some(b)) = (self.tuple.0.take(), self.tuple.1.take()) {
                    *self.out = Some((a, b));
                    Ok(())
                } else {
                    Err(E::unexpected("tuple should have 2 items"))
                }
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, T: Jayson<E>> Jayson<E> for Vec<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E: VisitorError, T: Jayson<E>> Visitor<E> for Place<Vec<T>> {
            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                Ok(Box::new(VecBuilder {
                    out: &mut self.out,
                    vec: Vec::new(),
                    element: None,
                }))
            }
        }

        struct VecBuilder<'a, T: 'a> {
            out: &'a mut Option<Vec<T>>,
            vec: Vec<T>,
            element: Option<T>,
        }

        impl<'a, T> VecBuilder<'a, T> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.vec.push(e);
                }
            }
        }

        impl<'a, E, T: Jayson<E>> Seq<E> for VecBuilder<'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                self.shift();
                Ok(Jayson::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<(), E> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.vec, Vec::new()));
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<E, K, V, H> Jayson<E> for HashMap<K, V, H>
where
    K: FromStr + Hash + Eq,
    V: Jayson<E>,
    H: BuildHasher + Default,
    E: VisitorError,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E, K, V, H> Visitor<E> for Place<HashMap<K, V, H>>
        where
            K: FromStr + Hash + Eq,
            V: Jayson<E>,
            H: BuildHasher + Default,
            E: VisitorError,
        {
            fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: HashMap::with_hasher(H::default()),
                    key: None,
                    value: None,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a, H: 'a> {
            out: &'a mut Option<HashMap<K, V, H>>,
            map: HashMap<K, V, H>,
            key: Option<K>,
            value: Option<V>,
        }

        impl<'a, K: Hash + Eq, V, H: BuildHasher> MapBuilder<'a, K, V, H> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, E, K, V, H> Map<E> for MapBuilder<'a, K, V, H>
        where
            K: FromStr + Hash + Eq,
            V: Jayson<E>,
            H: BuildHasher + Default,
            E: VisitorError,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<E>, E> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(E::unexpected(&format!("can not parse map key `{k}`"))),
                });
                Ok(Jayson::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<(), E> {
                self.shift();
                let substitute = HashMap::with_hasher(H::default());
                *self.out = Some(mem::replace(&mut self.map, substitute));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, K: FromStr + Ord, V: Jayson<E>> Jayson<E> for BTreeMap<K, V> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E, K, V> Visitor<E> for Place<BTreeMap<K, V>>
        where
            K: FromStr + Ord,
            V: Jayson<E>,
            E: VisitorError,
        {
            fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: BTreeMap::new(),
                    key: None,
                    value: None,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a> {
            out: &'a mut Option<BTreeMap<K, V>>,
            map: BTreeMap<K, V>,
            key: Option<K>,
            value: Option<V>,
        }

        impl<'a, K: Ord, V> MapBuilder<'a, K, V> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, E: VisitorError, K, V> Map<E> for MapBuilder<'a, K, V>
        where
            E: VisitorError,
            K: FromStr + Ord,
            V: Jayson<E>,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<E>, E> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(E::unexpected(&format!("can not parse map key `{k}`"))),
                });
                Ok(Jayson::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<(), E> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.map, BTreeMap::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<T, E> Jayson<E> for BTreeSet<T>
where
    E: VisitorError,
    T: Ord + Jayson<E>,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E, T> Visitor<E> for Place<BTreeSet<T>>
        where
            T: Jayson<E> + Ord,
            E: VisitorError,
        {
            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                Ok(Box::new(SeqBuilder {
                    out: &mut self.out,
                    set: BTreeSet::new(),
                    value: None,
                }))
            }
        }

        struct SeqBuilder<'a, T: 'a> {
            out: &'a mut Option<BTreeSet<T>>,
            set: BTreeSet<T>,
            value: Option<T>,
        }

        impl<'a, E, T> Seq<E> for SeqBuilder<'a, T>
        where
            T: Jayson<E> + Ord,
            E: VisitorError,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                if let Some(value) = self.value.take() {
                    self.set.insert(value);
                }
                Ok(Jayson::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<(), E> {
                if let Some(value) = self.value.take() {
                    self.set.insert(value);
                }
                *self.out = Some(mem::take(&mut self.set));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<T, E> Jayson<E> for HashSet<T>
where
    E: VisitorError,
    T: Hash + Eq + Jayson<E>,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
        impl<E, T> Visitor<E> for Place<HashSet<T>>
        where
            T: Jayson<E> + Hash + Eq,
            E: VisitorError,
        {
            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                Ok(Box::new(SeqBuilder {
                    out: &mut self.out,
                    set: HashSet::new(),
                    value: None,
                }))
            }
        }

        struct SeqBuilder<'a, T: 'a> {
            out: &'a mut Option<HashSet<T>>,
            set: HashSet<T>,
            value: Option<T>,
        }

        impl<'a, E, T> Seq<E> for SeqBuilder<'a, T>
        where
            T: Jayson<E> + Hash + Eq,
            E: VisitorError,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                if let Some(value) = self.value.take() {
                    self.set.insert(value);
                }
                Ok(Jayson::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<(), E> {
                if let Some(value) = self.value.take() {
                    self.set.insert(value);
                }
                *self.out = Some(mem::take(&mut self.set));
                Ok(())
            }
        }

        Place::new(out)
    }
}

#[cfg(feature = "serde_json")]
mod serde_json_impl {
    use super::*;

    use serde_json::Number;

    impl<E: VisitorError> Jayson<E> for serde_json::Value {
        fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<E> {
            impl<E: VisitorError> Visitor<E> for Place<serde_json::Value> {
                fn null(&mut self) -> Result<(), E> {
                    self.out = Some(serde_json::Value::Null);
                    Ok(())
                }

                fn boolean(&mut self, b: bool) -> Result<(), E> {
                    self.out = Some(serde_json::Value::Bool(b));
                    Ok(())
                }

                fn string(&mut self, s: &str) -> Result<(), E> {
                    self.out = Some(serde_json::Value::String(s.to_owned()));
                    Ok(())
                }

                fn negative(&mut self, n: i64) -> Result<(), E> {
                    self.out = Some(serde_json::Value::Number(Number::from(n)));
                    Ok(())
                }

                fn nonnegative(&mut self, n: u64) -> Result<(), E> {
                    self.out = Some(serde_json::Value::Number(Number::from(n)));
                    Ok(())
                }

                fn float(&mut self, n: f64) -> Result<(), E> {
                    if let Some(n) = Number::from_f64(n) {
                        self.out = Some(serde_json::Value::Number(n));
                        Ok(())
                    } else {
                        Err(E::unexpected("{} is not a valid Json floating point"))
                    }
                }

                fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>, E> {
                    struct VecBuilder<'a> {
                        out: &'a mut Option<serde_json::Value>,
                        vec: Vec<serde_json::Value>,
                        element: Option<serde_json::Value>,
                    }

                    impl<'a> VecBuilder<'a> {
                        fn shift(&mut self) {
                            if let Some(e) = self.element.take() {
                                self.vec.push(e);
                            }
                        }
                    }
                    impl<'a, E: VisitorError> Seq<E> for VecBuilder<'a> {
                        fn element(&mut self) -> Result<&mut dyn Visitor<E>, E> {
                            self.shift();
                            Ok(Jayson::begin(&mut self.element))
                        }

                        fn finish(&mut self) -> Result<(), E> {
                            self.shift();
                            let vec = std::mem::take(&mut self.vec);
                            *self.out = Some(serde_json::Value::Array(vec));
                            Ok(())
                        }
                    }
                    Ok(Box::new(VecBuilder {
                        out: &mut self.out,
                        vec: vec![],
                        element: None,
                    }))
                }

                fn map(&mut self) -> Result<Box<dyn Map<E> + '_>, E> {
                    struct MapBuilder<'a> {
                        out: &'a mut Option<serde_json::Value>,
                        map: serde_json::Map<String, serde_json::Value>,
                        key: Option<String>,
                        value: Option<serde_json::Value>,
                    }

                    impl<'a> MapBuilder<'a> {
                        fn shift(&mut self) {
                            if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                                self.map.insert(k, v);
                            }
                        }
                    }

                    impl<'a, E> Map<E> for MapBuilder<'a>
                    where
                        E: VisitorError,
                    {
                        fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<E>, E> {
                            self.shift();
                            self.key = Some(k.to_owned());
                            Ok(Jayson::begin(&mut self.value))
                        }

                        fn finish(&mut self) -> Result<(), E> {
                            self.shift();
                            let map = mem::take(&mut self.map);
                            *self.out = Some(serde_json::Value::Object(map));
                            Ok(())
                        }
                    }
                    Ok(Box::new(MapBuilder {
                        out: &mut self.out,
                        map: Default::default(),
                        key: None,
                        value: None,
                    }))
                }
            }

            Place::new(out)
        }
    }
}
