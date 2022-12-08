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
    "Using the Identity or Twox64Concat hasher requires a doc describing it's secure usage"
}

impl_lint_pass!(ExtrinsicsMustBeTagged => [EXTRINSICS_MUST_BE_TAGGED]);

#[derive(Clone, Default)]
pub struct ExtrinsicsMustBeTagged;

impl ExtrinsicsMustBeTagged {
    pub fn new() -> Self {
        Self
    }
}

//Plan:
// - Check if the Impl Itemkind is decorated with pallet::call (hopefully we can do this before macro expansion)
// - Check if all functions inside this implementation are decorated with "some tag"

// TODO: We are now checking if the last expression in the function is "with_transaction", but we should also check nothing (except for use statements) happen before that I guess

// New plan: Check if the Impl has selftype pallet and trait config. only then check if with_transaction is used.
impl<'tcx> LateLintPass<'tcx> for ExtrinsicsMustBeTagged {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        if let hir::ItemKind::Impl(the_impl) = item.kind {

            
            for the_impl_item in the_impl.items.into_iter() {
                if let Some(hir::Node::ImplItem(implItem)) = cx.tcx.hir().find(the_impl_item.id.hir_id()) {
                    if let hir::ImplItemKind::Fn(pff, bodyId) = &implItem.kind {
                        // hmmm... this just gives the comments.
                        // We need to find the bodies of these functions. For that we need hir::Item. Then its ItemKind (of type Fn), then we get a BodyId, which can give an hirID, which can hopefully give the body itself.
                        let body = cx.tcx.hir().find(bodyId.hir_id);

                        if let Some(hir::Node::Expr(expr)) = body {
                            if let hir::ExprKind::Block(block, _) = expr.kind {
                                if let Some(expr2) = block.expr {
                                    if let hir::ExprKind::Call(expr3, _) = expr2.kind {
                                        if let hir::ExprKind::Path(qpath) = &expr3.kind {
                                            if let hir::QPath::Resolved(_, path) = qpath {
                                                // println!("ATTRSS: {:?}\n", path);
                                                // println!("AND: {:?}\n", implItem.ident.as_str());

                                                
                                                let to_be_tagged_name = implItem.ident.as_str();
                                                // blabla if it is not "with_transaction" && it must_be_tagged, then report error.
                                                if must_be_tagged(to_be_tagged_name, cx, item) {
                                                    // Now check that it is not already using with_transaction:
                                                    for segment in path.segments {
                                                        if segment.ident.as_str() != "with_transaction" {
                                                            println!("MISSING TRANSACTIONAL MACRO ON FUNCTION {:?}", to_be_tagged_name);
                                                            break;
                                                        }
                                                    }
                                                }
                                                // This finds the with_transactional function. We must fix that it only finds it for extrinsics.
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Check if func_name is in the list of functions created by pallet::call macro that are exposed as extrinsics
pub fn must_be_tagged<'tcx>(func_name: &str, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) -> bool {
    let prnt_hir_id = cx.tcx.hir().find_parent_node(item.hir_id()).unwrap();
    if let Some(hir::Node::Item(hir::Item{kind, ..})) = cx.tcx.hir().find(prnt_hir_id) {
        if let hir::ItemKind::Mod(hir::Mod{item_ids, ..}) = kind {
            for item_id in item_ids.into_iter() {
                let mod_item_hir_id = item_id.hir_id();
                let mod_node = cx.tcx.hir().find(mod_item_hir_id);

                if let Some(hir::Node::Item(item)) = mod_node {
                    if let hir::ItemKind::Impl(the_impl) = item.kind {
                        for item_ref in the_impl.items.into_iter() {
                            if item_ref.ident.as_str() == "get_call_names" {
                                let item_ref_hir_id = item_ref.id.hir_id();
                                let item_ref_node = cx.tcx.hir().find(item_ref_hir_id);

                                if let Some(hir::Node::ImplItem(hir::ImplItem{kind, ..})) = item_ref_node {
                                    if let hir::ImplItemKind::Fn(_, bodyId) = kind {
                                        let body_node = cx.tcx.hir().find(bodyId.hir_id);

                                        if let Some(hir::Node::Expr(hir::Expr{kind, ..})) = body_node {
                                            if let hir::ExprKind::Block(hir::Block{expr, ..}, _) = kind {
                                                if let Some(expr2) = expr {
                                                    if let hir::ExprKind::AddrOf(_, _, expr3) = expr2.kind {
                                                        if let hir::ExprKind::Array(exprs) = expr3.kind {
                                                            for func_name_expr in exprs.into_iter() {
                                                                if let hir::ExprKind::Lit(
                                                                    rustc_span::source_map::Spanned{node, ..}) = &func_name_expr.kind {
                                                                    if let rustc_ast::ast::LitKind::Str(my_str, _) = node {
                                                                        if func_name == my_str.as_str() {
                                                                            return true;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
        }
    }
    false
}