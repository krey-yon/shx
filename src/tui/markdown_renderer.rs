//! Model answers are markdown; a terminal is not.
//!
//! Answers use four things — fenced code, inline code, bold, italic — plus
//! headings, and this module handles exactly those. Everything else is passed
//! through untouched. One pass over the text produces styled segments;
//! [`render_plain`] joins them into the words alone and [`render_with`] adds
//! colour, so both agree on the text and only differ on the escape codes.

use owo_colors::Style;

use crate::tui::theme::{self, Theme};

/// Whether the text is worth sending through the renderer at all.
#[must_use]
pub fn contains_markdown(input: &str) -> bool {
    input.contains("```") || input.contains("**") || input.contains("##")
}

/// The answer with the markup removed and no colour.
///
/// # Examples
///
/// ```
/// use shx::tui::markdown_renderer::render_plain;
///
/// assert_eq!(render_plain("**bold** and `code`"), "bold and code");
/// assert_eq!(render_plain("## Heading"), "Heading");
/// assert_eq!(render_plain("```\n  indented\n```"), "  indented");
/// ```
#[must_use]
pub fn render_plain(input: &str) -> String {
    plain(&segments(input))
}

/// The answer with the markup removed and colour added when this process is
/// allowed to write it.
///
/// # Examples
///
/// ```
/// use shx::tui::markdown_renderer::{render, render_plain};
///
/// assert!(render("**done**").contains("done"));
/// assert_eq!(render_plain("**done**"), "done");
/// ```
#[must_use]
pub fn render(input: &str) -> String {
    render_with(&Theme::detect(), input)
}

/// The answer as `theme` draws it, for a caller that has already decided
/// whether colour is wanted.
///
/// # Examples
///
/// ```
/// use shx::tui::markdown_renderer::render_with;
/// use shx::tui::theme::Theme;
///
/// let coloured = render_with(&Theme::new(true), "**done**");
/// assert!(coloured.contains('\u{1b}'));
/// assert_eq!(render_with(&Theme::new(false), "**done**"), "done");
/// ```
#[must_use]
pub fn render_with(theme: &Theme, input: &str) -> String {
    let segments = segments(input);
    if !theme.colour() {
        return plain(&segments);
    }
    let mut out = String::new();
    for segment in &segments {
        out.push_str(&segment.paint(theme));
    }
    out
}

struct Segment {
    text: String,
    /// `None` for text no style should touch, such as a line break.
    style: Option<Style>,
}

impl Segment {
    fn paint(&self, theme: &Theme) -> String {
        match self.style {
            Some(style) => theme.paint(style, &self.text),
            None => self.text.clone(),
        }
    }
}

fn plain(segments: &[Segment]) -> String {
    let mut out = String::new();
    for segment in segments {
        out.push_str(&segment.text);
    }
    out
}

fn segments(input: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut lines = 0;
    let mut fenced = false;
    for line in input.lines() {
        if is_fence(line) {
            fenced = !fenced;
            continue;
        }
        if lines > 0 {
            push(&mut segments, "\n", None);
        }
        lines += 1;
        if fenced {
            push(&mut segments, line, Some(theme::COMMAND));
        } else if let Some((level, body)) = heading(line) {
            push(&mut segments, body, Some(heading_style(level)));
        } else {
            segments.extend(inline(line));
        }
    }
    if input.ends_with('\n') {
        push(&mut segments, "\n", None);
    }
    segments
}

fn push(segments: &mut Vec<Segment>, text: &str, style: Option<Style>) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = segments.last_mut()
        && last.style == style
    {
        last.text.push_str(text);
        return;
    }
    segments.push(Segment {
        text: text.to_owned(),
        style,
    });
}

fn is_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// The level and text of a `#`, `##`, or `###` heading.
///
/// A marker has to be followed by a space and some text, so that `#1` in a
/// comment stays a number.
fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=3).contains(&level) {
        return None;
    }
    let body = line[level..].strip_prefix(' ')?;
    (!body.is_empty()).then_some((level, body))
}

