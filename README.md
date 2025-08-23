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
```wee create```  
This creates wee/mise configuration to setup and manage project environment. This is the first command to kickstart wee operations in a new project directory.


### Remove project wee setup completely
```wee destroy```  
This destroys wee/mise setup from current project.


### Show Current Environment
```wee show```

Displays following for current project setup with wee:
- Environment variables
- Startup and Cleanup commands run on project entry and exit respectively
- Bashit aliases/plugins
- Normal bash aliases
- Bash Functions


### Manage Environment Variables
```wee env <add|del> <var> [value]```

- With `add` → Add a variable with given [value]
- With `del` → Remove a variable.

**Examples:**

```
# Create DEBUG variable with value true
$ wee env add DEBUG true
# Create PORT variable with value 8080
$ wee env add PORT 8080
# Remove PORT variable
$ wee env del PORT # removes PORT
```

### Manage startup and cleanup commands
```wee <start|clean> <add|del> [cmd]```

- `start add [cmd]` → Update startup script with given command
- `clean add [cmd]` → Update cleanup script with given command
- `start del` → Remove a command from startup script
- `clean del` → Remove a command from cleanup script

**Examples:**
```
$ wee start add echo "Current branch: $(git branch)"
$ wee start add echo "Grand changes happening in $PWD!"
$ wee start del
1) echo "Current branch: $(git branch)" #cmd1
2) echo "Grand changes happening in $PWD!" #cmd2
Select a command by index to delete: 2
```

### Manage bash-it aliases and plugins
```wee bashit <alias|plugin> <add|del> <name>```

- `alias` → bash-it alias control
- `plugin` → bash-it plugin control
- `add` → Add alias/plugin enable and disable controls to startup and cleanup scripts respectively
- `del` → Remove alias/plugin enable and disable controls from startup and cleanup scripts respectively

**Examples:**
```
$ wee bashit alias add git
$ wee bashit alias del git
$ wee bashit plugin add git
$ wee bashit plugin del git
```

### Manage Aliases
```wee alias <add|del> <name> [cmd]```

- With `add` → Add an alias with given <name> and [cmd].
- With `del` → Remove an alias.

**Examples:**
```
$ wee alias add ll "ls -lha"
$ wee alias del ll
```

### Manage Functions
```wee func <add|del> <name>```

- With `add` → Add a new function (you’ll be prompted to enter the body; press `Ctrl+D` to finish).
- With `del` → Remove a function.

**Examples:**
```
$ wee func add greet

Enter function body:
  echo "Hello, $USER"

$ wee func del greet # removes function 'greet'
```

## 📂 Project Files

When run inside a project directory, `wee` manages:

- **`.mise.toml`**
  Project-specific configuration used by `mise`. Created from wee-template.toml if missing.

- **`.mise-setup.sh`**
  Sourced on entering project directory. Contains aliases and functions.

- **`.mise-cleanup.sh`**
  Sourced on leaving project directory. Removes aliases and functions.

## 🔄 rewee

Sometimes, after updating wee environment, wee shows this:
```
Run 'rewee' to reload environment immediately
```
Running `rewee` helps reload environment with any change immediately

