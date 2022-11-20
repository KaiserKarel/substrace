#![feature(test)] // compiletest_rs requires this attribute
#![feature(once_cell)]
#![feature(is_sorted)]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![warn(rust_2018_idioms, unused_lifetimes)]

use compiletest_rs as compiletest;
use compiletest_rs::common::Mode as TestMode;

use std::collections::HashMap;
use std::env::{self, remove_var, set_var, var_os};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use test_utils::IS_RUSTC_TEST_SUITE;

mod test_utils;

/// All crates used in UI tests are listed here
static TEST_DEPENDENCIES: &[&str] = &[
    "substrace_lints",
    "derive_new",
    "futures",
    "if_chain",
    "itertools",
    "quote",
    "regex",
    "serde",
    "serde_derive",
    "syn",
    "tokio",
    "parking_lot",
    "rustc_semver",
];

// Test dependencies may need an `extern crate` here to ensure that they show up
// in the depinfo file (otherwise cargo thinks they are unused)
#[allow(unused_extern_crates)]
extern crate derive_new;
#[allow(unused_extern_crates)]
extern crate futures;
#[allow(unused_extern_crates)]
extern crate if_chain;
#[allow(unused_extern_crates)]
extern crate itertools;
#[allow(unused_extern_crates)]
extern crate parking_lot;
#[allow(unused_extern_crates)]
extern crate quote;
#[allow(unused_extern_crates)]
extern crate rustc_semver;
#[allow(unused_extern_crates)]
extern crate substrace_lints;
#[allow(unused_extern_crates)]
extern crate syn;
#[allow(unused_extern_crates)]
extern crate tokio;

/// Produces a string with an `--extern` flag for all UI test crate
/// dependencies.
///
/// The dependency files are located by parsing the depinfo file for this test
/// module. This assumes the `-Z binary-dep-depinfo` flag is enabled. All test
/// dependencies must be added to Cargo.toml at the project root. Test
/// dependencies that are not *directly* used by this test module require an
/// `extern crate` declaration.
static EXTERN_FLAGS: LazyLock<String> = LazyLock::new(|| {
    let current_exe_depinfo = {
        let mut path = env::current_exe().unwrap();
        path.set_extension("d");
        fs::read_to_string(path).unwrap()
    };
    let mut crates: HashMap<&str, &str> = HashMap::with_capacity(TEST_DEPENDENCIES.len());
    for line in current_exe_depinfo.lines() {
        // each dependency is expected to have a Makefile rule like `/path/to/crate-hash.rlib:`
        let parse_name_path = || {
            if line.starts_with(char::is_whitespace) {
                return None;
            }
            let path_str = line.strip_suffix(':')?;
            let path = Path::new(path_str);
            if !matches!(path.extension()?.to_str()?, "rlib" | "so" | "dylib" | "dll") {
                return None;
            }
            let (name, _hash) = path.file_stem()?.to_str()?.rsplit_once('-')?;
            // the "lib" prefix is not present for dll files
            let name = name.strip_prefix("lib").unwrap_or(name);
            Some((name, path_str))
        };
        if let Some((name, path)) = parse_name_path() {
            if TEST_DEPENDENCIES.contains(&name) {
                // A dependency may be listed twice if it is available in sysroot,
                // and the sysroot dependencies are listed first. As of the writing,
                // this only seems to apply to if_chain.
                crates.insert(name, path);
            }
        }
    }
    let not_found: Vec<&str> = TEST_DEPENDENCIES
        .iter()
        .copied()
        .filter(|n| !crates.contains_key(n))
        .collect();
    assert!(
        not_found.is_empty(),
        "dependencies not found in depinfo: {:?}\n\
        help: Make sure the `-Z binary-dep-depinfo` rust flag is enabled\n\
        help: Try adding to dev-dependencies in Cargo.toml\n\
        help: Be sure to also add `extern crate ...;` to tests/compile-test.rs",
        not_found,
    );
    crates
        .into_iter()
        .map(|(name, path)| format!(" --extern {}={}", name, path))
        .collect()
});

fn base_config(test_dir: &str) -> compiletest::Config {
    let mut config = compiletest::Config {
        edition: Some("2021".into()),
        mode: TestMode::Ui,
        ..Default::default()
    };

    if let Ok(filters) = env::var("TESTNAME") {
        config.filters = filters.split(',').map(ToString::to_string).collect();
    }

    if let Some(path) = option_env!("RUSTC_LIB_PATH") {
        let path = PathBuf::from(path);
        config.run_lib_path = path.clone();
        config.compile_lib_path = path;
    }
    let current_exe_path = env::current_exe().unwrap();
    let deps_path = current_exe_path.parent().unwrap();
    let profile_path = deps_path.parent().unwrap();

    // Using `-L dependency={}` enforces that external dependencies are added with `--extern`.
    // This is valuable because a) it allows us to monitor what external dependencies are used
    // and b) it ensures that conflicting rlibs are resolved properly.
    let host_libs = option_env!("HOST_LIBS")
        .map(|p| format!(" -L dependency={}", Path::new(p).join("deps").display()))
        .unwrap_or_default();
    config.target_rustcflags = Some(format!(
        "--emit=metadata -Dwarnings -Zui-testing -L dependency={}{}{}",
        deps_path.display(),
        host_libs,
        &*EXTERN_FLAGS,
    ));

    config.src_base = Path::new("tests").join(test_dir);
    config.build_base = profile_path.join("test").join(test_dir);
    config.rustc_path = profile_path.join(if cfg!(windows) {
        "substrace-driver.exe"
    } else {
        "substrace-driver"
    });
    // config.bless = true; //overwrites stderr files to current error output.
    config
}

fn run_ui() {
    let mut config = base_config("ui");
    config.rustfix_coverage = true;
    // use tests/clippy.toml
    let _g = VarGuard::set("CARGO_MANIFEST_DIR", fs::canonicalize("tests").unwrap());
    let _threads = VarGuard::set(
        "RUST_TEST_THREADS",
        // if RUST_TEST_THREADS is set, adhere to it, otherwise override it
        env::var("RUST_TEST_THREADS").unwrap_or_else(|_| {
            std::thread::available_parallelism()
                .map_or(1, std::num::NonZeroUsize::get)
                .to_string()
        }),
    );
    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    set_var("CLIPPY_DISABLE_DOCS_LINKS", "true");
    run_ui();
}

/// Restores an env var on drop
#[must_use]
struct VarGuard {
    key: &'static str,
    value: Option<OsString>,
}

impl VarGuard {
    fn set(key: &'static str, val: impl AsRef<OsStr>) -> Self {
        let value = var_os(key);
        set_var(key, val);
        Self { key, value }
    }
}

impl Drop for VarGuard {
    fn drop(&mut self) {
        match self.value.as_deref() {
            None => remove_var(self.key),
            Some(value) => set_var(self.key, value),
        }
    }
}
