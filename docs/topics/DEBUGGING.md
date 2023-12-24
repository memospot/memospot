# Debugging Memos server

> This is intended for advanced users only.

## Introduction

As the Memos server runs as a child process of Memospot, it is not possible to inspect server console output with a regular Memospot build.

However, it is possible to log server output to a file.

## Enabling logging

> Enabling logging will intercept and parse all output from Memos server console. This will increase system resource usage. {style="warning"}

- Create a new `logging_config.yaml` in the Memospot data folder.

> Linux/macOS:
>
> ```bash
> touch ~/.memospot/logging_config.yaml
> ```

> Windows:
>
> ```powershell
> New-Item -ItemType File -Path "$Env:AppData\memospot\logging_config.yaml"
> ```

- Launch the Memospot app.

> If you launch Memospot with an empty or invalid logging_config.yaml, the file will be automatically populated with a default configuration. {style="note"}

## Default configuration {collapsible="true" default-state="collapsed"}

```yaml
appenders:
  file:
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)} - {h({l})}: {m}{n}"
    path: $ENV{MEMOSPOT_DATA}/memos.log
    kind: rolling_file
    policy:
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: fixed_window
        pattern: $ENV{MEMOSPOT_DATA}/memos.log.{}.gz
        count: 5
        base: 1
root:
  # debug | info | warn | error | off
  level: error
  appenders:
    - file
```

> If you use this configuration template, you must use absolute paths for the `appenders.file.path` and `appenders.file.policy.roller.pattern` fields, as the working directory of Memospot may be write-protected on some systems.
>
> For conveninence, the `$ENV{MEMOSPOT_DATA}` environment variable is available in the configuration file.

## Increasing log level

> To log _all_ requests handled by server, set `root.level` to `info` or `debug`.

Using [yq](https://github.com/mikefarah/yq) (Install via Homebrew, MacPorts, Winget, Chocolatey or Scoop):

- Linux/macOS:

  ```bash
  yq -i '.root.level = "debug"' "~/.memospot/logging_config.yaml"
  ```

- Windows:
  ```powershell
  yq -i '.root.level = "debug"' "$Env:AppData\memospot\logging_config.yaml"
  ```

Extra configurations options can be found in the [log4rs documentation](https://github.com/estk/log4rs#quick-start)

## Disabling logging

> It is recommended to disable logging when not needed.
>
> To disable logging entirely, delete the `logging_config.yaml` file.
> {style="note"}

- Linux/macOS:

  ```bash
  rm ~/.memospot/logging_config.yaml
  ```

- Windows:
  ```powershell
  Remove-Item "$Env:AppData\memospot\logging_config.yaml"
  ```

> Setting `root.level` to `off` will disable logging to file, but will not disable the server output parsing.
