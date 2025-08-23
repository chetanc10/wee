# wee – Work Environment Enhancer

`wee` is a lightweight Bash-based tool to **create and manage project-specific environments**.  
It integrates with [mise](https://mise.jdx.dev/) to automatically load your environment when you **enter a project directory** and unload it when you **leave**.

This ensures your **environment variables**, **aliases**, and **functions** remain scoped to a single project without affecting your global shell setup.

## ✨ Features
- Define and manage **environment variables** local to your project  
- Create and remove **aliases** for project-specific commands  
- Add, overwrite, or remove **functions** scoped to the project  
- Automatically manage **setup** and **cleanup** via `mise` hooks  
- Display all current project-specific configurations with a single command  

## 📦 Installation

Use basherbee to install: ```basherbee install chetanc10/wee```  
mise is automatically setup when installed with basherbee

## 🚀 Usage

```wee <command> [args...]```

### New project wee setup
```wee new```  
This creates wee/mise configuration to setup and manage project environment. This is the first command to kickstart wee operations in a new project directory.


### Manage Environment Variables
```wee env <VAR[=value]>```

- With `=value` → Add or update a variable.
- Without `=value` → Remove the variable.

**Examples:**

```
$ wee env DEBUG=true
$ wee env PORT=8080
$ wee env PORT # removes PORT
```

### Manage Aliases
```wee alias <name> [cmd...]```

- With `cmd...` → Add or update an alias.
- Without `cmd...` → Remove the alias.

**Examples:**
```
$ wee alias ll "ls -lah"
$ wee alias gs "git status"
$ wee alias ll # removes alias 'll'
```

### Manage startup and cleanup commands
```wee <start|clean> <a|d>```

- `start` → Update startup script
- `clean` → Update cleanup script
- `a` → Add a command to startup or cleanup script
- `d` → Remove a command from startup or cleanup script

**Examples:**
```
$ wee start a
Enter one-line bash command with args: echo "Current branch: $(git branch)"
$ wee clean a
Enter one-line bash command with args: echo "Checking for uncommited changes.. $(git status -uno)"
$ wee start d
1) echo "Current branch: $(git branch)" #cmd1
2) echo "Grand changes happening in $PWD!" #cmd2
Select a command by index to delete: 2
```

### Manage Functions
```wee func <name> [-]```

- With `-` → Add a new function (you’ll be prompted to enter the body; press `Ctrl+D` to finish).
- Without `-` → Remove the function.

**Examples:**
```
$ wee func greet -

Enter function body:
  echo "Hello, $USER"

$ wee func greet # removes function 'greet'
```

### Manage bash-it aliases and plugins
```wee bashit <a|d> <alias|plugin> <name>```

- `alias` → bash-it alias control
- `plugin` → bash-it plugin control
- `a` → Add alias/plugin enable and disable controls to startup and cleanup scripts respectively
- `d` → Remove alias/plugin enable and disable controls from startup and cleanup scripts respectively

**Examples:**
```
$ wee bashit a alias git
$ wee bashit d alias git
$ wee bashit a plugin git
$ wee bashit d plugin git
```

### Show Current Environment
```wee show```

Displays following for current project setup with wee:
- Environment variables
- Bashit aliases/plugins
- Aliases
- Functions

## 📂 Project Files

When run inside a project directory, `wee` manages:

- **`.mise.toml`**
  Project-specific configuration used by `mise`. Created from wee-template.toml if missing.

- **`.mise-setup.sh`**
  Sourced on entering project directory. Contains aliases and functions.

- **`.mise-cleanup.sh`**
  Sourced on leaving project directory. Removes aliases and functions.

## 🔄 Refreshing Environment

After adding or updating aliases or functions, you may tips similar to below:
```
Do this to refresh environment immediately:
 unalias llh; cd; cd -
```
Run such commands to apply changes immediately without restarting your shell.

