## system

The `.system` command executes an operating system command from within rsql. This is useful for running shell commands,
inspecting the environment, or integrating with other tools without leaving the CLI.

### Usage

```text
.system command [args]
```

### Description

The system command executes the specified operating system command. The command and any optional arguments are passed
to the operating system shell for execution. The output of the command is displayed to the defined output.

### When to use

- Use `.system` to run shell commands (e.g., `ls`, `pwd`, `cat file.txt`) without leaving rsql.
- Helpful for automation, scripting, or when you need to check files, directories, or system status during a session.

### Examples

Print the current working directory:

```text
.system pwd
```

List the current directory:

```text
.system ls -l
```

Run a script or external tool:

```text
.system ./myscript.sh arg1 arg2
```

### Troubleshooting

- If a command fails, check the syntax and ensure the command exists in your system's PATH.
- Output is sent to the current output destination (see `.output`).
- Some commands may behave differently depending on your OS (macOS, Linux, Windows).

### Related

- For output redirection, see [output](../output/index.md) and [tee](../tee/index.md).
- For scripting, see [read](../read/index.md) and [sleep](../sleep/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
