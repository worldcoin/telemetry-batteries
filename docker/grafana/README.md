# Grafana Observability Stack

Local Grafana stack for testing metrics and logs.

## Components

| Service | Port | Purpose |
|---------|------|---------|
| Grafana | 3000 | Dashboards & visualization |
| Prometheus | 9090 | Metrics storage & scraping |
| Loki | 3100 | Log aggregation |
| Promtail | - | Log collector (scrapes Docker logs) |

## Setup

1. Start the stack:
   ```bash
   docker compose up -d
   ```

2. Run your app with Prometheus metrics:
   ```bash
   TELEMETRY_PRESET=local \
   TELEMETRY_METRICS_BACKEND=prometheus \
   TELEMETRY_PROMETHEUS_LISTEN=0.0.0.0:9090 \
   cargo run --example basic --features metrics-prometheus
   ```

3. Open Grafana: http://localhost:3000
   - Default login: admin/admin
   - Anonymous access is enabled

## Datasources

Pre-configured:
- **Prometheus** - Metrics from your app
- **Loki** - Container logs

## Verify

Check Prometheus is scraping your app:
```bash
curl http://localhost:9090/api/v1/targets | jq '.data.activeTargets[] | {job: .labels.job, health: .health}'
```

Check Loki is receiving logs:
```bash
curl -G http://localhost:3100/loki/api/v1/labels
```

## Notes

- Your app's metrics endpoint must be accessible from Docker
- On macOS/Windows, `host.docker.internal` resolves to host machine
- On Linux, you may need `--add-host=host.docker.internal:host-gateway`
