//! Markdown tokenizer, migrated from `src/markdown.rs` into the new
//! `Highlighter` shape. Behaviour is unchanged; only the signature
//! changes to `(line, entry) -> (tokens, exit)`.

use super::{Highlighter, LineState, Token};

pub struct MarkdownLang;

impl Highlighter for MarkdownLang {
    fn tokenize_line(line: &str, entry: LineState) -> (Vec<(String, Token)>, LineState) {
        // Code fence handling: inside a fenced block, either continue
        // rendering as code or exit on closing fence.
        if entry == LineState::InMarkdownCodeFence {
            if line.trim_start().starts_with("```") {
                return (
                    vec![(line.to_string(), Token::MdCodeBlock)],
                    LineState::Normal,
                );
            }
            return (
                vec![(line.to_string(), Token::MdCodeBlock)],
                LineState::InMarkdownCodeFence,
            );
        }

        let mut tokens: Vec<(String, Token)> = Vec::new();

        if line.starts_with("# ") {
            tokens.push((line.to_string(), Token::MdHeading(1)));
            return (tokens, LineState::Normal);
        } else if line.starts_with("## ") {
            tokens.push((line.to_string(), Token::MdHeading(2)));
            return (tokens, LineState::Normal);
        } else if line.starts_with("### ") {
            tokens.push((line.to_string(), Token::MdHeading(3)));
            return (tokens, LineState::Normal);
        } else if line.starts_with("#### ") {
            tokens.push((line.to_string(), Token::MdHeading(4)));
            return (tokens, LineState::Normal);
        } else if line.starts_with("##### ") {
            tokens.push((line.to_string(), Token::MdHeading(5)));
            return (tokens, LineState::Normal);
        } else if line.starts_with("###### ") {
            tokens.push((line.to_string(), Token::MdHeading(6)));
            return (tokens, LineState::Normal);
        }

        if line.starts_with("```") {
            tokens.push((line.to_string(), Token::MdCodeBlock));
            return (tokens, LineState::InMarkdownCodeFence);
        }

        if line.starts_with("- [") && line.len() >= 5 {
            let checkbox_char = line.chars().nth(3);
            if checkbox_char == Some(' ') && line.chars().nth(4) == Some(']') {
                tokens.push((line.to_string(), Token::MdCheckboxUnchecked));
                return (tokens, LineState::Normal);
            } else if (checkbox_char == Some('X') || checkbox_char == Some('x'))
                && line.chars().nth(4) == Some(']')
            {
                tokens.push((line.to_string(), Token::MdCheckboxChecked));
                return (tokens, LineState::Normal);
            }
        }

        if line.starts_with("> ") {
            tokens.push((line.to_string(), Token::MdBlockquote));
            return (tokens, LineState::Normal);
        }

        if line.starts_with("- ")
            || line.starts_with("* ")
            || (line.len() > 2
                && line.chars().next().unwrap().is_ascii_digit()
                && &line[1..3] == ". ")
        {
            tokens.push((line.to_string(), Token::MdList));
            return (tokens, LineState::Normal);
        }

        let mut current = String::new();
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '`' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), Token::Normal));
                        current.clear();
                    }
                    current.push(ch);
                    while let Some(next_ch) = chars.next() {
                        current.push(next_ch);
                        if next_ch == '`' {
                            break;
                        }
                    }
                    tokens.push((current.clone(), Token::MdCode));
                    current.clear();
                }
                '*' if chars.peek() == Some(&'*') => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), Token::Normal));
                        current.clear();
                    }
                    current.push(ch);
                    current.push(chars.next().unwrap());
                    while let Some(next_ch) = chars.next() {
                        current.push(next_ch);
                        if next_ch == '*' && chars.peek() == Some(&'*') {
                            current.push(chars.next().unwrap());
                            break;
                        }
                    }
                    tokens.push((current.clone(), Token::MdBold));
                    current.clear();
                }
                '*' | '_' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), Token::Normal));
                        current.clear();
                    }
                    current.push(ch);
                    let delimiter = ch;
                    while let Some(next_ch) = chars.next() {
                        current.push(next_ch);
                        if next_ch == delimiter {
                            break;
                        }
                    }
                    tokens.push((current.clone(), Token::MdItalic));
                    current.clear();
                }
                '[' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), Token::Normal));
                        current.clear();
                    }
                    current.push(ch);
                    while let Some(next_ch) = chars.next() {
                        current.push(next_ch);
                        if next_ch == ')' && current.contains("](") {
                            break;
                        }
                    }
                    tokens.push((current.clone(), Token::MdLink));
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            tokens.push((current, Token::Normal));
        }

        if tokens.is_empty() {
            tokens.push((line.to_string(), Token::Normal));
        }

        (tokens, LineState::Normal)
    }
}
