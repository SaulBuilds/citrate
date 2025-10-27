# Citrate v3 Block Explorer

A comprehensive block explorer for the Citrate v3 AI-native Layer-1 BlockDAG with GhostDAG consensus.

## Features

### 🎯 Core Features
- **Real-time block and transaction tracking**
- **GhostDAG visualization in 3D vector space**
- **AI model registry explorer**
- **Semantic search powered by genesis model**
- **Network statistics and monitoring**

### 🧠 AI-Native Features
- **Genesis Model Integration**: Embedded BERT-tiny model for semantic operations
- **Semantic Search**: AI-powered search using chain-native embeddings
- **Model Activity Tracking**: Monitor deployed models and inference statistics
- **Intent Classification**: Automatic transaction intent detection

### 📊 Advanced Visualization
- **3D DAG Visualization**: Interactive Three.js-based DAG explorer
- **Blue Score Positioning**: Nodes positioned in 3D space based on blue scores
- **Real-time Updates**: WebSocket-based live data streaming
- **Network Topology**: Visual representation of parent/merge relationships

## Quick Start

### Prerequisites
- Node.js 20+
- PostgreSQL 15+
- Docker (optional)

### Installation

1. **Clone and navigate to explorer:**
```bash
cd citrate/explorer
```

2. **Install dependencies:**
```bash
npm install
```

3. **Set up environment:**
```bash
cp .env.example .env
# Edit .env with your configuration
```

4. **Run setup script:**
```bash
./setup.sh
```

This will:
- Start PostgreSQL (via Docker)
- Run database migrations
- Generate Prisma client
- Optionally start the explorer

### Manual Setup

1. **Start PostgreSQL:**
```bash
docker run -d -p 5432:5432 \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=citrate_explorer \
  postgres:15-alpine
```

2. **Run migrations:**
```bash
npx prisma migrate dev
npx prisma generate
```

3. **Start indexer:**
```bash
npm run indexer:dev
```

4. **Start explorer:**
```bash
npm run dev
```

### Docker Compose

```bash
docker-compose up
```

This starts:
- PostgreSQL database
- Redis cache
- Explorer web interface
- Indexer service

## Architecture

### Components

```
explorer/
├── src/
│   ├── app/              # Next.js app router
│   │   ├── api/          # API routes
│   │   └── (pages)/      # UI pages
│   ├── components/       # React components
│   │   ├── home/         # Homepage components
│   │   ├── search/       # Search components
│   │   └── layout/       # Layout components
│   ├── indexer/          # Chain indexer service
│   └── lib/              # Utilities
├── prisma/               # Database schema
└── public/              # Static assets
```

### Database Schema

- **Blocks**: Block data with GhostDAG fields
- **Transactions**: Transaction records with type classification
- **Models**: AI model registry
- **Inferences**: Inference execution history
- **DagStats**: Network statistics
- **SearchIndex**: Semantic search index

### API Endpoints

- `GET /api/blocks` - List blocks
- `GET /api/blocks/[hash]` - Get block details
- `GET /api/transactions` - List transactions
- `GET /api/models` - List AI models
- `GET /api/dag` - Get DAG visualization data
- `GET /api/stats` - Network statistics
- `POST /api/search/semantic` - Semantic search

## Configuration

### Environment Variables

```env
# Database
DATABASE_URL="postgresql://postgres:password@localhost:5432/citrate_explorer"

# RPC Endpoint
RPC_ENDPOINT="http://localhost:8545"

# Indexer
INDEXER_ENABLED="true"
INDEXER_START_BLOCK="0"

# Chain
CHAIN_ID="1337"
NETWORK_NAME="Citrate v3 Testnet"
```

### Genesis Model

The genesis model is embedded in the genesis block and provides:
- Text embeddings (128 dimensions)
- Semantic search capabilities
- Transaction intent classification
- Similarity scoring

Model specifications:
- Architecture: BERT-tiny
- Hidden size: 128
- Layers: 4
- Attention heads: 2
- Memory: ~45MB
- Inference time: ~5ms

## Development

### Running Tests
```bash
npm test
```

### Building for Production
```bash
npm run build
npm start
```

### Database Management
```bash
# Create migration
npx prisma migrate dev --name your_migration_name

# Reset database
npx prisma migrate reset

# View database
npx prisma studio
```

## Features in Detail

### 3D DAG Visualization
- Nodes positioned by blue score in Y-axis
- Block number determines X-axis spread
- Random Z-axis depth for 3D effect
- Interactive controls (orbit, zoom, pan)
- Color coding: Blue blocks (blue), Red blocks (red)
- Node size based on transaction count

### Semantic Search
- Powered by genesis BERT-tiny model
- Hybrid search combining semantic and keyword matching
- Real-time embeddings generation
- Similarity scoring with visual indicators
- Support for natural language queries

### Model Registry
- Track deployed AI models
- Monitor inference statistics
- View model metadata and permissions
- Track model operations history
- Performance metrics visualization

## Troubleshooting

### Database Connection Issues
```bash
# Check PostgreSQL status
docker ps | grep postgres

# View logs
docker logs lattice-explorer-db
```

### Indexer Not Syncing
```bash
# Check indexer logs
npm run indexer:dev

# Reset and resync
npx prisma migrate reset
npm run indexer
```

### Port Conflicts
```bash
# Change ports in .env
DATABASE_URL="postgresql://postgres:password@localhost:5433/citrate_explorer"
```

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## License

MIT License - see LICENSE file for details

## Support

- GitHub Issues: [citrate/issues](https://github.com/lattice/citrate/issues)
- Documentation: [docs.lattice.xyz](https://docs.lattice.xyz)
- Discord: [discord.gg/lattice](https://discord.gg/lattice)