use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use laurier::highlight::highlight_matched_text;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::{
    color::ColorTheme,
    git::detail::{display_path, ChangedFile},
};

#[derive(Debug, Clone)]
struct Node {
    path: PathBuf,
    depth: usize,
    file: Option<usize>,
    expanded: bool,
    matched_indices: Vec<usize>,
}

#[derive(Debug, Default)]
struct Branch {
    file: Option<usize>,
    children: BTreeMap<OsString, Branch>,
}

#[derive(Debug, Default, Clone)]
pub struct FileTreeState {
    nodes: Vec<Node>,
    visible: Vec<usize>,
    list: ListState,
    query: String,
    matches: Vec<usize>,
}

impl FileTreeState {
    pub fn new(files: &[ChangedFile]) -> Self {
        let mut root = Branch::default();
        for (index, file) in files.iter().enumerate() {
            let mut branch = &mut root;
            for component in file.path.components() {
                branch = branch
                    .children
                    .entry(component.as_os_str().into())
                    .or_default();
            }
            branch.file = Some(index);
        }
        let mut tree = Self::default();
        tree.append_branches(root, PathBuf::new(), 0);
        tree.rebuild_visible();
        // Start on a file so the patch is immediately useful when opening a commit.
        if let Some(index) = tree
            .visible
            .iter()
            .position(|i| tree.nodes[*i].file.is_some())
        {
            tree.list.select(Some(index));
        }
        tree
    }

    fn append_branches(&mut self, branch: Branch, parent: PathBuf, depth: usize) {
        let mut children: Vec<_> = branch.children.into_iter().collect();
        children.sort_by(|(a_name, a), (b_name, b)| {
            a.children
                .is_empty()
                .cmp(&b.children.is_empty())
                .then(a_name.cmp(b_name))
        });
        for (name, child) in children {
            let path = parent.join(name);
            self.nodes.push(Node {
                path: path.clone(),
                depth,
                file: child.file,
                expanded: true,
                matched_indices: Vec::new(),
            });
            self.append_branches(child, path, depth + 1);
        }
    }

    fn rebuild_visible(&mut self) {
        let mut hidden_below = None;
        self.visible.clear();
        for (index, node) in self.nodes.iter().enumerate() {
            if hidden_below.is_some_and(|depth| node.depth > depth) {
                continue;
            }
            hidden_below = None;
            self.visible.push(index);
            if node.file.is_none() && !node.expanded {
                hidden_below = Some(node.depth);
            }
        }
        let selected = self
            .list
            .selected()
            .unwrap_or(0)
            .min(self.visible.len().saturating_sub(1));
        self.list
            .select((!self.visible.is_empty()).then_some(selected));
    }

