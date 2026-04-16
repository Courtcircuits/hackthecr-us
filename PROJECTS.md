# Project Ideas — Distributed Systems Challenges

## 1. Federated Multi-Node API with Gossip-Based Sync

The most ambitious option, inspired by the kind of distributed systems challenges found in data-processing-at-scale courses.

### Concept

Deploy multiple API nodes across simulated regions, each owning a shard of the data. Nodes share updates with each other using a **gossip protocol** — no central coordinator, no master node.

A gossip protocol (also called epidemic protocol) is how distributed nodes share information without anyone being in charge. It works exactly like rumours spread in real life:

1. Node A learns something new
2. Node A randomly picks 2–3 neighbours and tells them
3. Each of those nodes randomly picks 2–3 neighbours and tells them
4. After a logarithmic number of rounds, every node knows

No master. No broadcast. No single point of failure.

### Architecture

Imagine 3 API nodes, one per major CROUS zone (Paris, Lyon, Bordeaux). Each node owns its local data but needs to stay in sync with the others.

```
Node Paris      Node Lyon       Node Bordeaux
   [DB-Paris]     [DB-Lyon]       [DB-Bordeaux]
       |               |                |
       └───── gossip ──┘────── gossip ──┘
```

A crawler scrapes new meals in Paris and pushes them to the Paris node. That node propagates this to Lyon and Bordeaux **without a central coordinator telling it to**.

The gossip cycle runs on each node every N seconds:

```
1. Pick a random peer from the known node list
2. Send: "here's my latest scrape_batch checksum for each region"
3. Peer replies: "here's mine"
4. Both compare — whoever is behind pulls the missing data
```

This is called **anti-entropy**: nodes continuously reconcile differences.

### How existing htc5 code fits in

| Existing piece | Role in gossip architecture |
|---|---|
| `scrape_batch.checksum` | Acts as a **version vector** — nodes compare checksums to detect who is behind without transferring the full dataset |
| Ed25519 signatures on PUT payloads | Acts as a **trust layer** — when Node Lyon receives data gossipped from Node Paris, it verifies the signature to confirm it came from a legitimate crawler, not a rogue node |

### Hard problems this forces you to solve

| Problem | Description |
|---|---|
| **Conflict resolution** | Two crawlers scraped the same restaurant at the same time and got different data. Who wins? (Last-write-wins? Checksum comparison?) |
| **Split-brain** | Lyon and Bordeaux can't reach Paris. They keep serving stale Paris data. When Paris comes back, how do you merge? |
| **Convergence guarantees** | How do you prove all nodes will *eventually* agree, even with packet loss and restarts? |
| **Membership management** | How does a new node announce itself? How do you detect a dead node vs. a slow one? |

### Implementation roadmap

You wouldn't start with full gossip. The natural progression:

**Step 1 — State exchange endpoint**
Each node exposes `GET /sync/state` returning all its `scrape_batch` checksums per region.

**Step 2 — Pull-based anti-entropy**
A background loop on each node periodically calls `/sync/state` on its peers, diffs the checksums, and pulls missing batches. This alone is a working distributed sync system.

**Step 3 — Dynamic membership**
Remove hardcoded peer lists. Nodes announce themselves on startup by gossiping to one known seed node, which spreads the membership info further.

**Step 4 — Failure detection**
If a peer doesn't respond N times, mark it *suspect*, then *dead*, and stop gossiping to it. This is the **SWIM protocol** — the same mechanism used by Consul and Cassandra.

Steps 1–2 alone constitute a solid distributed systems project. Steps 3–4 get you close to production-grade distributed membership.

---

## 2. Distributed Crawler Orchestration

Build a distributed work queue where multiple crawler workers race to claim CROUS regions:

- Use PostgreSQL's `SKIP LOCKED` or Redis streams as the work queue
- Add leader election so only one coordinator schedules batches
- Track worker heartbeats, handle failures and requeuing

## 3. Real-time Streaming Pipeline with Anomaly Detection

Replace batch scraping with an event-driven architecture:

- Each scraped meal becomes an event published to a message broker (Kafka, NATS, or Redpanda)
- A stream processor detects anomalies: sudden menu changes, restaurant closures, unusual meal patterns
- Downstream consumers update the DB, send Discord alerts, populate the search index

## 4. Geospatial Proximity Engine with H3

Your project already has GPS coordinates for schools and restaurants:

- Build a spatial index using **H3 hexagonal cells** over the restaurants and schools tables
- Expose a `GET /nearby?lat=X&lon=Y&radius=500m` endpoint
- Handle multi-resolution queries (broad search → refine) with PostGIS or a custom in-memory index

## 5. Event-Sourced Meal History with Time-Travel Queries

Redesign the write path as an append-only event log:

- Record every change (meal appeared, meal disappeared, data updated) as an immutable event
- Add a projection layer that rebuilds current state from events
- Expose historical queries: `GET /{region}/meals/{name}?at=2025-01-15`
- Forces you to solve eventual consistency between the event log and read models
