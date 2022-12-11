use super::auxiliary::paths;
use substrace_utils::diagnostics::{span_lint_and_help, span_lint_and_sugg};
use substrace_utils::source::{first_line_of_span, snippet_opt, line_span, span_extend_prev_str};
use substrace_utils::match_def_path;
use itertools::Itertools;
use rustc_ast::ast::{AttrKind, Attribute};
use rustc_ast::AstDeref;
use rustc_ast::ast as ast;
use rustc_ast::token::CommentKind;
use rustc_data_structures::fx::FxHashSet;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::source_map::{BytePos, Span, SourceMap, original_sp};
use rustc_span::{sym, Pos, DUMMY_SP};
use std::ops::Range;

declare_lint! {
    pub EXTRINSICS_MUST_BE_TAGGED,
    Deny,
    "Extrinsics need to be tagged using the pallet::call_index macro to prevent accidental reordering"
}

impl_lint_pass!(ExtrinsicsMustBeTagged => [EXTRINSICS_MUST_BE_TAGGED]);

#[derive(Clone, Default)]
pub struct ExtrinsicsMustBeTagged;

impl ExtrinsicsMustBeTagged {
    pub fn new() -> Self {
        Self
    }
}

// TODO: Check pallet::call_index is not in a comment? Relevant if it is 0?! But not a good idea to implement now, since we're most likely not gonna keep using this tacting of checking plain text.
impl<'tcx> LateLintPass<'tcx> for ExtrinsicsMustBeTagged {
    fn check_fn(&mut self,
                cx: &LateContext<'tcx>,
                fn_kind: hir::intravisit::FnKind<'tcx>,
                fn_decl: &'tcx hir::FnDecl<'tcx>,
                fn_body: &'tcx hir::Body<'tcx>,
                span: Span,
                hir_id: hir::hir_id::HirId) {
        if let hir::intravisit::FnKind::Method(rustc_span::symbol::Ident {name, ..}, fn_sig) = fn_kind 
            && is_extrinsic_name(name, cx)
            && let index = get_index_in_expansion(name, cx)
            && !get_index_in_macro(cx, fn_sig).is_some_and(|i| i == index) {

            let fn_sig_span_line = snippet_opt(cx, line_span(cx, fn_sig.span)); // also extend to include whitespace
            let suggestion = format!("#[pallet::call_index({})]\n{}", index, fn_sig_span_line.unwrap()); // We should always be able to find the span. If we cannot, there is a bug in this code, so panic and bug report.

            span_lint_and_sugg(
                cx,
                EXTRINSICS_MUST_BE_TAGGED,
                fn_sig.span,
                "Extrinsic not tagged",
                "Add the #[pallet::call_index(...)] macro to the top of your extrinsic definition",
                suggestion,
                Applicability::MachineApplicable, // Suggestion can be applied automatically
            );
        }
    }
}

//It checks if text above the function signature contains a call_index macro call.
pub fn get_index_in_macro<'tcx>(cx: &LateContext<'tcx>, fn_sig: &hir::FnSig<'tcx>) -> Option<u8> {
    let new_span = span_extend_prev_str(cx, fn_sig.span, "#[pallet::call_index(")?;
    let snip = snippet_opt(cx, new_span)?;

    (&snip[1..]).split(')').next()?.parse::<u8>().ok() // Removes first character ('(') and parses number until closing parenthesis
}

// Gets call index for extrinsic from code expanded by pallet::call macro
pub fn get_index_in_expansion<'tcx>(func_name_symbol: rustc_span::symbol::Symbol, cx: &LateContext<'tcx>) -> u8 {
    for item_id_ in cx.tcx.hir().root_module().item_ids.into_iter() { // Gets entire file
        if let Some(hir::Node::Item(parent_node)) = cx.tcx.hir().find(item_id_.hir_id())
        && let hir::ItemKind::Mod(mod_item) = parent_node.kind { //pallet module

            for mod_item_id in mod_item.item_ids.into_iter() {
                let node_in_mod = cx.tcx.hir().find(mod_item_id.hir_id());

                if let Some(hir::Node::Item(item_in_mod)) = node_in_mod
                    && let hir::ItemKind::Enum(hir::EnumDef{variants}, _) = &item_in_mod.kind { // Look for implement blocks

                    for variant in variants.into_iter() {
                        if variant.ident.name == func_name_symbol {
                                
                            for attrib in cx.tcx.hir().attrs(variant.hir_id).into_iter() {
                                if let ast::AttrKind::Normal(normalAttrib) = &attrib.kind
                                    && let ast::NormalAttr{item, ..} = normalAttrib.ast_deref()
                                    && let Some(ast::MetaItemKind::List(list_items)) = item.meta_kind() 
                                    && list_items.len() > 0 
                                    && let ast::NestedMetaItem::MetaItem(meta_item) = &list_items[0]
                                    && let ast::MetaItemKind::NameValue(lit) = &meta_item.kind 
                                    && let ast::LitKind::Int(the_index_wow, _) = lit.kind {

                                    return the_index_wow as u8; //casting is save here
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    panic!("Call index not present in expansion");
}

// Check if func_name_symbol is in the list of functions created by pallet::call macro that are exposed as extrinsics
// TODO: This is also used by missing_transactional. Should probably be moved to some other helper functions place?
pub fn is_extrinsic_name<'tcx>(func_name_symbol: rustc_span::symbol::Symbol, cx: &LateContext<'tcx>) -> bool {
    for item_id_ in cx.tcx.hir().root_module().item_ids.into_iter() { // Gets entire file
        if let Some(hir::Node::Item(parent_node)) = cx.tcx.hir().find(item_id_.hir_id())
        && let hir::ItemKind::Mod(mod_item) = parent_node.kind { //pallet module

            for mod_item_id in mod_item.item_ids.into_iter() {
                let node_in_mod = cx.tcx.hir().find(mod_item_id.hir_id());

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
    }

    false // func_name not found in get_call_names
}
