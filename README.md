# wee – Work Environment Enhancer

`wee` is a Rust-based tool to manage mise environment controller packages from GitHub.

## Inception

I'm primarily a bash user and had used `bash-it` for some time — it's a great repository combining aliases, functions, and plugins, but it's all Bash-only. As I started using PowerShell and other shells more, I wanted a lightweight way to share environment fragments (tomls, scripts, small binaries) across different shells and OSes.

wee works in conjunction with mise (see https://github.com/jdx/mise). mise manages shell environment using toml files; wee manages projects that package mise tomls, scripts, and OS-specific binaries so those fragments can be consumed by other projects.

Not just tomls: wee also enables shell scripts to be installed into .mise/conf.d/bin/ for each project that contains wee fragments.

Note: wee is useful where generic environment management and script sharing is needed across multiple platforms (as mise supports). For project-specific tomls/scripts, users can maintain them directly inside their project if preferred.

## Terminology

- `fragment` - Any file type that is recognized by wee and can be runtime loaded/removed for a project. File types recognized by wee:
  - .toml  - mise toml files
  - .bash  - bash script files (must have this suffix). See "Shell Scripts" section for details.
  - .zsh   - zsh script files (must have this suffix). See "Shell Scripts" section for details.
  - .fish  - fish script files (must have this suffix). See "Shell Scripts" section for details.
  - .ps1 / .pwsh - PowerShell script files (must have this suffix). See "Shell Scripts" section for details.
  - .linux - Linux OS executable binaries (planned; see "OS Binaries").
  - .win   - Windows OS executable binaries (planned; see "OS Binaries").
  - .macos - macOS executable binaries (planned; see "OS Binaries").
- `weeproj` - Any GitHub project that contains wee `fragments`. This identifier is used only in documentation and examples.
- `consumer` - Any local project that uses any `fragment` from a `weeproj`.
- `manifest` - A single manifest file maintained by wee to track installed `weeproj` repos, `fragment` files and `consumer` mappings.

## ✨ Features

- Install, upgrade and uninstall `weeproj` from GitHub
- Select and synchronize `fragments` (tomls, shell scripts, OS binaries) for a consumer project via an interactive selector
- Auto-refresh `fragment` whenever a `weeproj` is upgraded or when consumers are ported/refreshed
- If you don't choose fragments explicitly, the interactive selector can auto-select tomls and shell-specific scripts based on the current shell
- Show info about currently installed `weeproj` repos and `fragments` consumed by projects
- Dry-run and verbose modes for safe operations

## Installation

### Linux

```bash
curl -fsSL https://github.com/chetanc10/wee/releases/latest/download/wee-linux-x86_64 | sudo tee /usr/local/bin/wee > /dev/null && sudo chmod +x /usr/local/bin/wee
```

### Windows and macOS

Installers and packages for Windows and macOS are planned. For now, please check the Releases page for available binaries.

## `weeproj` Template

Refer to https://github.com/chetanc10/weegit for practical examples of `weeproj` and `fragment` templates.

### Sample `weeproj` directory tree

```
.sample-directory-tree/
├── toml
│   ├── base.toml
│   └── build-custom.toml
├── bash
│   ├── bulk-check.bash
│   └── inspect.bash
├── zsh
│   ├── bulk-check.zsh
│   └── inspect.zsh
├── fish
│   ├── bulk-check.fish
│   └── inspect.fish
├── pwsh
│   ├── bulk-check.pwsh
│   └── inspect.pwsh
├── LICENSE           # Ignored
├── README.md         # Ignored
├── dir1              # Other directories are ignored (dirs can contain src to build/deploy binaries; wee doesn't touch directories)
├── 2.sh              # Ignored (exact shell-suffixed files only are considered)
└── switch.c          # Ignored (other file types are ignored)
```

### Fragment Templates

#### TOML fragments
Every toml fragment should include a short description under the [_] table:

```toml
[_]
about = "description of toml"
```

The code requires this about entry to be present for toml fragments.

#### Shell fragments
Every shell fragment (bash, zsh, fish, pwsh) should include a single-line description at the top of the file using the "#about" prefix (no leading space after '#'):

Example:
```bash
#!/usr/bin/env bash
#about: single-liner description
```

The tool looks for lines starting with "#about" to extract the short description.

If a `weeproj` provides binaries and a toml needs to expose a per-project bin path, include:

```toml
[env]
_.path = ["{{ config_root }}/bin"]
```

This enables mise + wee to add a per-project bin directory into PATH.

### Shell Scripts

Shell scripts in a `weeproj` must use the following suffixes to indicate their target shell:

```
name1.bash  => bash script, copied to PWD/.mise/conf.d/bin/name1 (bin/ is added to PATH)
name1.zsh   => zsh script, copied to PWD/.mise/conf.d/bin/name1 (bin/ is added to PATH)
name1.fish  => fish script, copied to PWD/.mise/conf.d/bin/name1 (bin/ is added to PATH)
name1.ps1 or name1.pwsh => PowerShell script, copied to PWD/.mise/conf.d/bin/name1 (bin/ is added to PATH)
```

### OS Binaries

OS binary support is planned for future versions (suffixes: .linux, .win, .macos). Binaries follow the same per-OS selection rules.

## 🚀 Usage

Global flags:
- --dry-run : show what would happen without executing
- -v / --verbose : verbose output

Commands:

- Install
  ```
  wee install <git-url>
  ```
  Clone a weeproj into the registry.

- Uninstall
  ```
  wee uninstall <username/reponame>
  ```
  Remove a weeproj and all its references from all consumer projects.

- Upgrade
  ```
  wee upgrade <username/reponame>
  ```
  Upgrade a weeproj and update fragments in consumers.

- Frags (interactive fragment sync)
  ```
  wee frags [<username/reponame>]
  ```
  This is the interactive command to synchronize fragments from one (or all) installed weeproj(s) into the current project (consumer). Behavior:
  - Scans the specified repo (or all repos if none is given) for fragments matching the current OS and shell.
  - Presents an interactive TUI selection where you can toggle which fragments should be present in the current project.
  - After confirming, wee will copy newly-selected fragments into .mise/conf.d (tomls) and .mise/conf.d/bin (scripts/binaries), and remove fragments you deselect.
  - The TUI keys:
    - ↑ / ↓ : move
    - SPACE : toggle a fragment
    - a : toggle all
    - t : toggle all tomls
    - s : toggle all scripts for the detected shell
    - b : toggle all binaries for the detected OS
    - ENTER : confirm and apply changes
    - ESC : cancel (no change)
  - The command enforces that at least one toml fragment remains selected (to ensure mise has a toml to manage the project).

- Port
  ```
  wee port <path>
  ```
  Port (copy) a wee setup from an existing consumer to another directory, or vice versa. The command inspects whether the current PWD or the provided path is a consumer and copies .mise configuration and updates the manifest accordingly. It will refuse to port if both sides are active consumers or both are non-consumers.

- Refresh
  ```
  wee refresh [--subd] [--list <dir> ...]
  ```
  Refresh manifest and metadata for consumers. Use --subd to recurse into subdirectories and --list to provide specific directories. This is useful when .mise directories were moved/copied manually or when you want to re-sync manifest state.

- Info
  ```
  wee info
  ```
  Show manifest information about registry repos and consumer projects.

## Implementation notes (behavior observed from code)

- Fragment discovery is limited to directories named after the suffixes (toml, bash, zsh, fish, pwsh) at the top level of a weeproj. Files not matching expected suffixes are ignored.
- TOML fragments require the [_] table with an about value. Shell fragments require a "#about" line.
- The consumer-side fragments are copied into .mise/conf.d and .mise/conf.d/bin. When a project has no fragments left, wee will remove the .mise directory for that consumer.
- wee keeps a wee-enabled marker file .mise/wee_enabled.txt inside consumers to track which directories are active for manifest bookkeeping.
- The tool runs `mise reshell` after making changes (where appropriate) and will advise the user if they should run/reload the shell.

## Examples

- Interactive fragment sync for a specific repo:
  ```
  wee frags username/reponame
  ```

- Inspect and refresh current directory and its subdirectories:
  ```
  wee refresh --subd --list .
  ```

- Dry-run showing what would happen when synchronizing fragments:
  ```
  wee --dry-run frags username/reponame
  ```

---
