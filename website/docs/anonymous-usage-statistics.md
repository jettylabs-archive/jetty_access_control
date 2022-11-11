---
sidebar_position: 5
slug: './coming-soon'
---

# Anonymous Usage Statistics

We at Jetty Labs want to build the product you want to use, giving you tools and the control to adapt them to your needs. Customer feedback and connecting with you are critical components of the process, and we would love to hear from you!

Jetty is brand new to the world and we want to to build the best product along the way. Allowing us to collect anonymized usage data helps us help you manage permissions.

## What we collect

Starting in version `0.1.4`, we will introduce collection of metadata about each run of Jetty CLI.

We'll collect the arguments you supplied to Jetty or whether it failed to run, your operating system, information about the Jetty version you are running, and anonymous project-level and user-level identifiers. That's it!

Over time, we plan to add slightly more granularity but we will always note changes to anonymous usage statistics with the release.

## How to opt out

Jetty is founded on the idea that privacy and data can flourish together, but ultimately you are in control of what we collect.

If you want to opt out of collection, set `allow_anonymous_usage_statistics` to `false` in your project-level `jetty_config.yaml` file.

## A nod to our forbears

Great Expectations' [excellent post](https://greatexpectations.io/blog/anonymous-usage-statistics/) about their usage statistics guided this document and our approach in general. We owe them (and [dbt](https://www.getdbt.com/) before them) for laying the groundwork for us to build on.
