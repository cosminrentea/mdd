# Learnings 2026-05-08

---
type: reference
level: project
topic: performance tuning
weight: 0.6
weight-reason: quantifies WebClient refactoring impact
agent: main
---

### WebClient Bulk POST Throughput Improvement

Bulk POST rate improved from 23.7 to 97.0 products/s (4.1x).

Root causes:
- maxConcurrency raised from 5 to 10
- WebClient refactored to share rate limiter across reactive operators
- Total sync time: 40s to 24s (397 products)

### Reactor Thread Usage

- Threads used: 8 of 10 max (vs 5 previously)
- S3 read: 2333 products in 2.1s
- Transform: 0.1s
- Bulk POST: 4.1s

---
type: note
level: personal
topic: kafka configuration
weight: 0.4
weight-reason: common gotcha with parallel-consumer and Kafka 3.9.x
agent: main
---

### ParallelConsumer Kafka 3.9 Incompatibility

parallel-consumer 0.5.x + Kafka 3.9.x: Reflective access to `KafkaConsumer.coordinator.autoCommitEnabled` fails.

Workaround: `ignoreReflectiveAccessExceptionsForAutoCommitDisabledCheck(true)` in tests.

Alternative: pin kafka-clients to 3.8.1.

---
type: reference
level: team
topic: deployment
weight: 0.7
weight-reason: documents critical Adobe Pipeline auth requirement
agent: main
---

### Adobe Pipeline IMS Authentication

Adobe Pipeline (Kafka) requires IMS authentication:
- clientId, clientSecret, clientCode in config
- QA/STG bootstrap: `*-corp.int.pipeline.adobedc.net:443`
- PROD bootstrap: `*-secure.prd.pipeline.adobedc.net:9097`

Secure endpoints only reachable from inside k8s pods.
Topics cannot auto-create and must be manually provisioned.
