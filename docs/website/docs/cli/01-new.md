# new

Launch a guided flow to create a Jetty project in a new directory

### Usage

`jetty new [OPTIONS] [PROJECT_NAME]`

### Arguments

`[PROJECT_NAME]` Project name (optional)

### Options

| Flag                              | Description                                                                                                                                                                                                                              |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-f`, `--from` `<FROM>`           | Initialize from an existing jetty_config.yaml file. This will create a new directory based on the name specified in the config file. For this to work properly, you must also have an appropriate ~/.jetty/connectors.yaml file in place |
| `-o`, `--overwrite`               | Overwrite project directory if it exists                                                                                                                                                                                                 |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error                                                                                                                                                                                |
| `-h`, `--help`                    | Print help information                                                                                                                                                                                                                   |
