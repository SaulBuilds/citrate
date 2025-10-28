# Citrate Front-End Architecture Guide

**For Marketing Site & Documentation Portal**

*Version 1.0 - October 27, 2025*

---

## Table of Contents

1. [Overview](#overview)
2. [Technical Stack](#technical-stack)
3. [Architecture Principles](#architecture-principles)
4. [Design System](#design-system)
5. [Component Structure](#component-structure)
6. [Page Templates](#page-templates)
7. [Performance Optimization](#performance-optimization)
8. [SEO Strategy](#seo-strategy)
9. [Content Integration](#content-integration)
10. [Deployment](#deployment)

---

## Overview

### Project Structure

The Citrate front-end ecosystem consists of three interconnected web properties:

```
citrate/
├── marketing-site/          # Next.js marketing website
│   ├── Landing page (/)
│   ├── Features (/features)
│   ├── Whitepaper (/whitepaper)
│   ├── FAQ (/faq)
│   └── Blog (/blog)
│
├── docs-portal/             # Docusaurus documentation portal
│   ├── Getting Started
│   ├── API Reference
│   ├── Tutorials
│   └── Technical Guides
│
└── explorer/                # Next.js block explorer
    ├── Blocks
    ├── Transactions
    ├── Models
    └── DAG Visualizer
```

### Design Philosophy

**Core Principles**:
1. **AI-Native Aesthetic** - Futuristic, technical, yet approachable
2. **Performance First** - Lighthouse score 95+ on all metrics
3. **Developer-Focused** - Code examples, technical depth, dark mode default
4. **Trust Through Transparency** - Open-source ethos, technical accuracy
5. **Blockchain 3.0 Vibes** - Beyond DeFi, into AI/ML territory

**Visual Identity**:
- **Primary**: Deep purples and electric blues (representing AI + blockchain fusion)
- **Accent**: Bright cyan and magenta (high-tech, futuristic)
- **Neutral**: Dark grays and pure whites (technical, clean)
- **Code**: Monokai-inspired syntax highlighting

**Tone of Voice**:
- **Technical but accessible** - Explain complex concepts clearly
- **Confident but humble** - No hype, just facts and innovation
- **Community-driven** - "We" not "I", collaborative language
- **Action-oriented** - "Build", "Deploy", "Earn" over "Learn About"

---

## Technical Stack

### Marketing Site

**Framework**: Next.js 14 (App Router)
```json
{
  "next": "^14.0.0",
  "react": "^18.2.0",
  "react-dom": "^18.2.0"
}
```

**Why Next.js**:
- ✅ Server-side rendering (SSR) for SEO
- ✅ Static site generation (SSG) for performance
- ✅ Built-in image optimization
- ✅ API routes for dynamic content
- ✅ Excellent TypeScript support

**Styling**: Tailwind CSS + shadcn/ui components
```json
{
  "tailwindcss": "^3.4.0",
  "@radix-ui/react-*": "latest",
  "class-variance-authority": "^0.7.0",
  "clsx": "^2.0.0"
}
```

**Why Tailwind + shadcn**:
- ✅ Utility-first, rapid prototyping
- ✅ shadcn components are copy-paste (no npm bloat)
- ✅ Radix UI primitives (accessibility built-in)
- ✅ Easy dark mode with CSS variables

**Animation**: Framer Motion
```json
{
  "framer-motion": "^10.16.0"
}
```

**Why Framer Motion**:
- ✅ Performant 60fps animations
- ✅ Layout animations (reorder, shared elements)
- ✅ Scroll-triggered animations
- ✅ TypeScript support

**Data Visualization**: Recharts + D3.js
```json
{
  "recharts": "^2.10.0",
  "d3": "^7.8.5"
}
```

**Why**:
- ✅ Recharts for simple charts (tokenomics, roadmap)
- ✅ D3.js for complex DAG visualization
- ✅ SVG-based (scalable, accessible)

**Code Highlighting**: Shiki
```json
{
  "shiki": "^0.14.0"
}
```

**Why Shiki**:
- ✅ VS Code-quality syntax highlighting
- ✅ Static generation (no runtime cost)
- ✅ Multiple language support (Solidity, TypeScript, Rust)

### Documentation Portal

**Framework**: Docusaurus 3
```json
{
  "@docusaurus/core": "^3.0.0",
  "@docusaurus/preset-classic": "^3.0.0"
}
```

**Why Docusaurus**:
- ✅ Purpose-built for documentation
- ✅ Markdown-based content
- ✅ Versioning support (for API docs)
- ✅ Search integration (Algolia)
- ✅ React component embedding

**Plugins**:
```json
{
  "@docusaurus/plugin-content-docs": "^3.0.0",
  "@docusaurus/plugin-content-blog": "^3.0.0",
  "@docusaurus/theme-live-codeblock": "^3.0.0",
  "docusaurus-plugin-typedoc": "^0.20.0"
}
```

### Shared Dependencies

**TypeScript**:
```json
{
  "typescript": "^5.3.0",
  "@types/react": "^18.2.0",
  "@types/node": "^20.0.0"
}
```

**Linting & Formatting**:
```json
{
  "eslint": "^8.55.0",
  "prettier": "^3.1.0",
  "eslint-config-next": "^14.0.0"
}
```

**SDK Integration**:
```json
{
  "@citrate-ai/sdk": "latest",
  "ethers": "^6.9.0",
  "viem": "^1.20.0"
}
```

---

## Architecture Principles

### 1. Component-Driven Development

**Atomic Design Hierarchy**:
```
Atoms → Molecules → Organisms → Templates → Pages
```

**Example**:
```
Atoms:
- Button
- Input
- Badge
- Icon

Molecules:
- SearchBar (Input + Icon + Button)
- Card (Container + Title + Description)
- CodeSnippet (Code + CopyButton)

Organisms:
- HeroSection (Heading + Subheading + CTAs + Animation)
- FeatureGrid (Cards arranged in grid)
- CodeExample (Tabs + CodeSnippets + Output)

Templates:
- LandingPageTemplate
- DocumentationPageTemplate
- BlogPostTemplate

Pages:
- / (Landing)
- /features
- /whitepaper
```

### 2. Server-First Architecture

**Rendering Strategies**:

**Static Generation (SSG)** - Use for:
- Landing page
- Feature pages
- Whitepaper
- FAQ
- Blog posts

**Server-Side Rendering (SSR)** - Use for:
- Block explorer (real-time data)
- Model marketplace (dynamic content)

**Client-Side Rendering (CSR)** - Use for:
- Interactive DAG visualizer
- Code editor (Monaco)
- Wallet connection

**Example** (`app/page.tsx`):
```typescript
// Static generation (default in Next.js 14 App Router)
export default async function HomePage() {
  // This runs at build time
  const stats = await fetchNetworkStats();

  return <LandingPage stats={stats} />;
}

// Revalidate every hour
export const revalidate = 3600;
```

### 3. Data Fetching Patterns

**For Marketing Site**:
```typescript
// lib/api/network-stats.ts
export async function getNetworkStats() {
  const res = await fetch('https://api.citrate.ai/v1/stats', {
    next: { revalidate: 300 } // Cache for 5 minutes
  });

  return res.json();
}

// app/page.tsx
import { getNetworkStats } from '@/lib/api/network-stats';

export default async function HomePage() {
  const stats = await getNetworkStats();

  return (
    <div>
      <StatCard title="Total Validators" value={stats.validatorCount} />
      <StatCard title="Daily Transactions" value={stats.txCount} />
      <StatCard title="Models Deployed" value={stats.modelCount} />
    </div>
  );
}
```

**For Documentation Portal**:
```typescript
// Markdown-based, no API calls needed
// Content in docs/ directory, processed at build time
```

### 4. Progressive Enhancement

**Core Content First**:
1. HTML renders without JavaScript (SEO, accessibility)
2. CSS loads progressively (critical CSS inline)
3. JavaScript enhances (animations, interactivity)

**Example**:
```tsx
// components/CodeExample.tsx
export function CodeExample({ code, language }: Props) {
  // Works without JS (server-rendered HTML)
  return (
    <pre className="code-block">
      <code className={`language-${language}`}>
        {code}
      </code>
    </pre>
  );
}

// Enhanced with client-side copy button
'use client';
export function InteractiveCodeExample({ code, language }: Props) {
  const [copied, setCopied] = useState(false);

  return (
    <div className="relative">
      <CodeExample code={code} language={language} />
      <CopyButton onClick={() => {
        navigator.clipboard.writeText(code);
        setCopied(true);
      }} />
    </div>
  );
}
```

---

## Design System

### Color Palette

**Dark Theme (Default)**:
```css
:root {
  /* Primary - Blockchain Blue */
  --primary-50: #e6f1ff;
  --primary-100: #b3d9ff;
  --primary-500: #0066ff;
  --primary-700: #0047b3;
  --primary-900: #002966;

  /* Secondary - AI Purple */
  --secondary-50: #f3e6ff;
  --secondary-100: #d9b3ff;
  --secondary-500: #9333ea;
  --secondary-700: #6b21a8;
  --secondary-900: #4c1d95;

  /* Accent - Cyan */
  --accent-50: #e6ffff;
  --accent-500: #00d9ff;
  --accent-700: #00a3cc;

  /* Neutrals */
  --gray-50: #fafafa;
  --gray-100: #f4f4f5;
  --gray-200: #e4e4e7;
  --gray-300: #d4d4d8;
  --gray-400: #a1a1aa;
  --gray-500: #71717a;
  --gray-600: #52525b;
  --gray-700: #3f3f46;
  --gray-800: #27272a;
  --gray-900: #18181b;
  --gray-950: #09090b;

  /* Semantic Colors */
  --success: #10b981;
  --warning: #f59e0b;
  --error: #ef4444;
  --info: #3b82f6;

  /* Backgrounds */
  --bg-primary: var(--gray-950);
  --bg-secondary: var(--gray-900);
  --bg-tertiary: var(--gray-800);

  /* Text */
  --text-primary: var(--gray-50);
  --text-secondary: var(--gray-400);
  --text-tertiary: var(--gray-500);
}
```

**Light Theme**:
```css
[data-theme="light"] {
  --bg-primary: var(--gray-50);
  --bg-secondary: var(--gray-100);
  --bg-tertiary: var(--gray-200);

  --text-primary: var(--gray-900);
  --text-secondary: var(--gray-600);
  --text-tertiary: var(--gray-500);
}
```

### Typography

**Font Stack**:
```css
:root {
  /* Headlines - Geometric Sans */
  --font-display: 'Space Grotesk', -apple-system, sans-serif;

  /* Body - Sans Serif */
  --font-body: 'Inter', -apple-system, system-ui, sans-serif;

  /* Code - Monospace */
  --font-mono: 'JetBrains Mono', 'Fira Code', 'Monaco', monospace;
}
```

**Type Scale** (1.250 ratio):
```css
:root {
  --text-xs: 0.64rem;    /* 10.24px */
  --text-sm: 0.8rem;     /* 12.8px */
  --text-base: 1rem;     /* 16px */
  --text-lg: 1.25rem;    /* 20px */
  --text-xl: 1.563rem;   /* 25px */
  --text-2xl: 1.953rem;  /* 31.25px */
  --text-3xl: 2.441rem;  /* 39.06px */
  --text-4xl: 3.052rem;  /* 48.83px */
  --text-5xl: 3.815rem;  /* 61.04px */
  --text-6xl: 4.768rem;  /* 76.29px */
}
```

**Usage**:
```tsx
<h1 className="font-display text-6xl font-bold">
  The First AI-Native Blockchain
</h1>
<p className="font-body text-lg text-secondary">
  Built for the Age of Decentralized Intelligence
</p>
<code className="font-mono text-sm">
  npm install @citrate-ai/sdk
</code>
```

### Spacing System

**8px Base Grid**:
```css
:root {
  --space-1: 0.25rem;  /* 4px */
  --space-2: 0.5rem;   /* 8px */
  --space-3: 0.75rem;  /* 12px */
  --space-4: 1rem;     /* 16px */
  --space-5: 1.25rem;  /* 20px */
  --space-6: 1.5rem;   /* 24px */
  --space-8: 2rem;     /* 32px */
  --space-10: 2.5rem;  /* 40px */
  --space-12: 3rem;    /* 48px */
  --space-16: 4rem;    /* 64px */
  --space-20: 5rem;    /* 80px */
  --space-24: 6rem;    /* 96px */
  --space-32: 8rem;    /* 128px */
}
```

### Component Patterns

**Button**:
```tsx
// components/ui/button.tsx
import { cva, type VariantProps } from 'class-variance-authority';

const buttonVariants = cva(
  'inline-flex items-center justify-center rounded-lg font-medium transition-all focus-visible:outline-none focus-visible:ring-2 disabled:opacity-50 disabled:pointer-events-none',
  {
    variants: {
      variant: {
        primary: 'bg-primary-500 text-white hover:bg-primary-600 active:bg-primary-700',
        secondary: 'bg-secondary-500 text-white hover:bg-secondary-600 active:bg-secondary-700',
        outline: 'border-2 border-primary-500 text-primary-500 hover:bg-primary-500/10',
        ghost: 'text-primary-500 hover:bg-primary-500/10',
      },
      size: {
        sm: 'h-9 px-3 text-sm',
        md: 'h-11 px-5 text-base',
        lg: 'h-14 px-8 text-lg',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  }
);

export function Button({ variant, size, className, ...props }: Props) {
  return (
    <button
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}
```

**Card**:
```tsx
// components/ui/card.tsx
export function Card({ children, className, ...props }: Props) {
  return (
    <div
      className={cn(
        'rounded-xl border border-gray-800 bg-gray-900/50 backdrop-blur-sm p-6',
        'hover:border-primary-500/50 transition-colors',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}
```

**Code Block**:
```tsx
// components/ui/code-block.tsx
'use client';
import { useState } from 'react';
import { highlight } from 'shiki';

export function CodeBlock({ code, language, filename }: Props) {
  const [copied, setCopied] = useState(false);

  return (
    <div className="group relative">
      {filename && (
        <div className="flex items-center gap-2 rounded-t-lg bg-gray-800 px-4 py-2 text-sm text-gray-400">
          <FileIcon />
          {filename}
        </div>
      )}
      <pre className="overflow-x-auto rounded-lg bg-gray-900 p-4 font-mono text-sm">
        <code dangerouslySetInnerHTML={{ __html: highlightedCode }} />
      </pre>
      <button
        onClick={() => {
          navigator.clipboard.writeText(code);
          setCopied(true);
          setTimeout(() => setCopied(false), 2000);
        }}
        className="absolute right-4 top-4 opacity-0 group-hover:opacity-100 transition-opacity"
      >
        {copied ? <CheckIcon /> : <CopyIcon />}
      </button>
    </div>
  );
}
```

---

## Component Structure

### Marketing Site Components

**File Structure**:
```
marketing-site/
├── app/
│   ├── layout.tsx          # Root layout
│   ├── page.tsx            # Landing page
│   ├── features/
│   │   └── page.tsx
│   ├── whitepaper/
│   │   └── page.tsx
│   └── faq/
│       └── page.tsx
│
├── components/
│   ├── ui/                 # Base components (shadcn)
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── input.tsx
│   │   └── ...
│   │
│   ├── sections/           # Page sections
│   │   ├── Hero.tsx
│   │   ├── Features.tsx
│   │   ├── TechStack.tsx
│   │   ├── Tokenomics.tsx
│   │   ├── Roadmap.tsx
│   │   └── FAQ.tsx
│   │
│   ├── layout/             # Layout components
│   │   ├── Header.tsx
│   │   ├── Footer.tsx
│   │   └── Navigation.tsx
│   │
│   └── interactive/        # Interactive components
│       ├── DAGVisualizer.tsx
│       ├── CodePlayground.tsx
│       └── ModelExplorer.tsx
│
├── lib/
│   ├── api/                # API clients
│   ├── utils/              # Utilities
│   └── constants/          # Constants
│
└── public/
    ├── images/
    ├── fonts/
    └── videos/
```

### Key Component Implementations

**Hero Section**:
```tsx
// components/sections/Hero.tsx
import { Button } from '@/components/ui/button';
import { ArrowRight, Github } from 'lucide-react';
import { motion } from 'framer-motion';

export function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Animated background gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-primary-900/20 via-secondary-900/20 to-transparent" />

      {/* Floating particles (canvas animation) */}
      <ParticleBackground />

      <div className="relative z-10 container mx-auto px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="max-w-4xl mx-auto text-center"
        >
          <h1 className="font-display text-6xl md:text-7xl lg:text-8xl font-bold mb-6">
            The First{' '}
            <span className="bg-gradient-to-r from-primary-500 to-secondary-500 bg-clip-text text-transparent">
              AI-Native
            </span>{' '}
            Blockchain
          </h1>

          <p className="text-xl md:text-2xl text-gray-400 mb-12 max-w-3xl mx-auto">
            Citrate combines cutting-edge BlockDAG consensus with standardized AI model
            orchestration, creating the first blockchain purpose-built for artificial
            intelligence workloads.
          </p>

          <div className="flex flex-wrap gap-4 justify-center">
            <Button size="lg" className="gap-2">
              Get Started <ArrowRight className="w-5 h-5" />
            </Button>
            <Button size="lg" variant="outline" className="gap-2">
              <Github className="w-5 h-5" /> View on GitHub
            </Button>
            <Button size="lg" variant="ghost">
              Read Whitepaper
            </Button>
          </div>
        </motion.div>

        {/* Live network stats */}
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.3 }}
          className="mt-20 grid grid-cols-2 md:grid-cols-4 gap-6 max-w-4xl mx-auto"
        >
          <StatCard title="Validators" value="1,247" change="+12%" />
          <StatCard title="Daily TPS" value="12,340" change="+8%" />
          <StatCard title="Models" value="342" change="+25%" />
          <StatCard title="TVL" value="$24.5M" change="+15%" />
        </motion.div>
      </div>
    </section>
  );
}
```

**Features Grid**:
```tsx
// components/sections/Features.tsx
import { Zap, Lock, Globe, Code } from 'lucide-react';

const features = [
  {
    icon: Zap,
    title: 'GhostDAG Consensus',
    description: '10,000+ TPS through parallel block production with sub-12 second finality',
    link: '/features/consensus',
  },
  {
    icon: Code,
    title: 'EVM Compatible',
    description: '100% Solidity compatible. Deploy Ethereum contracts without modification',
    link: '/features/evm',
  },
  {
    icon: Globe,
    title: 'Model Context Protocol',
    description: 'Native MCP integration for standardized AI model orchestration',
    link: '/features/mcp',
  },
  {
    icon: Lock,
    title: 'Verifiable Inference',
    description: 'Cryptographic proofs ensure AI inference ran correctly',
    link: '/features/verification',
  },
];

export function Features() {
  return (
    <section className="py-24 bg-gradient-to-b from-gray-950 to-gray-900">
      <div className="container mx-auto px-4">
        <div className="max-w-3xl mx-auto text-center mb-16">
          <h2 className="font-display text-5xl font-bold mb-6">
            Three Core Innovations
          </h2>
          <p className="text-xl text-gray-400">
            Citrate combines cutting-edge blockchain consensus with AI-specific features
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
          {features.map((feature, i) => (
            <FeatureCard key={i} {...feature} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}

function FeatureCard({ icon: Icon, title, description, link, index }: Props) {
  return (
    <motion.a
      href={link}
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ delay: index * 0.1 }}
      className="group relative p-6 rounded-xl border border-gray-800 bg-gray-900/50 backdrop-blur hover:border-primary-500/50 transition-all hover:scale-105"
    >
      <div className="absolute inset-0 bg-gradient-to-br from-primary-500/10 to-secondary-500/10 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity" />

      <div className="relative">
        <Icon className="w-12 h-12 text-primary-500 mb-4" />
        <h3 className="font-display text-xl font-semibold mb-2">{title}</h3>
        <p className="text-gray-400">{description}</p>
      </div>
    </motion.a>
  );
}
```

**Code Example Component**:
```tsx
// components/sections/CodeExample.tsx
'use client';
import { useState } from 'react';
import { CodeBlock } from '@/components/ui/code-block';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';

const examples = {
  deploy: {
    label: 'Deploy Model',
    code: `import { ModelRegistry } from "@citrate-ai/sdk";

const registry = new ModelRegistry(client);

const model = await registry.deployModel({
  weights: "QmXnnyufdzAWL...",  // IPFS CID
  architecture: "transformer",
  metadata: {
    name: "Llama 3 70B Instruct",
    parameters: "70B",
  },
  accessPolicy: {
    type: "pay-per-inference",
    pricePerToken: 0.0001
  }
});

console.log(\`Model deployed: \${model.id}\`);`,
    language: 'typescript',
  },
  inference: {
    label: 'Run Inference',
    code: `const response = await client.inference.run({
  model: "llama-3-70b-instruct",
  input: "Explain quantum computing",
  maxTokens: 500,
  payment: 0.1  // LATT
});

console.log(response.output);
// "Quantum computing uses quantum mechanics..."`,
    language: 'typescript',
  },
  contract: {
    label: 'Smart Contract',
    code: `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract AIOracle {
    function predict(bytes memory data) external returns (bytes memory) {
        // Call AI precompile at 0x1001
        (bool success, bytes memory result) = address(0x1001).call(
            abi.encodeWithSignature("runInference(bytes)", data)
        );
        require(success, "Inference failed");
        return result;
    }
}`,
    language: 'solidity',
  },
};

export function CodeExample() {
  const [activeTab, setActiveTab] = useState('deploy');

  return (
    <section className="py-24">
      <div className="container mx-auto px-4">
        <div className="max-w-3xl mx-auto text-center mb-12">
          <h2 className="font-display text-5xl font-bold mb-6">
            Build with Familiar Tools
          </h2>
          <p className="text-xl text-gray-400">
            100% EVM compatible with AI-specific extensions
          </p>
        </div>

        <div className="max-w-4xl mx-auto">
          <Tabs value={activeTab} onValueChange={setActiveTab}>
            <TabsList className="mb-6">
              {Object.entries(examples).map(([key, { label }]) => (
                <TabsTrigger key={key} value={key}>
                  {label}
                </TabsTrigger>
              ))}
            </TabsList>

            {Object.entries(examples).map(([key, { code, language }]) => (
              <TabsContent key={key} value={key}>
                <CodeBlock code={code} language={language} />
              </TabsContent>
            ))}
          </Tabs>
        </div>
      </div>
    </section>
  );
}
```

---

## Page Templates

### Landing Page Structure

```tsx
// app/page.tsx
import { Hero } from '@/components/sections/Hero';
import { Features } from '@/components/sections/Features';
import { TechStack } from '@/components/sections/TechStack';
import { CodeExample } from '@/components/sections/CodeExample';
import { UseCases } from '@/components/sections/UseCases';
import { Tokenomics } from '@/components/sections/Tokenomics';
import { Roadmap } from '@/components/sections/Roadmap';
import { Community } from '@/components/sections/Community';
import { CTA } from '@/components/sections/CTA';

export default function HomePage() {
  return (
    <>
      <Hero />
      <Features />
      <TechStack />
      <CodeExample />
      <UseCases />
      <Tokenomics />
      <Roadmap />
      <Community />
      <CTA />
    </>
  );
}
```

### Whitepaper Page

```tsx
// app/whitepaper/page.tsx
import { MDXRemote } from 'next-mdx-remote/rsc';
import { readFile } from 'fs/promises';
import { TableOfContents } from '@/components/whitepaper/TableOfContents';
import { ShareButtons } from '@/components/whitepaper/ShareButtons';

export default async function WhitepaperPage() {
  const content = await readFile('docs/marketing/WHITEPAPER.md', 'utf-8');

  return (
    <div className="min-h-screen bg-gray-950">
      <div className="container mx-auto px-4 py-16">
        <div className="grid lg:grid-cols-[250px_1fr] gap-8">
          {/* Sticky sidebar */}
          <aside className="hidden lg:block">
            <div className="sticky top-24">
              <TableOfContents />
              <ShareButtons className="mt-8" />
            </div>
          </aside>

          {/* Main content */}
          <article className="prose prose-invert prose-lg max-w-none">
            <MDXRemote source={content} components={mdxComponents} />
          </article>
        </div>
      </div>
    </div>
  );
}

const mdxComponents = {
  h1: (props) => <h1 className="font-display text-5xl font-bold mb-6" {...props} />,
  h2: (props) => <h2 className="font-display text-4xl font-bold mt-12 mb-4" {...props} />,
  code: (props) => <CodeBlock {...props} />,
  pre: (props) => <div {...props} />,  // CodeBlock handles <pre>
  a: (props) => <a className="text-primary-500 hover:underline" {...props} />,
};
```

### FAQ Page

```tsx
// app/faq/page.tsx
'use client';
import { useState } from 'react';
import { Accordion, AccordionItem, AccordionTrigger, AccordionContent } from '@/components/ui/accordion';
import { Search } from 'lucide-react';

export default function FAQPage() {
  const [searchQuery, setSearchQuery] = useState('');

  const filteredFAQs = faqs.filter(faq =>
    faq.question.toLowerCase().includes(searchQuery.toLowerCase()) ||
    faq.answer.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="min-h-screen bg-gray-950 py-24">
      <div className="container mx-auto px-4 max-w-4xl">
        <div className="text-center mb-12">
          <h1 className="font-display text-5xl font-bold mb-6">
            Frequently Asked Questions
          </h1>
          <p className="text-xl text-gray-400">
            Everything you need to know about Citrate
          </p>
        </div>

        {/* Search */}
        <div className="relative mb-12">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 w-5 h-5" />
          <input
            type="text"
            placeholder="Search questions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-12 pr-4 py-3 bg-gray-900 border border-gray-800 rounded-lg focus:border-primary-500 focus:outline-none"
          />
        </div>

        {/* FAQ Accordion */}
        <Accordion type="single" collapsible className="space-y-4">
          {filteredFAQs.map((faq, i) => (
            <AccordionItem key={i} value={`item-${i}`}>
              <AccordionTrigger className="text-left">
                {faq.question}
              </AccordionTrigger>
              <AccordionContent className="prose prose-invert">
                <div dangerouslySetInnerHTML={{ __html: faq.answer }} />
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>

        {filteredFAQs.length === 0 && (
          <div className="text-center py-12 text-gray-400">
            No FAQs found matching "{searchQuery}"
          </div>
        )}
      </div>
    </div>
  );
}
```

---

## Performance Optimization

### 1. Image Optimization

**Use Next.js Image Component**:
```tsx
import Image from 'next/image';

<Image
  src="/images/hero-bg.jpg"
  alt="Hero background"
  width={1920}
  height={1080}
  priority  // For above-the-fold images
  placeholder="blur"
  blurDataURL="data:image/jpeg;base64,..."
/>
```

**Benefits**:
- ✅ Automatic WebP/AVIF conversion
- ✅ Lazy loading (except `priority` images)
- ✅ Responsive images (srcset)
- ✅ Blur-up placeholder

### 2. Font Optimization

**Use next/font**:
```tsx
// app/layout.tsx
import { Space_Grotesk, Inter, JetBrains_Mono } from 'next/font/google';

const spaceGrotesk = Space_Grotesk({
  subsets: ['latin'],
  display: 'swap',
  variable: '--font-display',
});

const inter = Inter({
  subsets: ['latin'],
  display: 'swap',
  variable: '--font-body',
});

export default function RootLayout({ children }) {
  return (
    <html lang="en" className={`${spaceGrotesk.variable} ${inter.variable}`}>
      <body className="font-body">{children}</body>
    </html>
  );
}
```

**Benefits**:
- ✅ Self-hosted fonts (no Google Fonts request)
- ✅ Automatic subsetting
- ✅ Zero layout shift (font-display: swap)

### 3. Code Splitting

**Dynamic Imports**:
```tsx
// Only load heavy components when needed
import dynamic from 'next/dynamic';

const DAGVisualizer = dynamic(() => import('@/components/interactive/DAGVisualizer'), {
  loading: () => <Skeleton />,
  ssr: false,  // Client-side only
});

export function Features() {
  const [showDAG, setShowDAG] = useState(false);

  return (
    <div>
      <button onClick={() => setShowDAG(true)}>Show DAG</button>
      {showDAG && <DAGVisualizer />}
    </div>
  );
}
```

### 4. Bundle Analysis

**Analyze Bundle Size**:
```bash
npm install @next/bundle-analyzer

# next.config.js
const withBundleAnalyzer = require('@next/bundle-analyzer')({
  enabled: process.env.ANALYZE === 'true',
});

module.exports = withBundleAnalyzer({
  // ...config
});

# Run analysis
ANALYZE=true npm run build
```

**Target Metrics**:
- First Contentful Paint (FCP): <1.5s
- Largest Contentful Paint (LCP): <2.5s
- Time to Interactive (TTI): <3.5s
- Cumulative Layout Shift (CLS): <0.1
- Total Bundle Size: <200 KB (initial JS)

---

## SEO Strategy

### 1. Metadata

**Generate Metadata**:
```tsx
// app/layout.tsx
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: {
    default: 'Citrate - The First AI-Native Blockchain',
    template: '%s | Citrate',
  },
  description: 'Citrate combines GhostDAG consensus with EVM compatibility and native AI model orchestration. Build decentralized AI applications with 10,000+ TPS.',
  keywords: ['blockchain', 'AI', 'GhostDAG', 'EVM', 'smart contracts', 'machine learning', 'DeFi', 'Web3'],
  authors: [{ name: 'Citrate Foundation' }],
  creator: 'Citrate Foundation',
  publisher: 'Citrate Foundation',
  openGraph: {
    type: 'website',
    locale: 'en_US',
    url: 'https://citrate.ai',
    siteName: 'Citrate',
    title: 'Citrate - The First AI-Native Blockchain',
    description: 'Build decentralized AI applications with 10,000+ TPS',
    images: [
      {
        url: 'https://citrate.ai/og-image.jpg',
        width: 1200,
        height: 630,
        alt: 'Citrate - AI-Native Blockchain',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    site: '@CitrateAI',
    creator: '@CitrateAI',
    title: 'Citrate - The First AI-Native Blockchain',
    description: 'Build decentralized AI applications with 10,000+ TPS',
    images: ['https://citrate.ai/twitter-image.jpg'],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  verification: {
    google: 'your-google-site-verification',
  },
};
```

### 2. Structured Data

**JSON-LD**:
```tsx
// components/StructuredData.tsx
export function StructuredData() {
  const schema = {
    '@context': 'https://schema.org',
    '@type': 'Organization',
    name: 'Citrate',
    url: 'https://citrate.ai',
    logo: 'https://citrate.ai/logo.png',
    sameAs: [
      'https://twitter.com/CitrateAI',
      'https://github.com/citrate-ai',
      'https://discord.gg/citrate',
    ],
    description: 'The first AI-native blockchain with GhostDAG consensus and EVM compatibility',
  };

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(schema) }}
    />
  );
}
```

### 3. Sitemap & Robots.txt

**Generate Sitemap**:
```tsx
// app/sitemap.ts
export default function sitemap() {
  return [
    {
      url: 'https://citrate.ai',
      lastModified: new Date(),
      changeFrequency: 'daily',
      priority: 1,
    },
    {
      url: 'https://citrate.ai/features',
      lastModified: new Date(),
      changeFrequency: 'weekly',
      priority: 0.8,
    },
    {
      url: 'https://citrate.ai/whitepaper',
      lastModified: new Date(),
      changeFrequency: 'monthly',
      priority: 0.9,
    },
    // ... more URLs
  ];
}
```

**Robots.txt**:
```tsx
// app/robots.ts
export default function robots() {
  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: ['/api/', '/admin/'],
    },
    sitemap: 'https://citrate.ai/sitemap.xml',
  };
}
```

---

## Content Integration

### How to Use the Marketing Documents

**1. Landing Page Content** (`LANDING_PAGE_CONTENT.md`):
```tsx
// Parse markdown sections and map to components

// Hero Section → components/sections/Hero.tsx
// Features Section → components/sections/Features.tsx
// Tech Stack → components/sections/TechStack.tsx
// etc.
```

**2. Whitepaper** (`WHITEPAPER.md`):
```tsx
// Render as MDX with rich components

import { MDXRemote } from 'next-mdx-remote/rsc';

export default async function WhitepaperPage() {
  const content = await readFile('docs/marketing/WHITEPAPER.md', 'utf-8');

  return <MDXRemote source={content} components={mdxComponents} />;
}
```

**3. FAQ** (`FAQ.md`):
```tsx
// Parse markdown into Q&A pairs

import { remark } from 'remark';
import html from 'remark-html';

const faqs = parseFAQs(markdownContent);  // Custom parser

export default function FAQPage() {
  return <Accordion items={faqs} />;
}
```

---

## Deployment

### Hosting Options

**Recommended: Vercel**
```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel --prod

# Automatic deployments on git push
```

**Alternative: Netlify**
```bash
# Install Netlify CLI
npm i -g netlify-cli

# Deploy
netlify deploy --prod
```

**Self-Hosted: Docker**
```dockerfile
FROM node:20-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

EXPOSE 3000
CMD ["npm", "start"]
```

### Environment Variables

```env
# .env.local
NEXT_PUBLIC_RPC_URL=https://rpc.citrate.ai
NEXT_PUBLIC_CHAIN_ID=1
NEXT_PUBLIC_EXPLORER_URL=https://explorer.citrate.ai
NEXT_PUBLIC_ANALYTICS_ID=your-analytics-id

# Private (server-side only)
DATABASE_URL=postgresql://...
API_SECRET_KEY=your-secret-key
```

### Build Configuration

```javascript
// next.config.js
module.exports = {
  output: 'standalone',  // For Docker
  images: {
    domains: ['citrate.ai', 'ipfs.io'],
    formats: ['image/avif', 'image/webp'],
  },
  experimental: {
    optimizeCss: true,
  },
  webpack: (config) => {
    // Custom webpack config
    return config;
  },
};
```

---

## Final Checklist

### Before Launch

- [ ] All pages render correctly (Desktop, Tablet, Mobile)
- [ ] Dark mode and light mode tested
- [ ] All links work (no 404s)
- [ ] Forms validated and tested
- [ ] Code examples syntax-highlighted correctly
- [ ] Images optimized (WebP/AVIF, compressed)
- [ ] Fonts loaded correctly (no FOIT/FOUT)
- [ ] Lighthouse score 95+ on all pages
- [ ] SEO metadata complete (title, description, OG tags)
- [ ] Sitemap generated and submitted
- [ ] Robots.txt configured
- [ ] Analytics integrated (Google Analytics, Plausible, etc.)
- [ ] Error tracking setup (Sentry, LogRocket)
- [ ] HTTPS enabled with valid certificate
- [ ] CDN configured (Cloudflare, etc.)
- [ ] DNS records configured correctly

### Accessibility

- [ ] Keyboard navigation works
- [ ] Screen reader tested (NVDA, VoiceOver)
- [ ] Color contrast ratio 4.5:1+ (WCAG AA)
- [ ] Focus indicators visible
- [ ] Alt text for all images
- [ ] ARIA labels where needed
- [ ] Semantic HTML (h1-h6, nav, main, footer)

---

**Ready to build?** Start with the marketing site landing page, then expand to whitepaper and FAQ. Good luck! 🚀
