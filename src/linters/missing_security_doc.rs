use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::match_def_path;
use clippy_utils::source::first_line_of_span;
use itertools::Itertools;
use rustc_ast::ast::{AttrKind, Attribute};
use rustc_ast::token::CommentKind;
use rustc_data_structures::fx::FxHashSet;
use rustc_hir as hir;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_lint, impl_lint_pass};
use rustc_span::edition::Edition;
use rustc_span::source_map::{BytePos, Span};
use rustc_span::{sym, Pos};
use std::ops::Range;

pub const STORAGE_MAP: [&str; 3] =
        ["frame_support", "storage", "StorageMap"];

declare_lint! {
    pub MISSING_SECURITY_DOC,
    Deny,
    "Using the Identity or Twox64Concat hasher requires a doc describing it's secure usage"
}

impl_lint_pass!(DocMarkdown => [MISSING_SECURITY_DOC]);

#[derive(Clone)]
pub struct DocMarkdown {
    suspicious_hash_functions: FxHashSet<String>,
    in_trait_impl: bool,
}

impl DocMarkdown {
    pub fn new(suspicious_hash_functions: FxHashSet<String>) -> Self {
        Self {
            suspicious_hash_functions,
            in_trait_impl: false,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for DocMarkdown {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        match item.kind {
            hir::ItemKind::TyAlias(ty, ..) => {
                if let hir::TyKind::TraitObject(ptr, ..) = ty.kind {
                    if let hir::def::Res::Def(_, id) = ptr[0].trait_ref.path.res {
                        if match_def_path(cx, id, &STORAGE_MAP) {
                            panic!("found the storage map")
                        }
                    }
                }
            }
            _ => (),
        }
        // let attrs = cx.tcx.hir().attrs(item.hir_id());
        // let headers = check_attrs(cx, &self.valid_idents, attrs);
    }
}

// fn lint_for_missing_headers<'tcx>(
//     cx: &LateContext<'tcx>,
//     def_id: LocalDefId,
//     span: impl Into<MultiSpan> + Copy,
//     sig: &hir::FnSig<'_>,
//     headers: DocHeaders,
//     body_id: Option<hir::BodyId>,
//     panic_span: Option<Span>,
// ) {
//     if !cx.access_levels.is_exported(def_id) {
//         return; // Private functions do not require doc comments
//     }

//     // do not lint if any parent has `#[doc(hidden)]` attribute (#7347)
//     if cx
//         .tcx
//         .hir()
//         .parent_iter(cx.tcx.hir().local_def_id_to_hir_id(def_id))
//         .any(|(id, _node)| is_doc_hidden(cx.tcx.hir().attrs(id)))
//     {
//         return;
//     }

//     if !headers.safety && sig.header.unsafety == hir::Unsafety::Unsafe {
//         span_lint(
//             cx,
//             MISSING_SAFETY_DOC,
//             span,
//             "unsafe function's docs miss `# Safety` section",
//         );
//     }
//     if !headers.panics && panic_span.is_some() {
//         span_lint_and_note(
//             cx,
//             MISSING_PANICS_DOC,
//             span,
//             "docs for function which may panic missing `# Panics` section",
//             panic_span,
//             "first possible panic found here",
//         );
//     }
//     if !headers.errors {
//         let hir_id = cx.tcx.hir().local_def_id_to_hir_id(def_id);
//         if is_type_diagnostic_item(cx, return_ty(cx, hir_id), sym::Result) {
//             span_lint(
//                 cx,
//                 MISSING_ERRORS_DOC,
//                 span,
//                 "docs for function returning `Result` missing `# Errors` section",
//             );
//         } else {
//             if_chain! {
//                 if let Some(body_id) = body_id;
//                 if let Some(future) = cx.tcx.lang_items().future_trait();
//                 let typeck = cx.tcx.typeck_body(body_id);
//                 let body = cx.tcx.hir().body(body_id);
//                 let ret_ty = typeck.expr_ty(&body.value);
//                 if implements_trait(cx, ret_ty, future, &[]);
//                 if let ty::Opaque(_, subs) = ret_ty.kind();
//                 if let Some(gen) = subs.types().next();
//                 if let ty::Generator(_, subs, _) = gen.kind();
//                 if is_type_diagnostic_item(cx, subs.as_generator().return_ty(), sym::Result);
//                 then {
//                     span_lint(
//                         cx,
//                         MISSING_ERRORS_DOC,
//                         span,
//                         "docs for function returning `Result` missing `# Errors` section",
//                     );
//                 }
//             }
//         }
//     }
// }

/// Cleanup documentation decoration.
///
/// We can't use `rustc_ast::attr::AttributeMethods::with_desugared_doc` or
/// `rustc_ast::parse::lexer::comments::strip_doc_comment_decoration` because we
/// need to keep track of
/// the spans but this function is inspired from the later.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn strip_doc_comment_decoration(
    doc: &str,
    comment_kind: CommentKind,
    span: Span,
) -> (String, Vec<(usize, Span)>) {
    // one-line comments lose their prefix
    if comment_kind == CommentKind::Line {
        let mut doc = doc.to_owned();
        doc.push('\n');
        let len = doc.len();
        // +3 skips the opening delimiter
        return (doc, vec![(len, span.with_lo(span.lo() + BytePos(3)))]);
    }

    let mut sizes = vec![];
    let mut contains_initial_stars = false;
    for line in doc.lines() {
        let offset = line.as_ptr() as usize - doc.as_ptr() as usize;
        debug_assert_eq!(offset as u32 as usize, offset);
        contains_initial_stars |= line.trim_start().starts_with('*');
        // +1 adds the newline, +3 skips the opening delimiter
        sizes.push((
            line.len() + 1,
            span.with_lo(span.lo() + BytePos(3 + offset as u32)),
        ));
    }
    if !contains_initial_stars {
        return (doc.to_string(), sizes);
    }
    // remove the initial '*'s if any
    let mut no_stars = String::with_capacity(doc.len());
    for line in doc.lines() {
        let mut chars = line.chars();
        for c in &mut chars {
            if c.is_whitespace() {
                no_stars.push(c);
            } else {
                no_stars.push(if c == '*' { ' ' } else { c });
                break;
            }
        }
        no_stars.push_str(chars.as_str());
        no_stars.push('\n');
    }

    (no_stars, sizes)
}

#[derive(Copy, Clone)]
struct DocHeaders {
    security: bool,
}

fn check_attrs<'a>(
    cx: &LateContext<'_>,
    valid_idents: &FxHashSet<String>,
    attrs: &'a [Attribute],
) -> DocHeaders {
    use pulldown_cmark::{BrokenLink, CowStr, Options};
    /// We don't want the parser to choke on intra doc links. Since we don't
    /// actually care about rendering them, just pretend that all broken links are
    /// point to a fake address.
    #[allow(clippy::unnecessary_wraps)] // we're following a type signature
    fn fake_broken_link_callback<'a>(_: BrokenLink<'_>) -> Option<(CowStr<'a>, CowStr<'a>)> {
        Some(("fake".into(), "fake".into()))
    }

    let mut doc = String::new();
    let mut spans = vec![];

    for attr in attrs {
        if let AttrKind::DocComment(comment_kind, comment) = attr.kind {
            let (comment, current_spans) =
                strip_doc_comment_decoration(&comment.as_str(), comment_kind, attr.span);
            spans.extend_from_slice(&current_spans);
            doc.push_str(&comment);
        } else if attr.has_name(sym::doc) {
            // ignore mix of sugared and non-sugared doc
            // don't trigger the safety or errors check
            return DocHeaders { security: true };
        }
    }

    let mut current = 0;
    for &mut (ref mut offset, _) in &mut spans {
        let offset_copy = *offset;
        *offset = current;
        current += offset_copy;
    }

    if doc.is_empty() {
        return DocHeaders { security: false };
    }

    let mut cb = fake_broken_link_callback;

    let parser = pulldown_cmark::Parser::new_with_broken_link_callback(
        &doc,
        Options::empty(),
        Some(&mut cb),
    )
    .into_offset_iter();
    // Iterate over all `Events` and combine consecutive events into one
    let events = parser.coalesce(|previous, current| {
        use pulldown_cmark::Event::Text;

        let previous_range = previous.1;
        let current_range = current.1;

        match (previous.0, current.0) {
            (Text(previous), Text(current)) => {
                let mut previous = previous.to_string();
                previous.push_str(&current);
                Ok((Text(previous.into()), previous_range))
            }
            (previous, current) => Err(((previous, previous_range), (current, current_range))),
        }
    });
    check_doc(cx, valid_idents, events, &spans)
}

