import {Config} from '@docusaurus/types';

const config: Config = {
  title: 'Citrate Network',
  tagline: 'Decentralized AI + Token Incentives',
  url: 'https://docs.citrate.ai',
  baseUrl: '/',
  favicon: 'img/favicon.ico',
  organizationName: 'citrate-network',
  projectName: 'citrate-docs',
  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',
  i18n: { defaultLocale: 'en', locales: ['en'] },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: require.resolve('./sidebars.ts'),
          editUrl: undefined,
          routeBasePath: '/',
        },
        blog: false,
        theme: { customCss: require.resolve('./src/css/custom.css') },
      },
    ],
  ],
  themeConfig: {
    navbar: {
      title: 'Citrate',
      logo: { alt: 'Citrate', src: 'img/logo.svg' },
      items: [
        { to: '/intro', label: 'Docs', position: 'left' },
        { href: 'https://github.com/citrate-ai', label: 'GitHub', position: 'right' },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        { title: 'Docs', items: [{ label: 'Intro', to: '/intro' }, { label: 'SDK', to: '/developers/sdk/javascript' }] },
        { title: 'Community', items: [{ label: 'Twitter', href: 'https://x.com' }] },
        { title: 'More', items: [{ label: 'GitHub', href: 'https://github.com/citrate-ai' }] },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Citrate Network`,
    },
  },
};

export default config;
