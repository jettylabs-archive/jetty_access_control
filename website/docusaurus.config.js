// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

const lightCodeTheme = require('prism-react-renderer/themes/github');
const darkCodeTheme = require('prism-react-renderer/themes/paleNight');

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

    plugins: [
        [
            require.resolve('docusaurus-gtm-plugin'),
            {
                id: 'GTM-NQ5FTQS', // GTM Container ID
            },
        ],
    ],
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
            announcementBar: {
                id: 'support_us',
                content: `<span class="custom-announcement"> If any of this looks interesting, we'd love to hear your thoughts!!<span class="custom-announcement-subtitle"><br><a target='_blank' rel='noopener noreferrer' id="survey-link" href='https://forms.gle/NHtvVYREAsV3gJsC9'>Let us know what you think</a> and enter to win an Amazon gift card 🎉🎉</span></span>`,
                backgroundColor: '#f47124',
                textColor: 'white',
                isCloseable: true,
            },
            metadata: [{ name: 'robots', content: 'noindex' }],
            colorMode: {
                defaultMode: 'light',
                disableSwitch: false,
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
                    { to: 'blog', label: 'Blog', position: 'right' },
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
                ],
            },
            footer: {
                style: 'dark',

                links: [
                    {
                        html: `
                        <div class="footer__custom_inner">
                            <div>Copyright © 2022 Jetty Labs, Inc.</div>
                            <div class="footer__custom_legal_links">
                                <div class="footer__custom_legal_link">
                                    <a href="https://www.get-jetty.com/privacy" target="_blank">Privacy Policy</a>
                                </div>
                                <div class="footer__custom_legal_link">
                                    <a href="https://www.get-jetty.com/terms-of-service" target="_blank">Terms of Service</a>
                                </div>
                            </div>
                        </div>
                        `,
                    },
                ],
            },
            prism: {
                theme: lightCodeTheme,
                darkTheme: darkCodeTheme,
            },
        }),
};

module.exports = config;
