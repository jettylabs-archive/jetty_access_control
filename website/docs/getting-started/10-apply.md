---
sidebar_position: 10
slug: './apply'
---

# Apply configurations

## Seeing planned changes

Once users, groups, tags, and policies are defined, you can see the changes that will be applied by opening your terminal and running `jetty plan`. `jetty plan` refreshes the known state of your data infrastructure, and then plans how to materialize your updated configurations into your infrastructure.

The output of this command shows what changes will happen in the system when the configuration is applied. It also leverages usage data (when made available by connectors) to warn you about users who will lose access to assets they have recently accessed. This will help you make sure that you don't accidentally revoke access to critical assets.

Because it fetches the current state of your data infrastructure, `jetty plan` can also help identify when real-world configurations have drifted from Jetty-defined configurations.

:::tip
To fetch the current state without actually creating a plan, you can run `jetty fetch`.
:::

## Applying configurations

:::note
If you are using Jetty's demo connectors, the apply step will fail. That's ok. It will work when you're using your real data tooling!
:::

Once you're happy with the how your configurations will be materialized in your infrastructure, you can run `jetty apply` to apply the changes specified in your configuration file. The apply process can use a recently created plan (from a recent run of `jetty plan`), but if needed, it will just generate a plan automatically.

If there is an error, the system will retry, and if unresolved, report the error state to the user.

**🎉🎉 Congratulations! 🎉🎉** You are now managing your data access policies in Jetty!

In the next section we'll talk about how you can use Jetty to test your configurations.
