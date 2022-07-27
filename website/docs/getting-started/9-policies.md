---
sidebar_position: 9
slug: './policies'
---

# Manage policies

In Jetty, policies are mechanism we use to actually grant (or revoke) data access. Policies are found in the `policies/` directory and can be split across as many files and subdirectories as you would like. The policies detected by `jetty bootstrap` are all stored in `policies/policies.yaml`. For the policies that already existed in, you may notice the `connectors` attribute (something like `connectors: include: -snowflake`). When we detect existing policies, those policies should only apply to the system in which they were detected. The `connectors` property makes sure of that.

Jetty policies fall into two broad categories: **allow** policies and **deny** policies. By default, users do not have access to any assets unless explicit policies are defined.

## Allow policies

Within Allow policies, there are three levels of access, or scope: metadata, read, and write.
Metadata access allows users to see the existence of assets within Jetty or other data governance tools. Read access allows users to read the actual data, and write access allows users to modify the data.

Allow policies must be defined on assets (and optionally tags) and are inherited only hierarchically. This means that if I explicitly grant access to a schema in Snowflake, access is also granted to the tables in the schema (this is hierarchical inheritance). Access is not automatically granted to derived assets (that would be derived inheritance). The inheritance of policies can be disabled by setting the `inherit` field to false.

Another thing to note is that allow policies are applied only in the platform in which the original asset exists. That means, for example, that a user having read access to a table in Snowflake doesn't mean that they will automatically have access to every dashboard in Tableau that uses that table. An allow policy in Tableau would be required to grant access to the dashboards.

An allow policy might look like this:

## Deny policies

Deny policies are behave slightly different than allow policies. They can be defined on assets and/or tags, and are inherited by downstream hierarchical _and_ derived assets. That means that if I deny access to a schema, access will be denied to the tables in the schema as well as to any derivatives of those tables. Additionally, deny policies _are_ automatically applied across platforms.

When tags are used to define policies, they can be used across all levels of the taxonomy hierarchy (defined in taxonomy.yaml). For example, a policy could be applied to the `PII` tag, which would include `Phone Number` as a child tag, or it could be applied directly on `Phone Number`.

A deny policy might look like this:

## Policy conflicts

In the case where multiple policies conflict, by default, the most specific policy will win out, with tags being classified as more specific than assets. Ties going to the Deny policy or the most permissive Allow policy. This might sound counter-intuitive, so we’ll clarify with some examples:
Policy 1: User A has write access to schema_1.table_b
Policy 2: User A is denied access to schema_1
Result: The table specification is more specific than the schema permission, so the user does have write access to table_b (Policy 1)
Policy 1: User A has write access to schema_1.table_b
Policy 2: User A is denied access to schema_1.table_b
Result: The specificity between the policies is equivalent, so the deny policy is applied
Policy 1: User A has write access to schema_1.table_b
Policy 2: User A is denied access to the PII tag, which has been applied to schema_1 (and its children)
Result: The tag specificity is applied as more specific than the asset specificity, so User A does not have access to table_b
Policy 1: User A has write access to schema_1.table_b with include_tags: [“PII”]
Policy 2: User A is denied access to the PII tag, which has been applied to schema_1 (and its children)
Result: The joint table & tag specification is more specific than the tag specification alone so the user is given write access to schema_1.table_b. They are still blocked from PII data elsewhere.
Policy 1: User A has write access to schema_1.table_b
Policy 2: User A has read access to schema_1.table_b
Result: In this case there are conflicting Allow permissions, and so the most permissive permission persists (write). This helps in cases where a user could be a part of several groups, each with escalating permissions.

:::info
In the case that a policy set natively in a a tool cannot be represented entirely in Jetty yet (for example, Snowflake row-access policy are not yet managed by Jetty), that policy still shows up, but has the attribute `managed: false` and includes a representation of the policy in the `connector_scope` field.
:::
