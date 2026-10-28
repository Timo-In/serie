use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::{
    color::ColorTheme,
    git::detail::{display_path, ChangedFile},
};

#[derive(Debug)]
struct Node {
    path: PathBuf,
    depth: usize,
    file: Option<usize>,
    expanded: bool,
}

#[derive(Debug, Default)]
struct Branch {
    file: Option<usize>,
    children: BTreeMap<OsString, Branch>,
}

#[derive(Debug, Default)]
pub struct FileTreeState {
    nodes: Vec<Node>,
    visible: Vec<usize>,
    list: ListState,
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
            .map(|i| {
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
                ListItem::new(Line::from(vec![
                    Span::raw(indent),
                    Span::styled(marker, Style::default().fg(color)),
                    Span::raw(name),
                ]))
            })
            .collect();
        let list = List::new(items)
            .block(block)
            .style(Style::default().fg(theme.fg))
            .highlight_style(
                Style::default()
                    .fg(theme.list_selected_fg)
                    .bg(theme.list_selected_bg)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_stateful_widget(list, area, &mut self.list);
    }
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
                assert_eq!(buffer[(3, 2)].fg, theme.list_selected_fg);
                assert_eq!(buffer[(3, 2)].bg, theme.list_selected_bg);
            }
        }
    }
}
