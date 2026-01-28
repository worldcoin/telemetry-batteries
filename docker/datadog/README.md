# Datadog Agent Setup

Local Datadog Agent for testing trace export.

## Setup

1. Copy `.env.example` to `.env` and add your API key:
   ```bash
   cp .env.example .env
   # Edit .env with your DD_API_KEY
   ```

2. Start the agent:
   ```bash
   docker compose up -d
   ```

3. Run your app with Datadog preset:
   ```bash
   TELEMETRY_PRESET=datadog \
   TELEMETRY_SERVICE_NAME=my-app \
   cargo run --example basic
   ```

## Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 8126 | TCP | APM traces |
| 8125 | UDP | DogStatsD metrics |

## Verify

Check the agent is receiving traces:
```bash
docker logs datadog-agent 2>&1 | grep -i trace
```

View in Datadog UI: https://app.datadoghq.com/apm/traces
