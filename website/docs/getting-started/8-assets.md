---
sidebar_position: 8
slug: './assets'
---

# Manage assets

In Jetty, data to which access can be managed, such as databases, schemas, tables, columns, dashboards, BI projects, etc. are all considered assets. The `assets/` project directory is where assets can be configured. In particular, assets can have tags applied and lineage updated. An asset configuration object looks like this:

```yaml
name: snow.analytics.raw.greenhouse_recruiting_xf
description: This is an optional description of the asset
tags:
	- pii
override_derived_from:
  - snow.analytcs.raw.greenhouse_recruiting_pipeline
```

## Tagging assets

As with groups, Jetty manages tags across multiple tools, but when appropriate applies them in the individual tools. Tags are applied to assets, and are automatically inherited through asset hierarchy and through data lineage. For example, a tag can be applied to a database table, and thanks to hierarchical inheritance, every column of that table will have the tag applied too. Thanks to derived asset inheritance, any other asset that Jetty can detect is built using data from this table will have the same tag applied.

There are times, however, when we might want to break this chain of inheritance. This can be done with a strip_tags object:

```yaml
name: snowflake.analytics.raw.greenhouse_recruiting_masked
tags:
	- masked-pii
strip_tags:
	- pii
```

The inheritance of tags can also be controlled on a tag-by-tag basis with tags_no_inherit, tags_no_lineage, and tags_no_hierarchy objects:

```yaml
name: snowflake.analytics.raw.greenhouse_recruiting_xf
description:
tags_no_inherit:
	- masked-pii
```

## Managing Asset Lineage

Jetty tries to detect asset lineage automatically, but in some cases it will miss something. In that case, you can override Jetty's automatic lineage graph for an asset with the `override_derived_from` property. This property expects a list of assets that are direct parents of the current asset.

Now that you understand how tags and lineage are managed, try updating some of the tags and lineage in the `assets/assets.yaml` file, and then use `jetty explore` to see how these changes affect downstream assets. For example, you could add a new tag, `hr_data` to the `snow.analytics.raw.greenhouse_recruiting_xf` table, and then see how that table shows up when you run:

```python
return Tag("hr_data").get_assets()
```

:::tip
When you ran this you might have noticed that explore provided two results - one for the current state of your infrastructure, and one for the current state of your configuration files. This is one of the ways to look at the effects that a config change will have.
:::

Can you see any derived assets that also inherited that tag?

Next we'll talk about policies.
