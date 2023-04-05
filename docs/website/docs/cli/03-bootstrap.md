# bootstrap

Fetch and build out initial configuration files for a project

### Usage

`jetty bootstrap [OPTIONS]`

### Arguments

`[PROJECT_NAME]` Project name (optional)

### Options

| Flag                              | Description                                                                     |
| --------------------------------- | ------------------------------------------------------------------------------- |
| `-n`, `--no-fetch`                | Don't fetch the current configuration before generating the configuration files |
| `-o`, `--overwrite`               | Overwrite files if they exists                                                  |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error                       |
| `-h`, `--help`                    | Print help information                                                          |
