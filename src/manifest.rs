use anyhow::{Context, bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::fs::OpenOptions;
use colored::Colorize;

use crate::frags;
use crate::git;

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

    if !trimmed.contains("://") && !trimmed.starts_with("git@") {
        return ("".to_string(), "".to_string())
    }

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

    return (user.to_string(), repo.to_string());
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
                let mut frags = frags::list_wee_repo_files(&path);
                for f in frags.iter_mut() {
                    print!(" {}", f.rsplit('/').next().unwrap_or(&f));
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
        if u == "" || r == "" {
            bail!("Invalid github url: '{}'", url.to_string());
        }
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

        let frags = frags::list_wee_repo_files(&repo_dir);
        if frags.is_empty() {
            fs::remove_dir_all(&repo_dir)?;
            bail!("no .toml files from {}", url);
        }

        if !frags::validate_frags(repo_dir.to_str().unwrap(), Some(frags.as_slice()), verbose) {
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

    pub fn upgrade_repo(&mut self, reponame: &str, dry_run: bool, verbose: bool)
        -> anyhow::Result<()> {
        let repodirs = frags::repo_name_to_path(Some(reponame), self);
        let repodir_valid = frags::validate_repodirs_list(repodirs.clone(), "upgrade".to_string(), dry_run);
        if !repodir_valid {
            return Ok(());
        }

        for repodir in repodirs {
            let reponame = format!("{}/{}",
                repodir.file_name().unwrap().to_string_lossy(), 
                repodir.parent().unwrap().file_name().unwrap().to_string_lossy()
            );
            if dry_run {
                println!("{} upgrade {} and corresponding fragments in consumers",
                    "would".yellow(), reponame);
            } else {
                // Upgrade repo first
                git::pull(&repodir)?;
            }

            // Update repo's fragment copies in relevant consumers
            let _ = frags::run(self,
                Some(&reponame),
                &std::env::current_dir()?.canonicalize()?,
                false,
                false,
                dry_run,
                verbose)?;
            if !dry_run {
                println!("{} upgraded {}", "✓".green(), reponame);
            }
        }

        Ok(())
    }

    pub fn del_repo(&mut self, repo_name: &str, dry_run: bool, verbose: bool) -> Result<()> {
        if !self.repo_present(repo_name) {
            bail!("Manifest has no such repo: '{}'", repo_name);
        }

        let repo_dir = self.file.parent().expect("REASON").join(repo_name);
        if !repo_dir.exists() {
            println!("{} repo {} not found!", "✗".red(), repo_dir.display());
            return Ok(());
        }

        if dry_run {
            println!("{} remove {} and its fragments from consumers",
                "would".yellow(), repo_dir.display());
        } else {
            // remove repo fragment references from consumers manifests
            self.consumers.iter_mut().for_each(|c| {
                let prefix = format!("{repo_name}:");
                let to_delete: Vec<String> = c.fragments.iter()
                    .filter(|t| t.starts_with(&prefix)).cloned().collect();
                for frag in &to_delete {
                    let path = PathBuf::from(&c.dir).join(frag);
                    let _ = std::fs::remove_file(&path);
                }
                c.fragments.retain(|t| t.strip_prefix(&prefix).is_none());
            });
        }
        // Remove repo specific files from all consumers
        let _ = frags::run(self,
            Some(&repo_name),
            &std::env::current_dir()?.canonicalize()?,
            false,
            true,
            dry_run,
            verbose)?;
        if !dry_run {
            // Remove the repo
            self.repos.retain(|r| r.name != repo_name);
            // Remove repo directory
            fs::remove_dir_all(&repo_dir)?;
            println!("{} uninstalled {}", "✓".green(), repo_name);
        }

        Ok(())
    }

    pub fn get_full_repo_list(&mut self) -> Vec<PathBuf> {
        let mut list = Vec::new();
        let manf_dir = match self.file.parent() {
            Some(d) => d,
            None => return list,
        };
        if let Ok(user_entries) = fs::read_dir(manf_dir) {
            for user_entry in user_entries.flatten() {
                let user_path = user_entry.path();
                if !user_path.is_dir() {
                    continue;
                }
                if let Ok(repo_entries) = fs::read_dir(&user_path) {
                    for repo_entry in repo_entries.flatten() {
                        let repo_path = repo_entry.path();
                        if repo_path.is_dir() && repo_path.join(".git").is_dir() {
                            list.push(repo_path);
                        }
                    }
                }
            }
        }

        list
    }

    pub fn repo_present(&mut self, repo_name: &str) -> bool {
        self.repos.iter().any(|r| r.name == repo_name)
    }

    // ------------ Fragment handlers -------------
    pub fn load_consumer_fragments(
        &mut self,
        consumer: &str,
        reponame: &String) -> Vec<String> {
        if let Some(c) = self.consumers.iter().find(|c| c.dir == consumer) {
            if !c.fragments.is_empty() {
                return c.fragments.iter().filter(|f| f.starts_with(reponame)).cloned().collect();
            }
        }
        return Vec::new();
    }

    pub fn save_consumer_fragments(
        &mut self,
        consumer: &str,
        add_fragments: &[String],
        del_fragments: &[String]) {
        if !add_fragments.is_empty() && !self.consumers.iter().any(|c| c.dir == consumer) {
            self.consumers.push(Consumer {
                dir: consumer.to_string(),
                fragments: vec![],
            });
        }
        let c: &mut Consumer = self.consumers.iter_mut().find(|c| c.dir == consumer).unwrap();
        for f in add_fragments {
            if !c.fragments.contains(&f) {
                c.fragments.push(f.clone());
            }
        }
        c.fragments.retain(|f| !del_fragments.contains(f));
    }

    // ------------ Consumer handlers -------------
    pub fn remove_consumer(&mut self, consumer: &str) {
        self.consumers.retain(|c| c.dir != consumer);
    }
}