    pub fn selected_file(&self) -> Option<usize> {
        let selected = self.list.selected()?;
        self.nodes[*self.visible.get(selected)?].file
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn search(&mut self, query: &str) {
        self.query = query.into();
        self.matches.clear();
        for (index, node) in self.nodes.iter_mut().enumerate() {
            let name = display_path(node.path.file_name().unwrap_or_default().as_ref());
            node.matched_indices = if node.file.is_some() {
                filename_matches(&name, query)
            } else {
                Vec::new()
            };
            if !node.matched_indices.is_empty() {
                self.matches.push(index);
            }
        }
        let selected = self.list.selected().map(|i| self.visible[i]).unwrap_or(0);
        if let Some(&target) = self
            .matches
            .iter()
            .find(|i| **i >= selected)
            .or(self.matches.first())
        {
            self.reveal(target);
        }
    }

    fn reveal(&mut self, target: usize) {
        let path = self.nodes[target].path.clone();
        for node in &mut self.nodes[..target] {
            if node.file.is_none() && path.starts_with(&node.path) {
                node.expanded = true;
            }
        }
        self.rebuild_visible();
        self.list
            .select(self.visible.iter().position(|i| *i == target));
    }

    pub fn next_match(&mut self, forward: bool, count: usize) {
        let total = self.matches.len();
        if total == 0 {
            return;
        }
        let selected = self.list.selected().map(|i| self.visible[i]).unwrap_or(0);
        let steps = count.saturating_sub(1) % total;
        let next = if forward {
            let first = self.matches.partition_point(|i| *i <= selected) % total;
            (first + steps) % total
        } else {
            let first = self
                .matches
                .partition_point(|i| *i < selected)
                .checked_sub(1)
                .unwrap_or(total - 1);
            (first + total - steps) % total
        };
        self.reveal(self.matches[next]);
    }

    pub fn search_status(&self) -> (usize, usize) {
        let selected = self.list.selected().map(|i| self.visible[i]);
        let current = self
            .matches
            .iter()
            .position(|i| Some(*i) == selected)
            .map_or(0, |i| i + 1);
        (current, self.matches.len())
    }

    pub fn move_selection(&mut self, down: bool, count: usize) {
        if let Some(selected) = self.list.selected() {
            let next = if down {
                selected
                    .saturating_add(count)
                    .min(self.visible.len().saturating_sub(1))
            } else {
                selected.saturating_sub(count)
            };
            self.list.select(Some(next));
        }
    }

    pub fn first(&mut self) {
        self.list.select((!self.visible.is_empty()).then_some(0));
    }

    pub fn last(&mut self) {
        self.list.select(self.visible.len().checked_sub(1));
    }

    pub fn expand(&mut self) {
        if let Some(selected) = self.list.selected() {
            let node = &mut self.nodes[self.visible[selected]];
            if node.file.is_none() {
                if node.expanded {
                    self.move_selection(true, 1);
                } else {
                    node.expanded = true;
                    self.rebuild_visible();
                }
            }
        }
    }

    pub fn activate(&mut self) -> Option<usize> {
        let selected = self.list.selected()?;
        let node = &mut self.nodes[self.visible[selected]];
        if let Some(index) = node.file {
            Some(index)
        } else {
            node.expanded = !node.expanded;
            self.rebuild_visible();
            None
        }
    }

    pub fn collapse(&mut self) {
        if let Some(selected) = self.list.selected() {
            let node = &mut self.nodes[self.visible[selected]];
            if node.file.is_none() && node.expanded {
                node.expanded = false;
                self.rebuild_visible();
            } else if node.depth > 0 {
                let depth = node.depth;
                if let Some(parent) = (0..selected)
                    .rev()
                    .find(|i| self.nodes[self.visible[*i]].depth < depth)
                {
                    self.list.select(Some(parent));
                }
            }
        }
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        files: &[ChangedFile],
        theme: &ColorTheme,
    ) {
        let items: Vec<ListItem> = self
            .visible
            .iter()
            .enumerate()
            .map(|(row, i)| {
                let node = &self.nodes[*i];
                let name = display_path(node.path.file_name().unwrap_or_default().as_ref());
                let indent = "  ".repeat(node.depth);
                let (marker, color) = if let Some(index) = node.file {
                    let status = files[index].status;
                    let color = match status {
                        'A' => theme.detail_file_change_add_fg,
                        'D' => theme.detail_file_change_delete_fg,
                        'R' | 'C' => theme.detail_file_change_move_fg,
                        _ => theme.detail_file_change_modify_fg,
                    };
                    (format!("{status} "), color)
                } else {
                    (if node.expanded { "▾ " } else { "▸ " }.into(), theme.fg)
                };
                let selected = self.list.selected() == Some(row);
                let base_style = Style::default().fg(if selected {
                    theme.list_selected_fg
                } else {
                    theme.list_subject_fg
                });
                let mut spans = vec![
                    Span::raw(indent),
                    Span::styled(marker, Style::default().fg(color)),
                ];
                spans.extend(
                    highlight_matched_text(vec![Span::raw(name)])
                        .matched_indices(node.matched_indices.clone())
                        .not_matched_style(base_style)
                        .matched_style(
                            Style::default()
                                .fg(theme.list_match_fg)
                                .bg(theme.list_match_bg),
                        )
                        .into_spans(),
                );
                let mut line = Line::from(spans);
                if selected {
                    line = line.style(
                        Style::default()
                            .fg(theme.list_selected_fg)
                            .bg(theme.list_selected_bg),
                    );
                }
                ListItem::new(line)
            })
            .collect();
        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(theme.fg));
        frame.render_stateful_widget(list, area, &mut self.list);
    }
}

