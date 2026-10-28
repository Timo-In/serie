use std::{fs, path::Path, process::Command};

use super::*;
use crate::{git::SortCommit, test_support::git::GitRepository};

fn git(path: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn load(path: &Path) -> Repository {
    Repository::load(path, SortCommit::Chronological, None, false).unwrap()
}

fn head(repository: &Repository) -> &Commit {
    let hash = match repository.head() {
        super::super::Head::Branch { name } => repository
            .all_refs()
            .into_iter()
            .find(|r| r.name() == name)
            .unwrap()
            .target(),
        _ => panic!("expected branch"),
    };
    repository.commit(hash).unwrap()
}

#[test]
fn root_patch_uses_literal_paths_and_keeps_filename_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let history = GitRepository::new(dir.path());
    history.init();
    fs::create_dir(dir.path().join("src")).unwrap();
    let path = "src/[a]\t换行\n.rs";
    fs::write(dir.path().join(path), "root content\n").unwrap();
    fs::write(dir.path().join("src/a.rs"), "must not match wildcard\n").unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("root", "2024-01-01");
    let repository = load(dir.path());
    let commit = head(&repository);
    let files = repository.detail_files(commit).unwrap();
    let file = files
        .iter()
        .find(|file| file.path == Path::new(path))
        .unwrap();
    assert_eq!(file.status, 'A');
    let patch = repository.detail_patch(commit, file).unwrap();
    assert!(patch.contains("+root content"));
    assert!(!patch.contains("must not match wildcard"));
    assert_eq!(
        fs::read(repository.detail_worktree_file(file).unwrap()).unwrap(),
        b"root content\n"
    );
    assert_eq!(file.label(), "src/[a]\\t换行\\n.rs");
    let nested = load(&dir.path().join("src"));
    let nested_commit = head(&nested);
    let nested_files = nested.detail_files(nested_commit).unwrap();
    let nested_file = nested_files
        .iter()
        .find(|file| file.path == Path::new(path))
        .unwrap();
    assert!(nested
        .detail_patch(nested_commit, nested_file)
        .unwrap()
        .contains("+root content"));
    assert_eq!(
        nested.detail_worktree_file(nested_file).unwrap(),
        repository.detail_worktree_file(file).unwrap()
    );
}

#[test]
fn rename_and_delete_patches_remain_historical_while_editor_uses_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let history = GitRepository::new(dir.path());
    history.init();
    fs::write(dir.path().join("before.txt"), "rename content\n").unwrap();
    fs::write(dir.path().join("deleted.txt"), "deleted content\n").unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("root", "2024-01-01");
    fs::rename(dir.path().join("before.txt"), dir.path().join("after.txt")).unwrap();
    fs::remove_file(dir.path().join("deleted.txt")).unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("rename and delete", "2024-01-02");
    fs::write(dir.path().join("after.txt"), "uncommitted edits\n").unwrap();
    let repository = load(dir.path());
    let commit = head(&repository);
    let files = repository.detail_files(commit).unwrap();
    let renamed = files.iter().find(|file| file.status == 'R').unwrap();
    assert_eq!(renamed.old_path.as_deref(), Some(Path::new("before.txt")));
    assert_eq!(renamed.path, Path::new("after.txt"));
    assert!(repository
        .detail_patch(commit, renamed)
        .unwrap()
        .contains("rename from before.txt"));
    assert_eq!(
        fs::read(repository.detail_worktree_file(renamed).unwrap()).unwrap(),
        b"uncommitted edits\n"
    );
    let deleted = files.iter().find(|file| file.status == 'D').unwrap();
    assert!(repository
        .detail_patch(commit, deleted)
        .unwrap()
        .contains("-deleted content"));
    assert!(repository.detail_worktree_file(deleted).is_err());
    assert!(!dir.path().join("deleted.txt").exists());
}