fn heading_style(level: usize) -> Style {
    match level {
        1 => Style::new().bold(),
        2 => Style::new().bold().dimmed(),
        _ => Style::new().dimmed(),
    }
}

fn inline(line: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut at = 0;
    while at < line.len() {
        let Some((marker, style)) = delimiter(line, at) else {
            let width = char_width(line, at);
            text.push_str(&line[at..at + width]);
            at += width;
            continue;
        };
        let body_start = at + marker.len();
        match line[body_start..].find(marker) {
            Some(offset) if offset > 0 => {
                let body = &line[body_start..body_start + offset];
                push(&mut segments, &text, Some(theme::EXPLANATION));
                push(&mut segments, body, Some(style));
                text.clear();
                at = body_start + offset + marker.len();
            }
            _ => {
                let width = char_width(line, at);
                text.push_str(&line[at..at + width]);
                at += width;
            }
        }
    }
    push(&mut segments, &text, Some(theme::EXPLANATION));
    segments
}

fn delimiter(line: &str, at: usize) -> Option<(&'static str, Style)> {
    let rest = &line[at..];
    if rest.starts_with("**") {
        Some(("**", Style::new().bold()))
    } else if rest.starts_with('`') {
        Some(("`", theme::COMMAND))
    } else if rest.starts_with('*') {
        Some(("*", Style::new().italic()))
    } else {
        None
    }
}

fn char_width(line: &str, at: usize) -> usize {
    line[at..].chars().next().map_or(1, char::len_utf8)
}

#[cfg(test)]
mod tests {
    use super::{Theme, contains_markdown, render, render_plain, render_with};

    fn strip_ansi(text: &str) -> String {
        let mut out = String::new();
        let mut escaping = false;
        for ch in text.chars() {
            match ch {
                '\u{1b}' => escaping = true,
                'm' if escaping => escaping = false,
                _ if !escaping => out.push(ch),
                _ => {}
            }
        }
        out
    }

    #[test]
    fn a_fenced_block_is_preserved_verbatim() {
        let input = "before\n```rust\nfn main() {\n    println!(\"hi\");\n}\n```\nafter";
        assert_eq!(
            render_plain(input),
            "before\nfn main() {\n    println!(\"hi\");\n}\nafter"
        );
    }

    #[test]
    fn indentation_inside_a_fence_survives() {
        let input = "```\n        deeply indented\n    four\n```";
        assert_eq!(render_plain(input), "        deeply indented\n    four");
    }

    #[test]
    fn two_fences_are_two_blocks() {
        let input = "```\nfirst\n```\nbetween\n```\nsecond\n```";
        assert_eq!(render_plain(input), "first\nbetween\nsecond");
    }

    #[test]
    fn markup_is_replaced_rather_than_duplicated() {
        let plain = render_plain("**bold** and *italic* and `code`");
        assert_eq!(plain, "bold and italic and code");
        assert!(!plain.contains('*'));
        assert!(!plain.contains('`'));
    }

    #[test]
    fn a_single_star_pair_is_italic() {
        assert_eq!(render_plain("a *b* c"), "a b c");
    }

    #[test]
    fn an_unclosed_marker_is_left_alone() {
        assert_eq!(render_plain("a * b"), "a * b");
        assert_eq!(render_plain("a ** b"), "a ** b");
        assert_eq!(render_plain("a ` b"), "a ` b");
    }

    #[test]
    fn an_empty_marker_is_left_alone() {
        assert_eq!(render_plain("****"), "****");
        assert_eq!(render_plain("``"), "``");
    }

    #[test]
    fn a_marker_inside_a_marker_keeps_its_own_text() {
        assert_eq!(render_plain("**bold `code` here**"), "bold `code` here");
    }

    #[test]
    fn a_marker_does_not_span_a_line_break() {
        assert_eq!(render_plain("**one\ntwo**"), "**one\ntwo**");
    }

