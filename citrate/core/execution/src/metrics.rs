// citrate/core/execution/src/metrics.rs

// Metrics for tracking execution and precompile calls
use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram, CounterVec, Histogram};

pub static VM_EXECUTIONS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "lattice_vm_executions_total",
        "Number of VM execution calls",
        &["status"]
    )
    .expect("register lattice_vm_executions_total")
});

pub static VM_GAS_USED: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!("lattice_vm_gas_used", "Gas used by VM execution")
        .expect("register lattice_vm_gas_used")
});

pub static PRECOMPILE_CALLS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "lattice_precompile_calls_total",
        "Total precompile calls",
        &["precompile", "method", "status"]
    )
    .expect("register lattice_precompile_calls_total")
});
