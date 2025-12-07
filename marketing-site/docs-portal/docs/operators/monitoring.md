---
id: monitoring
title: Node Monitoring
---

This guide sets up Prometheus + Grafana for Citrate nodes (docker compose profiles included).

- Start devnet node: `scripts/lattice.sh docker up`
- Start monitoring: `scripts/lattice.sh docker monitoring up`

Services
- Prometheus: http://localhost:9090 (scrapes `citrate-node-devnet:9100` and cluster nodes)
- Grafana: http://localhost:3001 (admin/admin)
  - Dashboards provisioned under `citrate/monitoring/grafana/dashboards` (e.g., Citrate Node Overview)

Cluster monitoring
- Start 5-node cluster: `scripts/lattice.sh docker cluster up`
- Start monitoring stack: `scripts/lattice.sh docker monitoring up`

Customize
- Edit `citrate/monitoring/prometheus.yml` to add/remove scrape targets.
- Grafana datasource is provisioned to Prometheus at `prometheus:9090`.
