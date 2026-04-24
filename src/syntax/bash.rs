//! Bash tokenizer. Heredoc body tracking is deferred per spec.

use super::common::*;
use super::{Highlighter, LineState, Token};

const KEYWORDS: &[&str] = &[
    "if", "then", "elif", "else", "fi", "for", "in", "do", "done", "while", "until", "case",
    "esac", "function", "return", "break", "continue", "local", "export", "declare", "readonly",
    "unset",
];

pub struct BashLang;

impl Highlighter for BashLang {
    fn tokenize_line(line: &str, _entry: LineState) -> (Vec<(String, Token)>, LineState) {
        let mut tokens: Vec<(String, Token)> = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            let b = bytes[i];

            if b == b'#' {
                tokens.push((line[i..].to_string(), Token::Comment));
                break;
            }

            if b == b'"' {
                let (end, _closed) = scan_double_quoted_string(line, i);
                tokens.push((line[i..end].to_string(), Token::String));
                i = end;
                continue;
            }

            if b == b'\'' {
                // Single quotes in bash: no escapes.
                let start = i;
                let mut j = i + 1;
                while j < bytes.len() && bytes[j] != b'\'' {
                    j += 1;
                }
                let end = if j < bytes.len() { j + 1 } else { bytes.len() };
                tokens.push((line[start..end].to_string(), Token::String));
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
            if matches!(two, "==" | "!=" | "<=" | ">=" | "&&" | "||" | "<<" | ">>") {
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
        (tokens, LineState::Normal)
    }
}
