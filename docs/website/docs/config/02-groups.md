# Groups

Group configurations are saved in `<project dir>/groups/groups.yaml`. A group configuration file has entries that look something like this:

```yaml title="groups/groups.yaml"
- ...
- name: My Special Group
  description: 'A special group for a special occasion'
  identifiers:
      snowflake: MY_SPECIAL_GROUP
  member of:
      - Party Planning Committee
```

You may notice the connector name prepended to the group names. When this format is used, the group is specific to a single connector (in the example above, Jetty won't try to create an All Users group in Snowflake; it knows that group is tableau-specific).

## Group Configurations

Groups are configured based on the following properties:

-   **name** (required) - The name used to reference a group throughout the Jetty configuration files. If a group name is prefixed with a connector name followed by two colons (e.g., snowflake::ACCOUNT_ADMIN), the group will only exist in the specified connector. Otherwise, the group will be created across all group-capable connectors
-   **description** (optional) - A description of the group
-   **identifiers** (optional) - A map of connector-specific names for the group. This allows you to great a Jetty group that is materialized with custom names in one or more connectors; any connector without an entry in this map will have a group created with the name specified in the `name` property
-   **member of** (optional) - A list of groups the group is a member of (groups must be referenced by their name, as specified in the groups configuration file); for connectors that do not support nested groups (like Tableau), users' inherited group membership will be applied directly in each group (i.e., if User A is a member of Group 1, and Group 1 is a member of Group 2, in Tableau, User A will be a direct member of both Group 1 and Group 2)

:::tip Changing the name of a group
If you would like to change the name of a group, you must also update all references to the group in your configuration. You can use [`jetty rename`](../cli/rename) to update any references for you.

If you would like to remove all references to a group you can use [`jetty remove`](../cli/remove).
:::
