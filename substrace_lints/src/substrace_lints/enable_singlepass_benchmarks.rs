use super::auxiliary::paths;
use substrace_utils::diagnostics::span_lint_and_sugg;
use substrace_utils::source::{snippet_opt, line_span};
use substrace_utils::match_def_path;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_ast as ast;
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::source_map::Span;

use super::extrinsics_must_be_tagged::is_extrinsic_name;

declare_lint! {
    pub ENABLE_SINGLEPASS_BENCHMARKS,
    Warn,
    "TODO"
}

impl_lint_pass!(EnableSinglepassBenchmarks => [ENABLE_SINGLEPASS_BENCHMARKS]);

#[derive(Clone, Default)]
pub struct EnableSinglepassBenchmarks;

// Check if extrinsics use with_transaction
impl EarlyLintPass for EnableSinglepassBenchmarks {
    fn check_attribute(&mut self, _: &EarlyContext<'_>, attr: &ast::Attribute) {
        println!("Check attribute: {:?}", attr);
    }
}