//! TypeScript tokenizer — adds TS keywords and built-in type names over
//! the shared JavaScript pipeline. JSX tags are deferred per spec.

use super::javascript::tokenize_line_with;
use super::{Highlighter, LineState, Token};

const TS_EXTRA_KEYWORDS: &[&str] = &[
    "interface",
    "type",
    "enum",
    "readonly",
    "implements",
    "public",
    "private",
    "protected",
    "abstract",
    "namespace",
    "declare",
    "keyof",
    "is",
    "satisfies",
    "never",
];

const TS_BUILTIN_TYPES: &[&str] = &[
    "string", "number", "boolean", "any", "unknown", "void", "never", "object",
];

pub struct TypeScriptLang;

impl Highlighter for TypeScriptLang {
    fn tokenize_line(line: &str, entry: LineState) -> (Vec<(String, Token)>, LineState) {
        tokenize_line_with(line, entry, TS_EXTRA_KEYWORDS, TS_BUILTIN_TYPES)
    }
}
