//! JavaScript tokenizer. `tokenize_line_with` is also used by TypeScript
//! with extra keyword / built-in type tables.

use super::common::*;
use super::{Highlighter, LineState, Token};

const JS_KEYWORDS: &[&str] = &[
    "var",
    "let",
    "const",
    "function",
    "class",
    "extends",
    "if",
    "else",
    "for",
    "while",
    "do",
    "switch",
    "case",
    "default",
    "break",
    "continue",
    "return",
    "throw",
    "try",
    "catch",
    "finally",
    "new",
    "delete",
    "typeof",
    "instanceof",
    "void",
    "in",
    "of",
    "this",
    "super",
    "async",
    "await",
    "yield",
    "import",
    "export",
    "from",
    "as",
    "static",
];

const JS_CONSTANTS: &[&str] = &["true", "false", "null", "undefined", "NaN", "Infinity"];

pub struct JavaScriptLang;

pub fn tokenize_line_with(
    line: &str,
    entry: LineState,
    extra_keywords: &[&str],
    extra_types: &[&str],
) -> (Vec<(String, Token)>, LineState) {
    let mut tokens: Vec<(String, Token)> = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;

    if entry == LineState::InBlockComment {
        if let Some(pos) = line.find("*/") {
            tokens.push((line[..pos + 2].to_string(), Token::Comment));
            i = pos + 2;
        } else {
            return (
                vec![(line.to_string(), Token::Comment)],
                LineState::InBlockComment,
            );
        }
    }

    if entry == LineState::InTemplateLiteral {
        let mut j = 0;
        let mut closed = false;
        while j < bytes.len() {
            if bytes[j] == b'\\' && j + 1 < bytes.len() {
                j += 2;
                continue;
            }
            if bytes[j] == b'`' {
                tokens.push((line[..j + 1].to_string(), Token::String));
                i = j + 1;
                closed = true;
                break;
            }
            j += 1;
        }
        if !closed {
            return (
                vec![(line.to_string(), Token::String)],
                LineState::InTemplateLiteral,
            );
        }
    }

    let mut state = LineState::Normal;

    while i < bytes.len() {
        let b = bytes[i];

        if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
            tokens.push((line[i..].to_string(), Token::Comment));
            break;
        }

        if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(rel) = line[i + 2..].find("*/") {
                let end = i + 2 + rel + 2;
                tokens.push((line[i..end].to_string(), Token::Comment));
                i = end;
                continue;
            } else {
                tokens.push((line[i..].to_string(), Token::Comment));
                state = LineState::InBlockComment;
                break;
            }
        }

        if b == b'"' {
            let (end, _closed) = scan_double_quoted_string(line, i);
            tokens.push((line[i..end].to_string(), Token::String));
            i = end;
            continue;
        }
        if b == b'\'' {
            let (end, _closed) = scan_single_quoted_string(line, i);
            tokens.push((line[i..end].to_string(), Token::String));
            i = end;
            continue;
        }
        if b == b'`' {
            let start = i;
            let mut j = i + 1;
            let mut closed = false;
            while j < bytes.len() {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    j += 2;
                    continue;
                }
                if bytes[j] == b'`' {
                    closed = true;
                    j += 1;
                    break;
                }
                j += 1;
            }
            tokens.push((line[start..j].to_string(), Token::String));
            i = j;
            if !closed {
                state = LineState::InTemplateLiteral;
                break;
            }
            continue;
        }

        if b.is_ascii_digit() {
            let end = scan_number(line, i);
            tokens.push((line[i..end].to_string(), Token::Number));
            i = end;
            continue;
        }

        if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
            let mut end = i + 1;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'$')
            {
                end += 1;
            }
            let ident = &line[i..end];
            let is_call = bytes.get(end) == Some(&b'(');
            let tok = if extra_keywords.contains(&ident) || JS_KEYWORDS.contains(&ident) {
                Token::Keyword
            } else if extra_types.contains(&ident) {
                Token::Type
            } else if JS_CONSTANTS.contains(&ident) {
                Token::Constant
            } else if is_all_caps(ident) {
                Token::Constant
            } else if is_call {
                Token::Function
            } else if !extra_types.is_empty() && starts_uppercase(ident) {
                // TS path treats capitalized idents as types. JS falls through.
                Token::Type
            } else {
                Token::Normal
            };
            tokens.push((ident.to_string(), tok));
            i = end;
            continue;
        }

        if b == b' ' || b == b'\t' {
            let start = i;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            tokens.push((line[start..i].to_string(), Token::Normal));
            continue;
        }

        let two = &line[i..(i + 2).min(bytes.len())];
        if matches!(
            two,
            "==" | "!="
                | "<="
                | ">="
                | "&&"
                | "||"
                | "=>"
                | "++"
                | "--"
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "&="
                | "|="
                | "^="
                | "??"
        ) {
            tokens.push((two.to_string(), Token::Operator));
            i += two.len();
            continue;
        }

        if let Some(is_punct) = classify_ascii_punct(b) {
            tokens.push((
                (b as char).to_string(),
                if is_punct {
                    Token::Punctuation
                } else {
                    Token::Operator
                },
            ));
            i += 1;
            continue;
        }

        let ch_len = line[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        tokens.push((line[i..i + ch_len].to_string(), Token::Normal));
        i += ch_len;
    }

    if tokens.is_empty() {
        tokens.push((line.to_string(), Token::Normal));
    }
    (tokens, state)
}

impl Highlighter for JavaScriptLang {
    fn tokenize_line(line: &str, entry: LineState) -> (Vec<(String, Token)>, LineState) {
        tokenize_line_with(line, entry, &[], &[])
    }
}
