//! # Mimir
//!
//! A serializable, multi-type cache.
//!
//! ```
//! use serde::{Deserialize, Serialize};
//! use mimir::{Cache, Item};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
//! struct SomeStruct {
//! 	id: i32,
//! }
//!
//! impl Item for SomeStruct {
//! 	type Key = i32;
//! 	const TYPE_KEY: &'static str = "struct SomeStruct";
//!
//! 	fn key(&self) -> Self::Key {
//! 		self.id
//! 	}
//! }
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
//! struct SomeOtherStruct {
//! 	id: u32,
//! }
//!
//! impl Item for SomeOtherStruct {
//! 	type Key = u32;
//! 	const TYPE_KEY: &'static str = "struct SomeOtherStruct";
//!
//! 	fn key(&self) -> Self::Key {
//! 		self.id
//! 	}
//! }
//!
//! let mut cache = Cache::new();
//!
//! let a = SomeStruct { id: 0 };
//! let b = SomeStruct { id: 1 };
//! let c = SomeOtherStruct { id: 2 };
//!
//! cache.insert(a);
//! cache.insert(b);
//! cache.insert(c);
//!
//! let ser = serde_json::to_string(&cache).unwrap();
//! let dser = serde_json::from_str::<Cache>(&ser).unwrap();
//!
//! assert_eq!(Some(a), dser.get::<SomeStruct>(a.id).copied());
//!
//! // mimir also provides helper functions for types that implement Clone or Copy
//! assert_eq!(Some(b), dser.copied::<SomeStruct>(b.id));
//! assert_eq!(Some(c), dser.cloned::<SomeOtherStruct>(c.id));
//! ```

extern crate serde;
extern crate serde_traitobject as t;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

/// # The `Item` Trait
///
/// Specifies an item that can be serialized. Needs the following:
///  - The type of the key which will be used for the object
///  - The unique key of the type, which will be used for serialization
///  - A function to get the key for the object
pub trait Item: t::Serialize + t::Deserialize + Serialize + for<'de> Deserialize<'de> {
    /// Type of the key that you want to use with your object.
    type Key: Hash + Eq + t::Serialize + t::Deserialize + Serialize + for<'de> Deserialize<'de>;

    /// The key used for serializing this type
    const TYPE_KEY: &'static str;

    /// The key for the current OBJECT. Should be unique for each OBJECT.
    fn key(&self) -> Self::Key;
}

/// # Cache
///
/// A multi-type serializable cache, using the `Item` trait.
pub struct Cache {
    // HashMap<TypeKey of T, HashMap<T::Key, T>>
    items: HashMap<String, t::Box<dyn t::Any>>,
}

type InnerHashMap<T> = HashMap<<T as Item>::Key, Box<T>>;

impl Cache {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn insert<T: Item + 'static>(&mut self, item: T) {
        let typekey = T::TYPE_KEY.to_string();
        let key = item.key();

        let items = self
            .items
            .entry(typekey)
            .or_insert_with(|| t::Box::new(InnerHashMap::<T>::new()))
            .as_any_mut()
            .downcast_mut::<InnerHashMap<T>>()
            .unwrap();

        items.insert(key, Box::new(item));
    }

    pub fn get<T: Item + 'static>(&self, key: T::Key) -> Option<&T> {
        self.items
            .get(T::TYPE_KEY)
            .and_then(|v| v.as_any().downcast_ref::<InnerHashMap<T>>())
            .and_then(|n| n.get(&key))
            .map(|n| &**n)
    }

    pub fn copied<T: Item + 'static>(&self, key: T::Key) -> Option<T>
    where
        T: Copy,
    {
        self.get(key).copied()
    }

    pub fn cloned<T: Item + 'static>(&self, key: T::Key) -> Option<T>
    where
        T: Clone,
    {
        self.get(key).cloned()
    }

    pub fn get_mut<T: Item + 'static>(&mut self, key: T::Key) -> Option<&mut T> {
        self.items
            .get_mut(T::TYPE_KEY)
            .and_then(|v| v.as_any_mut().downcast_mut::<InnerHashMap<T>>())
            .and_then(|n| n.get_mut(&key))
            .map(|n| &mut **n)
    }

    pub fn take<T: Item + 'static>(&mut self, key: T::Key) -> Option<T> {
        self.items
            .get_mut(T::TYPE_KEY)
            .and_then(|v| v.as_any_mut().downcast_mut::<InnerHashMap<T>>())
            .and_then(|n| n.remove(&key))
            .map(|n| *n)
    }
}

impl Serialize for Cache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.items.len()))?;

        for (key, value) in &self.items {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

struct CacheVisitor;

impl<'de> Visitor<'de> for CacheVisitor {
    type Value = Cache;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "HashMap<String, t::Box<dyn t::Any>>")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut this = Cache::new();

        while let Some((k, v)) = map.next_entry()? {
            this.items.insert(k, v);
        }

        Ok(this)
    }
}

impl<'de> Deserialize<'de> for Cache {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(CacheVisitor)
    }
}
