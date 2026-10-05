use anyhow::{Context, bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{env, fs};
use std::fs::OpenOptions;
use colored::Colorize;

use crate::frags;
use crate::git;
use crate::shell;

const MIN_MISE_VER: &str = "2026.8.10";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub file: PathBuf, //Manifest file path
    pub repos: Vec<Repo>, // Repos installed
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub consumers: Vec<Consumer>, // Consumers of repo fragments
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Consumer {
    pub dir: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fragments: Vec<String>,
}

fn mise_ver_older(curr: &str, min: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let (vcurr, vmin) = (parse(curr), parse(min));
    for i in 0..vcurr.len().max(vmin.len()) {
        let curr_ver = vcurr.get(i).copied().unwrap_or(0);
        let min_ver = vmin.get(i).copied().unwrap_or(0);
        if curr_ver < min_ver { return true; }
        if curr_ver > min_ver { return false; }
    }
    false
}

fn parse_user_repo(url: &str) -> (String, String) {
    let trimmed = url.trim_end_matches(['/', '\\']);
    let is_url = trimmed.contains("://") || trimmed.starts_with("git@");

    if is_url {
        // Strip scheme or git@ prefix to get the path portion
        let path_part = if let Some(idx) = trimmed.find("://") {
            &trimmed[idx + 3..]
        } else if let Some(idx) = trimmed.find('@') {
            &trimmed[idx + 1..]
        } else {
            trimmed
        };

        // Split by /, \, : → last two non-empty segments are user/repo
        let segs: Vec<&str> = path_part
            .split(['/', '\\', ':'])
            .filter(|s| !s.is_empty())
            .collect();

        let repo = segs.last().copied().unwrap_or("unknown")
            .strip_suffix(".git").unwrap_or("unknown");
        let user = segs.get(segs.len().saturating_sub(2))
            .copied().unwrap_or("unknown");

        (user.to_string(), repo.to_string())
    } else {
        let user = env::var("USER").unwrap_or_else(|_| "unknown".into());
        let name = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
        let repo = name.strip_suffix(".git").unwrap_or(name);

        (format!("local-{}", user), repo.to_string())
    }
}

fn validate_tomls(repo_dir: &str, fragments: Option<&[String]>) -> bool {
    for f in fragments.as_deref().unwrap_or(&[]) {
        if !f.ends_with(".toml") {
            continue;
        }
        let path = Path::new(repo_dir).join(f);
        let content: String = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                println!("{} read error: {}", f, e);
                return false;
            }
        };

        if let Err(e) = toml::from_str::<toml::Value>(&content) {
            println!("{} TOML parse error: {}", f, e);
            return false;
        }

        if f.split('.').count() > 2 {
            println!("{}: dot in filename breaks in mise 2027.8.10", f);
            return false;
        }
    }

    true
}

impl Manifest {
    // ------------ Manifest Load/Store/Show -------------
    pub fn load() -> Result<Self> {
        let manf_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("wee").join("manifest.toml");

        if !manf_path.exists() {
            // One-time check mise version requirement
            let mise_ver = std::process::Command::new("mise")
                .arg("version").output().ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|s| s.split_whitespace().nth(0).map(String::from));
            if mise_ver.as_deref()
                .map_or(true, |v| mise_ver_older(v, MIN_MISE_VER)) {
                bail!("Detected mise version: {}\nMinimum version needed: {}",
                    mise_ver.as_deref().unwrap_or("unknown"), MIN_MISE_VER);
            }

            // Create manifest file and return
            if let Some(parent) = manf_path.parent() {
                fs::create_dir_all(parent)?;
            }
            OpenOptions::new().create(true)
                .truncate(false).read(true).write(true).open(&manf_path)?;
            return Ok(Self {
                file: manf_path,
                repos: vec![],
                consumers: vec![] }
            );
        }

