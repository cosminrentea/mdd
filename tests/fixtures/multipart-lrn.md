# Learnings 2026-04-22

---
type: reference
level: project
topic: performance tuning
weight: 0.6
weight-reason: quantifies client refactoring throughput gains
agent: main
---

### HTTP Client Bulk Throughput Improvement

Bulk POST rate improved from 24 to 98 items/s (4.1x gain).

Root causes:
- maxConcurrency raised from 5 to 10
- Client refactored to share rate limiter across async operators
- Total sync time: 40s to 24s (400 items)

### Thread Pool Utilization

- Threads used: 8 of 10 max (vs 5 previously)
- Object read: 2300 items in 2.1s
- Transform: 0.1s
- Bulk POST: 4.1s

---
type: note
level: personal
topic: message broker configuration
weight: 0.4
weight-reason: common gotcha with consumer library and broker version 3.9.x
agent: main
---

### Consumer Library Broker 3.9 Incompatibility

parallel-consumer 0.5.x + broker 3.9.x: Reflective access to internal coordinator field fails.

Workaround: `ignoreReflectiveAccessExceptions(true)` in test config.

Alternative: pin broker-client library to 3.8.1.

---
type: reference
level: team
topic: deployment
weight: 0.7
weight-reason: documents critical notification service auth requirement
agent: main
---

### Notification Service Authentication

The notification service requires token-based authentication:
- clientId, clientSecret, clientCode in config
- QA/STG bootstrap: `notify-corp.int.example.net:443`
- PROD bootstrap: `notify-secure.prd.example.net:9097`

Secure endpoints only reachable from inside cluster pods.
Topics cannot auto-create and must be manually provisioned.