fn filename_matches(name: &str, query: &str) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let mut folded = String::new();
    let mut original = Vec::new();
    for (index, c) in name.char_indices() {
        let lower = c.to_lowercase().collect::<String>();
        original.extend(std::iter::repeat_n((index, c.len_utf8()), lower.len()));
        folded.push_str(&lower);
    }
    let mut indices = Vec::new();
    for (start, matched) in folded.match_indices(&query.to_lowercase()) {
        let (end, width) = original[start + matched.len() - 1];
        indices.extend(original[start].0..end + width);
    }
    indices.sort_unstable();
    indices.dedup();
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_and_expand_preserve_nested_selection() {
        let files = ["src/a.rs", "src/nested/b.rs", "README.md"].map(|path| ChangedFile {
            status: 'M',
            path: path.into(),
            old_path: None,
        });
        let mut tree = FileTreeState::new(&files);
        assert_eq!(tree.selected_file(), Some(1));
        tree.collapse();
        assert_eq!(tree.selected_file(), None);
        tree.collapse();
        assert_eq!(tree.visible.len(), 4);
        tree.collapse();
        tree.collapse();
        assert_eq!(tree.visible.len(), 2);
        tree.expand();
        tree.last();
        assert_eq!(tree.selected_file(), Some(2));
        tree.move_selection(true, usize::MAX);
        assert_eq!(tree.selected_file(), Some(2));
        tree.first();
        tree.expand();
        tree.expand();
        tree.expand();
        assert_eq!(tree.selected_file(), Some(1));
    }

    #[test]
    fn empty_tree_can_be_navigated() {
        let mut tree = FileTreeState::new(&[]);
        tree.first();
        tree.last();
        tree.expand();
        tree.collapse();
        tree.move_selection(true, 10);
        assert_eq!(tree.selected_file(), None);
        assert_eq!(tree.activate(), None);
    }

    #[test]
    fn enter_toggles_directories_and_returns_file_index() {
        let files = [ChangedFile {
            status: 'A',
            path: "src/a.rs".into(),
            old_path: None,
        }];
        let mut tree = FileTreeState::new(&files);
        assert_eq!(tree.activate(), Some(0));
        tree.first();
        assert_eq!(tree.activate(), None);
        assert_eq!(tree.visible.len(), 1);
        assert_eq!(tree.activate(), None);
        assert_eq!(tree.visible.len(), 2);
        tree.move_selection(true, 1);
        assert_eq!(tree.activate(), Some(0));
    }

    #[test]
    fn tree_renders_with_serie_theme_in_small_terminals() {
        use ratatui::{backend::TestBackend, Terminal};
        let files = [ChangedFile {
            status: 'A',
            path: "src/a.rs".into(),
            old_path: None,
        }];
        let theme = ColorTheme::default();
        for (width, height) in [(1, 1), (5, 2), (40, 10)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let mut tree = FileTreeState::new(&files);
            terminal
                .draw(|frame| tree.render(frame, frame.area(), Block::bordered(), &files, &theme))
                .unwrap();
            if width == 40 {
                let buffer = terminal.backend().buffer();
                assert_eq!(buffer[(5, 2)].fg, theme.list_selected_fg);
                assert_eq!(buffer[(5, 2)].bg, theme.list_selected_bg);
                assert_eq!(buffer[(35, 2)].bg, theme.list_selected_bg);
                assert!(!buffer[(5, 2)]
                    .modifier
                    .contains(ratatui::style::Modifier::BOLD));
            }
        }
    }

    #[test]
    fn filename_search_reveals_collapsed_matches_and_wraps() {
        let files = ["src/first.rs", "tests/first.rs", "first/other.txt"].map(|path| ChangedFile {
            status: 'M',
            path: path.into(),
            old_path: None,
        });
        let mut tree = FileTreeState::new(&files);
        tree.first();
        tree.activate();
        tree.search("FIRST");
        assert_eq!(tree.search_status(), (1, 2));
        assert_eq!(tree.selected_file(), Some(0));
        tree.next_match(true, 1);
        assert_eq!(tree.search_status(), (2, 2));
        assert_eq!(tree.selected_file(), Some(1));
        tree.next_match(true, 1);
        assert_eq!(tree.selected_file(), Some(0));
        tree.next_match(false, 1);
        assert_eq!(tree.selected_file(), Some(1));
        tree.next_match(true, usize::MAX);
        assert_eq!(tree.selected_file(), Some(0));
        tree.search("not-a-file");
        assert_eq!(tree.search_status(), (0, 0));
        tree.next_match(true, 1);
        assert_eq!(tree.selected_file(), Some(0));
        tree.search("");
        assert!(tree
            .nodes
            .iter()
            .all(|node| node.matched_indices.is_empty()));
        let mut empty = FileTreeState::new(&[]);
        empty.search("file");
        empty.next_match(false, 1);
        assert_eq!(empty.selected_file(), None);
    }

    #[test]
    fn filename_matches_preserve_utf8_boundaries_during_case_folding() {
        assert_eq!(filename_matches("中文.rs", "文"), vec![3, 4, 5]);
        assert_eq!(filename_matches("İ.rs", "i"), vec![0, 1]);
        assert_eq!(filename_matches("İİ.rs", "i"), vec![0, 1, 2, 3]);
        assert_eq!(filename_matches("First.rs", "FIRST"), vec![0, 1, 2, 3, 4]);
        assert!(filename_matches("first.rs", "").is_empty());
    }

    #[test]
    fn search_highlights_selected_unicode_filename_with_serie_match_colors() {
        use ratatui::{backend::TestBackend, Terminal};
        let files = [ChangedFile {
            status: 'A',
            path: "中文.rs".into(),
            old_path: None,
        }];
        let theme = ColorTheme::default();
        let mut tree = FileTreeState::new(&files);
        tree.search("文");
        let mut terminal = Terminal::new(TestBackend::new(30, 5)).unwrap();
        terminal
            .draw(|frame| tree.render(frame, frame.area(), Block::bordered(), &files, &theme))
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(3, 1)].fg, theme.list_selected_fg);
        assert_eq!(buffer[(5, 1)].fg, theme.list_match_fg);
        assert_eq!(buffer[(5, 1)].bg, theme.list_match_bg);
    }
}