        let content = std::fs::read_to_string(&manf_path)
            .with_context(|| format!("reading manifest at {}",
                    manf_path.display()))?;
        let m: Manifest = if content.trim().is_empty() {
            Self {
                file: manf_path,
                repos: vec![],
                consumers: vec![],
            }
        } else {
            toml::from_str(&content)
                .with_context(|| format!("parsing manifest at {}",
                        manf_path.display()))?
        };
        Ok(m)
    }

    pub fn save(&self) -> Result<()> {
        let p = &self.file;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&p, content)?;
        Ok(())
    }

    pub fn show(&self, verbose: bool) {
        if self.repos.is_empty() {
            println!("Manifest is empty");
            return;
        }

        println!("--------------- Repos");
        for r in &self.repos {
            println!("{}  →  {}", r.name, r.url);
            if verbose {
                print!("|-- fragments:");
                let path = self.file.parent()
                    .expect("parent").join(r.name.clone());
                let mut frags = shell::list_wee_repo_files(&path);
                for f in frags.iter_mut() {
                    let name = f.strip_suffix(".toml").unwrap_or(&f);
                    print!(" {}", name);
                }
                println!();
            }
            println!();
        }

        if self.consumers.is_empty() {
            return;
        }

        println!("--------------- Consumers");
        for c in &self.consumers {
            println!("{}  ({} fragments)", c.dir, c.fragments.len());
            if verbose {
                for f in &c.fragments {
                    println!("|- {}", f);
                }
                println!();
            }
            println!();
        }
    }

    // ------------ Repo handlers -------------
    pub fn add_repo(&mut self, url: &str, dry_run: bool, verbose: bool)
        -> anyhow::Result<()> {
        let (u, r) = parse_user_repo(&url);
        let repo_name = format!("{}/{}", u, r);   
        let repo_dir = self.file.parent().unwrap().join(&repo_name);

        if repo_dir.exists() {
            bail!("repo '{}' already exists at {}",
                repo_name, repo_dir.display());
        }

        if dry_run {
            println!("{} clone {} → {}",
                "would".yellow(), url, repo_dir.display());
            return Ok(());
        }

        if verbose {
            println!("cloning {} ...", url);
        }

        git::clone(url, &repo_dir)?;

        let frags = shell::list_wee_repo_files(&repo_dir);
        if frags.is_empty() {
            fs::remove_dir_all(&repo_dir)?;
            bail!("no .toml files from {}", url);
        }

        if !validate_tomls(repo_dir.to_str().unwrap(), Some(frags.as_slice())) {
            fs::remove_dir_all(&repo_dir)?;
            bail!("Cleaning up {} cloned from {}", repo_dir.display(), url);
        }

        self.repos.push(Repo {
            name: repo_name.to_string(),
            url: url.to_string(),
        });

        println!("{} installed {} - {} frags",
            "✓".green(), repo_name, frags.len());

        Ok(())
    }

    pub fn del_repo(&mut self, repo_name: &str, dry_run: bool) -> Result<()> {
        if !self.repo_present(repo_name) {
            bail!("Manifest has no such repo: '{}'", repo_name);
        }

        let repo_dir = self.file.parent().expect("REASON").join(repo_name);
        if !repo_dir.exists() {
            println!("{} repo {} not found!", "✗".red(), repo_dir.display());
            return Ok(());
        }

        if dry_run {
            println!("{} remove {} and all its symlinks",
                "would".yellow(), repo_dir.display());
            return Ok(());
        }

        // remove repo fragment references from consumers manifests
        self.consumers.iter_mut().for_each(|c| {
            let prefix = format!("{repo_name}/");
            let to_delete: Vec<String> = c.fragments.iter()
                .filter(|t| t.starts_with(&prefix)).cloned().collect();
            for frag in &to_delete {
                let path = PathBuf::from(&c.dir).join(frag);
                let _ = std::fs::remove_file(&path);
            }
            c.fragments.retain(|t| t.strip_prefix(&prefix).is_none());
        });
        // Remove repo specific files from all consumers
        let _ = frags::run(self, frags::FDEL,
            &repo_dir, &[], false, true)?;
        // Remove the repo
        self.repos.retain(|r| r.name != repo_name);
        // Remove repo directory
        fs::remove_dir_all(&repo_dir)?;

        println!("{} uninstalled {}", "✓".green(), repo_name);

        Ok(())
    }

    pub fn repo_present(&mut self, repo_name: &str) -> bool {
        self.repos.iter().any(|r| r.name == repo_name)
    }

    // ------------ Fragment handlers -------------
    pub fn add_fragments(
        &mut self,
        consumer: &str,
        repo: &String,
        fragments: &[String]) {
        let exists = self.consumers.iter().any(|c| c.dir == consumer);
        if !exists {
            self.consumers.push(Consumer {
                dir: consumer.to_string(),
                fragments: vec![],
            });
        }
        let c = self.consumers.iter_mut().find(|c| c.dir == consumer).unwrap();
        for f in fragments {
            let fragentry = format!("{}/{}", repo, f);
            if !c.fragments.contains(&fragentry) {
                c.fragments.push(fragentry.clone());
            }
        }
    }

    pub fn del_fragments(
        &mut self,
        consumer: &str,
        repo: &String,
        fragments: &[String]) {
        let full_fragments: Vec<String> = fragments.iter().map(|f|
                format!("{}/{}", repo, f)).collect();
        if let Some(c) = self.consumers.iter_mut().find(|c| c.dir == consumer) {
            c.fragments.retain(|f| !full_fragments.contains(f));
        }
        self.consumers.retain(|c| c.dir != consumer || !c.fragments.is_empty());
    }

    // ------------ Consumer handlers -------------
    pub fn remove_consumer(&mut self, consumer: &str) {
        self.consumers.retain(|c| c.dir != consumer);
    }
}
