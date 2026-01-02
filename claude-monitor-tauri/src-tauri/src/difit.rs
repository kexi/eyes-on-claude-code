use std::process::Command;

/// Diff types supported by the application
#[derive(Debug, Clone, Copy)]
pub enum DiffType {
    /// Unstaged changes (working directory vs HEAD)
    Unstaged,
    /// Latest commit diff (HEAD vs HEAD~1)
    LatestCommit,
    /// Branch diff (current branch vs main/master)
    Branch,
}

impl DiffType {
    /// Get the git diff command for this diff type
    fn to_git_diff_cmd(&self, branch: Option<&str>) -> String {
        match self {
            DiffType::Unstaged => "git diff".to_string(),
            DiffType::LatestCommit => "git diff HEAD~1 HEAD".to_string(),
            DiffType::Branch => {
                let base = branch.unwrap_or("main");
                format!("git diff {} HEAD", base)
            }
        }
    }
}

/// Open difit for the specified repository and diff type
/// This pipes git diff output to difit
pub fn open_difit(repo_path: &str, diff_type: DiffType, base_branch: Option<&str>) -> Result<(), String> {
    let git_diff_cmd = diff_type.to_git_diff_cmd(base_branch);

    // Build the shell command to pipe git diff to difit
    let full_cmd = format!("cd '{}' && {} | npx difit", repo_path, git_diff_cmd);

    let result = Command::new("sh")
        .arg("-c")
        .arg(&full_cmd)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to start difit: {}", e))
    }
}
