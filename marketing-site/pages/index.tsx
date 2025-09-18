import Head from 'next/head';
import Link from 'next/link';

export default function Home() {
  return (
    <>
      <Head>
        <title>Lattice Network</title>
        <meta name="description" content="Decentralized AI + Token Incentives" />
      </Head>
      <main style={{fontFamily:'Inter,ui-sans-serif', padding:'2rem'}}>
        <header style={{display:'flex',justifyContent:'space-between',alignItems:'center'}}>
          <h1>Lattice Network</h1>
          <nav style={{display:'flex',gap:'1rem'}}>
            <Link href="/sandbox">Sandbox</Link>
            <a href="../docs-portal/build" target="_blank">Docs</a>
            <a href="https://github.com/lattice-network" target="_blank">GitHub</a>
          </nav>
        </header>
        <section style={{marginTop:'4rem'}}>
          <h2>Distribute AI. Earn Tokens.</h2>
          <p>Build and deploy AI models to a decentralized network. Providers serve inference and earn based on usage and quality.</p>
        </section>
        <section style={{marginTop:'2rem',display:'grid',gridTemplateColumns:'repeat(3,1fr)',gap:'1.5rem'}}>
          <div>
            <h3>For Users</h3>
            <p>Discover AI-powered dApps and experiences. Use familiar wallets to interact.</p>
          </div>
          <div>
            <h3>For Developers</h3>
            <p>Ship contracts, use our SDK, and integrate Lattice RPCs. Test safely in the Sandbox.</p>
          </div>
          <div>
            <h3>For Providers</h3>
            <p>Register models, set pricing, and earn from serving inference on-chain.</p>
          </div>
        </section>
      </main>
    </>
  );
}

