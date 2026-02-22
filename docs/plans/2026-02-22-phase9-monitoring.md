# Phase 9 — Monitoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Prometheus metrics to the Rust engine and set up Grafana dashboards for full-stack observability.

**Architecture:** The Rust engine exposes a `/metrics` Prometheus endpoint on its existing Axum server. Prometheus scrapes the engine plus exporters for PostgreSQL, Redis, and Kafka. Grafana is auto-provisioned with a single "CraftStrat Overview" dashboard.

**Tech Stack:** metrics + metrics-exporter-prometheus (Rust), Prometheus, Grafana, postgres_exporter, redis_exporter, kafka_exporter

---

### Task 1: Add metrics dependencies and create metrics module

**Files:**
- Modify: `engine/Cargo.toml`
- Create: `engine/src/metrics.rs`

**Step 1: Add dependencies to Cargo.toml**

Add to `[dependencies]` in `engine/Cargo.toml`:

```toml
metrics = "0.24"
metrics-exporter-prometheus = "0.16"
```

**Step 2: Create the metrics module**

Create `engine/src/metrics.rs`:

```rust
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

pub fn init() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus metrics recorder")
}
```

**Step 3: Register the module in main.rs**

In `engine/src/main.rs`, add the module declaration after the existing ones:

```rust
mod metrics;
```

---

### Task 2: Wire /metrics endpoint into Axum API

**Files:**
- Create: `engine/src/api/handlers/metrics.rs`
- Modify: `engine/src/api/handlers/mod.rs`
- Modify: `engine/src/api/state.rs`
- Modify: `engine/src/api/mod.rs`
- Modify: `engine/src/main.rs`

**Step 1: Create the metrics handler**

Create `engine/src/api/handlers/metrics.rs`:

```rust
use std::sync::Arc;

use axum::extract::State;

use crate::api::state::ApiState;

pub async fn render(State(state): State<Arc<ApiState>>) -> String {
    state.prometheus.render()
}
```

**Step 2: Register the handler module**

In `engine/src/api/handlers/mod.rs`, add:

```rust
pub mod metrics;
```

**Step 3: Add PrometheusHandle to ApiState**

In `engine/src/api/state.rs`, add the import and field:

```rust
use metrics_exporter_prometheus::PrometheusHandle;
```

Add the field to the `ApiState` struct:

```rust
pub prometheus: PrometheusHandle,
```

**Step 4: Add the /metrics route**

In `engine/src/api/mod.rs`, add the route to the `router()` function, before `.with_state(state)`:

```rust
.route("/metrics", get(handlers::metrics::render))
```

**Step 5: Initialize metrics and pass handle to ApiState**

In `engine/src/main.rs`, initialize metrics early (before `spawn_all`), and add the handle to `ApiState`:

After `tracing_subscriber::fmt::init();`, add:

```rust
let prometheus_handle = metrics::init();
```

In the `ApiState` construction, add the field:

```rust
prometheus: prometheus_handle,
```

**Step 6: Verify it compiles**

Run: `cd engine && cargo check`
Expected: Compiles with no errors.

**Step 7: Commit**

```
feat(engine): add Prometheus /metrics endpoint

Add metrics + metrics-exporter-prometheus crates. Create /metrics
Axum handler that renders Prometheus exposition format.
```

---

### Task 3: Instrument strategy engine

**Files:**
- Modify: `engine/src/strategy/engine.rs`

**Step 1: Add tick and signal counters to the engine loop**

In `engine/src/strategy/engine.rs`, add import at top:

```rust
use metrics::{counter, histogram};
```

After the tick is deserialized successfully (after the `let tick: Tick = match ...` block, before reading assignments), add:

```rust
counter!("craftstrat_ticks_total").increment(1);
```

Wrap the Rayon parallel dispatch in a timing measurement. Replace the existing `let signals: Vec<EngineOutput> = assignments...collect();` block with:

