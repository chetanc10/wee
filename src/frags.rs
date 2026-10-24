use anyhow::{anyhow, Result};
use colored::Colorize;
use std::{
    env,
    env::consts,
    fs,
    path::Path,
    path::PathBuf,
    process::Command,
};
use walkdir::WalkDir;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::FragmentType;
use crate::manifest::Manifest;

const WEE_EN_FILEPATH: &str = ".mise/wee_enabled.txt";
const DIRS: [&str; 5] = ["toml", "bash", "zsh", "fish", "pwsh"];

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

fn prune_stale_mise_configs(verbose: bool) -> Result<()> {
    let home = dirs::home_dir().expect("no home dir");
    let state_dir = home.join(".local/state/mise");

    for subdir in ["tracked-configs", "trusted-configs", "ignored-configs"] {
        let dir = state_dir.join(subdir);
        if !dir.exists() { continue; }

        for entry in fs::read_dir(&dir)? {
            let path = entry?.path();
            if !path.is_symlink() { continue; }
            let target = fs::read_link(&path)?;
            if !target.exists() {
                if verbose {
                    println!("[VERBOSE] removing stale: {} -> {}",
                        path.display(), target.display());
                }
                fs::remove_file(&path)?;
            }
        }
    }

    Ok(())
}

fn mise_reshell(verbose: bool) -> Result <()> {
    let output = Command::new("mise").arg("reshell").output()?;
    let e = format!("{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr));
    let errors: String = e.lines().filter(|l| {!l.contains("no tasks")
        && !l.contains("Version:") && !l.contains("MISE_VERBOSE")})
        .collect::<Vec<_>>().join("\n");
    if errors.is_empty() {
        if verbose {
            println!("[VERBOSE] ran mise reshell");
        }
    } else {
        println!("{}", "NOTE: Run `mise reshell` or reload shell now".yellow());
    }

    Ok(())
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
            let p = format!("{}/bin/{}", conf_d, parts.join("-"));
            let p = p.strip_suffix(&shell_suf_pat).unwrap_or(&p).to_string();
            let p = p.strip_suffix(&os_suf_pat).unwrap_or(&p).to_string();
            return p;
        } else {
            return "".to_string();
    }
}

// Follows weeproj directory template and lists down all frags as
// absolute paths for known directories listed in DIRS
// If any non-DIR recognized files are present in a directory,
// it shall not be added to fragment collection
pub fn list_wee_repo_files(dir: &Path) -> Vec<String> {
    let os_suf = os_suffix();
    let shell_suf = shell_suffix();
    let targets: Vec<String> = vec!["toml".to_string(), shell_suf.to_string(), os_suf.to_string()];
    let mut results = Vec::new();

    let dentries: Vec<_> = match fs::read_dir(dir) {
        Ok(dl) => dl.flatten().filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            targets.iter().any(|t| t == &name) && e.path().is_dir()
        }).collect(),
        Err(_) => return results,
    };

    for d in dentries {
        let name = d.file_name().to_string_lossy().into_owned();
        let suffix = format!(".{}", name);

        if let Ok(files) = fs::read_dir(d.path()) {
            for f in files.flatten() {
                let fname = f.file_name().to_string_lossy().into_owned();
                if fname.ends_with(&suffix) && f.path().is_file() {
                    results.push(f.path().display().to_string());
                }
            }
        }
    }

    results
}

fn about_frag(path: &Path) -> Option<String> {
    let pathstr = path.to_string_lossy();
    let suf = pathstr.rsplit_once('.').map(|(_, e)| e);
    match suf {
        Some(s) if matches!(s, "toml" | "bash" | "zsh" | "fish" | "pwsh") => {
            let content: String = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_e) => return None, 
            };

            if suf == Some("toml") {
                let v: toml::Value = match toml::from_str::<toml::Value>(&content) {
                    Ok(v) => v,
                    Err(_e) => return None,
                };
                return v.get("_").and_then(|v| v.get("about"))
                    .and_then(|v| v.as_str()).map(|s| s.to_string());
            }
            return content.lines().find(|l| l.trim_start().starts_with("#about"))
                .and_then(|l| l.splitn(2, "#about = ").nth(1).map(|s| s.trim().to_string()));
            },
        _ => None,
    }
}

