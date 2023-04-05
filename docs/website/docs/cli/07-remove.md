# remove

Remove references to a group or user across all the configuration files

### Usage

`jetty_cli remove [OPTIONS] <NODE_TYPE> <NAME>`

### Arguments

`<NODE_TYPE>` The type of node that is being removed

Possible values:

-   group: Remove references to a group
-   user: Remove references to a user

`<NAME>` The name of the user or group that is being removed

### Options

| Flag                              | Description                                               |
| --------------------------------- | --------------------------------------------------------- |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error |
| `-h`, `--help`                    | Print help information (use `-h` for a summary)           |
