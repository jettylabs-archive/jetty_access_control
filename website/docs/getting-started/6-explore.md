---
sidebar_position: 6
slug: './explore'
---

# Explore your environment

Because Jetty connects to all your data tools, it can also give you visibility into access to those tools through a single interface, surfacing information about data assets, tags, groups, and users.

To begin exploring your data access environment with Jetty, open your terminal and run `jetty explore`. This will open an interactive prompt you can use to write explore functions. Explore functions look like standard Python code and support a [subset of the Python language](#). They can return or yield values that will be shown in the prompt.

The best way to get familiar with the explore command is to try it out, so below are some queries you can run on the data in the demo system. These exact queries will work if you're using our dummy connectors, but they may have to be modified if you're using your own data.

#### Who has access to the greenhouse_recruiting_xf table?

```python
return Asset("greenhouse_recruiting_xf").users_with_access()
```

#### Who can access any Snowflake data?

```python
for u in range(get_users()):
  if u.can_access_any(Connector("snow").get_assets()):
    yield u.get_email()
```

OR

```python
users = set()
for asset in range(Connector("snow").get_assets()):
  for user in range(asset.users_with_access()):
    users.add(user)
return users
```

#### What are all the assets that Elliot has access to?

```python
return User("elliot@haxorz.com").accessible_assets()
```

#### Who has access to data tagged “Phone Number”

```python
users = set()
for asset in range(Tag("Phone Number").get_assets()):
  for user in range(asset.users_with_access()):
    users.add(user)
return users
```

#### What kind of policies do we have set for phone numbers?

```python
return Tag("PII:Phone Number").get_policies()
```

#### What are all the tags that exist in the system?

```python
return get_tags()
```

#### Who is in the haxorz group and has access to PII data?

```python
users = set()
for user in range(Group("haxorz").get_users()):
  for asset in range(user.accessible_assets()):
    if asset.has_tag("PII:Phone Number"):
      users.add(user)
return users
```

#### What assets are derived from greenhouse_recruiting_xf?

```python
return Asset("greenhouse_recruiting_xf").derived_assets()
```

Now try writing some of your own queries (you can use the [Explore query documentation](#) for help)

:::tip
When you find useful queries, `jetty explore “<query text>”` to run that single query without opening the interactive shell. This can be especially useful for generating compliance reports. You can also persist useful queries in `.jt` files that can be run later with `jetty explore <filename>.jt`.
:::

## Audit data access with `explain_access()`

In addition to the `jetty explore` capabilities you have already seen, `explore` has an `explain_access()` function designed to describe why a user can or can’t access a particular data asset, highlighting specific configurations that affect the the final materialized access controls. `explain_access()` accepts two parameters: a user and an asset (or list of assets).

For example, if I want to understand why `elliot@gmail.com` does or doesn't have access to the asset `snow.analytics.raw` (a Snowflake schema), I can run:

```python
explain_access("elliot@gmail.com", "snow.analytics.raw")
```

In this case, the output describes that Elliot has partial access to the asset because he has access to some of the child assets. It also lists the policies that lead to that access.

If Elliot didn't have access to the schema, it would also explain why.

Now that you can explore your data, you are ready to learn more about managing users and groups.