fn allow_user_frag_selection(
    frag_list: &Vec<(String, String, String)>,
    selected: &mut Vec<bool>
)-> (Vec<bool>, bool) {
    let mut list_state = ListState::default().with_selected(Some(0));
    let mut done = false;
    let mut confirmed = false;
    let mut terminal = ratatui::init();
    let mut all_sel = false;
    let mut all_toml_sel = false;
    let mut all_scripts_sel = false;
    let mut all_bins_sel = false;

    loop {
        let _ = terminal.draw(|frame| {
            let items: Vec<ListItem> = frag_list.iter()
                .enumerate().map(|(i, (fname, _, desc))| {
                    let check = if selected[i] { "[✔]" } else { "[ ]" };
                    let label = format!("{:<4} {:<32} {}", check, fname, desc);
                    ListItem::new(Line::from(Span::raw(label)))
                }).collect();

            let list = List::new(items)
                .block(Block::default().title("Select Files").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Rgb(100, 200, 100)).fg(Color::Black))
                .highlight_symbol("> ");

            let help = Paragraph::new(
                "↑/↓ move · SPACE toggle · 'a' toggle all · 't' toggle all tomls · 's' toggle all scripts · 'b' toggle all binaries · ENTER confirm · ESC cancel"
            ).wrap(Wrap { trim: true }).style(Style::default().fg(Color::Yellow));

            let chunks = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(3),
            ]).split(frame.area());

            frame.render_stateful_widget(list, chunks[0], &mut list_state);
            frame.render_widget(help, chunks[1]);
        });

        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Up => { list_state.select_previous(); }
                KeyCode::Down => { list_state.select_next(); }
                KeyCode::Char(' ') => {
                    if let Some(i) = list_state.selected() {
                        selected[i] = !selected[i];
                    }
                }
                KeyCode::Char('a') => {
                    all_sel = !all_sel;
                    let len = selected.len();
                    selected[..len].fill(all_sel);
                }
                KeyCode::Char(c @ 't' | c @ 's' | c @ 'b') => {
                    let shell_sfx = format!(".{}", shell_suffix());
                    let os_sfx = format!(".{}", os_suffix());
                    let suffix = match c {
                        't' => ".toml",
                        's' => &shell_sfx,
                        'b' => &os_sfx,
                        _   => ".unknown",
                    };
                    let flag = match c {
                        't' => &mut all_toml_sel,
                        's' => &mut all_scripts_sel,
                        'b' => &mut all_bins_sel,
                        _   => &mut false,
                    };
                    *flag = !*flag;
                    for (i, (fname, _, _)) in frag_list.iter().enumerate() {
                        if fname.ends_with(suffix) {
                            selected[i] = *flag;
                        }
                    }    
                }
                KeyCode::Enter => { done = true; confirmed = true; }
                KeyCode::Esc => { done = true; }
                _ => {}
            }
        }
        if done { break; }
    }
    ratatui::restore();

    return (selected.to_vec(), confirmed);
}

