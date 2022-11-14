// #![allow(panics)]
// aux-build:frame-support.rs

extern crate frame_support;

use frame_support::{
    storage::{types::map::StorageMap as Bar},
    hash::Twox64Concat,
    Identity,
};

/// # Security
///
/// Twox64Concat is allowed because this is a test
pub type Foo<K, V> = Bar<Twox64Concat, K, V>;


pub type Foo2<K, V> = Bar<Twox64Concat, K, V>;

fn main() {}
