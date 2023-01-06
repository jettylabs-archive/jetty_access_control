# subgraph

Get the dot representation of a subgraph

### Usage

`jetty subgraph [OPTIONS] --id <ID>`

### Options

| Flag                              | Description                                                                            |
| --------------------------------- | -------------------------------------------------------------------------------------- |
| `-i`, `--id` `<ID>`               | The root node_id for the subgraph. You can get this from the url of the explore web UI |
| `-d`, `--depth` `<DEPTH>`         | The depth of the subgraph to collect [default: 1]                                      |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error                              |
| `-h`,` --help`                    | Print help information                                                                 |
