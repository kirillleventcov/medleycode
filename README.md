# MedleyCode

A lightweight, handwritten code editor built from scratch with [GPUI](https://crates.io/crates/gpui).

This is a fork of [MedleyText](https://github.com/) — the markdown-first editor — repurposed into a small, keyboard-driven code editor. The markdown editing path is preserved, and a hand-rolled syntax highlighter has been added for several common source languages.

## Features

- **File tree sidebar** with lazy folder expansion (toggle with `Ctrl+B`)
- **Fuzzy file finder** — quick-open palette (`Ctrl+P` / `Ctrl+O`) for instant navigation across the working directory
- **Hand-written syntax highlighting** for:
  - Rust, Python, JavaScript, TypeScript, Bash
  - Markdown (headings, bold, italic, code, links, lists, checkboxes, blockquotes, fenced code blocks)
  - Plain text fallback for unrecognized files
- Language detection from file extension and `#!` shebang
- Multi-line state tracking for block comments, triple-quoted strings, template literals, heredocs, and markdown code fences
- Find / replace, go-to-line, undo/redo, autocomplete menu
- Configurable theme with multiple presets and per-token color overrides
- Minimal interface, keyboard-driven workflow
- Zero external dependencies (except GPUI)

## Usage

```bash
medleycode [path]
```

`path` is optional and may be either a file or a directory:

- **File** — opens the file in the editor; the sidebar shows the file's containing directory.
- **Directory** — opens an empty buffer; the sidebar shows that directory's contents.
- **Missing path** — creates a new empty buffer with that filename.
- **No argument** — opens the current working directory.

**Keybindings:**

- `Ctrl+O` / `Ctrl+P` — Open fuzzy file finder
- `Ctrl+B` — Toggle file tree sidebar
- `Ctrl+S` — Save
- `Ctrl+Q` — Quit
- `Ctrl+A` — Select all
- `Ctrl+C` / `Ctrl+V` / `Ctrl+X` — Copy / Paste / Cut
- `Ctrl+Z` / `Ctrl+Shift+Z` (or `Ctrl+Y`) — Undo / Redo
- `Ctrl+G` — Go to line
- `Ctrl+F` — Find
- `F3` / `Shift+F3` — Find next / previous
- `Ctrl+←` / `Ctrl+→` — Move by word
- `Home` / `End` — Move to line start / end
- `Delete` — Forward delete
- `Tab` — Indent (2 spaces, or indent selected lines)
- `Shift+Tab` — Unindent selected lines
- Arrow keys — Navigate (`Shift` to select)
- Mouse drag — Select text
- Standard typing and editing

**Fuzzy file finder:**

- Type to search files with fuzzy matching
- `↑` / `↓` — Navigate results
- `Enter` — Open selected file
- `Esc` — Close palette

## Building

### macOS

Follow [Zed's MacOS build guide](https://github.com/zed-industries/zed/blob/main/docs/src/development/macos.md) to install the necessary dependencies.

```bash
cargo build --release
```

The resulting binary is `target/release/medleycode`.

## Configuration

MedleyCode reads optional settings from `~/.config/medleycode/config`. Each non-empty line uses either `key=value` or `key: value`. Lines beginning with `#` or `//` are ignored.

The file is created automatically with sensible defaults the first time you launch the editor, so you can open it and tweak values right away. Restart the editor (or close/open the window) after editing the config to load new values.

### Core options

- `font-size` — UI font size (clamped between 8 and 72, default `14`)

### Theme presets

Use a preset as the base palette, then override individual keys as needed:

```
theme.preset = catppuccin-mocha
```

Available presets: `default`, `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `catppuccin-latte`.

### Color overrides

Color values accept `#RRGGBB` or `0xRRGGBB`. Any unspecified key falls back to the current preset/default.

- **Editor surface**
  - `theme.editor.background`
  - `theme.editor.border`
  - `theme.editor.text`
  - `theme.editor.muted-text`
  - `theme.editor.cursor`
- **Highlights**
  - `theme.highlight.selection.background`
  - `theme.highlight.selection.foreground`
  - `theme.highlight.search-active.background`
  - `theme.highlight.search-active.foreground`
  - `theme.highlight.search-match.background`
  - `theme.highlight.search-match.foreground`
- **File tree sidebar**
  - `theme.file-tree.background`
  - `theme.file-tree.border`
  - `theme.file-tree.item-text`
  - `theme.file-tree.item-selected.background`
  - `theme.file-tree.item-selected.text`
  - `theme.file-tree.folder-text`
- **Panels (find dialog)**
  - `theme.panel.background`
  - `theme.panel.border`
  - `theme.panel.active-row.background`
  - `theme.panel.inactive-row.background`
  - `theme.panel.label-text`
  - `theme.panel.value-text`
  - `theme.panel.placeholder-text`
  - `theme.panel.status-text`
  - `theme.panel.shortcut-text`
- **Command palette**
  - `theme.palette.background`
  - `theme.palette.border`
  - `theme.palette.input-text`
  - `theme.palette.item.background`
  - `theme.palette.item.foreground`
  - `theme.palette.item-selected.background`
  - `theme.palette.item-selected.foreground`
  - `theme.palette.footer-text`
- **Autocomplete menu**
  - `theme.autocomplete.background`
  - `theme.autocomplete.border`
  - `theme.autocomplete.item.background`
  - `theme.autocomplete.item.foreground`
  - `theme.autocomplete.item-selected.background`
  - `theme.autocomplete.item-selected.foreground`
  - `theme.autocomplete.label-text`
- **Source code syntax** (Rust / Python / JS / TS / Bash)
  - `theme.syntax.code.keyword`
  - `theme.syntax.code.string`
  - `theme.syntax.code.number`
  - `theme.syntax.code.comment`
  - `theme.syntax.code.punctuation`
  - `theme.syntax.code.operator`
  - `theme.syntax.code.type`
  - `theme.syntax.code.function`
  - `theme.syntax.code.constant`
  - `theme.syntax.code.normal`
- **Markdown syntax**
  - `theme.syntax.heading1` … `theme.syntax.heading6` (or `theme.syntax.heading` for all levels)
  - `theme.syntax.bold`
  - `theme.syntax.italic`
  - `theme.syntax.code`
  - `theme.syntax.code-block`
  - `theme.syntax.link`
  - `theme.syntax.list`
  - `theme.syntax.checkbox-checked`
  - `theme.syntax.checkbox-unchecked`
  - `theme.syntax.blockquote`
  - `theme.syntax.normal`

### Example: Catppuccin Mocha + tweaks

```
font-size = 16
theme.preset = catppuccin-mocha

# Personal accents
theme.highlight.selection.background = #F5C2E7
theme.highlight.selection.foreground = #1E1E2E
theme.palette.item-selected.background = #B4BEFE
theme.palette.item-selected.foreground = #1E1E2E
theme.syntax.code.keyword = #F5C2E7
```

## Architecture notes

The syntax highlighter lives under `src/syntax/` and is intentionally tiny — each language is a single hand-written tokenizer that emits a `Vec<(String, Token)>` per line and threads a `LineState` enum across lines to track multi-line constructs. Adding a language means adding a module with a `tokenize_line` function and one match arm in `src/syntax/mod.rs`. There are no tree-sitter or regex grammar dependencies.

## Documentation

Built with [GPUI](https://docs.rs/gpui/latest/gpui/), a GPU-accelerated UI framework for Rust.
