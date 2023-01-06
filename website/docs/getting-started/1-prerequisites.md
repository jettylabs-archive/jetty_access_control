# Prerequisites

Below is the information you will need to connect to each connected platform. As a reminder, you don't need to use all of the connectors, but the more connectors you use, the more helpful Jetty can be.

<details>
  <summary><strong>Snowflake</strong></summary>
  <div>
    <p>To read the relevant metadata from Snowflake, Jetty needs to a user with a role that is able to read account metadata and use a warehouse. You can create a custom role with these permissions - we recommend following DataHub's <a href="https://datahubproject.io/docs/generated/ingestion/sources/snowflake#prerequisites">excellent documentation</a> to set this up.</p> 
    <p>If you would like to manage group membership and permissions (recommended), you will need the <code>SECURITYADMIN</code> role.</p>
    <p>To make setup easy, be ready with the following:</p>
    <ol>
      <li>Your Snowflake account identifier. This is the part of your Snowflake URL before <code>.snowflakecomputing.com</code> (it could be something like <code>cfa39421</code> or <code>xm82504.europe-west4.gcp</code>). See <a href="https://tinyurl.com/snow-account-id">the documentation</a> for more.</li>
      <li>The name of the Snowflake user you would like Jetty to use. We recommend creating a <a href="https://docs.snowflake.com/en/sql-reference/sql/create-user.html">new user</a> specifically for Jetty.</li>
      <li>The name of the Snowflake role you would like to use.</li>
      <li>The name of a warehouse your Jetty user has <code>USAGE</code> and <code>OPERATE</code> privileges on.</li>
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
    <p>Jetty uses dbt as a source for in-Snowflake lineage data. For this to work, Jetty needs to read metadata from your dbt project.</p>
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
      <strong>Note:</strong> Today Jetty only supports dbt Core projects. Please <a href="mailto:product@get-jetty.com">let us know</a> if you would like
      to use Jetty with dbt Cloud.
    </p>
  </div>
</details>

<details>
  <summary><strong>Tableau</strong></summary>
  <div>
    <p>To read and write the relevant metadata from Tableau, Jetty needs credentials to an account with at least a Site Administrator Explorer role.</p>
    <p>To make setup easy, be ready with the following:</p>
    <ol>
      <li>Your Tableau URL (something like <code>fs.online.tableau.com</code>).</li>
      <li>Your Tableau site name.</li>
      <li>A username and password or a personal access token name and secret for a user with the necessary permissions. A personal access token is the only supported authentication method if you use MFA. You can read more about personal access tokens <a href="https://help.tableau.com/current/pro/desktop/en-us/useracct.htm#create-and-revoke-personal-access-tokens">here</a>.</li>
    </ol>
  </div>
</details>

Now let's go ahead and start using Jetty!