    #[test]
    fn headings_lose_their_hashes() {
        assert_eq!(render_plain("# One"), "One");
        assert_eq!(render_plain("## Two"), "Two");
        assert_eq!(render_plain("### Three"), "Three");
        assert_eq!(render_plain("## Two\n\nbody"), "Two\n\nbody");
    }

    #[test]
    fn a_hash_that_is_not_a_heading_stays() {
        assert_eq!(render_plain("#nothash"), "#nothash");
        assert_eq!(render_plain("issue #42 here"), "issue #42 here");
        assert_eq!(render_plain("#### four"), "#### four");
        assert_eq!(render_plain("#"), "#");
    }

    #[test]
    fn a_hash_inside_a_fence_is_left_alone() {
        assert_eq!(render_plain("```\n# not a heading\n```"), "# not a heading");
    }

    #[test]
    fn an_unbalanced_fence_does_not_panic_and_keeps_the_text() {
        assert_eq!(
            render_plain("```\ncode line\nstill code"),
            "code line\nstill code"
        );
    }

    #[test]
    fn empty_input_is_empty() {
        assert_eq!(render_plain(""), "");
    }

    #[test]
    fn plain_text_passes_through_untouched() {
        let text = "Install ripgrep, then reload your shell so the new binary is on PATH.";
        assert_eq!(render_plain(text), text);
    }

    #[test]
    fn line_endings_are_preserved() {
        assert_eq!(render_plain("a\nb"), "a\nb");
        assert_eq!(render_plain("a\n"), "a\n");
        assert_eq!(render_plain("a\n\n"), "a\n\n");
        assert_eq!(render_plain("\n\n"), "\n\n");
    }

    #[test]
    fn a_lone_backtick_line_does_not_panic() {
        assert_eq!(render_plain("a ` b\n`"), "a ` b\n`");
    }

    #[test]
    fn multibyte_text_survives_markup_removal() {
        assert_eq!(render_plain("**héllo** `wörld`"), "héllo wörld");
    }

    #[test]
    fn contains_markdown_agrees_with_the_markers_it_renders() {
        assert!(contains_markdown("```rust\n```"));
        assert!(contains_markdown("**bold**"));
        assert!(contains_markdown("## Two"));
        assert!(!contains_markdown("just a sentence"));
        assert!(!contains_markdown(""));
    }

    #[test]
    fn the_styled_render_keeps_the_same_text() {
        let input = "# Title\n\n**bold** and `code`\n\n```\nfence\n```";
        let styled = render_with(&Theme::new(true), input);
        assert!(styled.contains('\u{1b}'), "{styled:?} should be styled");
        assert_eq!(strip_ansi(&styled), render_plain(input));
    }

    #[test]
    fn only_the_code_block_is_styled() {
        let theme = Theme::new(true);
        let styled = render_with(&theme, "a\n```\nb\n```\nc");
        assert_eq!(strip_ansi(&styled), "a\nb\nc");
        assert!(styled.contains(&theme.paint(crate::tui::theme::COMMAND, "b")));
        assert_eq!(
            styled.matches('\u{1b}').count(),
            2,
            "the prose around the fence should not be styled: {styled:?}"
        );
    }

    #[test]
    fn a_closing_fence_ends_the_code_block() {
        let theme = Theme::new(true);
        let styled = render_with(&theme, "```\nb\n```\nc");
        assert!(
            !styled.contains(&theme.paint(crate::tui::theme::COMMAND, "c")),
            "c is prose, not code: {styled:?}"
        );
    }

    #[test]
    fn colour_off_renders_exactly_the_plain_text() {
        let input = "# Title\n\n**bold** and `code`\n\n```\nfence\n```";
        assert_eq!(render_with(&Theme::new(false), input), render_plain(input));
    }

    #[test]
    fn render_follows_the_detected_theme() {
        let input = "**bold**";
        if Theme::detect().colour() {
            assert!(render(input).contains('\u{1b}'));
        } else {
            assert_eq!(render(input), "bold");
        }
    }
}
