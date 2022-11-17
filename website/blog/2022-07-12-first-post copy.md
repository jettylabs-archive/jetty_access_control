---
title: Hello Jetty
authors:
  - jk
tags: []
description: Hello Jetty
image: https://i.imgur.com/mErPwqL.png
hide_table_of_contents: true
---

We founded Jetty Labs to solve real problems at the intersection of data, systems, and privacy. Our CEO Isaac worked in data analytics at Lucid Software and Google and then built privacy-aware analytics tooling at a startup called DataFleets, and then LiveRamp (after an acquisition). Before founding Jetty Labs, I spent graduate school researching the intersection of augmented reality and privacy and worked as a production engineer on Meta’s AR/VR and privacy infrastructure teams. Isaac and I share a passion for building products and a belief in the importance of data privacy.

Well before starting Jetty Labs, we would talk about the data utility/data privacy dichotomy. Data is valuable, but privacy is critical; if we want the benefits of both, we as scientists and engineers and builders have to do better. A broad reaching solution will not be reached with a single tool, but by an ongoing commitment to face these conflicting interests head on. Effective data governance is a small part of what we see as that final destination in data privacy. Let me tell you how we arrived here.


### **Data, Systems, and Privacy**

In the age of generative AI, direct-to-consumer unicorns, and exceptionally good recommendation systems, data is everywhere. And it’s valuable – businesses tiny and huge count on personalized advertising to get their products in front of the people that want them. Executives rely on business intelligence to guide their decisions. Products and experiences can be tailored to each individual person that uses them. It’s never been more important to turn the data you collect into action, made possible today because of the infrastructural systems underlying computing.

I grew up with personal computing – the year I was born saw the World Wide Web, Python, and Vim all become publicly available. In the decades since then, we have built a new society on top of computer systems. Many people’s livelihood exists largely on the internet, even more so post-pandemic. We have come to directly depend on the efficiency, scalability, and redundancy of these systems in order to live and conduct business. It is intertwined in our private, social, and professional lives, meaning computers are more responsible now than before for protecting what makes us, us.

As our dependence on a computer-driven society has increased, privacy practices have come under increased scrutiny as people have become more aware of the ways in which their data can be used – with or without their consent. Society at large is intent on improving consumer privacy, and it’s starting to show up in policy around the world. GDPR and CCPA (now updated by CPRA) come to mind as early entrants, but they are not alone. Five US states have enacted GDPR-like privacy laws (California, Colorado, Connecticut, Utah, and Virginia),[^1] and in 2022 alone, 59 comprehensive consumer privacy bills were considered at the state level (compared to just 2 in 2018).[^2] On the global stage, 137 countries have implemented some sort of data privacy or protection legislation.[^3] And this is just the beginning - we are going to see more regulation as society reckons with how we define privacy and what control and safety look like and along the way, we will learn more difficult lessons like those we learned in the wake of the Cambridge Analytica scandal. It will take time to reach a state where people are both protected _and_ confident using internet tools, but we’ll get there.

As a business owner and a father, I feel an obligation to lift where I stand and improve the state of data governance and privacy more broadly. That’s why at Jetty Labs, we’re starting with full-stack tooling for data access control. Today, it can be challenging to understand and manage permissions across the stack. The seemingly endless collection of tracking spreadsheets, partially implemented data catalogs, SQL GRANT statements, and custom scripts that organizations find themselves relying on can be dizzying, and as the modern data stack continues to expand, access control is becoming more fragmented. The truth is that time spent monitoring and enforcing data access policies is not what yields the powerful business value data teams offer - too often governance is relegated to a side quest on that journey and often falls short of adequate prioritization. Too many organizations let what should be data democracy fall into data anarchy[^4] because understanding and maintaining complex permissions is not easy. We want to fix that. We want to make it enjoyable.


## **Enter Jetty**

Say hello to **Jetty**, our free CLI tool to give you more visibility into access control across your stack. Jetty makes it easy to answer questions like “What dashboards have been derived from sensitive data?” “Who are all the people that have access to the users table?” or “Why does this former employee still have access to the payments view?”


### **Data Teams and Access Control**

Jetty is built for data teams from the ground up. We know that data practitioners want to focus on delivering meaningful insights and driving marketing, operations, and product decisions. Jetty gives companies more time to focus on growing the business by streamlining data access control, helping data teams speak confidently about who has access to data and explain the policies behind that access.

Controlling access today across the data stack is labor-intensive and error-prone. Jetty helps users understand access policies and their effects across their data tools.


### **Developing Jetty**

Jetty is fast and flexible; to achieve this, we chose to write it in Rust, rather than Python or a JVM language. We have loved Rust’s elegant, powerful syntax, and have found that it offers a level of performance and safety that isn’t available in many of the more obvious or familiar candidates. If you have experience shipping Rust software in the real world, please reach out! We are hiring data-oriented engineers to help us build amazing data tooling.

We have also built Jetty to work across the data stack from the very beginning. With our first release, we’ve integrated with the APIs of three foundational data tools: Snowflake, dbt, and Tableau. This enables data teams to see and understand data permissions throughout the lifecycle of the data, from the warehouse, through transformations, and into BI tools. Our plan is to expand the footprint of platforms that we connect to until we can provide comprehensive data access management for any team’s stack.


### **Privacy**

We believe Jetty can make privacy protection a pleasure. Finding out who can access what data shouldn’t require SQL queries and manually-maintained spreadsheets. Jetty lets you fetch permissions and audit them together, and cross-stack understanding means that you can view the holistic picture of a person or group’s access all from a single tool.

With Jetty, you can navigate through your stack by user, group, tag, or asset, using asset hierarchy and lineage provided by your native tools. You can view all people who have access to an asset, and what policies have granted that access. You can even see users that have access to data derived from the original asset and users granted access through nested groups.

Tagging data with attributes such as “PII” lets you identify semantic access patterns without being constrained to the shape of your data. When the CISO asks “who has access to PII'' Jetty lets you answer with confidence. And because of the lightweight nature of Jetty, you can orchestrate regular updates so that you always have the most up-to-date information.


### Getting Started with Jetty

If any of this is intriguing to you try it out! Getting started with Jetty is as simple as running `pip install jetty-core`, which will install Jetty on your local machine and give you access to the Jetty CLI.

To create a Jetty project and connect to your data stack, run `jetty init`. After you finish the configuration walkthrough, you can fetch and explore your permissions by running `jetty explore --fetch`. That’s it!

For more information about getting started with Jetty, check out our documentation at https://docs.get-jetty.com.


### **What’s next?**

Today, Jetty gives data teams x-ray vision into the data permissions across their stack, but we are just getting started. The real challenge is actually configuring those permissions across the stack, and soon you’ll be able to use Jetty to do just that. If you’re interested in following along with our progress, [give us a shout](https://www.get-jetty.com/contact)! We’d love to hear from you.

And so this is a continuation of that original conversation Isaac and I have been having. Can we bring value to people through the data they provide and earn their trust along the way? We’re taking steps to make this easier, but there's still so much to do. We’re up to the challenge.



\- Jk

<!-- Footnotes themselves at the bottom. -->
## Notes

[^1]:
     https://iapp.org/resources/article/us-state-privacy-legislation-tracker/ 

[^2]:
     https://iapp.org/resources/article/the-growth-of-state-privacy-legislation-infographic/ 

[^3]:
     https://unctad.org/page/data-protection-and-privacy-legislation-worldwide 

[^4]:
     We first heard this framed as “data anarchy” by Nick Hudson of Cairn Analytics 
