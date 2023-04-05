# Jetty Documentation Site

The docs site is generated using [Docusaurus](https://docusaurus.io/)

## Development

### Prerequisites

-   A working version of Nodejs

### Process

To run the development server, change into the `website/` directory and run

```bash
npm install
npm run start
```

You can skip `npm install` if you've already installed the necessary dependencies

## Deployment

### Prerequisites

-   The Firebase CLI (`npm install -g firebase-tools`)

### Process

To deploy the site, install dependencies if needed and then, from the `website/` directory, build the site with

```bash
npm run build
```

Log into Firebase (if not already logged in):

```bash
firebase login
```

From the project root directory (where this README is), deploy site to Firebase with:

```bash
firebase deploy
```

Enjoy!