```rust
let eval_start = std::time::Instant::now();
let signals: Vec<EngineOutput> = assignments
    .par_iter()
    .filter_map(|a| {
        let mut state = match a.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!(
                    wallet_id = a.wallet_id,
                    strategy_id = a.strategy_id,
                    "mutex_poisoned_recovering"
                );
                poisoned.into_inner()
            }
        };
        let signal = interpreter::evaluate(&a.graph, &tick, &mut state);
        match signal {
            Signal::Hold => None,
            s => Some(EngineOutput {
                wallet_id: a.wallet_id,
                strategy_id: a.strategy_id,
                symbol: tick.symbol.clone(),
                signal: s,
            }),
        }
    })
    .collect();
histogram!("craftstrat_strategy_eval_duration_seconds").record(eval_start.elapsed().as_secs_f64());
```

After the `for output in signals` loop, add signal counters. Replace the loop with:

```rust
for output in signals {
    let signal_type = match &output.signal {
        Signal::Buy { .. } => "buy",
        Signal::Sell { .. } => "sell",
        Signal::Hold => "hold",
    };
    counter!("craftstrat_signals_total", "signal" => signal_type.to_string()).increment(1);

    tracing::info!(
        wallet_id = output.wallet_id,
        strategy_id = output.strategy_id,
        symbol = %output.symbol,
        signal = ?output.signal,
        "strategy_signal"
    );
    if signal_tx.send(output).await.is_err() {
        tracing::info!("signal_channel_closed");
        return Ok(());
    }
}
```

**Step 2: Verify it compiles**

Run: `cd engine && cargo check`

**Step 3: Commit**

```
feat(engine): instrument strategy engine with Prometheus metrics

Add craftstrat_ticks_total counter, craftstrat_signals_total counter (by signal
type), and craftstrat_strategy_eval_duration_seconds histogram.
```

---

### Task 4: Instrument execution pipeline

**Files:**
- Modify: `engine/src/execution/executor.rs`
- Modify: `engine/src/tasks/execution_tasks.rs`

**Step 1: Add order and execution duration metrics to executor**

In `engine/src/execution/executor.rs`, add import at top:

```rust
use metrics::{counter, gauge, histogram};
```

In the `run()` function, wrap the `submitter.submit(&order)` call with timing. Replace:

```rust
let result = match submitter.submit(&order).await {
```

With:

```rust
let exec_start = std::time::Instant::now();
let result = match submitter.submit(&order).await {
```

After the `match submitter.submit` block completes (after the closing `};` of the match), add:

```rust
histogram!("craftstrat_order_execution_duration_seconds").record(exec_start.elapsed().as_secs_f64());
let status_label = match result.status {
    OrderStatus::Filled => "filled",
    OrderStatus::Cancelled => "cancelled",
    OrderStatus::Failed => "failed",
    OrderStatus::Timeout => "timeout",
};
counter!("craftstrat_orders_total", "status" => status_label.to_string()).increment(1);
```

In the `update_position` function, after `state.pnl += pnl;`, add the PnL gauge update:

```rust
gauge!("craftstrat_pnl_usdc").increment(pnl);
```

(This requires adding `use metrics::{counter, gauge, histogram};` which is already done above.)

**Step 2: Add copy trade metrics to the signal bridge**

In `engine/src/tasks/execution_tasks.rs`, no signal-level metrics needed here (already counted in engine.rs). Skip.

**Step 3: Verify it compiles**

Run: `cd engine && cargo check`

**Step 4: Commit**

```
feat(engine): instrument execution with order and PnL metrics

Add craftstrat_orders_total counter (by status), craftstrat_order_execution_duration_seconds
histogram, and craftstrat_pnl_usdc gauge.
```

---

### Task 5: Instrument watcher and registry

**Files:**
- Modify: `engine/src/watcher/polymarket.rs`
- Modify: `engine/src/strategy/registry.rs`

**Step 1: Add copy trade counter to watcher**

In `engine/src/watcher/polymarket.rs`, add import at top:

```rust
use metrics::counter;
```

In the `run()` function, inside the loop where copy orders are pushed to the queue, after `q.push(order);` add:

```rust
counter!("craftstrat_copy_trades_total", "status" => "queued").increment(1);
```

