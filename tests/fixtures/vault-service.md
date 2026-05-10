---
type: reference
topic: persistence
last_updated: 2026-05-02
---

# Storefront Persistence Service

## What This Service Does

Consumes product catalog data from SQS queues and persists it into MongoDB Atlas.
After persistence, publishes change events to Kafka for downstream consumers.

In one sentence: SQS to MongoDB + Kafka event publishing.

## Architecture

```
SQS Queues --> JmsListener --> Processor --> MongoDB
                                         --> KafkaEventProducer
```

### Queue Configuration

| Queue | Entity Type | Tier |
|-------|------------|------|
| product-attributes-t1 | attributes | 1 |
| product-attributes-t2 | attributes | 2 |
| product-prices-t1 | prices | 1 |
| product-prices-t2 | prices | 2 |

### MongoDB Collections

- `productAttributes` -- main product data
- `productPrices` -- pricing information
- `productVariants` -- variant configurations

## Deployment

Deployed on both PaaS (Jenkins) and SaaS (Flex/ArgoCD).

### Environment Variables

```yaml
MONGO_URI: mongodb+srv://...
SQS_ENDPOINT: https://sqs.us-east-1.amazonaws.com
KAFKA_BOOTSTRAP: pipeline.adobedc.net:443
```

## Troubleshooting

### High SQS Backlog

Check consumer health via `/actuator/health` endpoint.
Monitor `sqs.messages.received` counter in Grafana.

### MongoDB Timeout

Verify connection pool settings:
- `maxPoolSize`: 50 (default)
- `connectTimeoutMS`: 10000