fn pick_fragments(
    repodir: &Path,
    known_frags: Vec<String>,
    verbose: bool,
    sel_types: &[FragmentType],
    enable_sel_dialog: bool,
    deleting_repo: bool
) -> Vec<String> {
    let os_suf = os_suffix();
    let shell_suf = shell_suffix();
    let targets: Vec<String> = vec!["toml".to_string(), shell_suf.to_string(), os_suf.to_string()];
    // Each frag_list entry: (filename, fragment, description-per-filename)
    // e.g. ("base.toml", "chetanc10/weegit:base.toml", "Shortcuts for most-used git commands")
    let mut frag_list: Vec<(String, String, String)> = Vec::new();
    let mut result: Vec<String> = Vec::new();

    let reponame = format!("{}/{}",
        repodir.parent().unwrap().file_name().unwrap().to_string_lossy(),
        repodir.file_name().unwrap().to_string_lossy()
    );
    // Collect fragments from given repodir as per OS & shell along with
    // the fragment consumption status for current project directory
    for entry in fs::read_dir(repodir).unwrap().flatten() {
        let dname = entry.file_name().to_string_lossy().into_owned();
        if !targets.contains(&dname) || !entry.path().is_dir()  { continue; }
        let dir = repodir.join(&dname);
        for f in fs::read_dir(&dir).into_iter().flatten() {
            let p = f.as_ref().expect("REASON").path();
            let fname: String = f.as_ref().expect("REASON").file_name().to_string_lossy().into_owned();
            if !p.is_file() || !fname.ends_with(&dname) { continue; }
            if let Some(s) = about_frag(&p) {
                frag_list.push(
                    (fname.to_string(), format!("{}:{}", reponame, fname.to_string()), s)
                );
            }
        }
    }

    let mut selected = vec![false; frag_list.len()];
    if !deleting_repo {
        // Mark frag_list entries that are already consumed by current project
        let mut i = 0;
        for (_, frag, _) in &frag_list {
            if verbose && known_frags.contains(&frag) {
                println!("[VERBOSE] {} is known", frag);
            }
            selected[i] = known_frags.contains(&frag);
            i += 1;
        }
    }
    if !sel_types.is_empty() {
        for s in sel_types {
            if *s == FragmentType::All { selected.fill(true); break; }
            let suffix: &str = match s {
                FragmentType::Toml   => ".toml",
                FragmentType::Script => shell_suffix(),
                FragmentType::Bin    => os_suffix(),
                FragmentType::All    => unreachable!(),
            };
            for (i, (fname, _, _)) in frag_list.iter().enumerate() {
                if fname.ends_with(&suffix) { selected[i] = true; }
            }
        }
    }
    let bak_selected = selected.clone();
    let mut confirmed = false;

    if enable_sel_dialog {
        (selected, confirmed) = allow_user_frag_selection(&frag_list, &mut selected);
    }

    if !confirmed && sel_types.is_empty() {
        if verbose {
            println!("[VERBOSE] {} User didn't save selection. Restoring original..", reponame);
        }
        selected = bak_selected;
    }
    for (i, (_, frag, _)) in frag_list.iter().enumerate() {
        if selected[i] {
            result.push(frag.clone());
        }
    }

    result
}

pub fn validate_frags(repodir: &str, fragments: Option<&[String]>, verbose: bool) -> bool {
    for f in fragments.as_deref().unwrap_or(&[]) {
        let fname = f.rsplit('/').next().unwrap_or(&f);
        let suf = f.rsplit_once('.').map(|(_, e)| e);
        match suf {
            Some(s) if matches!(s, "toml" | "bash" | "zsh" | "fish" | "pwsh") => {
                let path = Path::new(repodir).join(f);
                let content: String = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("{} {} read error: {}", "✗".red(), fname, e);
                        return false;
                    }
                };

                if suf == Some("toml") {
                    let v: toml::Value = match toml::from_str::<toml::Value>(&content) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("{} {} TOML parse error: {}", "✗".red(), fname, e);
                            return false;
                        }
                    };
                    if v.get("_").and_then(|v| v.get("about")).and_then(|v| v.as_str()).is_none() {
                            println!("{} {}: missing _.about string", "✗".red(), fname);
                            return false;
                    }
                } else if !content.lines().any(|l| l.trim_start().starts_with("#about")) {
                    println!("{} {}: missing '#about'", "✗".red(), fname);
                    return false;
                }

                if fname.split('.').count() > 2 {
                    println!("{} {}: dot in name breaks for mise >= 2027.8.10", "✗".red(), fname);
                    return false;
                }

                if verbose {
                    println!("{} {}: valid", "✓".green(), fname);
                }
            },
            _ => continue,
        }
    }

    true
}

pub fn repo_name_to_path(reponame: Option<&str>, manf: &mut Manifest) -> Vec<PathBuf> {
    // For empty reponame, return list with all installed repo dir paths
    let reponame = match reponame {
        Some(name) if !name.is_empty() => name,
        _ => return manf.get_full_repo_list(),
    };

    // For valid reponame, return list with corresponding repo dir path
    let parts: Vec<&str> = reponame.split('/').collect();
    if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        // reponame is correct (user/name) return it's path
        let p = Path::new(&manf.file).parent()
            .unwrap().to_path_buf().join(reponame);
        return vec![PathBuf::from(p)];
    }

    return vec![];
}

