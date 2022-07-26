---
sidebar_position: 3
slug: '../installation'
---

# Create a Jetty project

Jetty lets you manage data access declaratively, in much the same way that infrastructure-as-code tools let you manage your infrastructure. In Jetty configurations are written in a Jetty project and then applied using the `jetty` commend-line tool.

## Installation

:::note

To install Jetty on your local computer, make sure that you have Python 3 and pip installed. While not required we also recommend installing Jetty in a virtual environment.

:::

To install Jetty, open your terminal and run `pip install jetty-core`.

To see if it worked, try running `jetty --version`. You should see something like this:

```
> jetty --version

jetty core 0.1.2
```

:::tip
If it's not working for you, open a new terminal window and try running `jetty --version` again. If it's still not working, check out our [troubleshooting guide](../troubleshooting.md) or reach out to us on [Slack](https://slack.com/)
:::

### 🎉🎉 Congratulations! 🎉🎉

You have successfully installed Jetty!

## Creating a new project

Now that Jetty is installed, you can create a new project. In the terminal, go to your home directory (or wherever you want to create the project) and run `jetty init tutorial-project`. This will create a new directory for the project called `tutorial-project` (you are welcome to choose any project name you'd like).

In that directory there will be several files and directories where the project configurations and infrastructure state will be stored. We'll look at these in more depth in the coming sections.

To make sure that everything worked, try running `jetty status` in the terminal. You should see something like this:

```
> jetty status

Your project is up to date

Connector             Last Fetch                Connection Test
---                   ---                       ---

No connectors are set up. Run "jetty connect" to connect to your data tools
```

If it looks like everything is working, let's connect your data tools to Jetty!
