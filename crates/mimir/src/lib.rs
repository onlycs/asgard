extern crate serde;
extern crate serde_traitobject as t;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};
use std::{any::TypeId, collections::HashMap, hash::Hash};

/// # The `Item` Trait
///
/// This trait is used by any object that you want cached. To implement, we need a few things:
///	 - A unique id or key for each object
///  - A unique id or key for the type as a whole
///
/// To make this work
///  - The types of the keys are associated types.
///  - The object key must be Serializable, Deserializable, and Hashable.
///  - The type key must be able to convert to a `String`.
pub trait Item: t::Serialize + t::Deserialize + Serialize + for<'de> Deserialize<'de> {
    /// Type of the key that you want to use with your object.
    type Key: Hash + Eq + t::Serialize + t::Deserialize + Serialize + for<'de> Deserialize<'de>;

    /// Type of the TypeKey. Will be keyed as its string variant
    type TypeKey: ToString;

    /// The key that will be used to store the object while serializing
    /// which should be unique to each TYPE
    fn type_key() -> Self::TypeKey;

    /// The key for the current OBJECT. Should be unique for each OBJECT.
    fn key(&self) -> Self::Key;
}

/// # Cache
///
/// A serializable, multi-type, keyed cache.
///
/// example
/// ```
/// use serde::{Deserialize, Serialize};
/// use mimir::{Cache, Item};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
/// struct SomeStruct {
/// 	id: i32,
/// }
///
/// impl Item for SomeStruct {
/// 	type Key = i32;
/// 	type TypeKey = &'static str;
///
/// 	fn key(&self) -> Self::Key {
/// 		self.id
/// 	}
///
/// 	fn type_key() -> Self::TypeKey {
/// 		"struct SomeStruct"
/// 	}
/// }
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
/// struct SomeOtherStruct {
/// 	id: u32,
/// }
///
/// impl Item for SomeOtherStruct {
/// 	type Key = u32;
/// 	type TypeKey = &'static str;
///
/// 	fn key(&self) -> Self::Key {
/// 		self.id
/// 	}
///
/// 	fn type_key() -> Self::TypeKey {
/// 		"struct SomeOtherStruct"
/// 	}
/// }
///
/// let mut cache = Cache::new();
/// let a = SomeStruct { id: 0 };
/// let b = SomeStruct { id: 1 };
/// let c = SomeOtherStruct { id: 2 };
///
/// cache.insert(a);
/// cache.insert(b);
/// cache.insert(c);
///
/// let ser = serde_json::to_string(&cache).unwrap();
/// let dser = serde_json::from_str::<Cache>(&ser).unwrap();
///
/// println!("{ser}");
///
/// assert_eq!(Some(a), dser.get::<SomeStruct>(a.id).copied());
///
/// // mimir also provides helper functions for types that implement Clone or Copy
/// assert_eq!(Some(b), dser.copied::<SomeStruct>(b.id));
/// assert_eq!(Some(c), dser.cloned::<SomeOtherStruct>(c.id));
/// ```
pub struct Cache {
    deser: HashMap<String, t::Box<dyn t::Any>>,
    keys: HashMap<TypeId, String>,
    items: HashMap<TypeId, t::Box<dyn t::Any>>,
}

type InnerHashMap<T> = HashMap<<T as Item>::Key, Box<T>>;

impl Cache {
    pub fn new() -> Self {
        Self {
            deser: HashMap::new(),
            keys: HashMap::new(),
            items: HashMap::new(),
        }
    }

    fn extract_deser<T: Item + 'static>(&mut self) {
        if let Some(v) = self.deser.remove(&T::type_key().to_string()) {
            self.items.insert(TypeId::of::<T>(), v);
        }
    }

    fn immut_extract_deser<T: Item + 'static>(&self) -> Option<&t::Box<dyn t::Any>> {
        self.deser.get(&T::type_key().to_string())
    }

    pub fn insert<T: Item + 'static>(&mut self, item: T) {
        self.extract_deser::<T>();

        let type_id = TypeId::of::<T>();
        let key = item.key();

        self.keys.insert(type_id, T::type_key().to_string());

        let items = self
            .items
            .entry(type_id)
            .or_insert_with(|| t::Box::new(InnerHashMap::<T>::new()))
            .as_any_mut()
            .downcast_mut::<InnerHashMap<T>>()
            .unwrap();

        items.insert(key, Box::new(item));
    }

    pub fn get<T: Item + 'static>(&self, key: T::Key) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        self.immut_extract_deser::<T>()
            .or_else(|| self.items.get(&type_id))
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
        self.extract_deser::<T>();

        let type_id = TypeId::of::<T>();

        self.items
            .get_mut(&type_id)
            .and_then(|v| v.as_any_mut().downcast_mut::<InnerHashMap<T>>())
            .and_then(|n| n.get_mut(&key))
            .map(|n| &mut **n)
    }

    pub fn take<T: Item + 'static>(&mut self, key: T::Key) -> Option<T> {
        self.extract_deser::<T>();

        let type_id = TypeId::of::<T>();

        self.items
            .get_mut(&type_id)
            .map(|v| v.as_any_mut().downcast_mut::<InnerHashMap<T>>())
            .flatten()
            .map(|n| n.remove(&key))
            .flatten()
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
            map.serialize_entry(self.keys.get(key).unwrap(), value)?;
        }

        for (key, value) in &self.deser {
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
            this.deser.insert(k, v);
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