pub fn validate_repodirs_list(repodirs: Vec<PathBuf>, op_str: String, dry_run: bool) -> bool {
    if repodirs.is_empty() {
        eprintln!("{} No weeproj found!", "✗".red());
        return false;
    } else if repodirs.len() == 1 && !repodirs[0].is_dir() {
        eprintln!("{} repo for {} doesn't exist!", "✗".red(), repodirs[0].display());
        return false;
    } else if repodirs.len() > 1 {
        println!("{} No reponame given. Using all repos:", "⚠".yellow());
        for p in &repodirs {
            println!("{} ", p.display());
        }
        if dry_run { return true; }
        eprintln!("Press <ENTER> to do per-repo {} or Ctrl-c to exit..", op_str);
        // User can exit with Ctrl-C if user doesn't want to act on all repos
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }
    return true;
}

// Add/Remove/Upgrade fragments with selection or direct user-fragment list
// from a given repo or all repos (is user didn't specify a repo)
pub fn run(
    manf: &mut Manifest,
    reponame: Option<&str>,
    project: &PathBuf,
    sel_types: &[FragmentType],
    addreq_frags: &[String],
    delreq_frags: &[String],
    enable_sel_dialog: bool,
    deleting_repo: bool,
    dry_run: bool,
    verbose: bool
) -> Result<()> {
    let repodirs = repo_name_to_path(reponame, manf);
    let repodir_valid = validate_repodirs_list(
        repodirs.clone(),
        "fragment selection".to_string(),
        dry_run
    );
    if !repodir_valid {
        return Err(anyhow!("{} No valid repodir: sel_need {} frags_del {}",
                "✗".red(), enable_sel_dialog, deleting_repo));
    }
    if (!addreq_frags.is_empty() || !delreq_frags.is_empty()) && repodirs.len() > 1 {
        return Err(anyhow!("{} Cannot add/delete fragments from CLI without proper reponame",
                "✗".red()));
    }

    let conf_d = project.join(".mise/conf.d");

    for repodir in repodirs {
        let reponame = format!("{}/{}",
            repodir.parent().unwrap().file_name().unwrap().to_string_lossy(),
            repodir.file_name().unwrap().to_string_lossy()
        );
        // Get known frags consumed by current project and user-chosen frags
        // and if there's no difference between 2 lists, no change, try next repodir
        let known_frags = manf.load_consumer_fragments(
            &project.to_string_lossy().into_owned(), &reponame);
        if verbose {
            print!("[VERBOSE] known frags from {} (count={}):", reponame, known_frags.len());
            for f in &known_frags {
                if let Some((_, frag)) = f.split_once(':') {
                    print!(" {frag}");
                }
            }
            println!("");
        }
        let selected_frags: Vec<String> = if !addreq_frags.is_empty() {
            let os_suf = os_suffix();
            let shell_suf = shell_suffix();
            let targets: Vec<&str> = vec!["toml", &shell_suf, &os_suf];
            addreq_frags.iter()
                .filter(|n| { targets.iter().any(|t| n.ends_with(t)) })
                .map(|f| format!("{reponame}:{f}"))
                .collect()
        } else {
            pick_fragments(&repodir,
                known_frags.clone(),
                verbose,
                sel_types,
                enable_sel_dialog,
                deleting_repo)
        };

        // Check if atleast 1 fragment is toml
        if !selected_frags.iter().any(|s| s.ends_with("toml")) &&
           (addreq_frags.is_empty() || !addreq_frags.iter().any(|s| s.ends_with("toml"))) {
                eprintln!("{} No toml fragment selected from {}", "✗".red(), reponame);
                continue;
        }

        // First finalize added and deleted fragments
        let mut add_frags: Vec<String> = Vec::new();
        let mut del_frags: Vec<String> = Vec::new();
        for frag in &selected_frags {
            // selected_frags is user-selected either from CLI or runtime selection-dialog
            if !known_frags.contains(&frag.clone()) {
                if verbose {
                    println!("[VERBOSE] Add {}", &frag.clone());
                }
                add_frags.push(frag.clone());
            }
        }
        if delreq_frags.is_empty() {
            // delreq_frags empty => user deselected some known frags from selection-dialog
            for frag in known_frags {
                if !selected_frags.contains(&frag.clone()) {
                    if verbose {
                        println!("[VERBOSE] Del {}", &frag.clone());
                    }
                    del_frags.push(frag.clone());
                }
            }
        } else {
            // delreq_frags not empty => user specified deletable fragments from CLI
            for frag in delreq_frags {
                let f = format!("{}:{}", reponame, frag);
                if known_frags.contains(&f.clone()) {
                    if verbose {
                        println!("[VERBOSE] Del {}", &frag.clone());
                    }
                    del_frags.push(f.clone());
                }
            }
        }

        if verbose {
            println!("[VERBOSE] Changes were {} made in {} fragments for {}",
                if add_frags.is_empty() && del_frags.is_empty() { "not" } else { "" },
                reponame,
                project.display()
            );
        }
        if add_frags.is_empty() && del_frags.is_empty() {
            continue;
        }

        let conf_d_bin = conf_d.join("bin");
        // Ensure local .mise/conf_d/bin directory is created
        if !add_frags.is_empty() && !conf_d_bin.exists() && !dry_run {
            fs::create_dir_all(&conf_d_bin)?;
            fs::write(
                &project.join(".mise").join("wee_enabled.txt"),
                project.as_os_str().as_encoded_bytes()
            )?;
        }

        // Copy and Remove fragment files under project's .mise/conf_d/ path
        for frag in &add_frags { // frag="userX/repoY:frag-name.known-suffix"
            let fragname = frag.rsplitn(2, ':').next().unwrap();
            let fragsuf = fragname.rsplit_once('.').map(|(_, e)| e).unwrap();   
            let frag_path = repodir.join(fragsuf).join(fragname);
            let copy_path = get_copy_path(
                &repodir.to_string_lossy().into_owned(),
                &fragname.to_string(),
                &conf_d.display().to_string());
            if !dry_run {
                fs::copy(&frag_path, &copy_path)?;
                if verbose {
                    println!("[VERBOSE] copied {} → {}\n",
                        frag_path.display(), PathBuf::from(&copy_path).display());
                }
            } else {
                println!("[DRY-RUN] {} would be copied to {}",
                    frag_path.display(), PathBuf::from(&copy_path).display());
            }
        }
        for frag in &del_frags { // frag="userX/repoY:frag-name.known-suffix"
            let fragname = frag.rsplitn(2, ':').next().unwrap();
            let copy_path = get_copy_path(
                &repodir.to_string_lossy().into_owned(),
                &fragname.to_string(),
                &conf_d.display().to_string());
            if !PathBuf::from(&copy_path).exists() { continue; }
            if !dry_run {
                fs::remove_file(&copy_path)?;
                if verbose {
                    println!("[VERBOSE] removed {}\n", PathBuf::from(&copy_path).display());
                }
            } else {
                println!("[DRY-RUN] {} would be removed", PathBuf::from(&copy_path).display());
            }
        }

        if dry_run {
            continue;
        }

        // Update consumer fragment list in manifest
        manf.save_consumer_fragments(project.to_str().unwrap(), &add_frags, &del_frags);

        // Cleanup empty .mise/conf.d directory in project
        if !del_frags.is_empty() {
            let conf_d_empty = !fs::read_dir(&conf_d)
                .map(|d| d.filter_map(|e| e.ok()).any(|e| e.path().is_file()))
                .unwrap_or(false);
            let conf_d_bin_empty = !fs::read_dir(&conf_d_bin)
                .map(|d| d.filter_map(|e| e.ok()).any(|e| e.path().is_file()))
                .unwrap_or(false);
            if conf_d_empty && conf_d_bin_empty {
                fs::remove_dir_all(project.join(".mise"))?;
                if verbose {
                    println!("[VERBOSE] removed {}\n",
                        PathBuf::from(project.join(".mise")).display());
                }
            }
        }

        mise_reshell(verbose)?;
    }

    Ok(())
}

