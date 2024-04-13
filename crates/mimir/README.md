<!-- cargo-rdme start -->

# Mimir

A serializable, multi-type cache.

```rust
use serde::{Deserialize, Serialize};
use mimir::{Cache, Item};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct SomeStruct {
	id: i32,
}

impl Item for SomeStruct {
	type Key = i32;
	const TYPE_KEY: &'static str = "struct SomeStruct";

	fn key(&self) -> Self::Key {
		self.id
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
struct SomeOtherStruct {
	id: u32,
}

impl Item for SomeOtherStruct {
	type Key = u32;
	const TYPE_KEY: &'static str = "struct SomeOtherStruct";

	fn key(&self) -> Self::Key {
		self.id
	}
}

let mut cache = Cache::new();

let a = SomeStruct { id: 0 };
let b = SomeStruct { id: 1 };
let c = SomeOtherStruct { id: 2 };

cache.insert(a);
cache.insert(b);
cache.insert(c);

let ser = serde_json::to_string(&cache).unwrap();
let dser = serde_json::from_str::<Cache>(&ser).unwrap();

assert_eq!(Some(a), dser.get::<SomeStruct>(a.id).copied());

// mimir also provides helper functions for types that implement Clone or Copy
assert_eq!(Some(b), dser.copied::<SomeStruct>(b.id));
assert_eq!(Some(c), dser.cloned::<SomeOtherStruct>(c.id));
```

<!-- cargo-rdme end -->
