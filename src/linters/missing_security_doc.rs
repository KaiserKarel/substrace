use crate::paths;
use clippy_utils::diagnostics::{span_lint_and_help, span_lint_and_sugg};
use clippy_utils::source::first_line_of_span;
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
    pub MISSING_SECURITY_DOC,
    Warn,
    "Using the Identity or Twox64Concat hasher requires a doc describing it's secure usage"
}

impl_lint_pass!(DocMarkdown => [MISSING_SECURITY_DOC]);

#[derive(Clone, Default)]
pub struct DocMarkdown;

impl DocMarkdown {
    pub fn new() -> Self {
        Self
    }
}

impl<'tcx> LateLintPass<'tcx> for DocMarkdown {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'_>) {
        if let hir::ItemKind::TyAlias(ty, ..) = item.kind {
            if let hir::TyKind::TraitObject(ptr, ..) = ty.kind {
                if let Some(hir::def::Res::Def(_, id)) =
                    ptr.get(0).map(|poly| poly.trait_ref.path.res)
                {
                    if paths::is_like_storage_map(cx, id) {
                        let attrs = cx.tcx.hir().attrs(item.hir_id());
                        let headers = check_attrs(cx, &Default::default(), attrs);
                        if let Some(segments) = ptr.get(0).map(|poly| poly.trait_ref.path.segments)
                        {
                            for segment in segments {
                                if let Some(args) = segment.args {
                                    if paths::is_insecure_hash_function(cx, args)
                                        && !headers.security
                                    {
                                        span_lint_and_sugg(
                                            cx,
                                            MISSING_SECURITY_DOC,
                                            item.span,
                                            "Twox{64, 128, 256} and Identity are at not secure",
                                            "use Blake2, or add a # Security doc comment describing why the usage is correct",
                                            "/// # Security
/// Twox64Concat is safe because the ...".to_string(),
                                            Applicability::HasPlaceholders,
                                        );
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

// Taken from the actually clippy codebase. Performs parsing of doc comments. Most likely this may be slimmed down by a lot for this linter's purposes.
fn check_doc<'a, Events: Iterator<Item = (pulldown_cmark::Event<'a>, Range<usize>)>>(
    cx: &LateContext<'_>,
    _valid_idents: &FxHashSet<String>,
    events: Events,
    spans: &[(usize, Span)],
) -> DocHeaders {
    use pulldown_cmark::CowStr;
    use pulldown_cmark::Event::{
        Code, End, FootnoteReference, HardBreak, Html, Rule, SoftBreak, Start, TaskListMarker, Text,
    };
    use pulldown_cmark::Tag::{CodeBlock, Heading, Item, Link, Paragraph};

    let mut headers = DocHeaders { security: false };
    let mut in_code = false;
    let mut in_link = None;
    let mut in_heading = false;
    let mut ticks_unbalanced = false;
    let mut text_to_check: Vec<(CowStr<'_>, Span)> = Vec::new();
    let mut paragraph_span = spans
        .get(0)
        .expect("function isn't called if doc comment is empty")
        .1;
    for (event, range) in events {
        match event {
            Start(CodeBlock(_)) => {
                in_code = true;
            }
            End(CodeBlock(_)) => {
                in_code = false;
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