fn find_wee_enabled_files(consumers: &[PathBuf], recurse: bool) -> Vec<PathBuf> {
    let target = Path::new(WEE_EN_FILEPATH);
    let mut results = Vec::new();

    for root in consumers {
        if !recurse {
            let full = root.join(target);
            if full.is_file() {
                if let Ok(p) = fs::canonicalize(target) {
                    results.push(p);
                }
            }
        } else {
            WalkDir::new(root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().to_string_lossy().contains(target.to_str().unwrap()))
                .for_each(|e| { results.push(e.path().to_path_buf()); });
        }
    }

    results
}

pub fn refresh(
    manf: &mut Manifest,
    consumers: &[PathBuf],
    recurse: bool,
    dry_run: bool,
    verbose: bool
) -> Result<()> {
    let mut cleanup_mise_cfg: bool = false;
    let mut mise_do_reshell: bool = false;

    // For each WEE_EN_FILEPATH container, create new consumer metadata
    let wee_en_files = find_wee_enabled_files(&consumers, recurse);
    for weenfile in &wee_en_files {
        let mise_path = weenfile.parent().unwrap().to_path_buf();
        let proj_path = fs::canonicalize(mise_path.parent().unwrap().to_path_buf())?;
        let content = fs::read_to_string(weenfile)?.trim().to_string();
        let proj_path_from_file = Path::new(&content);
        if proj_path_from_file == proj_path { continue; }

        if dry_run {
            println!("[DRY-RUN] would update: {} → {}", proj_path.display(), weenfile.display());
        } else {
            fs::write(weenfile, format!("{}\n", proj_path.display()))?;
            if verbose {
                println!("[VERBOSE] Updated: {} → {}", proj_path.display(), weenfile.display());
            }
            manf.port_consumer_fragments(
                &proj_path_from_file.to_str().unwrap(),
                &proj_path.to_str().unwrap()
            )?;
            mise_do_reshell = true;
        }
    }

    // For each consumer from given consumer list, remove those without WEE_EN_FILEPATH
    for c in consumers {
        if c.join(".mise").join("wee_enabled.txt").is_file() { continue; }
        if dry_run {
            println!("[DRY-RUN] would remove: {}", c.display());
        } else {
            if manf.remove_consumer(&c.to_str().unwrap()) {
                cleanup_mise_cfg = true;
                if verbose {
                    println!("[VERBOSE] Removed: {}", c.display());
                }
            }
        }
    }

    // For each consumer from consumer list from wee metadata, remove those without WEE_EN_FILEPATH
    let consumers_from_metadata = manf.get_full_consumer_list();
    for c in &consumers_from_metadata {
        if c.join(".mise").join("wee_enabled.txt").is_file() { continue; }
        if dry_run {
            println!("[DRY-RUN] would remove: {}", c.display());
        } else {
            if manf.remove_consumer(&c.to_str().unwrap()) {
                cleanup_mise_cfg = true;
                if verbose {
                    println!("[VERBOSE] Removed: {}", c.display());
                }
            }
        }
    }

    // Cleanup mise trust configuration in case any consumer is removed from manifest
    if cleanup_mise_cfg && !dry_run {
        prune_stale_mise_configs(verbose)?;
    }

    if mise_do_reshell {
        mise_reshell(verbose)?;
    }

    Ok(())
}

