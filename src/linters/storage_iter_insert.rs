use clippy_utils::{diagnostics::span_lint_and_sugg, match_def_path};


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
        if is_storage_double_map(cx, fn_def_id) {
            self.iterating_over_storage_double_map = true;
            true
        } else if modifies_storage_double_map(cx, fn_def_id) {
            self.storage_double_map_mutated = true;
            true
        } else {
            false
        }
    }

    fn check_storage_map<'hir>(&mut self, cx: &LateContext<'hir>, fn_def_id: DefId) -> bool {
        if is_storage_map(cx, fn_def_id) {
            self.iterating_over_storage_map = true;
            true
        } else if modifies_storage_map(cx, fn_def_id) {
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

fn is_storage_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_ITER)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_DRAIN)
}

fn is_storage_double_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(
        cx,
        fn_def_id,
        &paths::ITERABLE_STORAGE_DOUBLE_MAP_ITER_PREFIX,
    ) || match_def_path(
        cx,
        fn_def_id,
        &paths::ITERABLE_STORAGE_DOUBLE_MAP_DRAIN_PREFIX,
    ) || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_ITER)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_DRAIN)
}

fn modifies_storage_double_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_SWAP)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_TAKE)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_INSERT)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_REMOVE)
        || match_def_path(
            cx,
            fn_def_id,
            &paths::ITERABLE_STORAGE_DOUBLE_MAP_REMOVE_PREFIX,
        )
        || match_def_path(
            cx,
            fn_def_id,
            &paths::ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE,
        )
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_MUTATE)
        || match_def_path(
            cx,
            fn_def_id,
            &paths::ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE_EXISTS,
        )
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_DOUBLE_MAP_APPEND)
}

fn modifies_storage_map(cx: &LateContext<'_>, fn_def_id: DefId) -> bool {
    match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_SWAP)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_REMOVE)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_TAKE)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_APPEND)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_INSERT)
        || match_def_path(cx, fn_def_id, &paths::ITERABLE_STORAGE_MAP_MIGRATE_KEY)
        || match_def_path(
            cx,
            fn_def_id,
            &paths::ITERABLE_STORAGE_MAP_MIGRATE_KEY_FROM_BLAKE,
        )
}

mod paths {
    pub const ITERABLE_STORAGE_MAP_ITER: [&str; 4] =
        ["frame_support", "storage", "IterableStorageMap", "iter"];
    pub const ITERABLE_STORAGE_MAP_DRAIN: [&str; 4] =
        ["frame_support", "storage", "IterableStorageMap", "drain"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_ITER_PREFIX: [&str; 4] = [
        "frame_support",
        "storage",
        "IterableStorageDoubleMap",
        "iter_prefix",
    ];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_DRAIN_PREFIX: [&str; 4] = [
        "frame_support",
        "storage",
        "IterableStorageDoubleMap",
        "drain_prefix",
    ];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_ITER: [&str; 4] = [
        "frame_support",
        "storage",
        "IterableStorageDoubleMap",
        "iter",
    ];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_DRAIN: [&str; 4] = [
        "frame_support",
        "storage",
        "IterableStorageDoubleMap",
        "drain",
    ];
    pub const ITERABLE_STORAGE_MAP_INSERT: [&str; 4] =
        ["frame_support", "storage", "StorageMap", "insert"];
    pub const ITERABLE_STORAGE_MAP_SWAP: [&str; 4] =
        ["frame_support", "storage", "StorageMap", "swap"];
    pub const ITERABLE_STORAGE_MAP_REMOVE: [&str; 4] =
        ["frame_support", "storage", "StorageMap", "remove"];
    pub const ITERABLE_STORAGE_MAP_TAKE: [&str; 4] =
        ["frame_support", "storage", "StorageMap", "take"];
    pub const ITERABLE_STORAGE_MAP_APPEND: [&str; 4] =
        ["frame_support", "storage", "StorageMap", "append"];
    pub const ITERABLE_STORAGE_MAP_MIGRATE_KEY: [&str; 4] =
        ["frame_support", "storage", "StorageMap", "migrate_key"];
    pub const ITERABLE_STORAGE_MAP_MIGRATE_KEY_FROM_BLAKE: [&str; 4] = [
        "frame_support",
        "storage",
        "StorageMap",
        "migrate_key_from_blake",
    ];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_SWAP: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "swap"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_TAKE: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "take"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_INSERT: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "insert"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_REMOVE: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "remove"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_REMOVE_PREFIX: [&str; 4] = [
        "frame_support",
        "storage",
        "StorageDoubleMap",
        "remove_prefix",
    ];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_MUTATE: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "mutate"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_APPEND: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "append"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE: [&str; 4] =
        ["frame_support", "storage", "StorageDoubleMap", "try_mutate"];
    pub const ITERABLE_STORAGE_DOUBLE_MAP_TRY_MUTATE_EXISTS: [&str; 4] = [
        "frame_support",
        "storage",
        "StorageDoubleMap",
        "try_mutate_exists",
    ];
}
