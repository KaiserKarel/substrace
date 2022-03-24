// aux-build:frame-support.rs

#[warn(
		clippy::disallowed_methods,
		clippy::indexing_slicing,
		clippy::todo,
		clippy::unwrap_used,
		clippy::panic
	)]
extern crate frame_support;

use frame_support::{Identity, Twox64Concat, storage::{IterableStorageMap, StorageMap as Bar, IterableStorageDoubleMap, StorageDoubleMap}};

/// # Security
/// 
/// Twox64Concat is allowed because this is a test
#[allow(bare_trait_objects)]
pub type Foo<K, V> = Bar<Twox64Concat, K, V, Query=()>;

#[allow(bare_trait_objects)]
pub type Foo2<K, V> = Bar<Twox64Concat, K, V, Query=()>;

fn main() {}