fn copy_dir_all(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ft = entry.file_type()?;

        if ft.is_symlink() {
            return Err(anyhow!("Aborting due to symlink {}", entry.path().display()));
        }
        if ft.is_dir() {
            copy_dir_all(&entry.path(), &to.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn port(
    manf: &mut Manifest,
    port_path: &PathBuf,
    dry_run: bool,
    verbose: bool
) -> Result<()> {
    let curr_path = std::env::current_dir()?.canonicalize()?;
    if curr_path == *port_path {
        return Err(anyhow!("{} Cannot port to/from local path!", "⚠".yellow()));
    }

    let wee_on_currp = manf.check_if_consumer(curr_path.to_str().unwrap());
    let wee_on_portp = manf.check_if_consumer(port_path.to_str().unwrap());

    // If both current and port paths are either non-consumers or active consumers
    // we cannot port fragments from one to the other either because of no fragments
    // or both having fragments which can't be overwritten
    if wee_on_currp == wee_on_portp {
        return Err(anyhow!("{} Port failed port-path & PWD are both active (or no) consumers!", "⚠".yellow()));
    }

    let (src_path, dst_path): (&Path, &Path) = if wee_on_currp {
        (curr_path.as_path(), port_path.as_path())
    } else {
        (port_path.as_path(), curr_path.as_path())
    };

    // Copy fragments from source to dstination and update manifest
    if dry_run {
        println!("[DRY-RUN] would copy frags from {} to {} ..",
            src_path.display(), dst_path.display());
    } else {
        if verbose {
            println!("[VERBOSE] Copying frags from {} to {} ..",
                src_path.display(), dst_path.display());
        }

        copy_dir_all(&src_path.join(".mise"), &dst_path.join(".mise"))?;
        manf.port_consumer_fragments(src_path.to_str().unwrap(), dst_path.to_str().unwrap())?;

        mise_reshell(verbose)?;
    }

    Ok(())
}
