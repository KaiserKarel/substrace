use substrace_utils::diagnostics::{span_lint_and_help, span_lint_and_sugg};
use substrace_utils::source::first_line_of_span;
use itertools::Itertools;
use rustc_ast::ast::{AttrKind, Attribute};
use rustc_ast::token::CommentKind;
use rustc_data_structures::fx::FxHashSet;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::source_map::{BytePos, Span};
use rustc_span::{sym, Pos};
use std::ops::Range;

declare_lint! {
    pub EXTRINSICS_MUST_BE_TAGGED,
    Warn,
    "Extrinsics must be tagged so that reordering of arguments will be noticed"
}

impl_lint_pass!(ExtrinsicsMustBeTagged => [EXTRINSICS_MUST_BE_TAGGED]);

#[derive(Clone, Default)]
pub struct ExtrinsicsMustBeTagged;

impl ExtrinsicsMustBeTagged {
    pub fn new() -> Self {
        Self
    }
}

impl<'tcx> LateLintPass<'tcx> for ExtrinsicsMustBeTagged {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        // Unimplemented
    }
}