In the `None` branch (skipped trades), after the `write_copy_trade` call, add:

```rust
counter!("craftstrat_copy_trades_total", "status" => "skipped").increment(1);
```

**Step 2: Add registry gauge updates**

In `engine/src/strategy/registry.rs`, add import at top:

```rust
use metrics::gauge;
```

At the end of the `activate()` function (after the `tracing::info!`), add:

```rust
let (wallets, assignments) = count_registry(&registry).await;
gauge!("craftstrat_active_wallets").set(wallets as f64);
gauge!("craftstrat_active_assignments").set(assignments as f64);
```

At the end of the `deactivate()` function (after the `tracing::info!`), add:

```rust
let (wallets, assignments) = count_registry(&registry).await;
gauge!("craftstrat_active_wallets").set(wallets as f64);
gauge!("craftstrat_active_assignments").set(assignments as f64);
```

Add a helper function after `deactivate()`:

```rust
async fn count_registry(registry: &AssignmentRegistry) -> (usize, usize) {
    let reg = registry.read().await;
    let mut wallet_ids = std::collections::HashSet::new();
    let mut assignment_count = 0usize;
    for assignments in reg.values() {
        for a in assignments {
            wallet_ids.insert(a.wallet_id);
            assignment_count += 1;
        }
    }
    (wallet_ids.len(), assignment_count)
}
```

**Step 3: Verify it compiles**

Run: `cd engine && cargo check`

**Step 4: Run existing tests**

Run: `cd engine && cargo test`
Expected: All existing tests pass (metrics macros are no-ops without an installed recorder in tests).

**Step 5: Commit**

```
feat(engine): instrument watcher and registry with Prometheus metrics

Add craftstrat_copy_trades_total counter (by status) and craftstrat_active_wallets
/ craftstrat_active_assignments gauges updated on activate/deactivate.
```

---

### Task 6: Add Prometheus and exporters to Docker

**Files:**
- Create: `infra/prometheus/prometheus.yml`
- Modify: `docker-compose.yml`

**Step 1: Create Prometheus config**

Create `infra/prometheus/prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'craftstrat-engine'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['engine:8080']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres_exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis_exporter:9121']

  - job_name: 'kafka'
    static_configs:
      - targets: ['kafka_exporter:9308']
```

**Step 2: Add services to docker-compose.yml**

Add after the `grafana` service (before `networks:`):

```yaml
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./infra/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    depends_on:
      - app
    networks:
      - craftstrat

  postgres_exporter:
    image: quay.io/prometheuscommunity/postgres-exporter:latest
    environment:
      DATA_SOURCE_NAME: "postgresql://craftstrat:craftstrat_secret@postgres:5432/craftstrat?sslmode=disable"
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - craftstrat

  redis_exporter:
    image: oliver006/redis_exporter:latest
    environment:
      REDIS_ADDR: "redis://redis:6379"
    depends_on:
      redis:
        condition: service_healthy
    networks:
      - craftstrat

  kafka_exporter:
    image: danielqsj/kafka-exporter:latest
    command: ["--kafka.server=kafka:9092"]
    depends_on:
      - kafka
    networks:
      - craftstrat
```

Add `prometheus_data:` to the `volumes:` section.

**Step 3: Commit**

```
feat(infra): add Prometheus and metric exporters

Add Prometheus server with scrape config for the engine, postgres_exporter,
redis_exporter, and kafka_exporter. All on the craftstrat Docker network.
```

---

### Task 7: Provision Grafana with datasource and dashboard

**Files:**
- Create: `infra/grafana/provisioning/datasources/datasources.yml`
- Create: `infra/grafana/provisioning/dashboards/dashboards.yml`
- Create: `infra/grafana/dashboards/craftstrat-overview.json`
- Modify: `docker-compose.yml` (grafana service)

**Step 1: Create Grafana datasource provisioning**

Create `infra/grafana/provisioning/datasources/datasources.yml`:

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    uid: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
```

**Step 2: Create Grafana dashboard provider**

Create `infra/grafana/provisioning/dashboards/dashboards.yml`:

```yaml
apiVersion: 1

