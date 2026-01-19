use anyhow::Result;
use image::ImageReader;
use once_cell::sync::Lazy;
use pathdiff::diff_paths;
use ratatui::{prelude::*, text::Text, widgets::*};
use ratatui_image::{picker::Picker, Resize, StatefulImage};
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use std::process::Command;
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};
use tui_markdown as md;
// v8 Picker re-exports ProtocolType from picker; no direct use needed here

pub struct Preview {
    pub text: Text<'static>,
    pub images: Vec<PathBuf>,
    pub videos: Vec<PathBuf>,
}

impl Preview {
    pub fn from_markdown(_path: &Path, src: &str) -> Result<Self> {
        // Normalize headings so `##Heading` (no space) becomes `## Heading`, and add spacing
        let normalized = normalize_headings(src);
        // Convert markdown → styled Text, then take an owned copy (no unsafe lifetime tricks)
        let text_parsed: Text<'_> = md::from_str(&normalized);
        let text_owned: Text<'static> = to_owned_text(text_parsed);
        let text: Text<'static> = apply_heading_styles(text_owned);

        let mut images = vec![];
        for (_alt, p) in find_md_images(src) {
            let abs = resolve_relative(_path, &p);
            if abs.exists() {
                images.push(abs);
            }
        }
        let mut videos = vec![];
        for (_alt, p) in find_md_videos(src) {
            let abs = resolve_relative(_path, &p);
            if abs.exists() {
                videos.push(abs);
            }
        }
        Ok(Self {
            text,
            images,
            videos,
        })
    }
}
fn to_owned_text(input: Text<'_>) -> Text<'static> {
    let mut out_lines: Vec<Line<'static>> = Vec::with_capacity(input.lines.len());
    for line in input.lines.iter() {
        let mut spans_owned: Vec<Span<'static>> = Vec::with_capacity(line.spans.len());
        for span in line.spans.iter() {
            spans_owned.push(Span::styled(span.content.to_string(), span.style));
        }
        out_lines.push(Line::from(spans_owned));
    }
    Text::from(out_lines)
}

fn apply_heading_styles(text: Text<'static>) -> Text<'static> {
    // Convert leading "# ", "## ", ... into styled heading lines without the markers.
    // This compensates for markdown backends that leave '#' in plain text.
    let is_light = std::env::var("SB_THEME")
        .ok()
        .map(|v| v.eq_ignore_ascii_case("light"))
        .unwrap_or(false);
    let mut new_lines: Vec<Line<'static>> = Vec::with_capacity(text.lines.len());
    for line in text.lines.into_iter() {
        // Reconstruct full content to detect heading markers
        let content: String = line
            .spans
            .iter()
            .map(|s| s.content.clone().into_owned())
            .collect();
        let trimmed = content.trim_end_matches('\n');
        // Count leading '#'
        let mut hashes = 0usize;
        for ch in trimmed.chars() {
            if ch == '#' && hashes < 6 {
                hashes += 1;
            } else {
                break;
            }
        }
        let is_heading = hashes > 0 && trimmed.chars().nth(hashes) == Some(' ');
        if is_heading {
            let title = trimmed[hashes + 1..].to_string();
            // Color scheme without bold/underline
            // Light: H1/H2 Blue, H3 DarkGray, others Gray
            // Dark: H1/H2 Yellow, H3 Cyan, others Cyan
            let style = if is_light {
                match hashes {
                    1 => Style::default().fg(Color::Blue),
                    2 => Style::default().fg(Color::Blue),
                    3 => Style::default().fg(Color::DarkGray),
                    _ => Style::default().fg(Color::Gray),
                }
            } else {
                match hashes {
                    1 => Style::default().fg(Color::Yellow),
                    2 => Style::default().fg(Color::Yellow),
                    3 => Style::default().fg(Color::Cyan),
                    _ => Style::default().fg(Color::Cyan),
                }
            };
            new_lines.push(Line::from(Span::styled(title, style)));
        } else {
            new_lines.push(line);
        }
    }
    Text::from(new_lines)
}

pub fn resolve_relative(base_file: &Path, rel: &str) -> PathBuf {
    let base_dir = base_file.parent().unwrap_or_else(|| Path::new("."));
    let p = Path::new(rel);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

fn normalize_headings(src: &str) -> String {
    // Insert a space after leading # if missing and add a blank line before top-level headings
    // This helps the markdown parser render headers and improves visual separation.
    let mut out = String::with_capacity(src.len() + 64);
    let mut prev_blank = true;
    for line in src.lines() {
        let bytes = line.as_bytes();
        // Count leading '#'
        let mut i = 0;
        while i < bytes.len() && bytes[i] == b'#' && i < 6 {
            i += 1;
        }
        if i > 0 {
            // If next char exists and isn't space
            if bytes
                .get(i)
                .map(|c| *c != b' ' && *c != b'#')
                .unwrap_or(false)
            {
                // Add separation before H1-H3
                if !prev_blank && i <= 3 {
                    out.push('\n');
                }
                out.push_str(&"#".repeat(i));
                out.push(' ');
                out.push_str(&line[i..]);
            } else {
                if !prev_blank && i <= 3 {
                    out.push('\n');
                }
                out.push_str(line);
            }
            out.push('\n');
            prev_blank = true; // we ensured a newline
            continue;
        }
        out.push_str(line);
        out.push('\n');
        prev_blank = line.trim().is_empty();
    }
    out
}

pub fn find_md_images(src: &str) -> Vec<(String, String)> {
    // Very small regex-free finder: ![alt](path)
    let mut out = vec![];
    let bytes = src.as_bytes();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if &bytes[i..i + 2] == b"![" {
            if let Some(j) = src[i + 2..].find(']') {
                let alt = &src[i + 2..i + 2 + j];
                if src[i + 2 + j..].starts_with("](") {
                    if let Some(k) = src[i + 3 + j..].find(')') {
                        let path = &src[i + 3 + j..i + 3 + j + k];
                        out.push((alt.to_string(), path.to_string()));
                        i += 3 + j + k;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

pub fn render_preview(f: &mut Frame, area: Rect, preview: &Preview) {
    // If opened file is a code file, show highlighted + diff view
    if try_render_code_preview(f, area).is_some() {
        return;
    }
    // Determine if we should overlay raw current line and dim rendered output
    // Show raw-line overlay only when explicitly enabled (e.g., during inline edit)
    let show_overlay = matches!(std::env::var("SB_OVERLAY").as_deref(), Ok("1"));
    // Text preview
    let mut paragraph = Paragraph::new(preview.text.clone())
        .wrap(Wrap { trim: true })
        .block(Block::default().title("Preview").borders(Borders::ALL));
    if show_overlay {
        paragraph = paragraph.style(Style::default().fg(Color::DarkGray));
    }

    // Split area vertically and render text + images + video placeholders
    let total_media = preview.images.len() + preview.videos.len();
    let mut rows = vec![Constraint::Min(
        area.height.saturating_sub((total_media as u16) * 12).max(3),
    )];
    if total_media > 0 {
        rows.extend(std::iter::repeat_n(Constraint::Length(12), total_media));
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(rows)
        .split(area);
    // Apply vertical scroll from env so caller can keep cursor in view
    let scroll_top: u16 = std::env::var("SB_PREVIEW_SCROLL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let paragraph = paragraph.scroll((scroll_top, 0));
    f.render_widget(paragraph, chunks[0]);

    // Always show a visible cursor line highlight in the preview's inner area
    if let Some(cursor) = std::env::var("SB_PREVIEW_CURSOR")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
    {
        let inner_y = chunks[0].y.saturating_add(1);
        let inner_x = chunks[0].x.saturating_add(1);
        let inner_h = chunks[0].height.saturating_sub(2);
        let inner_w = chunks[0].width.saturating_sub(2);
        if inner_h > 0 && inner_w > 0 && cursor >= scroll_top as usize {
            let rel = cursor - scroll_top as usize;
            if (rel as u16) < inner_h {
                let area = Rect {
                    x: inner_x,
                    y: inner_y + rel as u16,
                    width: inner_w,
                    height: 1,
                };
                let highlight = Paragraph::new("").style(Style::default().bg(Color::DarkGray));
                f.render_widget(highlight, area);
            }
        }
    }

    // If enabled, overlay a single raw line with a caret and gutter
    if show_overlay {
        if let (Ok(raw_line), Ok(cursor_str)) = (
            std::env::var("SB_CURRENT_TEXT"),
            std::env::var("SB_PREVIEW_CURSOR"),
        ) {
            if let Ok(cursor) = cursor_str.parse::<usize>() {
                let mut lines_iter = raw_line.lines();
                if let Some(raw0) = lines_iter.nth(cursor) {
                    let raw = sanitize_line(raw0);
                    // draw on top: a one-line block showing the raw markdown of the focused line
                    let inner_y = chunks[0].y.saturating_add(1);
                    let inner_x = chunks[0].x.saturating_add(1);
                    let inner_h = chunks[0].height.saturating_sub(2);
                    let inner_w = chunks[0].width.saturating_sub(2);
                    // place relative to scroll
                    let rel = cursor.saturating_sub(scroll_top as usize) as u16;
                    if rel >= inner_h {
                        return;
                    }
                    let y = inner_y + rel;
                    // Gutter with line number
                    let gutter_w: u16 = 6;
                    if inner_w > gutter_w {
                        let gutter_area = Rect {
                            x: inner_x,
                            y,
                            width: gutter_w,
                            height: 1,
                        };
                        let ln = format!("{:>4} ", cursor + 1);
                        let gutter = Paragraph::new(ln)
                            .style(Style::default().fg(Color::White).bg(Color::Blue));
                        f.render_widget(gutter, gutter_area);
                        // Raw line area beside gutter
                        let raw_area = Rect {
                            x: gutter_area.x + gutter_area.width,
                            y,
                            width: inner_w.saturating_sub(gutter_w),
                            height: 1,
                        };
                        let col = std::env::var("SB_PREVIEW_COL")
                            .ok()
                            .and_then(|s| s.parse::<usize>().ok())
                            .unwrap_or(0);
                        // Split raw into left of col, cursor char, and right for a visual caret block
                        let chars: Vec<char> = raw.chars().collect();
                        let col_idx = col.min(chars.len());
                        let (left, cur, right) = (
                            chars[..col_idx].iter().collect::<String>(),
                            chars.get(col_idx).cloned(),
                            if col_idx < chars.len() {
                                chars[col_idx + 1..].iter().collect::<String>()
                            } else {
                                String::new()
                            },
                        );
                        let mut spans = vec![Span::styled(
                            left,
                            Style::default().fg(Color::Black).bg(Color::Yellow),
                        )];
                        // Draw caret block on the current character or a block if at EOL
                        if let Some(c) = cur {
                            spans.push(Span::styled(
                                c.to_string(),
                                Style::default()
                                    .fg(Color::Black)
                                    .bg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        } else {
                            spans.push(Span::styled(
                                " ",
                                Style::default()
                                    .fg(Color::Black)
                                    .bg(Color::White)
                                    .add_modifier(Modifier::BOLD),
                            ));
                        }
                        spans.push(Span::styled(
                            right,
                            Style::default().fg(Color::Black).bg(Color::Yellow),
                        ));
                        let styled = Line::from(spans);
                        let para = Paragraph::new(styled).wrap(Wrap { trim: true });
                        f.render_widget(para, raw_area);
                    }
                }
            }
        }
    }

    fn sanitize_line(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_control() && c != '\t' { ' ' } else { c })
            .collect()
    }

    let mut idx = 1;
    for path in preview.images.iter() {
        if let Ok(reader) = ImageReader::open(path) {
            if let Ok(img) = reader.decode() {
                let picker = Picker::halfblocks();
                let mut state = picker.new_resize_protocol(img);
                let widget = StatefulImage::new().resize(Resize::Fit(None));
                f.render_stateful_widget(widget, chunks[idx], &mut state);
            } else {
                let line = Line::from(format!("(image decode failed) {}", path.display()));
                let p = Paragraph::new(line);
                f.render_widget(p, chunks[idx]);
            }
        } else {
            let line = Line::from(format!("(image load failed) {}", path.display()));
            let p = Paragraph::new(line);
            f.render_widget(p, chunks[idx]);
        }
        idx += 1;
    }
    for path in preview.videos.iter() {
        let line = Line::from(format!("video: {}", path.display()));
        let p = Paragraph::new(line);
        f.render_widget(p, chunks[idx]);
        idx += 1;
    }
}

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

fn try_render_code_preview(f: &mut Frame, area: Rect) -> Option<()> {
    // We need the current file path and buffer; pull from global if exposed
    // Since we don't have direct access to `App` here, detect via environment variables
    // Simplify: read env SB_CURRENT_FILE and SB_CURRENT_TEXT set by caller
    let path = std::env::var("SB_CURRENT_FILE").ok()?;
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let is_code = matches!(
        ext.as_str(),
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "rb"
            | "go"
            | "java"
            | "cpp"
            | "c"
            | "h"
            | "hpp"
            | "cs"
            | "php"
            | "swift"
            | "kt"
            | "scala"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "ps1"
            | "yml"
            | "yaml"
            | "toml"
            | "json"
            | "xml"
            | "html"
            | "css"
            | "scss"
            | "sass"
            | "sql"
            | "tex"
    );
    if !is_code {
        return None;
    }
    let text = std::env::var("SB_CURRENT_TEXT").ok().unwrap_or_default();

    // Get git diff if available
    let rel_for_git = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(
            std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new(".")),
        )
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });
    let spec = if let Some(root) = rel_for_git {
        let rel = diff_paths(&path, root).unwrap_or_else(|| std::path::PathBuf::from(&path));
        format!("HEAD:{}", rel.to_string_lossy())
    } else {
        format!("HEAD:{path}")
    };

    // Get original content from git if available
    let original = Command::new("git")
        .args(["show", &spec])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string());

    // Prepare syntax highlighting
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(&ext)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
    let theme_opt = THEME_SET
        .themes
        .get("base16-ocean.dark")
        .or_else(|| THEME_SET.themes.get("Solarized (dark)"))
        .or_else(|| THEME_SET.themes.get("Monokai"))
        .or_else(|| THEME_SET.themes.get("InspiredGitHub"))
        .or_else(|| THEME_SET.themes.values().next());

    let mut lines: Vec<Line> = Vec::new();
    let has_diff = original.is_some();

    // Get cursor position for highlighting
    let cursor_line = std::env::var("SB_PREVIEW_CURSOR")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    // If we have a diff, show unified inline diff with syntax highlighting
    if let Some(orig) = original {
        let diff = TextDiff::from_lines(&orig, &text);
        let mut line_num = 1;

        if let Some(theme) = theme_opt {
            let mut h = HighlightLines::new(syntax, theme);

            for change in diff.iter_all_changes() {
                let line_content = change.to_string_lossy();
                let line_trimmed = line_content.trim_end_matches('\n');

                // Get syntax highlighting for the line content
                let regions = h
                    .highlight_line(line_trimmed, &SYNTAX_SET)
                    .unwrap_or_default();
                let mut spans: Vec<Span> = Vec::new();

                // Check if this is the cursor line (only for non-deleted lines)
                let is_cursor_line = match change.tag() {
                    ChangeTag::Delete => false, // Deleted lines don't count for cursor position
                    ChangeTag::Insert | ChangeTag::Equal => line_num - 1 == cursor_line,
                };

                // Add diff marker and line number
                match change.tag() {
                    ChangeTag::Delete => {
                        let prefix_style =
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                        spans.push(Span::styled(format!("{line_num:4} - "), prefix_style));
                        // Apply syntax highlighting with red tint
                        for (style, segment) in regions {
                            let fg = style.foreground;
                            spans.push(Span::styled(
                                segment.to_string(),
                                Style::default()
                                    .fg(Color::Rgb(
                                        fg.r.saturating_sub(50),
                                        fg.g.saturating_sub(100),
                                        fg.b.saturating_sub(100),
                                    ))
                                    .add_modifier(Modifier::DIM),
                            ));
                        }
                    }
                    ChangeTag::Insert => {
                        let prefix_style = if is_cursor_line {
                            Style::default()
                                .fg(Color::Green)
                                .bg(Color::DarkGray)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD)
                        };
                        spans.push(Span::styled(format!("{line_num:4} + "), prefix_style));
                        // Apply syntax highlighting with green tint and cursor highlight
                        for (style, segment) in regions {
                            let fg = style.foreground;
                            let segment_style = if is_cursor_line {
                                Style::default()
                                    .fg(Color::Rgb(
                                        fg.r.saturating_sub(100),
                                        fg.g,
                                        fg.b.saturating_sub(100),
                                    ))
                                    .bg(Color::DarkGray)
                            } else {
                                Style::default().fg(Color::Rgb(
                                    fg.r.saturating_sub(100),
                                    fg.g,
                                    fg.b.saturating_sub(100),
                                ))
                            };
                            spans.push(Span::styled(segment.to_string(), segment_style));
                        }
                        line_num += 1;
                    }
                    ChangeTag::Equal => {
                        let prefix_style = if is_cursor_line {
                            Style::default().fg(Color::DarkGray).bg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        spans.push(Span::styled(format!("{line_num:4}   "), prefix_style));
                        // Normal syntax highlighting with cursor highlight
                        for (style, segment) in regions {
                            let fg = style.foreground;
                            let segment_style = if is_cursor_line {
                                Style::default()
                                    .fg(Color::Rgb(fg.r, fg.g, fg.b))
                                    .bg(Color::DarkGray)
                            } else {
                                Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b))
                            };
                            spans.push(Span::styled(segment.to_string(), segment_style));
                        }
                        line_num += 1;
                    }
                }
                lines.push(Line::from(spans));
            }
        } else {
            // Fallback without syntax highlighting
            for change in diff.iter_all_changes() {
                let (prefix, color) = match change.tag() {
                    ChangeTag::Delete => (format!("{line_num:4} - "), Color::Red),
                    ChangeTag::Insert => {
                        let p = format!("{line_num:4} + ");
                        line_num += 1;
                        (p, Color::Green)
                    }
                    ChangeTag::Equal => {
                        let p = format!("{line_num:4}   ");
                        line_num += 1;
                        (p, Color::Gray)
                    }
                };
                let content = change.to_string_lossy();
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(color)),
                    Span::styled(content.to_string(), Style::default().fg(color)),
                ]));
            }
        }
    } else {
        // No diff available, just show syntax highlighted code with line numbers
        if let Some(theme) = theme_opt {
            let mut h = HighlightLines::new(syntax, theme);
            let mut line_num = 1;
            for raw in text.lines() {
                let is_cursor_line = (line_num - 1) == cursor_line;
                let regions = h.highlight_line(raw, &SYNTAX_SET).unwrap_or_default();
                let mut spans: Vec<Span> = Vec::new();

                let prefix_style = if is_cursor_line {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                spans.push(Span::styled(format!("{line_num:4}   "), prefix_style));

                for (style, segment) in regions {
                    let fg = style.foreground;
                    let segment_style = if is_cursor_line {
                        Style::default()
                            .fg(Color::Rgb(fg.r, fg.g, fg.b))
                            .bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b))
                    };
                    spans.push(Span::styled(segment.to_string(), segment_style));
                }
                lines.push(Line::from(spans));
                line_num += 1;
            }
        } else {
            // Fallback: plain text with line numbers
            let mut line_num = 1;
            for raw in text.lines() {
                let is_cursor_line = (line_num - 1) == cursor_line;
                let prefix_style = if is_cursor_line {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let text_style = if is_cursor_line {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{line_num:4}   "), prefix_style),
                    Span::styled(raw.to_string(), text_style),
                ]));
                line_num += 1;
            }
        }
    }

    // Render as a single unified view
    let total_lines = text.lines().count();
    let title = if has_diff {
        format!(
            "Code (Diff vs HEAD) - {} (Line {}/{})",
            std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&path),
            cursor_line + 1,
            total_lines
        )
    } else {
        format!(
            "Code - {} (Line {}/{})",
            std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&path),
            cursor_line + 1,
            total_lines
        )
    };

    let para = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: false })
        .block(Block::default().title(title).borders(Borders::ALL))
        .scroll((
            std::env::var("SB_PREVIEW_SCROLL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            0,
        ));
    f.render_widget(para, area);

    Some(())
}

pub fn find_md_videos(src: &str) -> Vec<(String, String)> {
    // [video](path)
    let mut out = vec![];
    let bytes = src.as_bytes();
    let mut i = 0;
    while i + 4 < bytes.len() {
        if &bytes[i..i + 1] == b"[" {
            if let Some(j) = src[i + 1..].find(']') {
                let alt = &src[i + 1..i + 1 + j];
                if src[i + 1 + j..].starts_with("](") {
                    if let Some(k) = src[i + 2 + j..].find(')') {
                        let path = &src[i + 2 + j..i + 2 + j + k];
                        if alt.trim().eq_ignore_ascii_case("video") {
                            out.push((alt.to_string(), path.to_string()));
                        }
                        i += 2 + j + k;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

// Utility to compute a Rect for a given line index if we wanted to highlight it.
#[allow(dead_code)]
pub fn line_rect(area: Rect, line_idx: usize, total_lines: usize) -> Option<Rect> {
    if total_lines == 0 {
        return None;
    }
    let y = area.y.saturating_add(line_idx as u16);
    if y >= area.y + area.height {
        return None;
    }
    Some(Rect {
        x: area.x,
        y,
        width: area.width,
        height: 1,
    })
}