const RUST_CODE: &[&str] = &["rust", "no_run", "should_panic", "compile_fail"];

fn check_doc<'a, Events: Iterator<Item = (pulldown_cmark::Event<'a>, Range<usize>)>>(
    cx: &LateContext<'_>,
    _valid_idents: &FxHashSet<String>,
    events: Events,
    spans: &[(usize, Span)],
) -> DocHeaders {
    // true if a safety header was found
    use pulldown_cmark::Event::{
        Code, End, FootnoteReference, HardBreak, Html, Rule, SoftBreak, Start, TaskListMarker, Text,
    };
    use pulldown_cmark::Tag::{CodeBlock, Heading, Item, Link, Paragraph};
    use pulldown_cmark::{CodeBlockKind, CowStr};

    let mut headers = DocHeaders { security: false };
    let mut in_code = false;
    let mut in_link = None;
    let mut in_heading = false;
    let mut is_rust = false;
    let mut edition = None;
    let mut ticks_unbalanced = false;
    let mut text_to_check: Vec<(CowStr<'_>, Span)> = Vec::new();
    let mut paragraph_span = spans
        .get(0)
        .expect("function isn't called if doc comment is empty")
        .1;
    for (event, range) in events {
        match event {
            Start(CodeBlock(ref kind)) => {
                in_code = true;
                if let CodeBlockKind::Fenced(lang) = kind {
                    for item in lang.split(',') {
                        if item == "ignore" {
                            is_rust = false;
                            break;
                        }
                        if let Some(stripped) = item.strip_prefix("edition") {
                            is_rust = true;
                            edition = stripped.parse::<Edition>().ok();
                        } else if item.is_empty() || RUST_CODE.contains(&item) {
                            is_rust = true;
                        }
                    }
                }
            }
            End(CodeBlock(_)) => {
                in_code = false;
                is_rust = false;
            }
            Start(Link(_, url, _)) => in_link = Some(url),
            End(Link(..)) => in_link = None,
            Start(Heading(_, _, _) | Paragraph | Item) => {
                if let Start(Heading(_, _, _)) = event {
                    in_heading = true;
                }
                ticks_unbalanced = false;
                let (_, span) = get_current_span(spans, range.start);
                paragraph_span = first_line_of_span(cx, span);
            }
            End(Heading(_, _, _) | Paragraph | Item) => {
                if let End(Heading(_, _, _)) = event {
                    in_heading = false;
                }
                if ticks_unbalanced {
                    span_lint_and_help(
                        cx,
                        MISSING_SECURITY_DOC,
                        paragraph_span,
                        "backticks are unbalanced",
                        None,
                        "a backtick may be missing a pair",
                    );
                }
            }
            Start(_tag) | End(_tag) => (), // We don't care about other tags
            Html(_html) => (),             // HTML is weird, just ignore it
            SoftBreak | HardBreak | TaskListMarker(_) | Code(_) | Rule => (),
            FootnoteReference(text) | Text(text) => {
                let (begin, span) = get_current_span(spans, range.start);
                paragraph_span = paragraph_span.with_hi(span.hi());
                ticks_unbalanced |= text.contains('`') && !in_code;
                if Some(&text) == in_link.as_ref() || ticks_unbalanced {
                    // Probably a link of the form `<http://example.com>`
                    // Which are represented as a link to "http://example.com" with
                    // text "http://example.com" by pulldown-cmark
                    continue;
                }
                let trimmed_text = text.trim();
                headers.security |= in_heading && trimmed_text == "Security";
                if in_code {
                } else {
                    // Adjust for the beginning of the current `Event`
                    let span = span.with_lo(span.lo() + BytePos::from_usize(range.start - begin));
                    text_to_check.push((text, span));
                }
            }
        }
    }
    headers
}

fn get_current_span(spans: &[(usize, Span)], idx: usize) -> (usize, Span) {
    let index = match spans.binary_search_by(|c| c.0.cmp(&idx)) {
        Ok(o) => o,
        Err(e) => e - 1,
    };
    spans[index]
}
