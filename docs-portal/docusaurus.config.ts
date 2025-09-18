import {Config} from '@docusaurus/types';

const config: Config = {
  title: 'Lattice Network',
  tagline: 'Decentralized AI + Token Incentives',
  url: 'https://docs.lattice',
  baseUrl: '/',
  favicon: 'img/favicon.ico',
  organizationName: 'lattice-network',
  projectName: 'lattice-docs',
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
        },
        blog: false,
        theme: { customCss: require.resolve('./src/css/custom.css') },
      },
    ],
  ],
  themeConfig: {
    navbar: {
      title: 'Lattice',
      logo: { alt: 'Lattice', src: 'img/logo.svg' },
      items: [
        { to: '/docs/intro', label: 'Docs', position: 'left' },
        { href: 'https://github.com/lattice', label: 'GitHub', position: 'right' },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        { title: 'Docs', items: [{ label: 'Intro', to: '/docs/intro' }, { label: 'SDK', to: '/docs/developers/sdk/javascript' }] },
        { title: 'Community', items: [{ label: 'Twitter', href: 'https://x.com' }] },
        { title: 'More', items: [{ label: 'GitHub', href: 'https://github.com/lattice' }] },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Lattice Network`,
    },
  },
};

export default config;

