# wee – Work Environment Enhancer

`wee` is a lightweight Bash-based tool to **create and manage project-specific environments**.  
It integrates with bash 'cd' command to automatically load project-specific environment when **entering a project directory** and unload it when **leaving the project directory**.

This ensures **environment variables**, **aliases**, and **functions** remain scoped to a single project without affecting global shell and other project environment setup.

## ✨ Features
- Create and manage **environment variables** local to a project  
- Create and remove **aliases** for project-specific commands
- Add, overwrite, or remove **functions** local to a project
- Automatically manage **setup** and **cleanup** via `cd` hook
- Display all current project-specific configurations with a single command

## 📦 Installation

Use basherbee to install: ```basherbee install chetanc10/wee```  

## 🚀 Usage

```wee <command> [args...]```

### New project wee setup
```wee create```  
This creates wee configuration to setup and manage project environment in current directory. This is the first command to kickstart wee operations in a new project directory.


### Remove project wee setup completely
```wee destroy```  
This destroys wee configuration for current project directory.


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
$ wee env del PORT # removes PORT env variable
```

### Manage startup and cleanup commands
```wee <start|clean> <add|del> [cmd]```

- `start add [cmd]` → Update startup script with given command
- `clean add [cmd]` → Update cleanup script with given command
- `start del` → Remove a command from startup script
- `clean del` → Remove a command from cleanup script

**Examples:**
```
$ wee start add echo "git ls-files --others --exclude-standard"
$ wee start add echo "Grand changes happening in $PWD!"
$ wee start del
1) echo "git ls-files --others --exclude-standard" #cmd1
2) echo "Grand changes happening in ~/proj/curr-dir!" #cmd2
Select a command by index to delete: 2
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

- With `add` → Add a new function (user will be prompted to enter function body; press `Ctrl+D` to finish).
- With `del` → Remove a function.

**Examples:**
```
$ wee func add gcg

Enter function body:
  git config -l

$ wee func del gcg # removes function 'gcg'
```

## 📂 Per Project Wee Files

For each wee-configured project, `wee` manages two files list below:

- $HOME/.wee/\<absolute-path-to-project>-out.sh →  Sourced after a cd operation if OLDPWD is wee-configured 
- $HOME/.wee/\<absolute-path-to-project>-in.sh →  Sourced after a cd operation if PWD is wee-configured

Both the above files are created with ```wee create``` and removed with ```wee destroy```

## 🔄 rewee

Sometimes, after updating wee environment, wee shows this:
```
Run 'rewee' to reload environment immediately
```
Running `rewee` helps reload environment with any change immediately

