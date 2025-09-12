# Sprint 4: Network & P2P Layer Implementation

## Overview
Implement the peer-to-peer networking layer for block and transaction propagation, peer discovery, and network synchronization.

## Components to Implement

### 1. Network Core (`core/network/src/`)

#### `protocol.rs` - Wire Protocol Definition
- Message types (Block, Transaction, GetBlocks, etc.)
- Protocol versioning
- Message serialization/deserialization
- Handshake protocol

#### `peer.rs` - Peer Management
- Peer connection handling
- Peer state tracking
- Connection lifecycle management
- Peer scoring and reputation

#### `discovery.rs` - Peer Discovery
- Bootstrap node connectivity
- DHT-based peer discovery
- Peer exchange protocol
- Network topology management

#### `sync.rs` - Chain Synchronization
- Initial block download (IBD)
- Header-first synchronization
- Block request/response handling
- State sync for fast sync

#### `gossip.rs` - Gossip Protocol
- Transaction propagation
- Block announcement
- Gossip validation
- Broadcast optimization

#### `rpc.rs` - RPC Interface
- JSON-RPC server
- WebSocket support
- API endpoints for wallet/dapp interaction
- Event subscriptions

### 2. Message Types

```rust
pub enum NetworkMessage {
    // Handshake
    Hello { version: u32, genesis: Hash, height: u64 },
    Ack { peer_id: PeerId },
    
    // Blocks
    NewBlock(Block),
    GetBlocks { from: Hash, count: u32 },
    Blocks(Vec<Block>),
    
    // Transactions
    NewTransaction(Transaction),
    GetMempool,
    Mempool(Vec<Hash>),
    
    // Sync
    GetHeaders { from: Hash, count: u32 },
    Headers(Vec<BlockHeader>),
    
    // Discovery
    GetPeers,
    Peers(Vec<PeerInfo>),
}
```

### 3. Network Configuration

```rust
pub struct NetworkConfig {
    pub listen_addr: SocketAddr,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
    pub enable_discovery: bool,
    pub gossip_interval: Duration,
}
```

## Implementation Order

1. **Basic Networking**
   - TCP/UDP transport layer
   - Message framing and serialization
   - Basic peer connections

2. **Peer Management**
   - Connection pool
   - Peer state machine
   - Handshake protocol

3. **Message Handling**
   - Message routing
   - Request/response patterns
   - Event propagation

4. **Discovery & Gossip**
   - Peer discovery mechanisms
   - Gossip protocol for tx/block propagation
   - Network topology optimization

5. **Synchronization**
   - Block synchronization
   - State sync
   - Fork handling

6. **RPC Interface**
   - JSON-RPC server
   - WebSocket subscriptions
   - API documentation

## Testing Strategy

1. **Unit Tests**
   - Message serialization
   - Peer state transitions
   - Protocol compliance

2. **Integration Tests**
   - Multi-node network simulation
   - Sync scenarios
   - Network partitioning

3. **Performance Tests**
   - Throughput benchmarks
   - Latency measurements
   - Scalability testing

## Dependencies

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
libp2p = "0.53"  # Or custom implementation
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
futures = "0.3"
tracing = "0.1"
```

## Success Criteria

- [ ] Peers can discover and connect to each other
- [ ] Transactions propagate across the network
- [ ] Blocks are synchronized between nodes
- [ ] Network handles disconnections gracefully
- [ ] RPC interface allows external interaction
- [ ] All tests pass with >80% coverage