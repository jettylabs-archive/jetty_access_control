---
sidebar_position: 11
slug: "./test"
---

# Test configurations

Now that you can manage access policies with Jetty, it's important to make sure that your policies lead to the expected results. `jetty explore` and `jetty plan` can both help show the changes that your configuration will make. In addition to these tools, Jetty also allows users to create automated tests that will check to make sure that none of your assumptions are violated before applying any configuration changes.

Tests should be saved in the `test/` directory, and are comprised of a name, description and explore query that evaluates to true or false. Any result other than true will lead to a failed test.

For example, to make sure that nobody in the `haxorz` group has access to PII, you can write the following test:

```yaml
“Haxors PII Test”:
	description: Make sure that haxorz can’t access our PII data
	query: |
    violations = set()
    for user in range(Group("haxorz").get_users()):
      for asset in range(user.accessible_assets()):
        if asset.has_tag("PII"):
          violations.add(user)
          print(f"Violation: {user.name()} can access {asset.name()} \n")
    return len(users) == 0

```

You can run tests manually at any time by running `jetty test` inside your project, and all tests will be evaluated whenever you run `jetty plan`. Finally, all tests must pass before `jetty apply`.

:::tip
If you want to run a single test, you can just add the test name after `jetty test` (e.g., `jetty test "Haxors PII Test"`)
:::

# What's next?

**🎉🎉 Congratulations! 🎉🎉** You have finished the getting started tutorial. Feel free to explore the rest of the documentation to learn more about how Jetty can help you manage data access at your organization, and don't hesitate to connect with the Jetty team on [Slack](#).
