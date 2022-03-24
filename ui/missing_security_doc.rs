// aux-build:frame-support.rs

#[warn(
		clippy::disallowed_methods,
		clippy::indexing_slicing,
		clippy::todo,
		clippy::unwrap_used,
		clippy::panic
	)]
extern crate frame_support;

use frame_support::storage::{IterableStorageMap, StorageMap as Bar, IterableStorageDoubleMap, StorageDoubleMap};

// pub type Foo<K, V> = dyn Bar<K, V, Query=()>;

fn main() {}