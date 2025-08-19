# wee – Project Environment Helper

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

### Show Current Environment
```wee show```

Displays:
- Environment variables
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

After adding or updating aliases or functions, you may see this tip:
```
Do this to refresh env:
reload cd; cd -
```
Run it to apply changes immediately without restarting your shell.

