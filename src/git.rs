use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn clone(url: &str, dest: &Path) -> Result<()> {
    Command::new("git")
        .args(["clone", "--depth", "1", url, &dest.to_string_lossy()])
        .status()
        .context("running git clone")?
        .success()
        .then(|| ())
        .ok_or_else(|| anyhow::anyhow!("git clone failed"))
}

pub fn pull(dir: &Path) -> Result<()> {
    // use latest and hard reset to latest commit and loose older 2 commits
    // git fetch origin <branch-name> --depth=2
    // git reset --hard origin/<branch-name>

    // Get branch info to fetch that alone
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("failed to run git")
        .stdout;
    let branch = String::from_utf8_lossy(&branch).trim().to_string();

    // Fetch latest
    let out = Command::new("git")
        .args(["fetch", "origin", &branch])
        .current_dir(dir)
        .output()
        .expect("failed to run git fetch");
    if !out.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&out.stderr),
        ).into());
    }

    // Reset commit HEAD preserving older HEAD as oldest commit
    let out = Command::new("git")
        .args(["reset", "--hard", &format!("origin/{}", branch)])
        .current_dir(dir)
        .output()
        .expect("failed to run git reset");
    if !out.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&out.stderr).to_string(),
        ).into());
    }
    Ok(())
}
