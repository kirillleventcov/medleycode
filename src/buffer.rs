//! Per-file buffer state.
//!
//! Used for serializing buffer state when switching between open files.

use crate::find::FindPanelState;

/// Represents a single edit operation for undo/redo.
#[derive(Clone, Debug)]
pub struct EditOperation {
    pub old_content: String,
    pub new_content: String,
    pub old_cursor: usize,
    pub new_cursor: usize,
    pub old_selection: Option<usize>,
    pub new_selection: Option<usize>,
}

/// Snapshot of a buffer's state for multi-buffer support.
#[derive(Clone)]
pub struct Buffer {
    pub content: String,
    pub cursor_position: usize,
    pub selection_start: Option<usize>,
    pub is_dirty: bool,
    pub undo_stack: Vec<EditOperation>,
    pub redo_stack: Vec<EditOperation>,
    pub scroll_offset: f32,
    pub find_panel: Option<FindPanelState>,
}

impl Buffer {
    pub fn new(content: String) -> Self {
        Self {
            content,
            cursor_position: 0,
            selection_start: None,
            is_dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            scroll_offset: 0.0,
            find_panel: None,
        }
    }
}
