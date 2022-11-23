#![allow(dead_code)] // see https://github.com/rust-lang/rust/issues/46379

use std::path::PathBuf;
use std::sync::LazyLock;

pub static CARGO_SUBSTRACE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = std::env::current_exe().unwrap();
    assert!(path.pop()); // deps
    path.set_file_name("cargo-substrace");
    path
});

pub const IS_RUSTC_TEST_SUITE: bool = option_env!("RUSTC_TEST_SUITE").is_some();
