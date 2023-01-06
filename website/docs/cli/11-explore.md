# explore

Launch the exploration web UI

### Usage

`jetty explore [OPTIONS]`

### Arguments

`[PROJECT_NAME]` Project name (optional)

### Options

| Flag                              | Description                                                        |
| --------------------------------- | ------------------------------------------------------------------ |
| `-f`, `--fetch`                   | Fetch the current configuration before launching the UI            |
| `-b`, `--bind` `<BIND>`           | Select the ip and port to bind the server to (e.g. 127.0.0.1:3000) |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error          |
| `-h`, `--help`                    | Print help information                                             |
