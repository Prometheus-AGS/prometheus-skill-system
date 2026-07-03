// @ts-check
// Note: type annotations allow type checking and IDEs autocompletion

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'Prometheus Skill Pack',
  tagline: 'KnowMe-aligned AI skills, P2P sync, and the Feynman learning engine',
  favicon: 'img/favicon.ico',

  url: 'https://prometheus-skill-pack.prometheusags.ai',
  baseUrl: '/',

  organizationName: 'prometheusags',
  projectName: 'prometheus-skill-pack',

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

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
          editUrl: 'https://github.com/prometheusags/prometheus-skill-pack/edit/main/site/',
        },
        blog: false,
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'Prometheus Skill Pack',
        logo: {
          alt: 'KnowMe Conviction mark',
          src: 'img/knowme-conviction.svg',
          srcDark: 'img/knowme-conviction-dark.svg',
          width: 32,
          height: 32,
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'guideSidebar',
            position: 'left',
            label: 'Guide',
          },
          {
            type: 'docSidebar',
            sidebarId: 'learnSidebar',
            position: 'left',
            label: 'Learn Domain',
          },
          {
            type: 'docSidebar',
            sidebarId: 'sovereignSidebar',
            position: 'left',
            label: 'Sovereign Sync',
          },
          {
            href: 'https://github.com/prometheusags/prometheus-skill-pack',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              { label: 'Introduction', to: '/docs/guide/introduction' },
              { label: 'Learn Domain', to: '/docs/learn/overview' },
              { label: 'Sovereign Sync', to: '/docs/sovereign-sync/overview' },
            ],
          },
          {
            title: 'More',
            items: [
              { label: 'GitHub', href: 'https://github.com/prometheusags/prometheus-skill-pack' },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Prometheus AGS. MIT License.`,
      },
      prism: {
        theme: require('prism-react-renderer').themes.github,
        darkTheme: require('prism-react-renderer').themes.dracula,
        additionalLanguages: ['rust', 'toml', 'bash'],
      },
    }),
};

module.exports = config;
