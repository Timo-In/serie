use std::{collections::HashMap, rc::Rc};

use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::AppContext,
    color::ColorTheme,
    event::{AppEvent, Sender, UserEvent, UserEventWithCount},
    git::{detail::ChangedFile, Commit, Repository},
    view::{ListRefreshViewContext, RefreshViewContext},
    widget::{commit_list::CommitListState, file_tree::FileTreeState},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Panel {
    #[default]
    Files,
    Patch,
}

#[derive(Debug, Default)]
struct PanelState {
    focused: Panel,
    fullscreen_patch: bool,
    height: usize,
    width: usize,
    patch_y: usize,
    patch_x: usize,
}

impl PanelState {
    fn switch(&mut self) {
        self.fullscreen_patch = !self.fullscreen_patch;
        self.focused = if self.fullscreen_patch {
            Panel::Patch
        } else {
            Panel::Files
        };
    }

    fn areas(&self, area: Rect) -> (Option<Rect>, Option<Rect>) {
        if self.fullscreen_patch {
            return (None, Some(area));
        }
        let [files, patch] =
            Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
                .areas(area);
        (Some(files), Some(patch))
    }

    fn cancel(&mut self) -> bool {
        if self.fullscreen_patch {
            self.switch();
            false
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct GitDetailView<'a> {
    commit_list_state: Option<CommitListState<'a>>,
    repository: &'a Repository,
    commit: Commit,
    files: Vec<ChangedFile>,
    tree: FileTreeState,
    panels: PanelState,
    patches: HashMap<usize, Vec<Line<'static>>>,
    selected_file: Option<usize>,
    ctx: Rc<AppContext>,
    tx: Sender,
}

impl<'a> GitDetailView<'a> {
    pub fn new(
        commit_list_state: CommitListState<'a>,
        repository: &'a Repository,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        let commit = repository
            .commit(commit_list_state.selected_commit_hash())
            .unwrap()
            .clone();
        let mut view = Self {
            commit_list_state: Some(commit_list_state),
            repository,
            commit,
            files: Vec::new(),
            tree: FileTreeState::default(),
            panels: PanelState::default(),
            patches: HashMap::new(),
            selected_file: None,
            ctx,
            tx,
        };
        view.load_files();
        view
    }

    fn load_files(&mut self) {
        self.files = self
            .repository
            .detail_files(&self.commit)
            .unwrap_or_else(|error| {
                self.tx.send(AppEvent::NotifyError(error));
                Vec::new()
            });
        self.tree = FileTreeState::new(&self.files);
        self.patches.clear();
        self.selected_file = None;
        self.update_patch();
    }

    fn update_patch(&mut self) {
        let selected = self.tree.selected_file();
        if self.selected_file == selected {
            return;
        }
        self.selected_file = selected;
        self.panels.patch_x = 0;
        self.panels.patch_y = 0;
        if let Some(index) = selected {
            if !self.patches.contains_key(&index) {
                let lines = self
                    .repository
                    .detail_patch(&self.commit, &self.files[index])
                    .map(|patch| {
                        build_patch_lines(
                            &patch,
                            &self.ctx.color_theme,
                            self.ctx.core_config.user_command.tab_width as usize,
                        )
                    })
                    .unwrap_or_else(|error| vec![Line::raw(error)]);
                self.patches.insert(index, lines);
            }
        }
    }

    fn patch_lines(&self) -> &[Line<'static>] {
        self.selected_file
            .and_then(|index| self.patches.get(&index))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    fn scroll(&mut self, down: bool, count: usize) {
        match self.panels.focused {
            Panel::Files => self.tree.move_selection(down, count),
            Panel::Patch => {
                let max = self.patch_lines().len().saturating_sub(self.panels.height);
                self.panels.patch_y = if down {
                    self.panels.patch_y.saturating_add(count).min(max)
                } else {
                    self.panels.patch_y.saturating_sub(count)
                };
            }
        }
    }

    pub fn handle_event(&mut self, event: UserEventWithCount, _: KeyEvent) {
        match event.event {
            UserEvent::RefList | UserEvent::FocusPrevious => self.panels.switch(),
            UserEvent::Confirm
                if !self.panels.fullscreen_patch && self.tree.activate().is_some() =>
            {
                self.tx.send(AppEvent::OpenGitEditor);
            }
            UserEvent::Cancel | UserEvent::Close | UserEvent::Quit if self.panels.cancel() => {
                self.tx.send(AppEvent::CloseGitDetail);
            }
            UserEvent::EditFile
                if !self.panels.fullscreen_patch && self.selected_file.is_some() =>
            {
                self.tx.send(AppEvent::OpenGitEditor);
            }
            UserEvent::NavigateDown | UserEvent::ScrollDown => self.scroll(true, event.count),
            UserEvent::NavigateUp | UserEvent::ScrollUp => self.scroll(false, event.count),
            UserEvent::PageDown | UserEvent::HalfPageDown => {
                let step = self.page_step(event.event);
                self.scroll(true, step.saturating_mul(event.count));
            }
            UserEvent::PageUp | UserEvent::HalfPageUp => {
                let step = self.page_step(event.event);
                self.scroll(false, step.saturating_mul(event.count));
            }
            UserEvent::NavigateLeft => match self.panels.focused {
                Panel::Files => self.tree.collapse(),
                Panel::Patch => {
                    self.panels.patch_x = self.panels.patch_x.saturating_sub(event.count)
                }
            },
            UserEvent::NavigateRight => match self.panels.focused {
                Panel::Files => self.tree.expand(),
                Panel::Patch => {
                    let max = self
                        .patch_lines()
                        .iter()
                        .map(Line::width)
                        .max()
                        .unwrap_or(0)
                        .saturating_sub(self.panels.width)
                        .min(u16::MAX as usize);
                    self.panels.patch_x = self.panels.patch_x.saturating_add(event.count).min(max);
                }
            },
            UserEvent::GoToTop => match self.panels.focused {
                Panel::Files => self.tree.first(),
                Panel::Patch => self.panels.patch_y = 0,
            },
            UserEvent::GoToBottom => match self.panels.focused {
                Panel::Files => self.tree.last(),
                Panel::Patch => {
                    self.panels.patch_y =
                        self.patch_lines().len().saturating_sub(self.panels.height)
                }
            },
            UserEvent::SelectDown => self.tx.send(AppEvent::SelectOlderCommit),
            UserEvent::SelectUp => self.tx.send(AppEvent::SelectNewerCommit),
            UserEvent::GoToParent => self.tx.send(AppEvent::SelectParentCommit),
            UserEvent::MarkToggle => self.commit_list_state.as_mut().unwrap().toggle_mark(),
            UserEvent::ShortCopy | UserEvent::FullCopy => {
                let value = if event.event == UserEvent::ShortCopy {
                    self.commit.commit_hash.as_short_hash()
                } else {
                    self.commit.commit_hash.as_str()
                };
                self.tx.send(AppEvent::CopyToClipboard {
                    name: "Commit SHA".into(),
                    value: value.into(),
                });
            }
            UserEvent::UserCommand(n) => self.tx.send(AppEvent::OpenUserCommand(n)),
            UserEvent::HelpToggle => self.tx.send(AppEvent::OpenHelp),
            UserEvent::Refresh => self.refresh(),
            _ => {}
        }
        self.update_patch();
    }

    fn page_step(&self, event: UserEvent) -> usize {
        match event {
            UserEvent::HalfPageDown | UserEvent::HalfPageUp => (self.panels.height / 2).max(1),
            _ => self.panels.height.max(1),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [content, footer] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
        let (files_area, patch_area) = self.panels.areas(content);
        if let Some(area) = files_area {
            let title = format!(
                " Files · {} {} ",
                self.commit.commit_hash.as_short_hash(),
                self.commit.subject
            );
            let block = self.block(title, Panel::Files);
            self.panels.height = block.inner(area).height as usize;
            self.tree
                .render(frame, area, block, &self.files, &self.ctx.color_theme);
        }
        if let Some(area) = patch_area {
            let name = self
                .selected_file
                .map(|i| {
                    let file = &self.files[i];
                    if let Some(old) = &file.old_path {
                        format!(
                            "{} → {}",
                            crate::git::detail::display_path(old),
                            file.label()
                        )
                    } else {
                        file.label()
                    }
                })
                .unwrap_or_default();
            let block = self.block(format!(" Patch · {name} "), Panel::Patch);
            let inner = block.inner(area);
            self.panels.height = inner.height as usize;
            self.panels.width = inner.width as usize;
            self.panels.patch_y = self
                .panels
                .patch_y
                .min(self.patch_lines().len().saturating_sub(self.panels.height));
            let paragraph = if self.selected_file.is_none() {
                Paragraph::new(if self.files.is_empty() {
                    "No changed files"
                } else {
                    "Select a file to view its patch"
                })
            } else if self.patch_lines().is_empty() {
                Paragraph::new("No textual changes")
            } else {
                // Slice vertically rather than using Paragraph's u16 vertical scroll limit.
                Paragraph::new(
                    self.patch_lines()
                        .iter()
                        .skip(self.panels.patch_y)
                        .take(self.panels.height)
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            };
            frame.render_widget(
                paragraph
                    .block(block)
                    .scroll((0, self.panels.patch_x as u16)),
                area,
            );
        }
        let help = if self.panels.fullscreen_patch {
            " Tab/Esc: file tree  j/k: scroll  h/l: horizontal scroll "
        } else {
            " Tab: full patch  Enter: expand / edit file  e: edit file  Esc: back "
        };
        frame.render_widget(
            Paragraph::new(help).style(Style::default().fg(self.ctx.color_theme.status_info_fg)),
            footer,
        );
    }

    fn block(&self, title: String, panel: Panel) -> Block<'static> {
        let style = if self.panels.focused == panel {
            Style::default()
                .fg(self.ctx.color_theme.status_info_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.ctx.color_theme.divider_fg)
        };
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(style)
    }

    pub fn selected_file(&self) -> Option<&ChangedFile> {
        self.selected_file.map(|index| &self.files[index])
    }

    pub fn as_list_state(&self) -> &CommitListState<'a> {
        self.commit_list_state.as_ref().unwrap()
    }

    pub fn take_list_state(&mut self) -> CommitListState<'a> {
        self.commit_list_state.take().unwrap()
    }

    pub fn graph_image_ids_sorted(&self) -> Vec<u32> {
        self.as_list_state().graph_image_ids_sorted()
    }

    pub fn select_commit(&mut self, event: UserEvent) {
        let state = self.commit_list_state.as_mut().unwrap();
        match event {
            UserEvent::SelectDown => state.select_next(),
            UserEvent::SelectUp => state.select_prev(),
            UserEvent::GoToParent => state.select_parent(),
            _ => return,
        }
        self.commit = self
            .repository
            .commit(state.selected_commit_hash())
            .unwrap()
            .clone();
        self.load_files();
    }

    pub fn refresh(&self) {
        let list_context = ListRefreshViewContext::from(self.as_list_state());
        self.tx
            .send(AppEvent::Refresh(RefreshViewContext::GitDetail {
                list_context,
            }));
    }
}

fn build_patch_lines(patch: &str, theme: &ColorTheme, tab_width: usize) -> Vec<Line<'static>> {
    let mut in_hunk = false;
    patch
        .lines()
        .map(|line| {
            if line.starts_with("diff --git ") {
                in_hunk = false;
            } else if line.starts_with("@@") {
                in_hunk = true;
            }
            let color = if in_hunk && line.starts_with('+') {
                theme.detail_file_change_add_fg
            } else if in_hunk && line.starts_with('-') {
                theme.detail_file_change_delete_fg
            } else if line.starts_with("@@") {
                theme.status_info_fg
            } else if line.starts_with("--- ")
                || line.starts_with("+++ ")
                || line.starts_with("diff ")
            {
                theme.detail_label_fg
            } else {
                theme.fg
            };
            let text: String = line
                .chars()
                .flat_map(|c| {
                    if c == '\t' {
                        vec![' '; tab_width]
                    } else if c.is_control() {
                        c.escape_default().collect()
                    } else {
                        vec![c]
                    }
                })
                .collect();
            Line::styled(text, Style::default().fg(color))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_uses_serie_theme_and_escapes_control_characters() {
        use ratatui::style::Color;
        let theme = ColorTheme {
            detail_file_change_add_fg: Color::Blue,
            detail_file_change_delete_fg: Color::Magenta,
            status_info_fg: Color::Yellow,
            ..ColorTheme::default()
        };
        let lines = build_patch_lines("@@ hunk @@\n-old\n+new\t\x1b[31m", &theme, 2);
        assert_eq!(lines[0].style.fg, Some(Color::Yellow));
        assert_eq!(lines[1].style.fg, Some(Color::Magenta));
        assert_eq!(lines[2].style.fg, Some(Color::Blue));
        assert_eq!(lines[2].to_string(), "+new  \\u{1b}[31m");
        let lines = build_patch_lines("+++ b/file\n@@ hunk @@\n+++ added\n--- removed", &theme, 2);
        assert_eq!(lines[0].style.fg, Some(theme.detail_label_fg));
        assert_eq!(lines[2].style.fg, Some(Color::Blue));
        assert_eq!(lines[3].style.fg, Some(Color::Magenta));
    }

    #[test]
    fn tab_opens_full_patch_and_escape_restores_split() {
        let mut panels = PanelState::default();
        let area = Rect::new(0, 0, 100, 30);
        let (files, patch) = panels.areas(area);
        assert_eq!(files.unwrap().width, 35);
        assert_eq!(patch.unwrap().width, 65);
        panels.switch();
        assert_eq!(panels.areas(area), (None, Some(area)));
        assert_eq!(panels.focused, Panel::Patch);
        assert!(!panels.cancel());
        assert_eq!(panels.areas(area), (files, patch));
        assert_eq!(panels.focused, Panel::Files);
        assert!(panels.cancel());
        panels.switch();
        panels.switch();
        assert_eq!(panels.areas(area), (files, patch));
    }
}
