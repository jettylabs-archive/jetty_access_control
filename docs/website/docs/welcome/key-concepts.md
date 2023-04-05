---
sidebar_position: 1
slug: '/welcome/key-concepts'
---

# Key Concepts

Jetty provides a common interface to manage data access across multiple connected platforms. The interface provides an abstraction layer that is used to generate connector-specific configurations. In this section we'll go cover the core elements of this abstraction layer.

## Connectors

Jetty interfaces with external platforms via _connectors_. Connectors are the libraries that interface between Jetty and external products' APIs. They are responsible for fetching information about the external products and translating that information into [Jetty objects](#jetty-objects). They also transform Jetty configurations back into the appropriate product-specific representation and manage any required updates.

## Jetty Objects

Jetty represents the configurations and artifacts of connected platforms as Jetty-specific objects. These abstractions make it possible to provide a single interface that works across multiple platforms. These Jetty objects include:

### Users

Users in Jetty represent a single user or identity represented across one or more connected platforms. Jetty automatically merge users across platforms, and these compound identities can be further refined in each user's configuration file. Because these identities are cross-platform, it makes it possible to, for example, add them to a group in one place and have the change implemented across your full stack.

Jetty users can belong to [groups](#groups) and can have [policies](#policies) applied to them.

### Groups

Groups represent collections of users and groups that can be managed together in Jetty. These groups are implemented by connectors in different ways (e.g., as groups in Tableau, and as Roles in Snowflake). Jetty groups can have [policies](#policies) applied to them and can be members of other groups (Jetty takes care of this, even if the connected systems don't support nested groups)

### Assets

Assets represent the artifacts managed by the connected platforms (e.g., database tables, dashboards, or projects). They can have descendants and ancestors based on hierarchy (e.g., a table is a child of a schema) and lineage (e.g., a view is derived from a table) and can have [policies](#policies) applied to them. All assets have a globally unique identifier that can be used to tie together lineage and other information between systems (e.g., importing Snowflake lineage data based on a dbt project).

:::note Hierarchy vs Lineage
Assets have two types of genealogy: hierarchy-based and lineage-based.

**Hierarchy** refers to the way in which assets are nested or structured within the connected platform. You can think of this like the folder structure on a computer. This hierarchy helps to define the canonical identifier for the asset (like the file path to a file).

**Lineage** refers to relationships between assets that are derived from each other. This helps tie together data and systems and is especially helpful for auditing workflows. For example, it is via lineage that you can identify what dashboards are built from tables containing sensitive customer data.
:::

### Policies

Policies represent the policies as configured in a connected platform. Each policy applies to a single [asset](#assets), has one or more grantees (groups and/or users, depending on what is permitted by the connected platforms), and grants zero or more privileges. These privileges are the same privileges that are managed directly in the connected platform.

:::note Policies and effective permissions
Policies in Jetty are equivalent to policies set in the connected platforms. This means that, at times, the actual level of access that a user has may not match what a specific policy defines. For example, if a user a site administrator in Tableau, they have automatic access to all of the assets, even though that access is not explicitly controlled using individual policies.

[Jetty Explore](../cli/explore) has preview functionality to show a users effective permissions (only in versions prior to 2.5), but this is still experimental. If you want to know more about the current state of this feature, please [reach out](mailto:product@get-jetty.com).
:::

#### Default Policies

Default policies are a special type of policy that is applied to downstream assets of a given type. They have grantees and privileges, but also have a _path_ and a _target type_. The path specifies the path of assets that should be matched (starting from asset where this policy is configured). An example path could be `*/**` which would skip a level of hierarchy and then be applied to all assets at the next level of hierarchy and below. The _target type_ of a default policy determines the type of asset that the policy will be applied to.

Default policies can either be applied by Jetty only (translated into regular policies for all the affected assets), or by Jetty and by the asset's connected platform. For example, in Snowflake, a "connector-managed" default policy will be converted into regular policies by Jetty _and_ into future grants by Snowflake.

:::note Policy order of operations
Because overlapping policies can exist (for example, a default policy set on an asset may conflict with a regular policy set directly on the asset), Jetty tracks the specificity of each policy and only applies the most specific. The basic order of precedence is:

1. Regular policies set directly on an asset
1. Default policies set on an asset's parent, with a path ending in `/*`
1. Default policies set on an asset's parent, with a path ending in `/**`
1. Default policies set on an asset's grandparent, with a path ending in `/*`
1. _etc._

:::
