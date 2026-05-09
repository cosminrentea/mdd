# Learnings 2026-05-08

---
type: reference
level: project
topic: pes pb performance
weight: 0.6
weight-reason: quantifies the impact of WebClient refactoring; useful for future concurrency tuning
agent: main

---

### PES PB WebClient Refactoring: 4.1x Bulk POST Throughput

Commits `e67e728..d2832f2` on branch `pb1-2` validated via e2e on PaaS QA (2026-05-08):

- Bulk POST rate: 23.7 -> **97.0 products/s** (4.1x improvement)
- Root causes: `maxConcurrency` raised from 5 to 10 (env `PES_PB_MAX_CONCURRENCY`), WebClient refactored to share rate limiter across reactive operators via `RateLimiterOperator.of(rateLimiter)` in `Flux.flatMap`
- PB-side latency also dropped from 10-15s to ~4s per batch (appears to be PB infrastructure variability, not client-side)
- Total sync time: 40s -> **24s** (for 397 products, 8 batches of 50)
- Delete rate unchanged at 1.0 deletes/s (rate-limited, sequential) -- PB has no batch DELETE API
- Reactor threads used: 8 (of 10 max) vs 5 previously
- S3 read: 2333 products in 2.1s; transform: 0.1s; bulk POST: 4.1s; reconcile list: 3.5s