providers:
  - name: CraftStrat
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: false
```

**Step 3: Create the CraftStrat Overview dashboard**

Create `infra/grafana/dashboards/craftstrat-overview.json`:

```json
{
  "dashboard": {
    "id": null,
    "uid": "craftstrat-overview",
    "title": "CraftStrat Overview",
    "tags": ["craftstrat"],
    "timezone": "utc",
    "schemaVersion": 39,
    "version": 1,
    "refresh": "10s",
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "panels": [
      {
        "id": 100,
        "type": "row",
        "title": "Engine",
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 0},
        "collapsed": false
      },
      {
        "id": 1,
        "title": "Ticks / sec",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 0, "y": 1},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "rate(craftstrat_ticks_total[1m])",
            "legendFormat": "",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ops",
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "red", "value": null},
                {"color": "yellow", "value": 0.5},
                {"color": "green", "value": 1}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "area",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      },
      {
        "id": 2,
        "title": "Active Wallets",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 6, "y": 1},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "craftstrat_active_wallets",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "blue", "value": null}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "none",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      },
      {
        "id": 3,
        "title": "Active Strategies",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 12, "y": 1},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "craftstrat_active_assignments",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "purple", "value": null}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "none",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      },
      {
        "id": 4,
        "title": "Uptime",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 18, "y": 1},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "craftstrat_uptime_seconds",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "green", "value": null}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "none",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      },
      {
        "id": 5,
        "title": "Strategy Eval Latency",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 5},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(craftstrat_strategy_eval_duration_seconds_bucket[5m]))",
            "legendFormat": "p50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, rate(craftstrat_strategy_eval_duration_seconds_bucket[5m]))",
            "legendFormat": "p95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.99, rate(craftstrat_strategy_eval_duration_seconds_bucket[5m]))",
            "legendFormat": "p99",
            "refId": "C"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "custom": {
              "drawStyle": "line",
              "fillOpacity": 10,
              "pointSize": 5
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 6,
        "title": "Order Exec Latency",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 12, "y": 5},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "histogram_quantile(0.50, rate(craftstrat_order_execution_duration_seconds_bucket[5m]))",
            "legendFormat": "p50",
            "refId": "A"
          },
          {
            "expr": "histogram_quantile(0.95, rate(craftstrat_order_execution_duration_seconds_bucket[5m]))",
            "legendFormat": "p95",
            "refId": "B"
          },
          {
            "expr": "histogram_quantile(0.99, rate(craftstrat_order_execution_duration_seconds_bucket[5m]))",
            "legendFormat": "p99",
            "refId": "C"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "s",
            "custom": {
              "drawStyle": "line",
              "fillOpacity": 10,
              "pointSize": 5
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 101,
        "type": "row",
        "title": "Trading",
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 13},
        "collapsed": false
      },
      {
        "id": 7,
        "title": "Signals Rate",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 6, "x": 0, "y": 14},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "rate(craftstrat_signals_total[5m])",
            "legendFormat": "{{signal}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ops",
            "custom": {
              "drawStyle": "bars",
              "fillOpacity": 80,
              "stacking": {"mode": "normal"}
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 8,
        "title": "Orders Rate",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 6, "x": 6, "y": 14},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "rate(craftstrat_orders_total[5m])",
            "legendFormat": "{{status}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ops",
            "custom": {
              "drawStyle": "bars",
              "fillOpacity": 80,
              "stacking": {"mode": "normal"}
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 9,
        "title": "Copy Trades Rate",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 6, "x": 12, "y": 14},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "rate(craftstrat_copy_trades_total[5m])",
            "legendFormat": "{{status}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "ops",
            "custom": {
              "drawStyle": "bars",
              "fillOpacity": 80,
              "stacking": {"mode": "normal"}
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 10,
        "title": "PnL (USDC)",
        "type": "stat",
        "gridPos": {"h": 8, "w": 6, "x": 18, "y": 14},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "craftstrat_pnl_usdc",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "currencyUSD",
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "red", "value": null},
                {"color": "green", "value": 0}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "area",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      },
      {
        "id": 102,
        "type": "row",
        "title": "Infrastructure",
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 22},
        "collapsed": false
      },
      {
        "id": 11,
        "title": "Kafka Consumer Lag",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 12, "x": 0, "y": 23},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "kafka_consumergroup_lag",
            "legendFormat": "{{consumergroup}} / {{topic}}",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "custom": {
              "drawStyle": "line",
              "fillOpacity": 20
            },
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 100},
                {"color": "red", "value": 1000}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 12,
        "title": "Redis Memory",
        "type": "timeseries",
        "gridPos": {"h": 8, "w": 6, "x": 12, "y": 23},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "redis_memory_used_bytes",
            "legendFormat": "used",
            "refId": "A"
          },
          {
            "expr": "redis_memory_max_bytes",
            "legendFormat": "max",
            "refId": "B"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "bytes",
            "custom": {
              "drawStyle": "line",
              "fillOpacity": 15
            }
          },
          "overrides": []
        },
        "options": {
          "tooltip": {"mode": "multi"}
        }
      },
      {
        "id": 13,
        "title": "Redis Clients",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 18, "y": 23},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "redis_connected_clients",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "green", "value": null}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "none",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      },
      {
        "id": 14,
        "title": "PostgreSQL Connections",
        "type": "stat",
        "gridPos": {"h": 4, "w": 6, "x": 18, "y": 27},
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "targets": [
          {
            "expr": "pg_stat_activity_count",
            "refId": "A"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"color": "green", "value": null},
                {"color": "yellow", "value": 50},
                {"color": "red", "value": 90}
              ]
            }
          },
          "overrides": []
        },
        "options": {
          "colorMode": "value",
          "graphMode": "none",
          "reduceOptions": {"calcs": ["lastNotNull"]}
        }
      }
    ]
  },
  "overwrite": true
}
```

**Step 4: Update Grafana service in docker-compose.yml**

Replace the existing `grafana` service with:

```yaml
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer
    volumes:
      - grafana_data:/var/lib/grafana
      - ./infra/grafana/provisioning:/etc/grafana/provisioning
      - ./infra/grafana/dashboards:/var/lib/grafana/dashboards
    depends_on:
      - prometheus
    networks:
      - craftstrat
