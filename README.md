# wee – Work Environment Enhancer

`wee` is a rust-based tool to **manage mise environment controller packages from github**.



## Inception
I'm primarily a bash user and I had used `bash-it` from sometime. It is a good repository combining aliases/functions/plugins, but it's all on bash. As I started using powershell recently, I started porting some of such aliases/functions from bash-it, but it was cumbersome. And so I got the idea of generic cross-platform alias/function/script repository management tool using mise.

A very good example is git shortcuts. There are others..

wee works in conjunction with mise (refer to `https://github.com/jdx/mise`) - mise manages shell env as per tomls and wee manages projects that package mise tomls, scripts and OS specific binary releases.

Not just tomls, wee also enables shell scripts to be installed to .mise/conf.d/bin/ per project that has atleast wee toml placed.

Note: wee is useful where there are generic environment management and script sharing is needed across multiple platforms like mise supports. For project-specific tomls/scripts also, users can maintain separate repositories and share them and add them to projects so as to avoid adding such non-project files into their projects.



## Terminology
* `fragment` - Any file type that is recognized by wee is considered as a fragment to be runtime loaded/removed for any project. Following are the file types that wee considers as fragments.
  * .toml  - mise toml files 
  * .bash  - bash script files must always have this suffix. See `Shell Scripts` section for more details.
  * .zsh   - zsh script files must always have this suffix. See `Shell Scripts` section for more details.
  * .fish  - fish script files must always have this suffix. See `Shell Scripts` section for more details.
  * .ps1   - ps1 script files must always have this suffix. See `Shell Scripts` section for more details.
  * .linux - Linux OS executable binaries. See `OS Binaries` section for more details.
  * .win   - Windows OS executable binaries. See `OS Binaries` section for more details.
  * .macos - Mac OS executable binaries. See `OS Binaries` section for more details.
* `weeproj` - Any github project that contains wee `fragments` is considered as `weeproj`. This is not a tool intrinsic terminology, but an identifier used to documentation only, in README.md, issues, discussions to refer to projects that are understood by wee.
* `consumer` - Any local user project that uses any `fragment` from any `weeproj`
* `manifest` - A single manifest file is maintained by wee to track all current `weeproj` repos, `fragment` files and `consumer` mapping information to properly them all together.




## ✨ Features
- Install, upgrade and uninstall `weeproj` from github
- Add and Remove `fragments` to control environment setup within a project
- Auto refresh `fragment` whenever a `weeproj` is upgraded
- If a 'wee add' request doesn't specify 'fragment' names, wee auto-selects all tomls and all shell specific scripts (as per current shell)
- Show info of currently installed `weeproj` repos and `fragments` consumed by various projects




## Installation

### Linux
curl -fsSL https://github.com/chetanc10/wee/releases/latest/download/wee-linux-x86_64 | sudo tee /usr/local/bin/wee > /dev/null && sudo chmod +x /usr/local/bin/wee

### Windows and MacOS TODO


## `weeproj` Template
### Sample `weeproj` directory tree
```
.sample-directory-tree/
├── base.toml
├── gd1.bash // If script needed for bash
├── gd1.ps1 // If script needed for powershell
├── LICENSE // Ignored
├── README.md // Ignored
├── dir1 // directories are ignored (dirs can have src to build/deploy binaries, wee doesn't touch directories anyway)
├── 2.sh // sh files are ignored (exact shell suffixed files only are considered - refer to `Terminology`)
├── switch.c // Any other type files are ignored (refer to `Terminology`)
└── tag.toml
```

### toml template
Every toml file in a `weeproj` is expected to have following:
```
[env]
_.path = ["{{ config_root }}/bin"]
```
This enables wee in conjunction with mise to add a bin directory per project to PATH that contains shell scripts and OS binaries as long as it's in the same directory containing the .mise* entries.

### Shell Scripts
Shell scripts in `weeproj` must follow suffix templates - 
```
name1.bash => name1 is a bash shell script, copied to PWD/.mise/conf.d/bin/name1 and the bin/ is added to PATH
name1.zsh => name1 is a bash shell script, copied to PWD/.mise/conf.d/bin/name1 and the bin/ is added to PATH
name1.fish => name1 is a bash shell script, copied to PWD/.mise/conf.d/bin/name1 and the bin/ is added to PATH
name1.ps1 => name1 is a bash shell script, copied to PWD/.mise/conf.d/bin/name1 and the bin/ is added to PATH
```

### OS Binaries
OS Binary support shall come in future versions - it's a work in progress.



## 🚀 Usage

```wee <command> [args...]```

### Command Help Messages
```wee help [command]``` shows tool or command level help messages.


### `weeproj` management
#### Installing `weeproj`
```wee install <github-url>```
This clones a `weeproj` using given URL and confirms it has atleast 1 toml()  ion to setup and manage project environment in current directory. This is the first command to kickstart wee operations in a new project directory.


#### Removing `weeproj`
```wee destroy <username/reponame>```  
This removes specific `weeproj` from disk and removes references/copies of it's `fragment` files from all 'consumer' projects.


#### Upgrading `weeproj`
```wee upgrade <username/reponame>```  
This upgrades an existing `weeproj` and updates references/copies of it's `fragment` files across all 'consumer' projects.
**This is work in progress**



### `fragment` management
All `fragment` management commands are expected to run inside user projects (of course that's what mise tomls and thus wee demands).


#### Adding `fragments`
```wee add <username/reponame> frag1.toml binder.bash ledger.linux```
This validates given `fragments` - non-toml files must conform to current OS and shell detected by the tool - binder.bash checks against current shell 'bash' and ledger.linux against OS 'linux' as detected.
```wee add <username/reponame>```
This forces wee to get all valid `fragment` files from given `weeproj` and validates non-toml files conform to current OS and shell detected by the tool.

- After validation, wee ensures a ./.mise/conf.d/bin directory is created in current directorythese `fragments` are then copied
  - tomls under .mise/conf.d
  - non-tomls under .mise/conf.d/bin (executables are to be inside bin/ that is added to PATH by mise)


Note: If a non-conformant `fragment` is specified, a warning is issued and that `fragment` is discared.


#### Removing `fragment`
```wee remove <username/reponame> frag1.toml binder.bash ledger.linux```
```wee remove <username/reponame>```
- If `fragments` are not specified, wee defaults to taking all `fragments` from given `weeproj`.
- Each `fragment` is checked and removed from current project
- `manifest` is edited against current `consumer` to update `fragment` to `consumer` mapping
- If no `fragments` remain in PWD/.mise/conf.d/ PWD/.mise/conf.d/bin, PWD/.mise folder is removed

