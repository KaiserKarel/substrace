use super::auxiliary::paths;
use substrace_utils::diagnostics::span_lint_and_sugg;
use substrace_utils::source::{snippet_opt, line_span};
use substrace_utils::match_def_path;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::source_map::Span;

use super::extrinsics_must_be_tagged::is_extrinsic_name;

declare_lint! {
    pub MISSING_TRANSACTIONAL,
    Warn,
    "All extrinsics must use the #[transactional] macro."
}

impl_lint_pass!(MissingTransactional => [MISSING_TRANSACTIONAL]);

#[derive(Clone, Default)]
pub struct MissingTransactional;

// Check if extrinsics use with_transaction
impl<'tcx> LateLintPass<'tcx> for MissingTransactional {
    fn check_fn(&mut self,
        cx: &LateContext<'tcx>,
        fn_kind: hir::intravisit::FnKind<'tcx>,
        _: &'tcx hir::FnDecl<'tcx>,
        fn_body: &'tcx hir::Body<'tcx>,
        _: Span,
        _: hir::hir_id::HirId) {
        if let hir::intravisit::FnKind::Method(rustc_span::symbol::Ident {name, ..}, fn_sig) = fn_kind 
            && is_extrinsic_name(name, cx) {

            // If it uses a with_transaction function as final expression, then no flag.
            if let hir::ExprKind::Block(body_block, _) = fn_body.value.kind
                && let Some(block_expr) = body_block.expr 
                && let hir::ExprKind::Call(call_expr, _) = block_expr.kind 
                && let hir::ExprKind::Path(qpath) = &call_expr.kind 
                && let hir::QPath::Resolved(_, path) = qpath
                && let hir::def::Res::Def(_, def_id) = path.res
                && match_def_path(cx, def_id, &paths::WITH_TRANSACTION) {

                return;
            }
            
            let fn_sig_span_str = snippet_opt(cx, line_span(cx, fn_sig.span)).expect("Should be a valid span.");

            let suggestion = format!("#[transactional]\n{fn_sig_span_str}");
            span_lint_and_sugg(
                cx,
                MISSING_TRANSACTIONAL,
                fn_sig.span,
                "substrace: Missing #[transactional] on extrinsic",
                "Add the #[transactional] macro to the top of your extrinsic definition",
                suggestion,
                Applicability::MachineApplicable, // Suggestion can be applied automatically
            );
        }
    }
}