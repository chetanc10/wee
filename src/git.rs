use anyhow::{bail, Context, Result};
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
    let status = Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(dir)
        .status()
        .context("running git pull")?;
    if !status.success() {
        bail!("git pull failed in {}", dir.display());
    }
    Ok(())
}

pub fn diff_last(dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "HEAD~1"])
        .current_dir(dir)
        .output()
        .context("running git diff")?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn reset_hard_head(dir: &Path, ref_: &str) -> Result<()> {
    Command::new("git")
        .args(["reset", "--hard", ref_])
        .current_dir(dir)
        .status()
        .context("running git reset")?
        .success()
        .then(|| ())
        .ok_or_else(|| anyhow::anyhow!("git reset failed"))
}
