// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/dracula');

/** @type {import('@docusaurus/types').Config} */
const config = {
    title: 'Jetty Core',
    tagline: 'Governance for the modern data stack',
    url: 'https://docs.get-jetty.com',
    baseUrl: '/',
    onBrokenLinks: 'throw',
    onBrokenMarkdownLinks: 'warn',
    favicon: 'img/favicon.ico',

    // GitHub pages deployment config.
    // If you aren't using GitHub pages, you don't need these.
    organizationName: 'jettylabs', // Usually your GitHub org/user name.
    projectName: 'jetty_docs', // Usually your repo name.

    // Even if you don't use internalization, you can use this field to set useful
    // metadata like html lang. For example, if your site is Chinese, you may want
    // to replace "en" with "zh-Hans".
    i18n: {
        defaultLocale: 'en',
        locales: ['en'],
    },

    presets: [
        [
            'classic',
            /** @type {import('@docusaurus/preset-classic').Options} */
            ({
                docs: {
                    sidebarPath: require.resolve('./sidebars.js'),
                    routeBasePath: '/',
                    // Please change this to your repo.
                    // Remove this to remove the "edit this page" links.
                    // editUrl:
                    //   'https://github.com/jettylabs/jetty_docs/tree/main/website',
                },
                blog: {
                    showReadingTime: true,
                    // Please change this to your repo.
                    // Remove this to remove the "edit this page" links.
                    // editUrl:
                    //   'https://github.com/jettylabs/jetty_docs/tree/main/website',
                },
                theme: {
                    customCss: require.resolve('./src/css/custom.css'),
                },
            }),
        ],
    ],

    themeConfig:
        /** @type {import('@docusaurus/preset-classic').ThemeConfig} */

        ({
            metadata: [{ name: 'robots', content: 'noindex' }],
            colorMode: {
                defaultMode: 'light',
                disableSwitch: true,
            },
            navbar: {
                // title: 'Jetty Labs',
                logo: {
                    alt: 'Jetty Labs Logo',
                    src: 'img/logo.png',
                    href: 'https://www.get-jetty.com',
                    target: '_self',
                },
                items: [
                    {
                        type: 'doc',
                        docId: 'getting-started/index',
                        position: 'right',
                        label: 'Documentation',
                    },
                    {
                        to: 'https://www.get-jetty.com/about',
                        label: 'About',
                        position: 'right',
                        target: '_self',
                    },
                    {
                        to: 'https://www.get-jetty.com/contact',
                        label: 'Contact',
                        position: 'right',
                        target: '_self',
                    },
                    {
                        to: 'https://www.get-jetty.com/jetty-cloud',
                        label: 'Jetty Cloud',
                        position: 'right',
                        target: '_self',
                    },
                    // { to: '#', label: 'Blog', position: 'right' },
                ],
            },
            footer: {
                style: 'dark',
                // links: [
                //     {
                //         title: 'Docs',
                //         items: [
                //             {
                //                 label: 'Tutorial',
                //                 to: '/docs/getting-started/',
                //             },
                //         ],
                //     },
                //     {
                //         title: 'Community',
                //         items: [
                //             {
                //                 label: 'Stack Overflow',
                //                 href: '#',
                //             },
                //             {
                //                 label: 'Discord',
                //                 href: '#',
                //             },
                //             {
                //                 label: 'Twitter',
                //                 href: '#',
                //             },
                //         ],
                //     },
                //     {
                //         title: 'More',
                //         items: [
                //             {
                //                 label: 'Blog',
                //                 to: '#',
                //             },
                //             {
                //                 label: 'GitHub (',
                //                 href: '#',
                //             },
                //         ],
                //     },
                // ],
                copyright: `Copyright © ${new Date().getFullYear()} Jetty Labs, Inc.`,
            },
            prism: {
                theme: lightCodeTheme,
                darkTheme: darkCodeTheme,
            },
        }),
};

module.exports = config;
