const { execSync } = require('child_process');
const { readdirSync, statSync } = require('fs');
const { join } = require('path');

// Simple exporter: uses markdown-to-pdf to generate a single PDF per doc
function walk(dir, files = []) {
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry);
    const st = statSync(p);
    if (st.isDirectory()) walk(p, files); else if (p.endsWith('.md') || p.endsWith('.mdx')) files.push(p);
  }
  return files;
}

const docsDir = join(process.cwd(), 'docs');
const files = walk(docsDir);
for (const f of files) {
  const out = f.replace(/\.(md|mdx)$/i, '.pdf');
  console.log(`Exporting ${f} -> ${out}`);
  execSync(`npx md-to-pdf "${f}" --dest "${out}"`, { stdio: 'inherit' });
}

