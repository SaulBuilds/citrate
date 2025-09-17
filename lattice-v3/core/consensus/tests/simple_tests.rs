// Simple working tests for consensus module

#[test]
fn test_consensus_module_loads() {
    // This test simply verifies the module can be imported
    use lattice_consensus::*;
    
    // Verify GhostDagParams exists and has expected defaults
    let params = GhostDagParams::default();
    assert_eq!(params.k, 18);
    assert_eq!(params.max_parents, 10);
    assert_eq!(params.max_blue_score_diff, 1000);
    assert_eq!(params.pruning_window, 100000);
    assert_eq!(params.finality_depth, 100);
}