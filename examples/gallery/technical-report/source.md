---
title: Latency Regression Postmortem
subtitle: Checkout service, incident INC-1183
author: Reliability Engineering
date: 2026-05-28
---

# Summary

On 2026-05-26 the checkout service experienced elevated p99 latency (480 ms →
2.1 s) for 47 minutes, affecting an estimated 12% of purchase attempts. The root
cause was an unbounded connection pool interacting with a slow downstream.[^scope]

[^scope]: Impact estimated from the difference between expected and observed
completed-checkout counts during the window.

# Timeline

| Time (UTC) | Event                                             |
|:-----------|:--------------------------------------------------|
| 14:02      | Deploy of checkout `v3.8.0`                        |
| 14:09      | p99 latency alert fires                            |
| 14:18      | On-call engaged; downstream identified as degraded|
| 14:49      | Connection pool cap lowered; latency recovers      |

# Root cause

The release raised the database pool ceiling from 50 to 500. When the payments
provider slowed, requests held connections while waiting, exhausting the database
and creating a feedback loop.

```text
client → checkout → [pool: 500] → db
                         │
                  payments (slow) ──► held connections pile up
```

# Corrective actions

- [x] Cap the pool at 80 with a 250 ms acquire timeout
- [x] Add a circuit breaker around the payments client
- [ ] Load-test the degraded-downstream scenario in staging
- [ ] Add a runbook entry for pool exhaustion

# Lessons

> Capacity limits are safety limits. A larger pool did not add headroom; it
> removed the backpressure that previously contained a downstream failure.
