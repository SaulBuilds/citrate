---
id: monitoring
title: Node Monitoring
---

This guide sets up Prometheus + Grafana for Lattice nodes (docker compose profiles included).

- Start devnet node: `scripts/lattice.sh docker up`
- Start monitoring: `scripts/lattice.sh docker monitoring up`

Services
- Prometheus: http://localhost:9090 (scrapes `lattice-node-devnet:9100` and cluster nodes)
- Grafana: http://localhost:3001 (admin/admin)
  - Dashboards provisioned under `lattice-v3/monitoring/grafana/dashboards` (e.g., Lattice Node Overview)

Cluster monitoring
- Start 5-node cluster: `scripts/lattice.sh docker cluster up`
- Start monitoring stack: `scripts/lattice.sh docker monitoring up`

Customize
- Edit `lattice-v3/monitoring/prometheus.yml` to add/remove scrape targets.
- Grafana datasource is provisioned to Prometheus at `prometheus:9090`.
