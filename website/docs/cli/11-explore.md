# explore

This command will start a local server and open a browser window to a search page. You can use the search page to find specific users, groups, data assets, and tags, and view detailed information about them. This information can be filtered, sorted, and downloaded. The displayed information includes:

#### Users

-   User-accessible assets, including specific access privileges (_in preview_)
-   Groups the user is a member of (both directly, and through nested group relationships)
-   Tagged assets the user has access to (grouped by tag)

#### Groups

-   Directly assigned members of the Group
-   Members of the group, including those inherited via nested group relationships
-   Groups a given group is a member of (directly and through inheritance)

#### Assets

-   Tags applied to the asset
-   Users with direct asset to the asset (_in preview_)
-   Users with access to an asset derived from the original asset (for example, a dashboard built from the original table) (_in preview_)
-   Hierarchy of the asset (e.g., the projects a Tableau workbook is part of, and any sheets or metrics that are children of the workbook)
-   Lineage of the asset (e.g., the assets that the original asset is derived from and the assets then derived from it)

#### Tags

-   Tag inheritance rules (passed via lineage and/or hierarchy)
-   Assets that have the tag applied (directly or through inheritance)
-   Users with access to the tagged assets (_in preview_)

#### Answering Questions

The Explore UI can help find answers to countless questions. Here are a few example:

-   What views or dashboards have been derived from sensitive data?
    1. Search for the table that the sensitive data originates from
    1. Go to the _Lineage_ tab to see all down-stream derived assets
-   Who has access to dashboards, views, or tables, that have been derived from sensitive data?
    1. Search for the table that the sensitive data originates from
    1. Use the _Any Access_ tab to see all users with any access to that table, including via downstream assets
-   What combination of access rules has resulted in an unauthorized user having access to a sensitive data asset?
    1. Search for the user in questions
    1. Filter the _Assets_ tab to find the asset in question, and look at the permissions granted and explanations

:::note
We will continue to improve the Explore UI - if there is more you wish you could do [let us know](mailto:product@get-jetty.com)!
:::

### Usage

`jetty explore [OPTIONS]`

### Arguments

`[PROJECT_NAME]` Project name (optional)

### Options

| Flag                              | Description                                                        |
| --------------------------------- | ------------------------------------------------------------------ |
| `-f`, `--fetch`                   | Fetch the current configuration before launching the UI            |
| `-b`, `--bind` `<BIND>`           | Select the ip and port to bind the server to (e.g. 127.0.0.1:3000) |
| `-l`, `--log-level` `<LOG_LEVEL>` | Specify the log level. Can be debug, info, warn, or error          |
| `-h`, `--help`                    | Print help information                                             |
