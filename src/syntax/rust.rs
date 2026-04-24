//! Rust tokenizer.

use super::common::*;
use super::{Highlighter, LineState, Token};

const KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while", "loop", "return",
    "break", "continue", "struct", "enum", "trait", "impl", "pub", "use", "mod", "as", "where",
    "self", "Self", "super", "crate", "async", "await", "move", "ref", "in", "type", "dyn",
    "unsafe", "extern", "box",
];

const BUILTIN_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str", "String",
];

const CONSTANTS: &[&str] = &["true", "false", "None", "Some", "Ok", "Err"];

pub struct RustLang;

impl Highlighter for RustLang {
    fn tokenize_line(line: &str, entry: LineState) -> (Vec<(String, Token)>, LineState) {
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

            // Raw strings: r"..." or r#"..."#
            if b == b'r' && (bytes.get(i + 1) == Some(&b'"') || bytes.get(i + 1) == Some(&b'#')) {
                let mut j = i + 1;
                let mut hashes = 0;
                while j < bytes.len() && bytes[j] == b'#' {
                    hashes += 1;
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'"' {
                    let content_start = j + 1;
                    let mut k = content_start;
                    let closing = format!("\"{}", "#".repeat(hashes));
                    let closing_bytes = closing.as_bytes();
                    let mut closed = false;
                    while k < bytes.len() {
                        if k + closing_bytes.len() <= bytes.len()
                            && &bytes[k..k + closing_bytes.len()] == closing_bytes
                        {
                            k += closing_bytes.len();
                            tokens.push((line[i..k].to_string(), Token::String));
                            i = k;
                            closed = true;
                            break;
                        }
                        k += 1;
                    }
                    if !closed {
                        tokens.push((line[i..].to_string(), Token::String));
                        i = bytes.len();
                    }
                    continue;
                }
            }

            // Byte strings: b"..."
            if b == b'b' && bytes.get(i + 1) == Some(&b'"') {
                let (end, _closed) = scan_double_quoted_string(line, i + 1);
                tokens.push((line[i..end].to_string(), Token::String));
                i = end;
                continue;
            }

            // Double-quoted strings
            if b == b'"' {
                let (end, _closed) = scan_double_quoted_string(line, i);
                tokens.push((line[i..end].to_string(), Token::String));
                i = end;
                continue;
            }

            // Char literal (heuristic: treat unterminated as lifetime punctuation)
            if b == b'\'' {
                let (end, closed) = scan_single_quoted_string(line, i);
                if closed {
                    tokens.push((line[i..end].to_string(), Token::String));
                    i = end;
                    continue;
                }
            }

            if b.is_ascii_digit() {
                let end = scan_number(line, i);
                tokens.push((line[i..end].to_string(), Token::Number));
                i = end;
                continue;
            }

            if b.is_ascii_alphabetic() || b == b'_' {
                let end = scan_identifier(line, i);
                let ident = &line[i..end];
                let after = bytes.get(end).copied();
                let is_call = after == Some(b'(');

                let tok = if KEYWORDS.contains(&ident) {
                    Token::Keyword
                } else if BUILTIN_TYPES.contains(&ident) {
                    Token::Type
                } else if CONSTANTS.contains(&ident) {
                    Token::Constant
                } else if is_all_caps(ident) {
                    Token::Constant
                } else if starts_uppercase(ident) {
                    Token::Type
                } else if is_call {
                    Token::Function
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
                    | "::"
                    | "->"
                    | "=>"
                    | ".."
                    | "<<"
                    | ">>"
                    | "+="
                    | "-="
                    | "*="
                    | "/="
                    | "%="
                    | "&="
                    | "|="
                    | "^="
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
}
