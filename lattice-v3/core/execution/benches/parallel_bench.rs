use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lattice_consensus::types::{Hash, PublicKey, Signature, Transaction};
use std::collections::HashMap;

fn mk_tx(sender: u8, nonce: u64) -> Transaction {
    let mut h = [0u8; 32];
    h[0] = sender;
    h[1..9].copy_from_slice(&nonce.to_le_bytes());
    Transaction {
        hash: Hash::new(h),
        nonce,
        from: PublicKey::new([sender; 32]),
        to: None,
        value: 0,
        gas_limit: 21000,
        gas_price: 1_000_000_000,
        data: vec![],
        signature: Signature::new([1; 64]),
        tx_type: None,
    }
}

// Local helpers to avoid cross-crate imports for benches
fn group_by_sender_sequential(txs: &[Transaction]) -> Vec<Vec<&Transaction>> {
    let mut map: HashMap<PublicKey, Vec<&Transaction>> = HashMap::new();
    for tx in txs {
        map.entry(tx.from).or_default().push(tx);
    }
    // Sort by nonce within each sender for determinism
    for v in map.values_mut() {
        v.sort_by_key(|t| t.nonce);
    }
    map.into_values().collect()
}

fn plan_round_robin<'a>(groups: &'a [Vec<&'a Transaction>]) -> Vec<&'a Transaction> {
    // Simple round-robin merge of sender groups
    let mut indices = vec![0usize; groups.len()];
    let mut out = Vec::new();
    loop {
        let mut progressed = false;
        for (i, g) in groups.iter().enumerate() {
            if indices[i] < g.len() {
                out.push(g[indices[i]]);
                indices[i] += 1;
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
    }
    out
}

fn gen_txs(senders: usize, per_sender: usize) -> Vec<Transaction> {
    let mut v = Vec::with_capacity(senders * per_sender);
    for s in 0..senders {
        for n in 0..per_sender {
            v.push(mk_tx((s % 250) as u8, n as u64));
        }
    }
    // Shuffle not necessary; grouping sorts per sender
    v
}

fn bench_grouping(c: &mut Criterion) {
    let txs = gen_txs(64, 128);
    c.bench_function("group_by_sender_sequential 64x128", |b| {
        b.iter(|| {
            let groups = group_by_sender_sequential(black_box(&txs));
            black_box(groups);
        })
    });
}

fn bench_round_robin(c: &mut Criterion) {
    let txs = gen_txs(64, 128);
    let groups = group_by_sender_sequential(&txs);
    c.bench_function("plan_round_robin 64x128", |b| {
        b.iter(|| {
            let plan = plan_round_robin(black_box(&groups));
            black_box(plan);
        })
    });
}

criterion_group!(benches, bench_grouping, bench_round_robin);
criterion_main!(benches);
