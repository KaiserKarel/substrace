//! This module contains paths to types and functions Substrace needs to know
//! about.
//!
//! Whenever possible, please consider diagnostic items over hardcoded paths.

pub const DEFAULT_TRAIT_METHOD: [&str; 4] = ["core", "default", "Default", "default"];
pub const SLICE_INTO_VEC: [&str; 4] = ["alloc", "slice", "<impl [T]>", "into_vec"];
pub const CONVERT_IDENTITY: [&str; 3] = ["core", "convert", "identity"];
pub const VEC_FROM_ELEM: [&str; 3] = ["alloc", "vec", "from_elem"];
pub const VEC_NEW: [&str; 4] = ["alloc", "vec", "Vec", "new"];
pub const WEAK_ARC: [&str; 3] = ["alloc", "sync", "Weak"];
pub const WEAK_RC: [&str; 3] = ["alloc", "rc", "Weak"];
