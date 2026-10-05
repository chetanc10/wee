use std::{env, env::consts, fs, path::Path, path::PathBuf};

use crate::manifest::Manifest;

const EXTENSIONS: &[&str] = &[
    "toml",  // for mise
    "bash",  // for bash script
    "zsh",   // for zsh script
    "fish",  // for fish script
    "pwsh",  // for powershell script
    "linux", // for Linux OS
    "win",   // for Windows OS
    "macos", // for Mac OS
];
pub fn os_suffix() -> &'static str {
    match consts::OS { // output strings tightly coupled with EXTENSIONS
        "linux" => "linux",
        "windows" => "win",
        "macos" => "macos",
        _ => "",
    }
}

pub fn shell_suffix() -> &'static str {
    match env::var("SHELL") { // output strings tightly coupled with EXTENSIONS
        Ok(s) if s.contains("fish") => "fish",
        Ok(s) if s.contains("zsh") => "zsh",
        Ok(s) if s.contains("bash") => "bash",
        Ok(s) if s.contains("pwsh") || s.contains("powershell") => "pwsh",
        _ => "",
    }
}

pub fn get_copy_path(repo: &String, name: &String, conf_d: &String) -> String {
    let os_suf = os_suffix();
    let os_suf_pat = format!(".{}", os_suf);
    let shell_suf = shell_suffix();
    let shell_suf_pat = format!(".{}", shell_suf);

    if name.ends_with(".toml") {
        let n = format!("{}/{}", repo, name);
        let mut parts: Vec<_> = n.rsplit('/').take(3).collect();
        parts.reverse();
        return format!("{}/{}", conf_d, parts.join("-"));
    } else if (!shell_suf.is_empty() && name.ends_with(&shell_suf_pat)) ||
        (!os_suf.is_empty() && name.ends_with(&os_suf_pat)) {
        let mut parts: Vec<_> = name.rsplit('/').take(2).collect();
        parts.reverse();
        let mut p = format!("{}/bin/{}", conf_d, parts.join("-"));
        let p = p.strip_suffix(&shell_suf_pat).unwrap_or(&p).to_string();
        let p = p.strip_suffix(&os_suf_pat).unwrap_or(&p).to_string();
        return p;
    } else {
        return "".to_string();
    }
}

pub fn list_wee_repo_files(dir: &Path) -> Vec<String> {
    let os_suf = os_suffix();
    let os_suf_pat = format!(".{}", os_suf);
    let shell_suf = shell_suffix();
    let shell_suf_pat = format!(".{}", shell_suf);

    fs::read_dir(dir).ok().into_iter().flatten().filter_map(|e|
        e.ok()).filter_map(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        if name.ends_with(".toml") {
            Some(name.to_string())
        } else if !shell_suf.is_empty() && name.ends_with(&shell_suf_pat) {
            Some(name.to_string())
        } else if !os_suf.is_empty() && name.ends_with(&os_suf_pat) {
            Some(name.to_string())
        } else {
            None
        }
    }).collect()
}

fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for ext in EXTENSIONS {
        let candidate = dir.join(format!("{}.{}", name, ext));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn parse_source(source: &str, manf: &Manifest) -> PathBuf {
    if !source.starts_with('/') && !source.starts_with('~') {
        // username/repo => extract full path from wee registry
        let s = Path::new(&manf.file)
            .parent().unwrap().to_path_buf().join(source);
        PathBuf::from(s)
    } else {
        // full local path given
        PathBuf::from(source)
    }
}
