use anyhow::{bail, Result};
use colored::Colorize;
use std::{fs, path::PathBuf, process::Command};

use crate::manifest::Manifest;
use crate::shell;

pub const FADD : u8 = 0x01;
pub const FDEL : u8 = 0x02;
pub const FREF : u8 = 0x04;

fn frag_op_str(op: u8) -> &'static str {
    match op {
        FADD => "Add",
        FDEL => "Del",
        FREF => "Refresh",
        _ => "",
    }
}

pub fn run(
    manf: &mut Manifest,
    op: u8,
    repo: &PathBuf,
    fragments: &[String],
    dry_run: bool,
    verbose: bool
) -> Result<()> {
    if !fs::exists(repo)? {
        eprintln!("repo path {} does not exist", repo.display());
        return Ok(());
    }

    let wee_repo = format!("{}/{}",
        repo.parent().unwrap().file_name().unwrap().display(),
        repo.file_name().unwrap().display());

    let frags: Vec<String> = if fragments.is_empty() {
        let mut frags = shell::list_wee_repo_files(&repo);
        if frags.is_empty() {
            bail!("no fragments user-specified nor found in repo {}",
                repo.display());
        }
        print!("No fragments specified! Trying all fragment files:");
        for f in frags.iter_mut() { print!(" {}", f); }
        println!("\n");
        frags
    } else {
        let frags = fragments.to_vec();
        println!("");
        frags
    };

    let project = std::env::current_dir()?.canonicalize()?;
    let conf_d = project.join(".mise/conf.d");

    if dry_run {
        match op {
            FADD => {
                println!("{} create {} and copy: {}",
                    "would".yellow(), conf_d.display(), frags.join(", "));
                return Ok(());
            },
            FREF => {
                println!("{} restore {} frags into {}",
                    "would".yellow(), frags.len(), conf_d.display());
                return Ok(());
            },
            FDEL => {},
            _ => {
                bail!("{} operation", "Unknown fragment".red().bold());
            }
        }
    }

    // Ensure local .mise/conf_d/bin directory is created already
    let conf_d_bin = conf_d.join("bin");
    fs::create_dir_all(&conf_d_bin)?;

    // Add / Del / Refresh frags
    let mut changes_made = false;
    for fragname in &frags {
        let frag = fragname.rsplit('/').next().unwrap();
        let mut frag_path = PathBuf::from("."); // dummy
        if op != FDEL {
            frag_path = match PathBuf::from(&manf.file).parent() {
                Some(p) => p.join(&wee_repo).join(fragname),
                None => {
                    println!("{} fragment '{}' not in registry",
                        "⚠".yellow(), fragname);
                    continue;
                }
            };
        }

        let copy_path = shell::get_copy_path(&wee_repo,
            &fragname, &conf_d.display().to_string());
        if copy_path == "" && verbose {
            println!("{} Discarding unrecognized fragment '{}' ",
                "⚠".yellow(), fragname);
            continue;
        }

        if PathBuf::from(&copy_path).exists() {
            if op == FDEL && dry_run { // dry run for delete
                println!("{} remove {}",
                    "would".yellow(), PathBuf::from(&copy_path).display());
                return Ok(());
            }
            fs::remove_file(&copy_path)?;
        } else if op == FDEL {
            println!("{} '{}' not in current project", "⚠".yellow(), frag);
            continue;
        }

        if op == FDEL {
            if verbose {
                println!("{} removed {}", "✓".green(), frag);
            }
        } else {
            fs::copy(&frag_path, &copy_path)?;
            if verbose {
                println!("copied {} → {}\n",
                    frag_path.display(), PathBuf::from(&copy_path).display());
            }
        }

        if !dry_run {
            changes_made = true;
        }
    }

    if !changes_made {
        println!("No fragment changes made!");
        return Ok(());
    }
    // Update consumer info in manifest
    if op == FADD {
        manf.add_fragments(project.to_str().unwrap(), &wee_repo, &frags);
    } else if op == FDEL {
        let conf_d_empty = !fs::read_dir(&conf_d)
            .map(|d| d.filter_map(|e| e.ok()).any(|e| e.path().is_file()))
            .unwrap_or(false);
        let conf_d_bin_empty = !fs::read_dir(&conf_d_bin)
            .map(|d| d.filter_map(|e| e.ok()).any(|e| e.path().is_file()))
            .unwrap_or(false);
        if conf_d_empty && conf_d_bin_empty {
            fs::remove_dir_all(project.join(".mise"))?;
        }
        manf.del_fragments(project.to_str().unwrap(), &wee_repo, &frags);
    } // don't update fragment in manifest for 'refresh', it's already there!

    // Auto reshell
    // let status = Command::new("mise").arg("reshell").status();
    let output = Command::new("mise").arg("reshell").output()?;
    let e = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));
    let errors: String = e.lines().filter(|l| {!l.contains("no tasks")
        && !l.contains("Version:") && !l.contains("MISE_VERBOSE")})
    .collect::<Vec<_>>().join("\n");
    if errors.is_empty() {
        if verbose {
            println!("ran mise reshell");
        }
    } else {
        println!("{}",
            "NOTE: Run `mise reshell` or reload shell now".yellow());
    }

    println!("{}: {} {} frags",
        frag_op_str(op), "✓".green(), frags.len());

    Ok(())
}