```

**Step 5: Commit**

```
feat(infra): provision Grafana with Prometheus datasource and CraftStrat Overview dashboard

Auto-provision Grafana with Prometheus datasource and a 14-panel dashboard
covering engine metrics, trading activity, and infrastructure health.
```

---

### Task 8: Add uptime metric to engine

**Files:**
- Modify: `engine/src/strategy/engine.rs`

**Step 1: Add uptime gauge to the tick processing loop**

The `craftstrat_uptime_seconds` metric needs a source. Add it to the strategy engine tick loop since it runs continuously. At the top of the `run()` function, after `tracing::info!("strategy_engine_started");`, add:

```rust
let engine_start = std::time::Instant::now();
```

Inside the loop, after `counter!("craftstrat_ticks_total").increment(1);`, add:

```rust
metrics::gauge!("craftstrat_uptime_seconds").set(engine_start.elapsed().as_secs_f64());
```

Add `metrics` to the existing import:

```rust
use metrics::{counter, gauge, histogram};
```

**Step 2: Verify and commit**

Run: `cd engine && cargo check`

```
feat(engine): add craftstrat_uptime_seconds gauge
```

---

### Task 9: Verify full stack

**Step 1: Build the Rust engine**

Run: `cd engine && cargo build`
Expected: Compiles with no errors.

**Step 2: Run all engine tests**

Run: `cd engine && cargo test`
Expected: All tests pass. The `metrics` macros are safe to call without an installed recorder (they become no-ops).

**Step 3: Verify Docker Compose config**

Run: `docker compose config --quiet`
Expected: No errors.

**Step 4: Final commit**

```
feat: Phase 9 — Monitoring complete

Prometheus metrics in Rust engine, exporters for PG/Redis/Kafka,
and auto-provisioned Grafana dashboard.
```
