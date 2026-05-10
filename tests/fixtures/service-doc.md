---
type: reference
topic: ingestion
last_updated: 2026-04-18
---

# Nexus Ingestion Pipeline

## Overview

Receives upstream events from message queues and transforms them into document records.
Publishes completion notifications downstream for indexing consumers.

In one sentence: Queue consumer to document store plus event fanout.

## Architecture

```
Message Queues --> EventListener --> Transformer --> DocumentStore
                                                 --> NotificationProducer
```

### Queue Bindings

| Queue | Entity Type | Priority |
|-------|------------|----------|
| entity-alpha-p1 | alpha | 1 |
| entity-alpha-p2 | alpha | 2 |
| entity-beta-p1 | beta | 1 |
| entity-beta-p2 | beta | 2 |

### Document Collections

- `entityAlpha` -- primary entity records
- `entityBeta` -- secondary classification data
- `entityGamma` -- variant configurations

## Deployment

Deployed via CI pipeline to staging and production clusters.

### Environment Variables

```yaml
STORE_URI: docstore+srv://cluster.example.net
QUEUE_ENDPOINT: https://queue.region.example.com
NOTIFY_BOOTSTRAP: notify.internal.example.net:9093
```

## Troubleshooting

### High Queue Backlog

Check consumer health via `/health/ready` endpoint.
Monitor `queue.messages.consumed` counter in the dashboard.

### Store Timeout

Verify connection pool settings:
- `maxPoolSize`: 50 (default)
- `connectTimeoutMS`: 10000
