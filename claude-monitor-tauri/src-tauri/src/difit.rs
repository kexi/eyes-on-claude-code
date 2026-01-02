use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::Duration;

/// Default base branch for branch diff comparison
const DEFAULT_BASE_BRANCH: &str = "main";

/// Default port for difit server
const DEFAULT_DIFIT_PORT: u16 = 4966;

/// Diff types supported by the application
#[derive(Debug, Clone, Copy)]
pub enum DiffType {
    /// Unstaged changes (working directory vs index)
    Unstaged,
    /// Staged changes (index vs HEAD)
    Staged,
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
            DiffType::Staged => Ok(vec!["diff".to_string(), "--cached".to_string()]),
            DiffType::LatestCommit => Ok(vec![
                "diff".to_string(),
                "HEAD~1".to_string(),
                "HEAD".to_string(),
            ]),
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

/// Result of starting a difit server
pub struct DifitServerInfo {
    pub url: String,
    pub process: Child,
}

/// Registry to track running difit processes by window label
pub struct DifitProcessRegistry {
    processes: Mutex<HashMap<String, Child>>,
    next_port: Mutex<u16>,
}

impl DifitProcessRegistry {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            next_port: Mutex::new(DEFAULT_DIFIT_PORT),
        }
    }

    /// Get the next available port
    pub fn get_next_port(&self) -> u16 {
        let mut port = self.next_port.lock().unwrap();
        let current = *port;
        *port = port.wrapping_add(1);
        if *port < DEFAULT_DIFIT_PORT {
            *port = DEFAULT_DIFIT_PORT;
        }
        current
    }

    /// Register a difit process with a window label
    pub fn register(&self, window_label: String, process: Child) {
        if let Ok(mut processes) = self.processes.lock() {
            processes.insert(window_label, process);
        }
    }

    /// Kill and remove a difit process by window label
    pub fn kill(&self, window_label: &str) {
        if let Ok(mut processes) = self.processes.lock() {
            if let Some(mut process) = processes.remove(window_label) {
                let _ = process.kill();
                let _ = process.wait(); // Reap the zombie process
            }
        }
    }

    /// Kill all registered difit processes
    pub fn kill_all(&self) {
        if let Ok(mut processes) = self.processes.lock() {
            for (_, mut process) in processes.drain() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }
    }
}

impl Default for DifitProcessRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Start a difit server for the specified repository and diff type
/// Returns the URL and process handle for management
pub fn start_difit_server(
    repo_path: &str,
    diff_type: DiffType,
    base_branch: Option<&str>,
    port: u16,
) -> Result<DifitServerInfo, String> {
    let git_args = diff_type.to_git_diff_args(base_branch)?;

    // Run git diff and capture output
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

    // Start difit with --no-open flag so it doesn't open browser
    let mut difit_process = Command::new("npx")
        .args(["difit", "--no-open", "--port", &port.to_string()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(repo_path)
        .spawn()
        .map_err(|e| format!("Failed to start difit: {}", e))?;

    // Write git diff to stdin
    {
        let mut stdin = difit_process
            .stdin
            .take()
            .ok_or("Failed to capture difit stdin")?;
        stdin
            .write_all(&diff_content)
            .map_err(|e| format!("Failed to write to difit stdin: {}", e))?;
    } // stdin is dropped here, closing the pipe

    // Read stderr to find the server URL with timeout
    let stderr = difit_process
        .stderr
        .take()
        .ok_or("Failed to capture difit stderr")?;

    // Use a channel to receive the port from a background thread
    let (tx, rx) = mpsc::channel();
    let expected_port = port;
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().take(10) {
            if let Ok(line) = line {
                // Look for "difit server started on http://localhost:XXXX"
                if line.contains("difit server started on") {
                    if let Some(url_start) = line.find("http://") {
                        let url = &line[url_start..];
                        // Extract port from URL
                        if let Some(port_str) = url.strip_prefix("http://localhost:") {
                            if let Ok(p) = port_str.trim().parse::<u16>() {
                                let _ = tx.send(p);
                                return;
                            }
                        }
                    }
                }
            }
        }
        // Send expected port if we couldn't find the actual one
        let _ = tx.send(expected_port);
    });

    // Wait for up to 5 seconds for the server to start
    let actual_port = rx.recv_timeout(Duration::from_secs(5)).unwrap_or(port);
    let url = format!("http://localhost:{}", actual_port);

    Ok(DifitServerInfo {
        url,
        process: difit_process,
    })
}
