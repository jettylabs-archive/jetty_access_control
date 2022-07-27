---
sidebar_position: 5
slug: "./bootstrap"
---

# Bootstrap your project

Systems like Snowflake and Tableau already have some groups or roles and access policies configured by default. You may have added more controls on top of that already. Bootstrapping your project takes these existing permissions and turns them into configuration files that can serve as the base for your system configuration.

Once connectors are all configured, all we need to do is run `jetty bootstrap`.
:::tip
If you add or update a connector, you can bootstrap just that connector by with `jetty bootstrap <connector namespace>` (e.g., `jetty bootstrap snow`)
:::

If you have edited any configuration files, the system will ask if you would like to back up your existing configuration.

```
>jetty bootstrap

Would you like to back up your existing configuration files?(Y/n)
```

:::tip
You can skip the prompt and automatically back up existing configuration files by running `jetty bootstrap --backup`. You can turn on backup by default by adding `backup: true` in the Jetty config.
:::

Once you confirm/reject the config backup, Jetty will begin fetching the relevant information from all of your connectors. The time this takes will depend on the connectors you're using, but in some cases can take a minute or more. Once the process is complete, your config files will have been updated to reflect the current state of your data infrastructure. Below is an explanation of the project files and their contents:

**jetty-config.yaml** - This file contains information about your project, including connectors, and isn't modified by `jetty bootstrap`

**taxonomy.yaml** - This file lists all data tags and attributes present in your system (for example, Snowflake or Tableau tags). Jetty starts from a pre-defined hierarchy of tags, and tries to put your existing tags into the right place in that hierarchy (for example, `email address` should be a child tag to `PII`). This hierarchy allows you to write policies that can apply to across groups of similar data classes and across the tools in your stack.

**assets/assets.yaml** - This file is populated with all the assets that have had tags applied directly to them in their source systems (it doesn't include inherited tags from Snowflake, for example). Only the tagged assets are shown so that the file can provide useful information without overwhelming the user. Some organizations have hundreds of thousands of assets, and displaying them all in a yaml file would not be particularly valuable. Assets that don't appear in the file can still be referenced in policies, and can be added to the assets file without any negative side-effects. You may also notice that assets are specified as `managed: false`. This means that by default, Jetty provides you insight into your controls but doesn't manage them itself. Again, the goal is to allow you to adopt Jetty as gradually as you need.

It is in this file that tags and custom lineage can be applied to assets. We'll discuss all of this later in the tutorial.

:::info
As your Jetty configs grow, assets.yaml will get quite large. To learn how we recommend splitting it up and structuring your assets directory, see [Managing a Large Jetty Project](#).
:::

**users/users.yaml** - This file serves as a reference of all of the users in your different data tools, along with properties to show which tools they exist in. User provisioning isn't yet managed by Jetty, so changes to this file will not add users to or remove users from a particular tool. In this file, however, you can add metadata attributes about users.

**groups/groups.yaml** - Groups are a Jetty abstraction that can mean different things in different tools. For example, in Snowflake, Jetty groups are equivalent to roles. The `groups.yaml` file is populated with all of the existing groups and their membership (users or other groups) from all platforms that leverage this abstraction. If you look at this file, you'll notice that by default, each role is has an attribute `managed: false` and specifies that it is only applied one connector `connectors: include: -snow`. `managed: false` means that while Jetty will observe the membership of these groups, it will not enforce them. If a user is added to a Snowflake role by Okta, for example, Jetty won’t try to manage membership based on this file and kick them back out. This lets you gradually begin managing groups in Jetty by removing the `managed: false` attribute for each group only when you are ready.

**policies/policies.yaml** - This file is populated with the access controls that exist within your data tools. Jetty’s permissions are structured as objects with an `agent` (who is the policy applied to), a `target` (the assets the policy applies to), and a `scope` (the level of access granted). The basic scopes are `deny`, `none`, `metadata`, `read`, and `write`, each one granting more access than the previous. There is also a `connector_scope` field that allows a JSON object to specify connector-specific permissions. Jetty management of these permissions is limited, but it allows users to apply custom, high-granularity permissions in a native tool and see that reflected in Jetty.

**/.state/** - This directory contains a metadata snapshot of your Jetty ecosystem, including details that aren’t shown in the configuration files (for example, all assets, not just the tagged ones are represented here). This metadata can be accessed using `jetty explore` (we'll get there next).