#[test]
fn merge_compares_against_first_parent_and_empty_commit_has_no_files() {
    let dir = tempfile::tempdir().unwrap();
    let history = GitRepository::new(dir.path());
    history.init();
    history.commit("empty root", "2024-01-01");
    let repository = load(dir.path());
    assert!(repository
        .detail_files(head(&repository))
        .unwrap()
        .is_empty());
    history.checkout_b("feature");
    fs::write(dir.path().join("feature.txt"), "feature\n").unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("feature", "2024-01-02");
    history.checkout("master");
    fs::write(dir.path().join("master.txt"), "master\n").unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("master", "2024-01-03");
    history.merge(&["feature"], "2024-01-04");
    let repository = load(dir.path());
    let commit = head(&repository);
    assert_eq!(commit.parent_commit_hashes.len(), 2);
    let files = repository.detail_files(commit).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, Path::new("feature.txt"));
    assert!(repository
        .detail_patch(commit, &files[0])
        .unwrap()
        .contains("+feature"));
}

#[test]
fn binary_and_large_patch_are_handled_without_unbounded_display() {
    let dir = tempfile::tempdir().unwrap();
    let history = GitRepository::new(dir.path());
    history.init();
    fs::write(dir.path().join("binary"), b"a\0b").unwrap();
    fs::write(
        dir.path().join("large"),
        "a long line for a bounded patch\n".repeat(100_000),
    )
    .unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("root", "2024-01-01");
    let repository = load(dir.path());
    let commit = head(&repository);
    let files = repository.detail_files(commit).unwrap();
    let binary = files
        .iter()
        .find(|file| file.path == Path::new("binary"))
        .unwrap();
    assert!(repository
        .detail_patch(commit, binary)
        .unwrap()
        .contains("Binary files"));
    let large = files
        .iter()
        .find(|file| file.path == Path::new("large"))
        .unwrap();
    let patch = repository.detail_patch(commit, large).unwrap();
    assert!(patch.ends_with("[Patch truncated at 2 MiB]\n"));
    assert!(patch.len() < PATCH_LIMIT + 100);
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn non_utf8_filename_roundtrips() {
    use std::os::unix::ffi::OsStringExt;
    let dir = tempfile::tempdir().unwrap();
    let history = GitRepository::new(dir.path());
    history.init();
    let path = PathBuf::from(OsString::from_vec(b"non-utf8-\xff.rs".to_vec()));
    fs::write(dir.path().join(&path), "content\n").unwrap();
    git(dir.path(), &["add", "-A"]);
    history.commit("root", "2024-01-01");
    let repository = load(dir.path());
    let commit = head(&repository);
    let files = repository.detail_files(commit).unwrap();
    assert_eq!(files[0].path, path);
    assert_eq!(
        fs::read(repository.detail_worktree_file(&files[0]).unwrap()).unwrap(),
        b"content\n"
    );
}

#[cfg(unix)]
#[test]
fn parser_preserves_non_utf8_path_bytes() {
    use std::os::unix::ffi::OsStrExt;
    let files = parse_changed_files(b"M\0non-utf8-\xff.rs\0").unwrap();
    assert_eq!(files[0].path.as_os_str().as_bytes(), b"non-utf8-\xff.rs");
}

#[cfg(unix)]
#[test]
fn editor_rejects_directories_and_symlinks() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let history = GitRepository::new(dir.path());
    history.init();
    history.commit("root", "2024-01-01");
    fs::create_dir(dir.path().join("directory")).unwrap();
    fs::write(dir.path().join("target"), "content").unwrap();
    symlink("target", dir.path().join("link")).unwrap();
    let repository = load(dir.path());
    for name in ["directory", "link"] {
        let file = ChangedFile {
            status: 'M',
            path: name.into(),
            old_path: None,
        };
        assert!(repository
            .detail_worktree_file(&file)
            .unwrap_err()
            .contains("not a regular"));
    }
}

#[test]
fn editor_arguments_keep_shell_syntax_in_filenames_literal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("space ' \"; $(touch injected) [a].rs");
    let output = editor_command("printf '%s'", &path)
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, path.to_string_lossy().as_bytes());
    assert!(!dir.path().join("injected").exists());
}
