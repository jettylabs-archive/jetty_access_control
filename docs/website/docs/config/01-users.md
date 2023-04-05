# Users

User configurations are saved in the `<project dir>/users/` directory, with one user per file. Jetty generates file names for each user, but these names are user configurable and have no impact on the operation of the tool (as long as the files maintain a `.yaml` or `.yml` extension). A user configuration file looks something like this:

```yaml title="users/product@get-jetty.com.yaml"
name: product@get-jetty.com
identifiers:
    snowflake: PRODUCT_GUY
    tableau: 4147d13e-5979-49a3-a7d6-e3552f16ef9c
member of:
    - snowflake::ANALYTICS
    - tableau::All Users
    - tableau::Analysts
```

## User Configurations

Users are configured based on the following properties:

-   **name** (required) - The name used to reference a user throughout the Jetty configuration files
-   **identifiers** (required) - A map of connector-specific user identifiers that should be treated as a single user
-   **member of** (optional) - A list of groups the user is a member of (groups can be referenced by their name, as specified in the groups configuration file)

:::tip Changing the name of a user
If you would like to change the name of a user, you must also update all references to the user in your configuration. You can use [`jetty rename`](../cli/rename) to update any references for you.

If you would like to remove all references to a user you can use [`jetty remove`](../cli/remove).
:::
