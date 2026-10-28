use std::{
    ffi::OsString,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::{Commit, Repository};

const PATCH_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub status: char,
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
}

impl ChangedFile {
    pub fn label(&self) -> String {
        display_path(&self.path)
    }
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

impl Repository {
    // Keep the fork's Git operations separate from the upstream detail implementation.
    fn detail_diff_command(&self, commit: &Commit) -> Command {
        let mut command = Command::new("git");
        command
            .current_dir(&self.path)
            .args(["--no-pager", "diff-tree"])
            .args([
                "--root",
                "--no-commit-id",
                "-r",
                "--find-renames",
                "--no-relative",
            ]);
        if let Some(parent) = commit.parent_commit_hashes.first() {
            command.arg(parent.as_str());
        }
        command.arg(commit.commit_hash.as_str());
        command
    }

    pub fn detail_files(&self, commit: &Commit) -> Result<Vec<ChangedFile>, String> {
        let output = self
            .detail_diff_command(commit)
            .args(["--name-status", "-z", "--"])
            .output()
            .map_err(|e| format!("Failed to list changed files: {e}"))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().into());
        }
        parse_changed_files(&output.stdout)
    }

    pub fn detail_patch(&self, commit: &Commit, file: &ChangedFile) -> Result<String, String> {
        let mut command = self.detail_diff_command(commit);
        command.args([
            "--patch",
            "--color=never",
            "--no-ext-diff",
            "--no-textconv",
            "--",
        ]);
        if let Some(old_path) = &file.old_path {
            command.arg(top_literal_path(old_path));
        }
        command.arg(top_literal_path(&file.path));
        // Bound output so a large patch cannot exhaust the UI's memory.
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to load patch: {e}"))?;
        let mut bytes = Vec::new();
        let read_result = child
            .stdout
            .take()
            .expect("patch stdout should be piped")
            .take(PATCH_LIMIT as u64 + 1)
            .read_to_end(&mut bytes);
        let truncated = bytes.len() > PATCH_LIMIT;
        if truncated || read_result.is_err() {
            let _ = child.kill();
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        read_result.map_err(|e| e.to_string())?;
        if !status.success() && !truncated {
            return Err("Git could not load this patch".into());
        }
        bytes.truncate(PATCH_LIMIT);
        let mut patch = String::from_utf8_lossy(&bytes).into_owned();
        if truncated {
            patch.push_str("\n[Patch truncated at 2 MiB]\n");
        }
        Ok(patch)
    }

    pub fn detail_worktree_file(&self, file: &ChangedFile) -> Result<PathBuf, String> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| format!("Failed to locate working tree: {e}"))?;
        if !output.status.success() {
            return Err("Cannot edit files without a working tree".into());
        }
        let root = output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout);
        let path = path_from_bytes(root)?.join(&file.path);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|e| format!("Cannot open working-tree file {}: {e}", file.label()))?;
        if !metadata.is_file() {
            return Err(format!(
                "{} is not a regular working-tree file",
                file.label()
            ));
        }
        Ok(path)
    }

    pub fn detail_editor_command(&self, path: &Path) -> Command {
        let editor = ["VISUAL", "EDITOR"]
            .into_iter()
            .filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "vi".into());
        let mut command = editor_command(&editor, path);
        command.current_dir(&self.path);
        command
    }
}

fn editor_command(editor: &str, path: &Path) -> Command {
    // Editor settings may include arguments. Pass the filename separately so
    // repository paths cannot be interpreted as shell syntax.
    let mut command = Command::new("sh");
    command
        .args(["-c", &format!("{editor} \"$@\""), "serie-editor"])
        .arg(path);
    command
}

fn parse_changed_files(bytes: &[u8]) -> Result<Vec<ChangedFile>, String> {
    let mut fields = bytes.split(|b| *b == 0).filter(|field| !field.is_empty());
    let mut files = Vec::new();
    while let Some(status) = fields.next() {
        let status = char::from(status[0]);
        let first = fields.next().ok_or("Missing path in Git diff output")?;
        let (path, old_path) = if matches!(status, 'R' | 'C') {
            let second = fields.next().ok_or("Missing rename destination")?;
            (path_from_bytes(second)?, Some(path_from_bytes(first)?))
        } else {
            (path_from_bytes(first)?, None)
        };
        files.push(ChangedFile {
            status,
            path,
            old_path,
        });
    }
    Ok(files)
}

fn top_literal_path(path: &Path) -> OsString {
    let mut spec = OsString::from(":(top,literal)");
    spec.push(path);
    spec
}

fn path_from_bytes(bytes: &[u8]) -> Result<PathBuf, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(OsString::from_vec(bytes.to_vec()).into())
    }
    #[cfg(not(unix))]
    {
        String::from_utf8(bytes.to_vec())
            .map(PathBuf::from)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests;
