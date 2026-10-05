mod git;
mod frags;
mod manifest;
mod shell;

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
    /// Clone a fragment repo into the registry
    Install {
        /// Git URL of the fragment repo
        url: String,
    },
    /// Remove a fragment repo and all its references from all projects
    Uninstall {
        /// Name of the repo to remove
        name: String,
    },
    #[command(verbatim_doc_comment)]
    /// Copy fragment(s) from existing repo in registry into current project
    Add(AddOrDelArgs),
    #[command(verbatim_doc_comment)]
    /// Remove fragment(s) from current project
    Remove(AddOrDelArgs),
    /// Show Manifest info from registry repos and consumer projects
    Info,
}

#[derive(clap::Args)]
struct AddOrDelArgs {
    /// <github-user>/<github-repo> or /absolute/path/to/local-only-repo
    #[arg(value_name = "SOURCE")]
    source: String,
    /// Space-separated fragment (file) names. Unless specified, it
    /// defaults to all toml, shell-script and OS executable files in repo.
    /// Recognized fragments are any of below:
    ///     > mise toml files (suffixed 'toml')
    ///     > shell scripts (suffixed 'bash' 'zsh' 'fish' 'pwsh')
    ///     > OS specific binaries (suffixed 'linux', 'win', 'macos')
    ///     Note: Any other suffixed files are discarded
    #[arg(value_name = "FRAGMENTS", num_args = 0.., verbatim_doc_comment)]
    fragments: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut manf = Manifest::load()?;

    let result = match &cli.command {
        Commands::Install { url } => {
            manf.add_repo(url, cli.dry_run, cli.verbose)
        },
        Commands::Uninstall { name } => {
            manf.del_repo(name, cli.dry_run)
        },
        Commands::Add(args) => {
            let repo = shell::parse_source(&args.source, &manf);
            Ok(frags::run(&mut manf, frags::FADD, &repo,
                    &args.fragments, cli.dry_run, cli.verbose)?)
        },
        Commands::Remove(args) => {
            let repo = shell::parse_source(&args.source, &manf);
            Ok(frags::run(&mut manf, frags::FDEL, &repo,
                    &args.fragments, cli.dry_run, cli.verbose)?)
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
