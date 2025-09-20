import Head from 'next/head';

const GH_OWNER = process.env.NEXT_PUBLIC_GH_OWNER || 'lattice-network';
const GH_REPO = process.env.NEXT_PUBLIC_GH_REPO || 'lattice-v3';
const RELEASE_URL = `https://github.com/${GH_OWNER}/${GH_REPO}/releases/latest`;

function osName() {
  if (typeof navigator === 'undefined') return 'unknown';
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes('mac')) return 'macos';
  if (ua.includes('win')) return 'windows';
  if (ua.includes('linux')) return 'linux';
  return 'unknown';
}

export default function Downloads() {
  const detected = osName();
  return (
    <>
      <Head>
        <title>Downloads — Lattice</title>
        <meta name="description" content="Download Lattice Node, CLI, and GUI installers" />
      </Head>
      <main style={{fontFamily:'Inter,ui-sans-serif', padding:'2rem', maxWidth: 960, margin: '0 auto'}}>
        <header style={{display:'flex',justifyContent:'space-between',alignItems:'center'}}>
          <h1>Downloads</h1>
          <a href={RELEASE_URL} target="_blank" rel="noopener noreferrer">All Releases</a>
        </header>

        <p style={{color:'#555'}}>Detected platform: <b>{detected}</b>. Choose your installer below or visit the releases page.</p>

        <section style={{display:'grid',gridTemplateColumns:'repeat(auto-fit,minmax(260px,1fr))',gap:'1rem',marginTop:'1rem'}}>
          <div style={{border:'1px solid #eee',borderRadius:12,padding:'1rem'}}>
            <h3>macOS</h3>
            <ul>
              <li>GUI (Tauri): .dmg (Apple Silicon/Intel)</li>
              <li>CLI + Node: tar.gz</li>
            </ul>
            <a className="btn" href={RELEASE_URL} target="_blank" rel="noopener noreferrer">Open Latest Release</a>
          </div>
          <div style={{border:'1px solid #eee',borderRadius:12,padding:'1rem'}}>
            <h3>Windows</h3>
            <ul>
              <li>GUI (Tauri): .msi/.exe</li>
              <li>CLI + Node: .zip</li>
            </ul>
            <a className="btn" href={RELEASE_URL} target="_blank" rel="noopener noreferrer">Open Latest Release</a>
          </div>
          <div style={{border:'1px solid #eee',borderRadius:12,padding:'1rem'}}>
            <h3>Linux</h3>
            <ul>
              <li>GUI (Tauri): AppImage / .deb / .rpm</li>
              <li>CLI + Node: tar.gz</li>
            </ul>
            <a className="btn" href={RELEASE_URL} target="_blank" rel="noopener noreferrer">Open Latest Release</a>
          </div>
        </section>

        <section style={{marginTop:'2rem'}}>
          <h3>Command-line install (alternatives)</h3>
          <pre style={{background:'#0f172a',color:'#e2e8f0',padding:'1rem',borderRadius:8,overflow:'auto'}}>
{`curl -sSfL https://sh.rustup.rs | sh  # Rust toolchain\n`}
{`git clone https://github.com/${GH_OWNER}/${GH_REPO}.git\n`}
{`cd ${GH_REPO}/lattice-v3 && cargo build --release -p lattice-node -p lattice-cli`}
          </pre>
          <p style={{color:'#666'}}>We will add Homebrew, Scoop, winget, and APT/YUM repos after the first public release.</p>
        </section>

        <style jsx>{`
          .btn { display:inline-block; margin-top:.5rem; padding:.5rem .75rem; background:#111827; color:#fff; border-radius:8px; text-decoration:none; }
          .btn:hover { background:#1f2937; }
        `}</style>
      </main>
    </>
  );
}

