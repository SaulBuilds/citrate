import React from 'react'
import { DocsThemeConfig } from 'nextra-theme-docs'

const config: DocsThemeConfig = {
  logo: <span style={{ fontWeight: 'bold', fontSize: '1.2em' }}>⟡ Lattice AI Docs</span>,
  project: {
    link: 'https://github.com/lattice-ai/lattice-v3',
  },
  chat: {
    link: 'https://discord.gg/lattice',
  },
  docsRepositoryBase: 'https://github.com/lattice-ai/lattice-v3/tree/main/developer-tools/documentation-portal',
  footer: {
    text: 'Lattice AI Blockchain Documentation © 2024',
  },
  primaryHue: 240,
  primarySaturation: 100,
  useNextSeoProps() {
    return {
      titleTemplate: '%s – Lattice AI Docs'
    }
  },
  head: (
    <>
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />
      <meta property="og:title" content="Lattice AI Blockchain Documentation" />
      <meta property="og:description" content="Complete developer guide for Lattice AI blockchain" />
      <link rel="icon" href="/favicon.ico" />
    </>
  ),
  sidebar: {
    titleComponent({ title, type }) {
      if (type === 'separator') {
        return <span className="cursor-default">{title}</span>
      }
      return <>{title}</>
    },
    defaultMenuCollapseLevel: 1,
    toggleButton: true
  },
  navigation: {
    prev: true,
    next: true
  },
  toc: {
    backToTop: true
  },
  editLink: {
    text: 'Edit this page on GitHub →'
  },
  feedback: {
    content: 'Question? Give us feedback →',
    labels: 'feedback'
  },
  search: {
    placeholder: 'Search documentation...'
  },
  gitTimestamp: ({ timestamp }) => (
    <>Last updated on {timestamp.toLocaleDateString()}</>
  ),
  darkMode: true,
  nextThemes: {
    defaultTheme: 'dark'
  }
}

export default config