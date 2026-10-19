mod git;
mod frags;
mod manifest;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::manifest::Manifest;

#[derive(Parser)]
#[command(name = "wee", version,
    about = "Work Environment Enhancer with mise and tomls, scripts, OS-specific-binaries")]
struct Cli {
    /// Show what would happen without executing
    #[arg(long, global = true)]
    dry_run: bool,
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone a weeproj into the registry
    Install {
        /// Git URL of the weeproj
        url: String,
    },
    /// Remove a weeproj and all its references from all projects
    Uninstall {
        /// Name of the repo to remove
        reponame: Option<String>,
    },
    /// Upgrage a weeproj and all its references from all projects
    Upgrade {
        /// Name of the repo to remove
        reponame: Option<String>,
    },
    /// Copy or Remove fragments from existing weeproj(s) for current project
    Frags {
        /// Optional repo-name
        /// Format: <github-user>/<github-repo>
        /// Opens a selection dialog listing fragments suited for current OS/Shell
        /// and ensures user-selected fragments alone are present in current project
        ///   - mise toml files (suffixed 'toml')
        ///   - shell scripts (suffixed 'bash' 'zsh' 'fish' 'pwsh')
        ///   - OS specific binaries (suffixed 'linux', 'win', 'macos')
        ///   Note: Other suffixed files are discarded
        #[arg(verbatim_doc_comment)]
        reponame: Option<String>,
    },
    /// Port a wee setup from an existing consumer to a newbie consumer
    Port {
        /// Path is interpreted as either wee configuration source or destination by
        /// checking Path and PWD wee configuration presence:
        ///   - PWD has no WEE config && Path has WEE config => Copy config from Path to PWD
        ///   - Path has no WEE config && PWD has WEE config => Copy config from PWD to Path
        /// Any other case is effectively a no-op.
        #[arg(verbatim_doc_comment, value_parser = arg_pathbuf_to_dir)]
        path: PathBuf,
    },
    /// Refresh wee setup for consumer(s) in manifest
    Refresh {
        /// subd flag forces recursive refresh with sub-directoryes under target(s)
        /// Default is non-recursive
        #[arg(long, verbatim_doc_comment)]
        subd: bool,
        /// Space separated list of relative/absolute directories;
        /// Needed when user did copy/move/removal of wee enabled directory or wee-created .mise directory
        /// manually without using Port command
        /// This command takes from few millisec to seconds as per depth
        /// Default is current directory
        #[arg(long, verbatim_doc_comment, value_parser = arg_pathbuf_to_dir)]
        list: Vec<PathBuf>,
    },
    /// Show Manifest info from registry repos and consumer projects
    Info,
}

fn arg_pathbuf_to_dir(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.is_dir() {
        Err(format!("not a directory: {}", path.display()))
    } else {
        Ok(path)
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut manf = Manifest::load()?;

    let result = match &cli.command {
        Commands::Install { url } => {
            manf.add_repo(url, cli.dry_run, cli.verbose)
        },
        Commands::Uninstall { reponame } => {
            manf.del_repo(reponame.as_deref().unwrap_or(""), cli.dry_run, cli.verbose)
        },
        Commands::Upgrade { reponame } => {
            manf.upgrade_repo(reponame.as_deref().unwrap_or(""), cli.dry_run, cli.verbose)
        },
        Commands::Frags{ reponame } => {
            Ok(frags::run(&mut manf,
                    reponame.as_deref(),
                    &std::env::current_dir()?.canonicalize()?,
                    true,
                    false,
                    cli.dry_run,
                    cli.verbose)?)
        },
        Commands::Refresh { subd, list } => {
            let dirs: Vec<PathBuf> = if list.is_empty() {
                vec![PathBuf::from(".")]
            } else {
                list.to_vec()
            };
            frags::refresh(&mut manf, &dirs, *subd, cli.dry_run, cli.verbose)
        },
        Commands::Port { path } => {
            frags::port(&mut manf, &path, cli.dry_run, cli.verbose)
        },
        Commands::Info => {
            Ok(manf.show(cli.verbose))
        },
    };

    if let Err(e) = result {
        eprintln!("{} {e:#}", "error:".red().bold());
        std::process::exit(1);
    }

    if !matches!(cli.command, Commands::Info) {
        return Ok(manf.save()?);
    }

    return Ok(());
}
