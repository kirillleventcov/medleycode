//! Python tokenizer.

use super::common::*;
use super::{Highlighter, LineState, Token};

const KEYWORDS: &[&str] = &[
    "def", "class", "if", "elif", "else", "for", "while", "return", "yield", "import", "from",
    "as", "pass", "break", "continue", "with", "try", "except", "finally", "raise", "lambda",
    "global", "nonlocal", "async", "await", "and", "or", "not", "is", "in",
];

const BUILTIN_TYPES: &[&str] = &[
    "int",
    "str",
    "float",
    "bool",
    "list",
    "dict",
    "tuple",
    "set",
    "bytes",
    "bytearray",
];

const CONSTANTS: &[&str] = &["True", "False", "None"];

pub struct PythonLang;

fn scan_triple_string(line: &str, start: usize, quote: u8) -> (usize, bool) {
    let bytes = line.as_bytes();
    let mut i = start;
    while i + 2 < bytes.len() {
        if bytes[i] == quote && bytes[i + 1] == quote && bytes[i + 2] == quote {
            return (i + 3, true);
        }
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
        } else {
            i += 1;
        }
    }
    (bytes.len(), false)
}

impl Highlighter for PythonLang {
    fn tokenize_line(line: &str, entry: LineState) -> (Vec<(String, Token)>, LineState) {
        let mut tokens: Vec<(String, Token)> = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0;

        if let LineState::InTripleString(q) = entry {
            let quote = q as u8;
            let (end, closed) = scan_triple_string(line, 0, quote);
            tokens.push((line[..end].to_string(), Token::String));
            if !closed {
                return (tokens, LineState::InTripleString(q));
            }
            i = end;
        }

        let mut state = LineState::Normal;

        while i < bytes.len() {
            let b = bytes[i];

            if b == b'#' {
                tokens.push((line[i..].to_string(), Token::Comment));
                break;
            }

            // String prefixes: f, r, b, u (1-2 chars, case-insensitive).
            let mut prefix_len = 0;
            if matches!(b, b'f' | b'F' | b'r' | b'R' | b'b' | b'B' | b'u' | b'U') {
                let next = bytes.get(i + 1).copied();
                if next == Some(b'"') || next == Some(b'\'') {
                    prefix_len = 1;
                } else if let Some(c) = next {
                    if matches!(c, b'f' | b'F' | b'r' | b'R' | b'b' | b'B') {
                        let n2 = bytes.get(i + 2).copied();
                        if n2 == Some(b'"') || n2 == Some(b'\'') {
                            prefix_len = 2;
                        }
                    }
                }
            }

            let quote_idx = i + prefix_len;
            if prefix_len > 0
                && (bytes.get(quote_idx) == Some(&b'"') || bytes.get(quote_idx) == Some(&b'\''))
            {
                let q = bytes[quote_idx];
                if bytes.get(quote_idx + 1) == Some(&q) && bytes.get(quote_idx + 2) == Some(&q) {
                    let (end, closed) = scan_triple_string(line, quote_idx + 3, q);
                    tokens.push((line[i..end].to_string(), Token::String));
                    if !closed {
                        state = LineState::InTripleString(q as char);
                        break;
                    }
                    i = end;
                    continue;
                }
                let (end, _closed) = if q == b'"' {
                    scan_double_quoted_string(line, quote_idx)
                } else {
                    scan_single_quoted_string(line, quote_idx)
                };
                tokens.push((line[i..end].to_string(), Token::String));
                i = end;
                continue;
            }

            if b == b'"' {
                if bytes.get(i + 1) == Some(&b'"') && bytes.get(i + 2) == Some(&b'"') {
                    let (end, closed) = scan_triple_string(line, i + 3, b'"');
                    tokens.push((line[i..end].to_string(), Token::String));
                    if !closed {
                        state = LineState::InTripleString('"');
                        break;
                    }
                    i = end;
                    continue;
                }
                let (end, _closed) = scan_double_quoted_string(line, i);
                tokens.push((line[i..end].to_string(), Token::String));
                i = end;
                continue;
            }
            if b == b'\'' {
                if bytes.get(i + 1) == Some(&b'\'') && bytes.get(i + 2) == Some(&b'\'') {
                    let (end, closed) = scan_triple_string(line, i + 3, b'\'');
                    tokens.push((line[i..end].to_string(), Token::String));
                    if !closed {
                        state = LineState::InTripleString('\'');
                        break;
                    }
                    i = end;
                    continue;
                }
                let (end, _closed) = scan_single_quoted_string(line, i);
                tokens.push((line[i..end].to_string(), Token::String));
                i = end;
                continue;
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
                let is_call = bytes.get(end) == Some(&b'(');

                let tok = if KEYWORDS.contains(&ident) {
                    Token::Keyword
                } else if BUILTIN_TYPES.contains(&ident) {
                    Token::Type
                } else if CONSTANTS.contains(&ident) {
                    Token::Constant
                } else if is_all_caps(ident) {
                    Token::Constant
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
                    | "//"
                    | "**"
                    | "->"
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
