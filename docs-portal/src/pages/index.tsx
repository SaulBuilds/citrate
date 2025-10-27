import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

export default function Home(): JSX.Element {
  return (
    <Layout title="Lattice Docs" description="Citrate Network Documentation">
      <main style={{padding: '2rem'}}>
        <h1>Citrate Network Documentation</h1>
        <p>Start with the introduction to learn about Lattice.</p>
        <Link className="button button--primary" to="/intro">Get Started</Link>
      </main>
    </Layout>
  );
}

