use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub branch: String,
    pub latest_commit_hash: String,
    pub latest_commit_time: String,
    pub has_unstaged_changes: bool,
    pub is_git_repo: bool,
}

impl Default for GitInfo {
    fn default() -> Self {
        Self {
            branch: String::new(),
            latest_commit_hash: String::new(),
            latest_commit_time: String::new(),
            has_unstaged_changes: false,
            is_git_repo: false,
        }
    }
}

/// Get git information for a repository
pub fn get_git_info(repo_path: &str) -> GitInfo {
    let path = Path::new(repo_path);
    if !path.exists() {
        return GitInfo::default();
    }

    // Check if it's a git repo
    let is_git_repo = run_git_command(repo_path, &["rev-parse", "--git-dir"]).is_some();
    if !is_git_repo {
        return GitInfo::default();
    }

    let branch = get_current_branch(repo_path).unwrap_or_default();
    let (latest_commit_hash, latest_commit_time) = get_latest_commit(repo_path);
    let has_unstaged_changes = check_unstaged_changes(repo_path);

    GitInfo {
        branch,
        latest_commit_hash,
        latest_commit_time,
        has_unstaged_changes,
        is_git_repo: true,
    }
}

fn run_git_command(repo_path: &str, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", repo_path])
        .args(args)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn get_current_branch(repo_path: &str) -> Option<String> {
    run_git_command(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
}

fn get_latest_commit(repo_path: &str) -> (String, String) {
    let hash = run_git_command(repo_path, &["rev-parse", "--short", "HEAD"])
        .unwrap_or_default();

    let time = run_git_command(repo_path, &["log", "-1", "--format=%cr"])
        .unwrap_or_default();

    (hash, time)
}

fn check_unstaged_changes(repo_path: &str) -> bool {
    // Check for unstaged changes (modified, deleted, untracked)
    let status = run_git_command(repo_path, &["status", "--porcelain"]);
    match status {
        Some(output) => !output.is_empty(),
        None => false,
    }
}
