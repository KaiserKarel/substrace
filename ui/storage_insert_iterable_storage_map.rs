// aux-build:frame-support.rs

extern crate frame_support;

#[warn(
    clippy::disallowed_method,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unwrap_used,
    clippy::panic
)]

use frame_support::storage::{IterableStorageMap, StorageMap, IterableStorageDoubleMap, StorageDoubleMap};

fn main() {
    fn foo<T: IterableStorageMap + StorageMap<u32, u32>>() {
        T::iter();
        T::swap(1, 2);
    }

    fn bar<T: IterableStorageDoubleMap<u32, u32, u32> + StorageDoubleMap<u32, u32, u32>>() {
        T::iter();
        T::swap(1, 2, 3, 4);
    }

    // Acceptable because mutation then iter handle is fine.
    fn baz<T: IterableStorageDoubleMap<u32, u32, u32> + StorageDoubleMap<u32, u32, u32>>() {
        T::swap(1, 2, 3, 4);
        T::iter();
    }
}
