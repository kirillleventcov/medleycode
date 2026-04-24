//! Shared scan helpers used by per-language tokenizers. Each helper takes
//! a byte-slice iterator position and returns the end offset of the match.
//! These are utilities — nothing in here dictates control flow in a
//! language module.

/// Returns the byte offset at the end of an identifier starting at `start`.
/// An identifier is `[A-Za-z_][A-Za-z0-9_]*`. Returns `start` if no ident.
pub fn scan_identifier(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    if start >= bytes.len() {
        return start;
    }
    let c = bytes[start];
    if !(c.is_ascii_alphabetic() || c == b'_') {
        return start;
    }
    let mut i = start + 1;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' {
            i += 1;
        } else {
            break;
        }
    }
    i
}

/// Returns the byte offset at the end of a number literal starting at `start`.
/// Covers decimal, hex (0x), octal (0o), binary (0b), floats, underscore
/// separators, and trailing language suffixes made of alphanumerics.
pub fn scan_number(s: &str, start: usize) -> usize {
    let bytes = s.as_bytes();
    if start >= bytes.len() || !bytes[start].is_ascii_digit() {
        return start;
    }
    let mut i = start;
    // Prefixed bases: 0x, 0o, 0b
    if bytes[i] == b'0' && i + 1 < bytes.len() {
        match bytes[i + 1] {
            b'x' | b'X' | b'o' | b'O' | b'b' | b'B' => {
                i += 2;
                while i < bytes.len() && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'_') {
                    i += 1;
                }
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                return i;
            }
            _ => {}
        }
    }
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        if i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                i += 1;
            }
        }
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    // Numeric suffix (e.g. i32, u64, f32, n for BigInt)
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i
}

/// Scans a double-quoted string literal (with `\` escapes) starting at `start`
/// where `bytes[start] == b'"'`. Returns `(end_exclusive, closed)` — `closed`
/// is `false` when the line ended before the closing quote.
pub fn scan_double_quoted_string(s: &str, start: usize) -> (usize, bool) {
    let bytes = s.as_bytes();
    debug_assert!(bytes.get(start) == Some(&b'"'));
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'"' => return (i + 1, true),
            _ => i += 1,
        }
    }
    (i, false)
}

/// Scans a single-quoted string literal (with `\` escapes) starting at `start`
/// where `bytes[start] == b'\''`. Returns `(end_exclusive, closed)`.
pub fn scan_single_quoted_string(s: &str, start: usize) -> (usize, bool) {
    let bytes = s.as_bytes();
    debug_assert!(bytes.get(start) == Some(&b'\''));
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'\'' => return (i + 1, true),
            _ => i += 1,
        }
    }
    (i, false)
}

/// Returns true if `ident` is entirely ASCII uppercase / digits / underscores
/// with at least one letter — the convention for module-level constants.
pub fn is_all_caps(ident: &str) -> bool {
    let mut has_alpha = false;
    for b in ident.bytes() {
        if b.is_ascii_lowercase() {
            return false;
        }
        if b.is_ascii_uppercase() {
            has_alpha = true;
        } else if !b.is_ascii_digit() && b != b'_' {
            return false;
        }
    }
    has_alpha
}

/// Returns true if `ident` starts with an ASCII uppercase letter.
pub fn starts_uppercase(ident: &str) -> bool {
    ident
        .as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_uppercase())
}

/// Returns the punctuation/operator class for a single ASCII byte. `None` means
/// the byte is neither. `Some(true)` is punctuation; `Some(false)` is operator.
pub fn classify_ascii_punct(b: u8) -> Option<bool> {
    match b {
        b'(' | b')' | b'{' | b'}' | b'[' | b']' | b';' | b',' | b'.' | b':' => Some(true),
        b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>' | b'!' | b'&' | b'|' | b'^'
        | b'~' | b'?' | b'@' => Some(false),
        _ => None,
    }
}
