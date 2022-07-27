---
sidebar_position: 4
slug: "./connectors"
---

# Connect your tools

## Connectors

Jetty connects to your tools using external connectors. There are generally two types of connectors: policy connectors and metadata connectors.

You can think of **Policy connectors** as read/write connectors. They allow Jetty to read metadata from the source system and manage data access policies. Snowflake and Tableau both use policy connectors.

**Metadata connectors**, on the other hand, can be thought of as read-only connectors. They provide metadata to the system, but they don't allow Jetty to directly manage any access policies. dbt uses a metadata connector because it provides valuable information to inform your policies, but it doesn't actually let Jetty manage data access permissions directly within dbt.

In this tutorial you'll use the Snowflake, dbt, and Tableau connectors. You can see all available Jetty connectors in the [Connector](../connectors/index.md) section of the documentation.

:::tip
Is a connector missing for your stack? Learn about writing your own connector by checking out our [developer documentation](#) or reach out on [Slack](https://www.slack.com).
:::

## Connecting to Snowflake with the CLI

:::info
For the connector to work, you must log in to Snowflake with a user with the `SECURITYADMIN` role. We recommend making a Jetty-specific user. Read [this](#) for help configuring your Jetty user.
:::

The Snowflake connector provides user, role, asset, and policy metadata to Jetty, and also allows Jetty to set policies itself. To connect to Snowflake, open your terminal, navigate to the project directory and run `jetty connect snowflake` then follow the prompts to set up the connector.
:::tip
You can also just run `jetty connect` without specifying a connector. In that case, your first prompt will ask you to choose from the available connectors.
:::

```buttonless
What should the namespace for this connector be? (snow)
>
```

:::tip
A connectors namespace will help you refer to its assets more easily throughout the configuration process.
:::

```buttonless
What is your Snowflake account identifier (typically something like https://acme-marketing-test-account.snowflakecomputing.com)?
>
```

:::tip
To connect to our demo Snowflake endpoint, enter `demo`. If you go this route, the rest of the steps will be skipped.
:::

```buttonless
Authenticate with
> Oauth (recommended)
  Username & Password
```

Follow the authentication prompts (specific to your authentication method) and after a quick connection check, Snowflake should be connected!

:::tip
`jetty connect` sets up your connection by downloading the relevant connector library and adding an entry to the `jetty-config.yaml` file. You can also install connectors (e.g. `pip install jetty-core-snowflake`) and add connections to the file manually (don't worry, it won't mess anything up).
:::

If you run `jetty status` you should now see your new connector listed:

```buttonless
> jetty status

Your project is out of date

Connector             Last Fetch                Connection Test
snowflake (demo)      ---                       ✅

One or more connectors haven't been fetched in a while. Run "jetty fetch" to fetch from all connectors.
```

## Connecting to dbt Core with the CLI

The dbt connector improves the Jetty experience by providing data lineage and description metadata. As you might have guessed, you can set up a connection to dbt Core with `jetty connect dbt-core`. For dbt Core, the configuration is pretty simple. You just set a namespace and the path to your dbt project.

```buttonless
What should the namespace for this connector be? (dbt)
>
```

```buttonless
What is the path to your dbt project (this can be a file path or a github url)?
>
```

:::tip reminder
To connect to our demo dbt project, enter `demo`.
:::

After a quick connection test, dbt should now be connected!

:::info
The dbt connector needs access to the files generated after dbt is run. If those aren't available, the connection test will fail.
:::

Running `jetty status` should now show the dbt connector as well:

```buttonless
> jetty status

Your project is out of date

Connector             Last Fetch                Connection Test
dbt-core (demo)       ---                       ✅
snowflake (demo)      ---                       ✅

One or more connectors haven't been fetched in a while. Run "jetty fetch" to fetch from all connectors.
```

## Connecting to Tableau Server with the CLI

The Tableau Server connector provides metadata about users, groups, projects, and workbooks, and also allows Jetty to manage access policies. You can set up your connection by running `jetty connect tableau-server` and following the (now familiar) prompts.

```
What should the namespace for this connector be? (tableau)
>
```

```
What is your Tableau Server address?
>
```

:::tip reminder
To connect to our demo Tableau Server endpoint, enter `demo`. If you go this route, the rest of the steps will be skipped.
:::

```
Enter your Username
>
```

```
Enter your Password
>
```

After a quick connection check, Tableau Server should be connected!

If you run `jetty status` you should now see all three of our connectors listed:

```
> jetty status

Your project is out of date

Connector             Last Fetch                Connection Test
dbt-core (demo)       ---                       ✅
snowflake (demo)      ---                       ✅
tableau-server (demo) ---                       ✅

One or more connectors haven't been fetched in a while. Run "jetty fetch" to fetch from all connectors.
```

With our connectors configured, you can now bootstrap your environment!
