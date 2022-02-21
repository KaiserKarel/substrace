#![feature(rustc_private)]
#![allow(unused_extern_crates)]
// Since Dylint is dynamically loading linters, we cannot allow for panics.
#![warn(
    clippy::disallowed_method,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unwrap_used,
    clippy::panic
)]
dylint_linting::dylint_library!();

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_hir_pretty;
extern crate rustc_index;
extern crate rustc_infer;
extern crate rustc_lexer;
extern crate rustc_lint;
extern crate rustc_middle;
// smoelius: The renaming of `rustc_mir` has caused problems for the Dylint tests. The crate is used
// by only one Clippy lint, `redundant_clone`. So I am disabling it for now.
// extern crate rustc_mir_dataflow;
extern crate rustc_parse;
extern crate rustc_parse_format;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;
extern crate rustc_typeck;

mod linters;

#[doc(hidden)]
#[no_mangle]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[linters::no_panics::PANICS]);
    lint_store.register_late_pass(|| Box::new(linters::no_panics::Panics::new()));
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
