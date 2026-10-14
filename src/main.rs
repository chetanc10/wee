mod git;
mod frags;
mod manifest;

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
    #[command(verbatim_doc_comment)]
    /// Copy or Remove fragments from existing weeproj(s) for current project
    Frags {
        /// Optional repo-name
        /// Format: <github-user>/<github-repo>
        /// Opens a selection dialog listing fragments suited for current OS/Shell
        /// and ensures user-selected fragments alone are present in current project
        ///   * mise toml files (suffixed 'toml')
        ///   * shell scripts (suffixed 'bash' 'zsh' 'fish' 'pwsh')
        ///   * OS specific binaries (suffixed 'linux', 'win', 'macos')
        ///   Note: Other suffixed files are discarded
        reponame: Option<String>,
    },
    /// Show Manifest info from registry repos and consumer projects
    Info,
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
