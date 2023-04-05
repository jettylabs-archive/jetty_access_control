# rename

Rename a group or user across all the configuration files

### Usage

`jetty rename [OPTIONS] <NODE_TYPE> <OLD> <NEW>`

### Arguments

`<NODE_TYPE>`
The type of node that is being modified

Possible values:

-   group: Rename a group
-   user: Rename a user

`<OLD>` The name of the user or group that will be updated

`<NEW>` The new name of that user or group

### Options

| Flag                              | Description                                               |
| --------------------------------- | --------------------------------------------------------- |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error |
| `-h`, `--help`                    | Print help information (use `-h` for a summary)           |
