---
sidebar_position: 5
slug: './anonymous-statistics'
---

# Anonymous Usage Statistics

It's pretty simple - we want to build a product you want to use. One of the tools we use to justify our investment in Jetty and to define our product direction is anonymous usage information collected from Jetty installations.

## What we collect

We collect information including what Jetty CLI commands are run, whether Jetty failed due to an internal error/bug, and basic information such as your operating system and the types of connectors that you use. This information is associated with randomly generated identifiers to help us understand usage patterns over time.

Here's an example of what we collect:

```json
{
    "environment": "dev",
    "event": {
        "name": "invoked_fetch",
        "properties": {
            "connector_types": [
                "dbt",
                "snowflake",
                "tableau"
            ]
        }
    }
    "jetty_version": "0.1.0",
    "platform": "mac",
    "project_id": "8c48dc5c-7762-46f3-bc84-524bc85ef5a8",
    "schema_version": "0.0.1",
    "created": "2022-11-11T00:16:03.399191000Z",
    "user_id": "e677e873-2870-49c5-bc28-c1a8be520465"
}
```

## What we don't collect

We do not collect any personally identifying information such as names, email addresses, or IP addresses.

## How to opt out

Jetty is founded on the idea that privacy and data can flourish together, but ultimately you are in control of what we collect.

If you want to opt out of collection, set `allow_anonymous_usage_statistics` to `false` in your project-level `jetty_config.yaml` file.

## A nod to our forebears

Great Expectations' [excellent post](https://greatexpectations.io/blog/anonymous-usage-statistics/) about their usage statistics guided this document and our approach in general. We owe them (and [dbt](https://www.getdbt.com/) before them) for laying the groundwork for us to build on.

## Contact us

User feedback is a critical part of our development process, and we would love to hear from you! Please feel free to [reach out to us](mailto:product@get-jetty.com) with any questions, suggestions, or concerns.
