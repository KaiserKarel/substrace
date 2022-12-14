use substrace_utils::diagnostics::span_lint_and_sugg;
use enumset::{EnumSet, EnumSetType};
use if_chain::if_chain;
use rustc_ast::{self as ast, *};
use rustc_errors::Applicability;
use substrace_utils::source::snippet_opt;

use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::{hygiene::SyntaxContext, symbol::sym, BytePos, Span};
use std::str::FromStr;

declare_lint! {
    pub PANICS,
    Warn,
    "any type of panicking code may not be present in the runtime."
}

impl_lint_pass!(Panics => [PANICS]);

/// Clippy attributes which must be enabled in crates made for the substrate runtime or pallets.
#[derive(EnumSetType, Debug)]
pub enum RequiredAttributes {
    DisallowedMethod,
    IndexingSlicing,
    Todo,
    UnwrapUsed,
    Panic,
}

impl FromStr for RequiredAttributes {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use RequiredAttributes::*;

        match s {
            "disallowed_methods" => Ok(DisallowedMethod),
            "indexing_slicing" => Ok(IndexingSlicing),
            "todo" => Ok(Todo),
            "unwrap_used" => Ok(UnwrapUsed),
            "panic" => Ok(Panic),
            _ => Err(()),
        }
    }
}

impl ToString for RequiredAttributes {
    fn to_string(&self) -> String {
        let s = match self {
            RequiredAttributes::DisallowedMethod => "disallowed_methods",
            RequiredAttributes::IndexingSlicing => "indexing_slicing",
            RequiredAttributes::Todo => "todo",
            RequiredAttributes::UnwrapUsed => "unwrap_used",
            RequiredAttributes::Panic => "panic",
        };
        s.to_owned()
    }
}

fn format_help(diff: EnumSet<RequiredAttributes>) -> String {
    let mut help = String::from("#![warn(\n");
    for item in diff {
        help.push_str(&format!("    clippy::{},\n", item.to_string()));
    }
    help.push_str(")]\n");
    help
}

#[derive(Debug, Default)]
pub struct Panics {
    seen_attributes: EnumSet<RequiredAttributes>,
}

impl Panics {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'hir> LateLintPass<'hir> for Panics {
    fn check_attribute(&mut self, _: &LateContext<'_>, attr: &ast::Attribute) {
        if let Some(items) = &attr.meta_item_list() {
            if let Some(ident) = attr.ident() {
                let ident = &*ident.as_str();
                if matches!(ident, "warn" | "deny" | "forbid") {
                    items.iter().for_each(|item| {
                        if let Some(attr1) = extract_clippy_lint(item) {
                            self.seen_attributes.insert(attr1);
                        }
                    });
                }
            }
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'hir>) {
        let diff = EnumSet::<RequiredAttributes>::all().difference(self.seen_attributes);

        let span = Span::new(BytePos(0), BytePos(1), SyntaxContext::root(), None);

        if !diff.is_empty()
            && let Some(span_text) = snippet_opt(cx, span) {
            
            let mut suggestion = format_help(diff);
            suggestion.push_str(&span_text);

            span_lint_and_sugg(
                cx,
                PANICS,
                span,
                "substrace: clippy must be configured to warn or deny about any panicking code TODOTODO",
                "insert attributes at the root of the crate",
                suggestion,
                Applicability::MachineApplicable,
            );
        }
    }
}

fn extract_clippy_lint(lint: &NestedMetaItem) -> Option<RequiredAttributes> {
    if_chain! {
        if let Some(meta_item) = lint.meta_item();
        if meta_item.path.segments.len() > 1;
        if let tool_name = meta_item.path.segments[0].ident;
        if tool_name.name.as_str() == "clippy";
        let lint_name = meta_item.path.segments.last().unwrap().ident.name;
        then {
            return RequiredAttributes::from_str(&lint_name.as_str()).ok();
        }
    }
    None
}
