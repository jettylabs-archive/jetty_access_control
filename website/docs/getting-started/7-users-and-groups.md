---
sidebar_position: 7
slug: './users-and-groups'
---

# Manage users and groups

## Users

In Jetty, users represent users of your data tools, human or otherwise. The primary key for users is their email address and other user information can also be, also be associated with users in the form of attribute key-value pairs. These attributes can be configured directly in the `users.yaml` configuration file or can be provided as metadata from connectors. As we mentioned in the [section on bootstrapping](./5-bootstrap.md), user records can also specify the platform that the user exists in.

Users are defined in the `/users/` directory. When you bootstrap the system, a `users.yaml` file will be created in that directory, but you are welcome to split your groups across any number of files in any number of sub-directories. A user object is structured as follows:

```yaml
elliot@gmail.com:
  connectors:
    - snow
    - tableau
    - dbt
  attributes:
    team: Analytics Team
    gdpr_training: false
```

## Groups

In Jetty, groups are managed at the system level, but applied across each connector individually, when applicable. For example, in Snowflake, the groups I define in Jetty will be materialized into roles and access will be granted to those roles. Groups have a name (the object key) and an optional description and can have users or other groups as members.

Groups are defined in the `/groups/` directory, and as with users, you are welcome to split your groups across any number of files in any number of sub-directories. A group object is structured as follows:

```yaml
Special Group:
  description: This is the description of the group
  managed: true # this is optional as groups are managed by default
  members:
    users:
      - elliot@gmail.com
    groups:
      - haxorz
```

Member users and groups can be defined explicitly or can also be defined using the jetty explore query syntax, for example:

```yaml
Special Group:
  description: This is the description of the group
  managed: true # this is optional as groups are managed by default
  members:
    users_functions:
      - | # this is a great way to write multi-line strings in yaml that respect leading whitespace
        for u in range(get_users()):
          if u.has_attribute({"gdpr_training": true}):
            yield u
```

:::info
Groups that exist in the target platforms but aren’t specified in Jetty generally aren’t affected, though Jetty will override policies set on those groups.
:::

Now that you understand how Jetty manages users and groups, go ahead and create a group of your own. Next, we'll talk about managing assets.
