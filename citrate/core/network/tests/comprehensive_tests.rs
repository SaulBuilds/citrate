// Comprehensive tests for the network module

use citrate_network::{
    peer::Direction, NetworkMessage, Peer, PeerId, PeerInfo, PeerManager, PeerManagerConfig,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

#[cfg(test)]
mod network_tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_manager_creation() {
        let config = PeerManagerConfig::default();
        let peer_manager = PeerManager::new(config);

        let peers = peer_manager.get_all_peers();
        assert_eq!(peers.len(), 0);
    }

    #[tokio::test]
    async fn test_peer_id_generation() {
        let id1 = PeerId::random();
        let id2 = PeerId::random();

        // Each peer should have unique ID
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_add_peer() {
        let config = PeerManagerConfig::default();
        let pm = PeerManager::new(config);
        let peer_id = PeerId::random();
        let addr: SocketAddr = "127.0.0.1:9002".parse().unwrap();

        // Create channels for the peer
        let (send_tx, recv_rx) = mpsc::channel(100);
        let peer_info = PeerInfo::new(peer_id.clone(), addr, Direction::Outbound);
        let peer = Arc::new(Peer::new(peer_info, send_tx, recv_rx));

        let result = pm.add_peer(peer).await;
        assert!(result.is_ok());

        let peers = pm.get_all_peers();
        assert_eq!(peers.len(), 1);
        assert!(pm.get_peer(&peer_id).is_some());
    }

    #[tokio::test]
    async fn test_remove_peer() {
        let config = PeerManagerConfig::default();
        let pm = PeerManager::new(config);
        let peer_id = PeerId::random();
        let addr: SocketAddr = "127.0.0.1:9004".parse().unwrap();

        // Create and add a peer
        let (send_tx, recv_rx) = mpsc::channel(100);
        let peer_info = PeerInfo::new(peer_id.clone(), addr, Direction::Outbound);
        let peer = Arc::new(Peer::new(peer_info, send_tx, recv_rx));
        pm.add_peer(peer).await.unwrap();
        assert_eq!(pm.get_all_peers().len(), 1);

        pm.remove_peer(&peer_id).await;
        assert_eq!(pm.get_all_peers().len(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let config = PeerManagerConfig::default();
        let pm = PeerManager::new(config);

        // Add some peers
        for i in 0..3 {
            let peer_id = PeerId::random();
            let addr: SocketAddr = format!("127.0.0.1:900{}", 6 + i).parse().unwrap();
            let (send_tx, recv_rx) = mpsc::channel(100);
            let peer_info = PeerInfo::new(peer_id, addr, Direction::Outbound);
            let peer = Arc::new(Peer::new(peer_info, send_tx, recv_rx));
            pm.add_peer(peer).await.unwrap();
        }

        // Broadcast a message
        let msg = NetworkMessage::Ping { nonce: 12345 };
        let result = pm.broadcast(&msg).await;

        // Should attempt to send to all peers
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_peer_connection_limit() {
        let config = PeerManagerConfig {
            max_peers: 5,
            ..Default::default()
        };
        let pm = PeerManager::new(config);

        // Try to add many peers
        for i in 0..10 {
            let peer_id = PeerId::random();
            let addr: SocketAddr = format!("127.0.0.1:{}", 9100 + i).parse().unwrap();
            let (send_tx, recv_rx) = mpsc::channel(100);
            let peer_info = PeerInfo::new(peer_id, addr, Direction::Outbound);
            let peer = Arc::new(Peer::new(peer_info, send_tx, recv_rx));
            let _ = pm.add_peer(peer).await;
        }

        // Should respect max peer limit
        let peers = pm.get_all_peers();
        assert!(peers.len() <= 5);
    }

    #[tokio::test]
    async fn test_network_message_types() {
        // Test different message types can be created
        let ping = NetworkMessage::Ping { nonce: 1 };
        let pong = NetworkMessage::Pong { nonce: 1 };

        match ping {
            NetworkMessage::Ping { nonce } => assert_eq!(nonce, 1),
            _ => panic!("Wrong message type"),
        }

        match pong {
            NetworkMessage::Pong { nonce } => assert_eq!(nonce, 1),
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_peer_operations() {
        let config = PeerManagerConfig::default();
        let pm = Arc::new(PeerManager::new(config));

        let mut handles = vec![];

        // Spawn tasks to add peers concurrently
        for i in 0..10 {
            let pm_clone = pm.clone();
            let handle = tokio::spawn(async move {
                let peer_id = PeerId::random();
                let addr: SocketAddr = format!("127.0.0.1:{}", 9200 + i).parse().unwrap();
                let (send_tx, recv_rx) = mpsc::channel(100);
                let peer_info = PeerInfo::new(peer_id, addr, Direction::Outbound);
                let peer = Arc::new(Peer::new(peer_info, send_tx, recv_rx));
                pm_clone.add_peer(peer).await
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let _ = handle.await;
        }

        let peers = pm.get_all_peers();
        assert!(peers.len() > 0);
    }

    #[tokio::test]
    async fn test_peer_scoring() {
        let config = PeerManagerConfig::default();
        let pm = PeerManager::new(config);
        let peer_id = PeerId::random();
        let addr: SocketAddr = "127.0.0.1:9031".parse().unwrap();

        // Create and add a peer
        let (send_tx, recv_rx) = mpsc::channel(100);
        let peer_info = PeerInfo::new(peer_id.clone(), addr, Direction::Outbound);
        let peer = Arc::new(Peer::new(peer_info, send_tx, recv_rx));
        pm.add_peer(peer.clone()).await.unwrap();

        // Update peer score
        pm.update_peer_score(&peer_id, 10).await;

        // Check the score via the peer's info
        let score = peer.info.read().await.score;
        assert!(score >= 0);
    }

    #[tokio::test]
    async fn test_max_peers_config() {
        let config = PeerManagerConfig {
            max_peers: 10,
            max_inbound: 5,
            max_outbound: 5,
            ..Default::default()
        };

        let pm = PeerManager::new(config.clone());

        // Verify the configuration is applied
        assert_eq!(pm.max_peers(), 10);

        // Test peer counts
        let (total, inbound, outbound) = pm.get_peer_counts().await;
        assert_eq!(total, 0);
        assert_eq!(inbound, 0);
        assert_eq!(outbound, 0);
    }
}
