---
sidebar_position: 7
slug: './users-and-groups'
---

# Manage users and groups

## Users

## Groups

In Jetty, groups are managed at the system level, but applied across each connector individually, when applicable. For example, in Snowflake, the groups I define in Jetty will be materialized into roles and access will be granted to those roles. Groups have a name and descirption and can have users or other groups as members.

Groups are defined in the `/groups/` directory. When you bootstrap the system, a `groups.yaml` file will be created in that directory, but you are welcome to split your groups across any number of files in any number of sub-directories. A group object is structured as follows:

```yaml
name: NameOfGroup
description: This is the description of the group
managed: true # this is optional as groups are managed by default
members:
	users:
		- elliot@gmail.com
	groups:
		- haxorz
```

Users and groups can be defined explicitly or can also be defined using the jetty explore query syntax, for example:

```yaml
name: NameOfGroup
description: This is the description of the group
managed: true # this is optional as groups are managed by default
members:
  users_functions:
    - | # this is a great way to write multi-line strings in yaml that respect leading whitespace
      for u in range(get_users()):
        if u.has_attribute("Analytics Team"):
          yield u
```

:::info
Groups that exist in the target platforms but aren’t specified in Jetty generally aren’t affected, though Jetty will override policies set on those groups.
:::
