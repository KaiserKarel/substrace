#![feature(array_windows)]
#![feature(binary_heap_into_iter_sorted)]
#![feature(box_patterns)]
#![feature(control_flow_enum)]
#![feature(drain_filter)]
#![feature(iter_intersperse)]
#![feature(let_chains)]
#![feature(let_else)]
#![feature(lint_reasons)]
#![feature(never_type)]
#![feature(once_cell)]
#![feature(rustc_private)]
#![feature(stmt_expr_attributes)]
#![feature(is_some_and)]
#![recursion_limit = "512"]
#![cfg_attr(feature = "deny-warnings", deny(warnings))]
#![allow(clippy::missing_docs_in_private_items, clippy::must_use_candidate)]
#![warn(trivial_casts, trivial_numeric_casts)]
// warn on lints, that are included in `rust-lang/rust`s bootstrap
#![warn(rust_2018_idioms, unused_lifetimes)]
// warn on rustc internal lints
#![warn(rustc::internal)]
// Disable this rustc lint for now, as it was also done in rustc
#![allow(rustc::potential_query_instability)]

// FIXME: switch to something more ergonomic here, once available.
// (Currently there is no way to opt into sysroot crates without `extern crate`.)
extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

#[macro_use]
extern crate substrace_utils;

use rustc_data_structures::fx::FxHashSet;
use rustc_lint::LintId;
use rustc_semver::RustcVersion;
use rustc_session::Session;

/// Macro used to declare a Substrace lint.
///
/// Every lint declaration consists of 4 parts:
///
/// 1. The documentation, which is used for the website
/// 2. The `LINT_NAME`. See [lint naming][lint_naming] on lint naming conventions.
/// 3. The `lint_level`, which is a mapping from *one* of our lint groups to `Allow`, `Warn` or
///    `Deny`. The lint level here has nothing to do with what lint groups the lint is a part of.
/// 4. The `description` that contains a short explanation on what's wrong with code where the
///    lint is triggered.
///
/// Currently the categories `style`, `correctness`, `suspicious`, `complexity` and `perf` are
/// enabled by default. As said in the README.md of this repository, if the lint level mapping
/// changes, please update README.md.
///
/// # Example
///
/// ```
/// #![feature(rustc_private)]
/// extern crate rustc_session;
/// use rustc_session::declare_tool_lint;
/// use substrace_lints::declare_substrace_lint;
///
/// declare_substrace_lint! {
///     /// ### What it does
///     /// Checks for ... (describe what the lint matches).
///     ///
///     /// ### Why is this bad?
///     /// Supply the reason for linting the code.
///     ///
///     /// ### Example
///     /// ```rust
///     /// Insert a short example of code that triggers the lint
///     /// ```
///     ///
///     /// Use instead:
///     /// ```rust
///     /// Insert a short example of improved code that doesn't trigger the lint
///     /// ```
///     pub LINT_NAME,
///     pedantic,
///     "description"
/// }
/// ```
/// [lint_naming]: https://rust-lang.github.io/rfcs/0344-conventions-galore.html#lints
#[macro_export]
macro_rules! declare_substrace_lint {
    { $(#[$attr:meta])* pub $name:tt, style, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Warn, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, correctness, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Deny, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, suspicious, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Warn, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, complexity, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Warn, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, perf, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Warn, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, pedantic, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Allow, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, restriction, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Allow, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, cargo, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Allow, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, nursery, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Allow, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, internal, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Allow, $description, report_in_external_macro: true
        }
    };
    { $(#[$attr:meta])* pub $name:tt, internal_warn, $description:tt } => {
        declare_tool_lint! {
            $(#[$attr])* pub substrace::$name, Warn, $description, report_in_external_macro: true
        }
    };
}

#[cfg(feature = "internal")]
pub mod deprecated_lints;
#[cfg_attr(feature = "internal", allow(clippy::missing_clippy_version_attribute))]
mod utils;

// begin lints modules, do not remove this comment, it’s used in `update_lints`
// end lints modules, do not remove this comment, it’s used in `update_lints`

mod substrace_lints;
use substrace_lints::{
    extrinsics_must_be_tagged,
    missing_security_doc,
    missing_transactional,
    no_panics,
};

pub use crate::utils::conf::Conf;
use crate::utils::conf::{format_error, TryConf};

/// Register all pre expansion lints
///
/// Pre-expansion lints run before any macro expansion has happened.
///
/// Note that due to the architecture of the compiler, currently `cfg_attr` attributes on crate
/// level (i.e `#![cfg_attr(...)]`) will still be expanded even when using a pre-expansion pass.
///
/// Used in `./src/driver.rs`.
pub fn register_pre_expansion_lints(store: &mut rustc_lint::LintStore, sess: &Session, conf: &Conf) {

}

#[doc(hidden)]
pub fn read_conf(sess: &Session) -> Conf {
    let file_name = match utils::conf::lookup_conf_file() {
        Ok(Some(path)) => path,
        Ok(None) => return Conf::default(),
        Err(error) => {
            sess.struct_err(&format!("error finding Substrace's configuration file: {}", error))
                .emit();
            return Conf::default();
        },
    };

    let TryConf { conf, errors, warnings } = utils::conf::read(&file_name);
    // all conf errors are non-fatal, we just use the default conf in case of error
    for error in errors {
        sess.err(&format!(
            "error reading Substrace's configuration file `{}`: {}",
            file_name.display(),
            format_error(error)
        ));
    }

    for warning in warnings {
        sess.struct_warn(&format!(
            "error reading Substrace's configuration file `{}`: {}",
            file_name.display(),
            format_error(warning)
        ))
        .emit();
    }

    conf
}

/// Register all lints and lint groups with the rustc plugin registry
///
/// Used in `./src/driver.rs`.
pub fn register_plugins(store: &mut rustc_lint::LintStore, sess: &Session, conf: &Conf) {

    // Allows to enable or disable lints in code
    // store.register_lints(&[no_panics::PANICS]);

    store.register_late_pass(|_| Box::new(extrinsics_must_be_tagged::ExtrinsicsMustBeTagged));
    store.register_late_pass(|_| Box::new(missing_security_doc::DocMarkdown));
    store.register_late_pass(|_| Box::new(missing_transactional::MissingTransactional));
    // store.register_late_pass(|_| Box::new(no_panics::Panics::new()));
}

// only exists to let the dogfood integration test works.
// Don't run substrace as an executable directly
#[allow(dead_code)]
fn main() {
    panic!("Please use the cargo-substrace executable");
}
