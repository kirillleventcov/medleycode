//! Syntax highlighting dispatch and shared types.
//!
//! Each supported language lives in its own submodule and exposes a
//! `tokenize_line(line, entry) -> (Vec<(String, Token)>, LineState)` function
//! matching the `Highlighter` trait shape. The free `tokenize_line` function
//! below is the single match-on-`Language` dispatch used by the editor.

use std::path::Path;

pub mod bash;
pub mod common;
pub mod javascript;
pub mod markdown;
pub mod python;
pub mod rust;
pub mod typescript;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    // Core
    Keyword,
    String,
    Number,
    Comment,
    Punctuation,
    Operator,
    Normal,
    // Secondary
    Type,
    Function,
    Constant,
    // Markdown-only tokens
    MdHeading(u8),
    MdBold,
    MdItalic,
    MdCode,
    MdLink,
    MdList,
    MdCheckboxChecked,
    MdCheckboxUnchecked,
    MdBlockquote,
    MdCodeBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineState {
    #[default]
    Normal,
    InBlockComment,
    InTripleString(char),
    InTemplateLiteral,
    InHeredoc,
    InMarkdownCodeFence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    PlainText,
    Markdown,
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Bash,
}

pub trait Highlighter {
    fn tokenize_line(line: &str, entry: LineState) -> (Vec<(String, Token)>, LineState);
}

pub fn tokenize_line(
    lang: Language,
    line: &str,
    entry: LineState,
) -> (Vec<(String, Token)>, LineState) {
    match lang {
        Language::PlainText => (vec![(line.to_string(), Token::Normal)], LineState::Normal),
        Language::Markdown => markdown::MarkdownLang::tokenize_line(line, entry),
        Language::Rust => rust::RustLang::tokenize_line(line, entry),
        Language::Python => python::PythonLang::tokenize_line(line, entry),
        Language::JavaScript => javascript::JavaScriptLang::tokenize_line(line, entry),
        Language::TypeScript => typescript::TypeScriptLang::tokenize_line(line, entry),
        Language::Bash => bash::BashLang::tokenize_line(line, entry),
    }
}

pub fn detect_language(path: Option<&Path>, first_line: Option<&str>) -> Language {
    if let Some(p) = path {
        match p.extension().and_then(|e| e.to_str()) {
            Some("rs") => return Language::Rust,
            Some("py") | Some("pyi") => return Language::Python,
            Some("ts") | Some("tsx") | Some("mts") | Some("cts") => return Language::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") | Some("cjs") => return Language::JavaScript,
            Some("sh") | Some("bash") | Some("zsh") => return Language::Bash,
            Some("md") | Some("markdown") => return Language::Markdown,
            _ => {}
        }
    }
    if let Some(line) = first_line {
        if let Some(rest) = line.strip_prefix("#!") {
            if rest.contains("python") {
                return Language::Python;
            }
            if rest.contains("bash") || rest.contains("/sh") || rest.contains("zsh") {
                return Language::Bash;
            }
            if rest.contains("node") {
                return Language::JavaScript;
            }
        }
    }
    Language::PlainText
}

/// Recomputes `line_states` from scratch for the given content. Returns a vector
/// of length `num_lines + 1` where `line_states[i]` is the state entering line `i`
/// and `line_states[num_lines]` is the state after the last line.
pub fn recompute_all_line_states(lang: Language, content: &str) -> Vec<LineState> {
    let mut states = vec![LineState::Normal];
    let mut entry = LineState::Normal;
    for line in content.split('\n') {
        let (_, exit) = tokenize_line(lang, line, entry);
        entry = exit;
        states.push(entry);
    }
    states
}
