use criterion::{criterion_group, criterion_main, Criterion, black_box};
use lattice_execution::parallel::{group_by_sender_sequential, plan_round_robin};
use lattice_consensus::types::{Transaction, PublicKey, Hash, Signature};

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
    }
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

