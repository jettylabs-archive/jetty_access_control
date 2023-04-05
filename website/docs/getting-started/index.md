# Getting Started with Jetty

## Welcome

In this guide, you will learn everything you need to know to get started with Jetty. In the next 15 minutes, you will learn how to:

1.  Install Jetty
1.  Connect to your existing data tools
1.  Automatically generate configuration files based on your new current data access controls
1.  Edit group membership and data access policy configurations
1.  Plan and (optionally) apply the changes in your environment

:::note

In this walkthrough, we will connect to Snowflake, dbt, and Tableau. We will continue to add support for additional data tools, so please [let us know](mailto:product@get-jetty.com) if this stack doesn't meet your needs!

:::

## Why Jetty?

Traditionally, access management for the data stack has meant countless SQL _GRANT_ statements and clicking through error-prone UIs, typically done without any sort of version control and with minimal oversight.

Jetty brings configurations from different tools together into a single platform and lets you mange your permissions through a simple, auditable, version-controlled process. By centralizing your access management, Jetty can also help you answer critical questions like:

-   What views have been derived from sensitive data?
-   What dashboards leverage data derived sensitive tables?
-   Who has access to dashboards, views, or tables, that have been derived from sensitive data?
-   Why doesn't a key stakeholder have access to the information they need?

Let's get started!
