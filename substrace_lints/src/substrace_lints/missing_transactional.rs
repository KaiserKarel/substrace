use super::auxiliary::paths;
use substrace_utils::diagnostics::{span_lint_and_help, span_lint_and_sugg};
use substrace_utils::source::{first_line_of_span, snippet_opt};
use substrace_utils::match_def_path;
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
    pub MISSING_TRANSACTIONAL,
    Warn,
    "All extrinsics must use the #[transactional] macro."
}

impl_lint_pass!(MissingTransactional => [MISSING_TRANSACTIONAL]);

#[derive(Clone, Default)]
pub struct MissingTransactional;

impl MissingTransactional {
    pub fn new() -> Self {
        Self
    }
}

// Check if extrinsics use with_transaction
impl<'tcx> LateLintPass<'tcx> for MissingTransactional {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        if let hir::ItemKind::Impl(impl_block) = item.kind {
            for impl_item in impl_block.items.into_iter() {
                if let Some(hir::Node::ImplItem(item_in_impl)) = cx.tcx.hir().find(impl_item.id.hir_id()) //Using RFC-2497 if-let-chain syntax
                    && let hir::ImplItemKind::Fn(fn_sig, body_id) = &item_in_impl.kind
                    && let this_fn_name_symbol = item_in_impl.ident.name 
                    && is_extrinsic_name(this_fn_name_symbol, cx, item) // TODO: Vanaf hieronder moet alles dus kloppen!

                    && let fn_body_in_impl = cx.tcx.hir().find(body_id.hir_id) // Body of function in impl block
                    && let Some(hir::Node::Expr(body_expr)) = fn_body_in_impl
                    && let hir::ExprKind::Block(body_block, _) = body_expr.kind   // TODO: What if it isn't a block? We should use || here I think.
                    && let Some(block_expr) = body_block.expr 
                    && let hir::ExprKind::Call(call_expr, _) = block_expr.kind 
                    && let hir::ExprKind::Path(qpath) = &call_expr.kind 
                    && let hir::QPath::Resolved(_, path) = qpath
                    && let hir::def::Res::Def(_, def_id) = path.res
                    && !match_def_path(cx, def_id, &paths::WITH_TRANSACTION) // Now check if with_transaction function is missing
                    && let Some(fn_sig_span_str) = snippet_opt(cx, fn_sig.span) {

                    let suggestion = format!("#[transactional]
        {fn_sig_span_str}");
                    span_lint_and_sugg(
                        cx,
                        MISSING_TRANSACTIONAL,
                        fn_sig.span,
                        "Missing #[transactional] on extrinsic",
                        "Add the #[transactional] macro to the top of your extrinsic definition",
                        suggestion,
                        Applicability::MachineApplicable, // Suggestion can be applied automatically
                    );
                    break;
                }
            }
        }
    }
}

// Check if func_name_symbol is in the list of functions created by pallet::call macro that are exposed as extrinsics
// "item" must be a child of the mod level that get_call_names is also part of
pub fn is_extrinsic_name<'tcx>(func_name_symbol: rustc_span::symbol::Symbol, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) -> bool {
    if let Some(parent_hir_id) = cx.tcx.hir().find_parent_node(item.hir_id()) // pallet module hir_id
        && let Some(hir::Node::Item(parent_node)) = cx.tcx.hir().find(parent_hir_id)
        && let hir::ItemKind::Mod(mod_item) = parent_node.kind {

        for mod_item_id in mod_item.item_ids.into_iter() {
            let mod_item_hir_id = mod_item_id.hir_id();
            let node_in_mod = cx.tcx.hir().find(mod_item_hir_id);

            if let Some(hir::Node::Item(item_in_mod)) = node_in_mod
                && let hir::ItemKind::Impl(impl_block) = item_in_mod.kind { // Look for implement blocks

                for item_ref in impl_block.items.into_iter() {
                    if item_ref.ident.as_str() == "get_call_names"
                        && let item_ref_hir_id = item_ref.id.hir_id()
                        && let item_ref_node = cx.tcx.hir().find(item_ref_hir_id)
                        && let Some(hir::Node::ImplItem(item_in_impl)) = item_ref_node
                        && let hir::ImplItemKind::Fn(_, body_id) = item_in_impl.kind
                        && let body_node = cx.tcx.hir().find(body_id.hir_id) // Body of get_call_names
                        && let Some(hir::Node::Expr(body_expr)) = body_node
                        && let hir::ExprKind::Block(body_block, _) = body_expr.kind
                        && let Some(block_expr) = body_block.expr
                        && let hir::ExprKind::AddrOf(_, _, ref_expr) = block_expr.kind
                        && let hir::ExprKind::Array(extrinsic_name_exprs) = ref_expr.kind {

                        println!("Item_ref: {:?}", item_ref); // TODO: This path is of course different every time, since the path to get_call_names depends on the pallet you're in...

                        for extrinsic_name_expr in extrinsic_name_exprs.into_iter() { // Check for each extrinsic whether func_name_symbol is in it
                            if let hir::ExprKind::Lit(spanned) = &extrinsic_name_expr.kind
                                && let rustc_ast::ast::LitKind::Str(extrinsic_symbol, _) = spanned.node
                                && func_name_symbol == extrinsic_symbol { 
                                
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false // func_name not found in get_call_names
}