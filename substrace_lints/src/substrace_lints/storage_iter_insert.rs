use substrace_utils::diagnostics::span_lint_and_sugg;

use super::auxiliary::paths;
use rustc_errors::Applicability;
use rustc_hir::{def_id::DefId, Body, Expr};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};

declare_lint! {
    pub STORAGE_ITER_INSERT,
    Warn,
    "iterating over storage and mutating it at the same time causes undefined results"
}

impl_lint_pass!(StorageIterInsert => [STORAGE_ITER_INSERT]);

#[derive(Debug, Default)]
pub struct StorageIterInsert {
    storage_map_mutated: bool,
    storage_double_map_mutated: bool,
    iterating_over_storage_map: bool,
    iterating_over_storage_double_map: bool,
}

impl StorageIterInsert {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'hir> LateLintPass<'hir> for StorageIterInsert {
    fn check_expr(&mut self, cx: &LateContext<'hir>, expr: &'hir Expr<'hir>) {
        if let Some(fn_def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) {
            self.check_expr(cx, fn_def_id);
            self.finalize(cx, expr);
        }
    }

    fn check_body_post(&mut self, _: &LateContext<'hir>, _: &'hir Body<'hir>) {
        self.iterating_over_storage_map = false;
        self.storage_map_mutated = false;
        self.iterating_over_storage_double_map = false;
        self.storage_double_map_mutated = false;
    }
}

impl StorageIterInsert {
    fn finalize<'hir>(&mut self, cx: &LateContext<'hir>, expr: &'hir Expr<'hir>) {
        if self.storage_map_mutated && self.iterating_over_storage_map
            || self.storage_double_map_mutated && self.iterating_over_storage_double_map
        {
            span_lint_and_sugg(
                cx,
                STORAGE_ITER_INSERT,
                expr.span,
                "iterating and modifying storage has undefined results",
                "restructure code, or specifically describe why this isn't undefined behaviour",
                "#[allow(storage_iter_insert)]...".to_string(),
                Applicability::HasPlaceholders,
            );
        }
    }

    fn check_storage_double_map<'hir>(&mut self, cx: &LateContext<'hir>, fn_def_id: DefId) -> bool {
        if paths::is_storage_double_map(cx, fn_def_id) {
            if !self.iterating_over_storage_double_map {
                self.storage_double_map_mutated = false;
            }

            self.iterating_over_storage_double_map = true;
            true
        } else if paths::modifies_storage_double_map(cx, fn_def_id) {
            self.storage_double_map_mutated = true;
            true
        } else {
            false
        }
    }

    fn check_storage_map<'hir>(&mut self, cx: &LateContext<'hir>, fn_def_id: DefId) -> bool {
        if paths::is_storage_map(cx, fn_def_id) {
            if !self.iterating_over_storage_map {
                self.storage_map_mutated = false;
            }
            self.iterating_over_storage_map = true;
            true
        } else if paths::modifies_storage_map(cx, fn_def_id) {
            self.storage_map_mutated = true;
            true
        } else {
            false
        }
    }

    fn check_expr<'hir>(&mut self, cx: &LateContext<'hir>, fn_def_id: DefId) -> bool {
        self.check_storage_map(cx, fn_def_id) || self.check_storage_double_map(cx, fn_def_id)
    }
}