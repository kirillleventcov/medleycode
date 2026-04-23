//! Left sidebar file tree for workspace navigation.

use gpui::{Rgba, rgb};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A node in the file tree.
#[derive(Clone, Debug)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let is_dir = path.is_dir();
        Some(Self {
            path,
            name,
            is_dir,
            children: Vec::new(),
        })
    }

    fn load_children(&mut self) {
        if !self.is_dir {
            return;
        }
        self.children.clear();
        if let Ok(entries) = std::fs::read_dir(&self.path) {
            let mut nodes: Vec<TreeNode> = entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.file_name()?.to_string_lossy().starts_with('.') {
                        return None;
                    }
                    TreeNode::from_path(path)
                })
                .collect();
            nodes.sort_by(|a, b| {
                // Directories first, then alphabetical
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });
            self.children = nodes;
        }
    }
}

/// File tree sidebar state.
pub struct FileTree {
    root: TreeNode,
    expanded: HashSet<PathBuf>,
    selected_path: Option<PathBuf>,
    pub should_open: Option<PathBuf>,
    pub scroll_offset: f32,
    pub width: f32,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut root = TreeNode::from_path(root_path).unwrap_or_else(|| TreeNode {
            path: PathBuf::from("."),
            name: String::from("."),
            is_dir: true,
            children: Vec::new(),
        });
        root.load_children();
        let mut expanded = HashSet::new();
        expanded.insert(root.path.clone());
        Self {
            root,
            expanded,
            selected_path: None,
            should_open: None,
            scroll_offset: 0.0,
            width: 220.0,
        }
    }

    pub fn root_path(&self) -> &Path {
        &self.root.path
    }

    pub fn toggle_expand(&mut self, path: &Path) {
        if self.expanded.contains(path) {
            self.expanded.remove(path);
        } else {
            self.expanded.insert(path.to_path_buf());
            // Lazily load children when expanding for the first time.
            if let Some(node) = Self::find_node_mut(&mut self.root, path) {
                if node.children.is_empty() {
                    node.load_children();
                }
            }
        }
    }

    fn find_node_mut<'a>(node: &'a mut TreeNode, target: &Path) -> Option<&'a mut TreeNode> {
        if node.path == target {
            return Some(node);
        }
        for child in &mut node.children {
            if let Some(found) = Self::find_node_mut(child, target) {
                return Some(found);
            }
        }
        None
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded.contains(path)
    }

    pub fn select(&mut self, path: PathBuf) {
        self.selected_path = Some(path);
    }

    pub fn selected(&self) -> Option<&PathBuf> {
        self.selected_path.as_ref()
    }

    pub fn click(&mut self, path: &Path) {
        if path.is_dir() {
            self.toggle_expand(path);
        } else {
            self.selected_path = Some(path.to_path_buf());
            self.should_open = Some(path.to_path_buf());
        }
    }

    pub fn move_selection_up(&mut self) {
        let flat = self.flatten_visible();
        if flat.is_empty() {
            return;
        }
        let idx = flat
            .iter()
            .position(|p| self.selected_path.as_ref() == Some(p))
            .unwrap_or(0);
        let new_idx = idx.saturating_sub(1);
        self.selected_path = Some(flat[new_idx].clone());
    }

    pub fn move_selection_down(&mut self) {
        let flat = self.flatten_visible();
        if flat.is_empty() {
            return;
        }
        let idx = flat
            .iter()
            .position(|p| self.selected_path.as_ref() == Some(p))
            .unwrap_or(0);
        let new_idx = (idx + 1).min(flat.len() - 1);
        self.selected_path = Some(flat[new_idx].clone());
    }

    pub fn confirm_selection(&mut self) {
        if let Some(path) = self.selected_path.clone() {
            if path.is_dir() {
                self.toggle_expand(&path);
            } else {
                self.should_open = Some(path);
            }
        }
    }

    /// Flatten visible nodes into a list of paths for keyboard nav.
    fn flatten_visible(&self) -> Vec<PathBuf> {
        let mut result = Vec::new();
        self.flatten_node(&self.root, &mut result);
        result
    }

    fn flatten_node(&self, node: &TreeNode, out: &mut Vec<PathBuf>) {
        out.push(node.path.clone());
        if self.expanded.contains(&node.path) {
            for child in &node.children {
                self.flatten_node(child, out);
            }
        }
    }

    pub fn ensure_visible(&mut self, path: &Path) {
        let mut current = path.parent();
        while let Some(p) = current {
            self.expanded.insert(p.to_path_buf());
            current = p.parent();
        }
        self.select(path.to_path_buf());
    }

    /// Re-scan the root directory (e.g. after external changes).
    pub fn refresh(&mut self) {
        let root_path = self.root.path.clone();
        let mut root = TreeNode::from_path(root_path).unwrap_or_else(|| TreeNode {
            path: PathBuf::from("."),
            name: String::from("."),
            is_dir: true,
            children: Vec::new(),
        });
        root.load_children();
        self.root = root;
    }

    /// Collect visible rows for rendering: (depth, path, name, is_dir, is_selected, is_expanded)
    pub fn visible_rows(&self) -> Vec<(usize, PathBuf, String, bool, bool, bool)> {
        let mut rows = Vec::new();
        self.rows_for_node(&self.root, 0, &mut rows);
        rows
    }

    fn rows_for_node(
        &self,
        node: &TreeNode,
        depth: usize,
        rows: &mut Vec<(usize, PathBuf, String, bool, bool, bool)>,
    ) {
        let is_selected = self.selected_path.as_ref() == Some(&node.path);
        let is_expanded = self.expanded.contains(&node.path);
        rows.push((depth, node.path.clone(), node.name.clone(), node.is_dir, is_selected, is_expanded));
        if is_expanded {
            for child in &node.children {
                self.rows_for_node(child, depth + 1, rows);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileTreeTheme {
    pub background: Rgba,
    pub border: Rgba,
    pub item_text: Rgba,
    pub item_selected_background: Rgba,
    pub item_selected_text: Rgba,
    pub folder_text: Rgba,
    pub scroll_indicator: Rgba,
}

impl Default for FileTreeTheme {
    fn default() -> Self {
        Self {
            background: rgb(0x1f1f1f),
            border: rgb(0x454545),
            item_text: rgb(0xd4d4d4),
            item_selected_background: rgb(0x094771),
            item_selected_text: rgb(0xffffff),
            folder_text: rgb(0x89b4fa),
            scroll_indicator: rgb(0x454545),
        }
    }
}

impl FileTreeTheme {
    pub fn catppuccin_mocha() -> Self {
        Self {
            background: rgb(0x181825),
            border: rgb(0x313244),
            item_text: rgb(0xcdd6f4),
            item_selected_background: rgb(0x585b70),
            item_selected_text: rgb(0xf5c2e7),
            folder_text: rgb(0x89b4fa),
            scroll_indicator: rgb(0x313244),
        }
    }

    pub fn catppuccin_latte() -> Self {
        Self {
            background: rgb(0xedeff3),
            border: rgb(0xe6e9ef),
            item_text: rgb(0x4c4f69),
            item_selected_background: rgb(0x209fb5),
            item_selected_text: rgb(0xe6e9ef),
            folder_text: rgb(0x1e66f5),
            scroll_indicator: rgb(0xe6e9ef),
        }
    }

    pub fn catppuccin_frappe() -> Self {
        Self {
            background: rgb(0x292c3c),
            border: rgb(0x414559),
            item_text: rgb(0xc6d0f5),
            item_selected_background: rgb(0x51576d),
            item_selected_text: rgb(0xf4b8e4),
            folder_text: rgb(0x8caaee),
            scroll_indicator: rgb(0x414559),
        }
    }

    pub fn catppuccin_macchiato() -> Self {
        Self {
            background: rgb(0x1e2030),
            border: rgb(0x363a4f),
            item_text: rgb(0xcad3f5),
            item_selected_background: rgb(0x494d64),
            item_selected_text: rgb(0xf4c2c2),
            folder_text: rgb(0x8aadf4),
            scroll_indicator: rgb(0x363a4f),
        }
    }
}
