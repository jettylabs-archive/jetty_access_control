# Getting Started with Jetty

## Welcome

In this guide, you will learn everything you need to know to get started with Jetty. By the end of this guide you will know how to:

1.  Install Jetty
1.  Connect to your existing data tools
1.  Explore data access permissions across your stack
1.  Configure Tags to understand data access across platforms

:::note

In this walkthrough, we will connect to Snowflake, dbt, and Tableau. We will continue to add support for additional data tools, so please [let us know](mailto:support@get-jetty.com) if this stack doesn't meet your needs!

:::

## A use case

Jetty makes it easy to understand who has access to what data, all the way across your data stack.

For example, imagine that you have sensitive business or customer data in your warehouse. This data is transformed via dbt and accessed via Tableau dashboards by executive stakeholders, providing critical insights into the business. Due to the sensitive nature of the data, however, it is critical to understand who has what level of access to this data.

Jetty provides a single platform to help you understand permissions and access across your stack, allowing you to answer questions like:

-   What views have been derived from sensitive data?
-   What dashboards leverage data derived sensitive tables?
-   Who has access to dashboards, views, or tables, that have been derived from sensitive data?
-   Why doesn't a key stakeholder have access to the information they need?
-   What combination of access rules has resulted in an unauthorized user having access to a sensitive data asset?

In a few minutes you'll be a few mouse clicks away from the answers to these and many other questions!

## Prerequisites

Below is the information you will need to connect to each connected platform. As a reminder, you don't need to use all of the connectors, but the more connectors you use, the more helpful Jetty can be.

<details>
  <summary><strong>Snowflake</strong></summary>
  <div>
    <p>To read the relevant metadata from Snowflake, Jetty needs to use an account with the <code>SECURITYADMIN</code> role and usage permissions on a warehouse.</p>
    <p>To make setup easy, be ready with the following:</p>
    <ol>
      <li>Your Snowflake account identifier. This is easiest to get in SQL with <code>SELECT current_account();</code>. This field can be the account locator (like <code>cea29483</code>) or org account name, dash-separated (like <code>MRLDK-ESA98348</code>) See <a href="https://tinyurl.com/snow-account-id">the documentation</a> for more.</li>
      <li>The name of the Snowflake user you would like Jetty to use. We recommend creating a <a href="https://docs.snowflake.com/en/sql-reference/sql/create-user.html">new user</a> specifically for Jetty.</li>
      <li>The name of a warehouse your Jetty user has the <code>USAGE</code> privilege on.</li>
    </ol>
  </div>
</details>

<details>
  <summary><strong>dbt</strong></summary>
  <div>
    <p>
      <strong>Note:</strong> A Snowflake connector must also be configured in order to connect to dbt.
    </p>
    <hr />
    <p>Jetty uses dbt as a source for in-Snowflake lineage data. For this to work, Jetty needs access to your dbt project.</p>
    <p>To make setup easy, be ready with the following:</p>
    <ol>
      <li>The path to your dbt project</li>
      <li>Your Snowflake account identifier. This helps link your dbt project to the right Snowflake account.</li>
    </ol>
    <p>Once Jetty can access your dbt project, it will check for the <code>target/manifest.json</code> file, and if it can't find one, ask you to generate one with <code>dbt docs generate</code>.</p>
    <p>
        You can read more about setting up a dbt project with Snowflake <a href="https://docs.getdbt.com/docs/get-started/getting-started/getting-set-up/setting-up-snowflake">here</a>.
    </p>
    <hr />
    <p>
      <strong>Note:</strong> Today Jetty only supports dbt Core projects. Please <a href="mailto:support@get-jetty.com">let us know</a> if you would like
      to use Jetty with dbt Cloud.
    </p>
  </div>
</details>

<details>
  <summary><strong>Tableau</strong></summary>
  <div>
    <p>To read the relevant metadata from Tableau, Jetty needs credentials to an account with account or site admin privileges.</p>
    <p>To make setup easy, be ready with the following:</p>
    <ol>
      <li>Your Tableau URL (something like <code>fs.online.tableau.com</code>).</li>
      <li>Your Tableau site name.</li>
      <li>The username of the user Jetty will use to connect.</li>
      <li>The password of the user Jetty will use to connect.</li>
    </ol>
  </div>
</details>

Now let's go ahead and start using Jetty!
