use std::io::Write;
use std::process::{Command, Stdio};

/// Default base branch for branch diff comparison
const DEFAULT_BASE_BRANCH: &str = "main";

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
    /// Get the git diff arguments for this diff type
    fn to_git_diff_args(&self, branch: Option<&str>) -> Result<Vec<String>, String> {
        match self {
            DiffType::Unstaged => Ok(vec!["diff".to_string()]),
            DiffType::LatestCommit => Ok(vec!["diff".to_string(), "HEAD~1".to_string(), "HEAD".to_string()]),
            DiffType::Branch => {
                let base = branch.unwrap_or(DEFAULT_BASE_BRANCH);
                // Validate branch name to prevent git option injection
                if base.starts_with('-') {
                    return Err(format!("Invalid branch name: {}", base));
                }
                Ok(vec!["diff".to_string(), base.to_string(), "HEAD".to_string()])
            }
        }
    }
}

/// Open difit for the specified repository and diff type
/// This pipes git diff output to difit without using shell interpolation
pub fn open_difit(repo_path: &str, diff_type: DiffType, base_branch: Option<&str>) -> Result<(), String> {
    let git_args = diff_type.to_git_diff_args(base_branch)?;

    // Run git diff and capture output (no shell interpolation - safe from injection)
    let git_output = Command::new("git")
        .args(&git_args)
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to run git diff: {}", e))?;

    if !git_output.status.success() {
        let stderr = String::from_utf8_lossy(&git_output.stderr);
        return Err(format!("git diff failed: {}", stderr));
    }

    let diff_content = git_output.stdout;

    if diff_content.is_empty() {
        return Err("No diff content to display".to_string());
    }

    // Spawn npx difit and pipe the diff content to stdin (no shell interpolation)
    // Note: difit process runs asynchronously - we don't wait for it to complete
    let mut difit_process = Command::new("npx")
        .arg("difit")
        .stdin(Stdio::piped())
        .current_dir(repo_path)
        .spawn()
        .map_err(|e| format!("Failed to start difit: {}", e))?;

    // Write git diff output to difit's stdin
    let mut stdin = difit_process
        .stdin
        .take()
        .ok_or("Failed to capture difit stdin")?;

    stdin
        .write_all(&diff_content)
        .map_err(|e| format!("Failed to write to difit stdin: {}", e))?;

    Ok(())
}
