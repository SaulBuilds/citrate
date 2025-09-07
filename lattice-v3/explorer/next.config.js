/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  experimental: {
    serverActions: true,
  },
  async rewrites() {
    return [
      {
        source: '/rpc/:path*',
        destination: process.env.RPC_ENDPOINT || 'http://localhost:8545/:path*',
      },
    ];
  },
  env: {
    RPC_ENDPOINT: process.env.RPC_ENDPOINT || 'http://localhost:8545',
    DATABASE_URL: process.env.DATABASE_URL || 'postgresql://postgres:password@localhost:5432/lattice_explorer',
    INDEXER_ENABLED: process.env.INDEXER_ENABLED || 'true',
  },
};

module.exports = nextConfig;