# Oddex — Technical Specification
> Polymarket automated trading platform: data ingestion, strategy engine, no-code builder, backtesting, multi-wallet execution

---

## 1. Overview

Oddex is a SaaS platform that allows users to create automated trading strategies on Polymarket prediction markets via a no-code interface, backtest them against historical data, and run them live across multiple wallets simultaneously.

### 1.1 Core Value Proposition
- No-code strategy builder (form-based + advanced node editor)
- Real-time execution engine across N wallets in parallel
- Historical backtesting with real order book data
- Revenue sharing via Polymarket Builder Program

### 1.2 Tech Stack

| Layer | Technology |
|---|---|
| Business App | Laravel 12 (PHP) + Inertia.js |
| Frontend | React 19 + TailwindCSS (dans Laravel via Inertia) |
| Trading Engine | Rust (Tokio, Rayon, Axum) |
| Message Bus | Apache Kafka |
| Time-series DB | ClickHouse 26.1 |
| Business DB | PostgreSQL 17 |
| State Store | Redis 7 |
| Monitoring | Grafana |
| Infra | Docker + Docker Compose |

---

## 2. Repository Structure

```
oddex/
├── docker-compose.yml                     # Orchestration de tous les services
├── .env / .env.example                    # Variables partagées (Laravel + Engine)
├── CLAUDE.md                              # Contexte AI
│
├── web/                                   # Laravel 12 (monolith avec Inertia)
│   ├── app/
│   │   ├── Http/
│   │   │   ├── Controllers/
│   │   │   │   ├── Auth/
│   │   │   │   ├── StrategyController.php
│   │   │   │   ├── WalletController.php
│   │   │   │   ├── BacktestController.php
│   │   │   │   └── BillingController.php
│   │   │   └── Middleware/
│   │   │       └── CheckPlanLimits.php
│   │   ├── Models/
│   │   │   ├── User.php
│   │   │   ├── Strategy.php
│   │   │   ├── Wallet.php
│   │   │   ├── WalletStrategy.php
│   │   │   ├── Trade.php
│   │   │   ├── BacktestResult.php
│   │   │   └── Subscription.php
│   │   └── Services/
│   │       ├── EngineService.php          # HTTP calls to Rust engine
│   │       ├── WalletService.php          # Génération + chiffrement clés
│   │       └── BillingService.php         # Stripe via Cashier
│   ├── resources/
│   │   └── js/                            # React 19 via Inertia
│   │       ├── pages/
│   │       │   ├── dashboard.tsx
│   │       │   ├── strategy/
│   │       │   │   ├── index.tsx
│   │       │   │   ├── builder.tsx        # No-code form builder
│   │       │   │   └── node-editor.tsx    # Advanced React Flow
│   │       │   ├── backtest/
│   │       │   │   ├── index.tsx
│   │       │   │   └── result.tsx
│   │       │   ├── wallets/
│   │       │   │   └── index.tsx
│   │       │   └── billing/
│   │       │       └── index.tsx
│   │       ├── components/
│   │       │   ├── strategy/
│   │       │   │   ├── form-builder.tsx   # SI/ET/ALORS form mode
│   │       │   │   ├── rule-row.tsx
│   │       │   │   └── node-editor.tsx    # React Flow node editor
│   │       │   ├── charts/
│   │       │   │   ├── pnl-chart.tsx
│   │       │   │   └── backtest-chart.tsx
│   │       │   └── ui/                    # shadcn/ui components
│   │       ├── layouts/
│   │       │   └── app-layout.tsx
│   │       └── app.tsx                    # Inertia bootstrap
│   ├── database/
│   │   └── migrations/
│   ├── routes/
│   │   └── web.php                        # Routes Inertia (pas d'API REST séparée)
│   ├── config/
│   ├── public/
│   ├── storage/
│   ├── tests/
│   ├── artisan
│   ├── composer.json
│   ├── package.json
│   └── vite.config.ts
│
├── engine/                                # Rust — trading engine
│   ├── src/
│   │   ├── main.rs
│   │   ├── fetcher/
│   │   │   ├── mod.rs
│   │   │   ├── polymarket.rs              # API client
│   │   │   └── batch.rs                   # ClickHouse batch writer
│   │   ├── kafka/
│   │   │   ├── producer.rs
│   │   │   └── consumer.rs
│   │   ├── strategy/
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs                  # Main dispatch loop
│   │   │   ├── interpreter.rs             # JSON graph interpreter
│   │   │   ├── indicators.rs              # EMA, SMA, RSI, VWAP...
│   │   │   └── state.rs                   # Stateful strategy state
│   │   ├── execution/
│   │   │   ├── mod.rs
│   │   │   ├── queue.rs                   # Priority queue + throttle
│   │   │   ├── orders.rs                  # Order signing + submission
│   │   │   └── wallet.rs                  # Multi-wallet manager
│   │   ├── watcher/
│   │   │   ├── mod.rs
│   │   │   └── polymarket.rs              # Poll trades des wallets externes surveillés
│   │   ├── storage/
│   │   │   ├── clickhouse.rs
│   │   │   └── redis.rs
│   │   ├── backtest/
│   │   │   └── runner.rs                  # Replay ticks from ClickHouse
│   │   └── api/
│   │       └── server.rs                  # Axum HTTP server (internal)
│   ├── Cargo.toml
│   └── Cargo.lock
│
├── infra/                                 # Infrastructure
│   ├── docker/
│   │   └── app.Dockerfile                 # PHP 8.4 + Nginx + Node 22
│   ├── nginx/
│   │   └── default.conf
│   └── clickhouse/
│       └── init.sql                       # slot_snapshots table
│
└── docs/
    ├── SPEC.md
    └── plans/
```

---

## 3. Database Schemas

### 3.1 PostgreSQL — Business Data

```sql
-- Users
CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    email           VARCHAR(255) UNIQUE NOT NULL,
    password        VARCHAR(255) NOT NULL,
    name            VARCHAR(255),
    plan            VARCHAR(20) DEFAULT 'free' CHECK (plan IN ('free','starter','pro','enterprise')),
    stripe_id       VARCHAR(255) NULL,
    team_id         BIGINT NULL,                    -- nullable, pas de teams en V1
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Wallets (wallets Polygon générés par la plateforme)
CREATE TABLE wallets (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label           VARCHAR(255),
    address         VARCHAR(255) NOT NULL UNIQUE,
    private_key_enc TEXT NOT NULL,                  -- clé privée chiffrée AES-256 + pgcrypto
    balance_usdc    NUMERIC(18,6) DEFAULT 0,
    is_active       BOOLEAN DEFAULT TRUE,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Strategies
CREATE TABLE strategies (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT NULL,
    graph           JSONB NOT NULL,                 -- strategy graph (voir section 5)
    mode            VARCHAR(10) DEFAULT 'form' CHECK (mode IN ('form','node')),
    is_active       BOOLEAN DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_strategies_graph ON strategies USING GIN (graph);

-- Wallet <-> Strategy assignments
CREATE TABLE wallet_strategies (
    id                  BIGSERIAL PRIMARY KEY,
    wallet_id           BIGINT NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    strategy_id         BIGINT NOT NULL REFERENCES strategies(id) ON DELETE CASCADE,
    markets             JSONB DEFAULT '[]',
    max_position_usdc   NUMERIC(18,6) DEFAULT 100,
    is_running          BOOLEAN DEFAULT FALSE,
    started_at          TIMESTAMPTZ NULL,
    UNIQUE (wallet_id, strategy_id)
);

-- Wallets externes surveillés pour le copy trading (n'importe quelle adresse Polygon)
CREATE TABLE watched_wallets (
    id              BIGSERIAL PRIMARY KEY,
    address         VARCHAR(255) NOT NULL UNIQUE,
    label           VARCHAR(255) NULL,              -- nom donné par l'user (ex: "Whale #1")
    follower_count  INT DEFAULT 0,
    win_rate        NUMERIC(5,4) NULL,
    total_pnl_usdc  NUMERIC(18,6) NULL,
    avg_slippage    NUMERIC(8,6) NULL,              -- affiché aux followers avant de copier
    last_seen_at    TIMESTAMPTZ NULL,
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Copy trading : un wallet Oddex suit un wallet externe
CREATE TABLE copy_relationships (
    id                  BIGSERIAL PRIMARY KEY,
    follower_wallet_id  BIGINT NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    watched_wallet_id   BIGINT NOT NULL REFERENCES watched_wallets(id) ON DELETE CASCADE,
    size_mode           VARCHAR(20) DEFAULT 'proportional' CHECK (size_mode IN ('fixed','proportional')),
    size_value          NUMERIC(18,6) NOT NULL,
    max_position_usdc   NUMERIC(18,6) DEFAULT 100,
    markets_filter      JSONB NULL,                 -- null = copier tous les marchés
    is_active           BOOLEAN DEFAULT TRUE,
    created_at          TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (follower_wallet_id, watched_wallet_id)
);

-- Audit de chaque copy trade (transparence slippage)
CREATE TABLE copy_trades (
    id                      BIGSERIAL PRIMARY KEY,
    copy_relationship_id    BIGINT NOT NULL REFERENCES copy_relationships(id),
    follower_trade_id       BIGINT NULL REFERENCES trades(id),
    -- trade détecté sur le wallet externe
    leader_address          VARCHAR(255) NOT NULL,
    leader_market_id        VARCHAR(100),
    leader_outcome          VARCHAR(10),
    leader_price            NUMERIC(10,6),
    leader_size_usdc        NUMERIC(18,6),
    leader_tx_hash          VARCHAR(255),           -- hash Polygon pour vérification on-chain
    -- exécution follower
    follower_price          NUMERIC(10,6) NULL,
    slippage_pct            NUMERIC(8,6) NULL,      -- (follower_price - leader_price) / leader_price
    status                  VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending','filled','failed','skipped')),
    skip_reason             VARCHAR(255) NULL,
    detected_at             TIMESTAMPTZ NOT NULL,
    executed_at             TIMESTAMPTZ NULL,
    created_at              TIMESTAMPTZ DEFAULT NOW()
);

-- Trades Oddex (strategy-driven ou copy-driven)
CREATE TABLE trades (
    id                      BIGSERIAL PRIMARY KEY,
    wallet_id               BIGINT NOT NULL REFERENCES wallets(id),
    strategy_id             BIGINT NULL REFERENCES strategies(id),       -- NULL si copy trade
    copy_relationship_id    BIGINT NULL REFERENCES copy_relationships(id), -- NULL si strategy trade
    market_id               VARCHAR(100),
    side                    VARCHAR(10) CHECK (side IN ('buy','sell')),
    outcome                 VARCHAR(10) CHECK (outcome IN ('UP','DOWN')),
    price                   NUMERIC(10,6),
    size_usdc               NUMERIC(18,6),
    order_type              VARCHAR(20) CHECK (order_type IN ('market','limit','stoploss','take_profit')),
    status                  VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending','filled','cancelled')),
    polymarket_order_id     VARCHAR(255) NULL,
    fee_bps                 SMALLINT NULL,
    executed_at             TIMESTAMPTZ NULL,
    created_at              TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_trades_wallet ON trades (wallet_id, created_at DESC);
CREATE INDEX idx_trades_market ON trades (market_id);

-- Backtest results
CREATE TABLE backtest_results (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    strategy_id     BIGINT NOT NULL REFERENCES strategies(id),
    market_filter   JSONB NULL,
    date_from       TIMESTAMPTZ NULL,
    date_to         TIMESTAMPTZ NULL,
    total_trades    INT,
    win_rate        NUMERIC(5,4),
    total_pnl_usdc  NUMERIC(18,6),
    max_drawdown    NUMERIC(5,4),
    sharpe_ratio    NUMERIC(8,4),
    result_detail   JSONB NULL,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Subscriptions (Laravel Cashier / Stripe)
CREATE TABLE subscriptions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type            VARCHAR(255) NOT NULL,
    stripe_id       VARCHAR(255) UNIQUE NOT NULL,
    stripe_status   VARCHAR(255) NOT NULL,
    stripe_price    VARCHAR(255) NULL,
    quantity        INT DEFAULT 1,
    trial_ends_at   TIMESTAMPTZ NULL,
    ends_at         TIMESTAMPTZ NULL,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE subscription_items (
    id                  BIGSERIAL PRIMARY KEY,
    subscription_id     BIGINT NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
    stripe_id           VARCHAR(255) UNIQUE NOT NULL,
    stripe_product      VARCHAR(255) NULL,
    stripe_price        VARCHAR(255) NOT NULL,
    quantity            INT DEFAULT 1,
    created_at          TIMESTAMPTZ DEFAULT NOW(),
    updated_at          TIMESTAMPTZ DEFAULT NOW()
);
```

### 3.2 ClickHouse — Time-series Market Data

```sql
CREATE TABLE slot_snapshots (
    captured_at       DateTime64(3),
    symbol            LowCardinality(String),       -- e.g. "btc-updown-15m-1770138000"
    slot_ts           UInt32,                        -- Unix timestamp slot start
    slot_duration     UInt32,                        -- 300 | 900 | 3600 | 14400 | 86400 (seconds)
    minutes_into_slot Float32,
    pct_into_slot     Float32,                       -- 0.0 to 1.0 (position in slot)

    -- Order book — Level 1
    bid_up            Float32,
    ask_up            Float32,
    bid_down          Float32,
    ask_down          Float32,
    bid_size_up       Float32,
    ask_size_up       Float32,
    bid_size_down     Float32,
    ask_size_down     Float32,
    spread_up         Float32,
    spread_down       Float32,

    -- Order book — Level 2 & 3
    bid_up_l2         Float32,
    ask_up_l2         Float32,
    bid_up_l3         Float32,
    ask_up_l3         Float32,
    bid_down_l2       Float32,
    ask_down_l2       Float32,
    bid_down_l3       Float32,
    ask_down_l3       Float32,

    -- Derived
    mid_up            Float32,                       -- (bid_up + ask_up) / 2
    mid_down          Float32,
    size_ratio_up     Float32,                       -- bid_size_up / ask_size_up
    size_ratio_down   Float32,

    -- Price context
    chainlink_price   Float32,
    dir_move_pct      Float32,
    abs_move_pct      Float32,

    -- Time context
    hour_utc          UInt8,
    day_of_week       UInt8,

    -- Volume
    market_volume_usd Float32,

    -- Market result (filled at slot close, nullable during slot)
    winner            Nullable(Enum8('UP' = 1, 'DOWN' = 2)),
    btc_price_start   Float32,
    btc_price_end     Float32

) ENGINE = MergeTree()
PARTITION BY toYYYYMM(captured_at)
ORDER BY (symbol, slot_duration, captured_at)
SETTINGS index_granularity = 8192;
```

---

## 4. Rust Engine — Core Types

```rust
// src/fetcher/polymarket.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub captured_at: DateTime<Utc>,
    pub symbol: String,
    pub slot_ts: u32,
    pub slot_duration: u32,
    pub minutes_into_slot: f32,
    pub pct_into_slot: f32,

    pub bid_up: f32,
    pub ask_up: f32,
    pub bid_down: f32,
    pub ask_down: f32,
    pub bid_size_up: f32,
    pub ask_size_up: f32,
    pub bid_size_down: f32,
    pub ask_size_down: f32,
    pub spread_up: f32,
    pub spread_down: f32,

    pub chainlink_price: f32,
    pub dir_move_pct: f32,
    pub abs_move_pct: f32,
    pub hour_utc: u8,
    pub day_of_week: u8,
    pub market_volume_usd: f32,
}

// src/strategy/mod.rs

#[derive(Debug, Clone)]
pub enum Signal {
    Buy  { outcome: Outcome, size_usdc: f64, order_type: OrderType },
    Sell { outcome: Outcome, size_usdc: f64, order_type: OrderType },
    Hold,
}

#[derive(Debug, Clone)]
pub enum Outcome { Up, Down }

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit { price: f64 },
    StopLoss { trigger_price: f64 },
    TakeProfit { trigger_price: f64 },
}

pub trait Strategy: Send {
    fn on_tick(&mut self, tick: &Tick) -> Signal;
    fn reset(&mut self);
    fn name(&self) -> &str;
}

// src/strategy/state.rs

pub struct StrategyState {
    pub window: VecDeque<Tick>,         // sliding window of last N ticks
    pub window_size: usize,
    pub position: Option<Position>,     // current open position
    pub pnl: f64,
    pub ema_values: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub outcome: Outcome,
    pub entry_price: f64,
    pub size_usdc: f64,
    pub entry_at: DateTime<Utc>,
    pub stoploss: Option<f64>,
    pub take_profit: Option<f64>,
}
```

---

## 5. Strategy Graph JSON Format

Strategies are stored as a JSON graph in PostgreSQL and interpreted by the Rust engine at runtime.

### 5.1 Form Mode (simple)

```json
{
  "mode": "form",
  "conditions": [
    {
      "type": "AND",
      "rules": [
        {
          "indicator": "abs_move_pct",
          "operator": ">",
          "value": 3.0
        },
        {
          "indicator": "pct_into_slot",
          "operator": "between",
          "value": [0.1, 0.4]
        },
        {
          "indicator": "spread_up",
          "operator": "<",
          "value": 0.05
        }
      ]
    }
  ],
  "action": {
    "signal": "buy",
    "outcome": "UP",
    "size_mode": "fixed",
    "size_usdc": 50,
    "order_type": "market"
  },
  "risk": {
    "stoploss_pct": 30,
    "take_profit_pct": 80,
    "max_position_usdc": 200,
    "max_trades_per_slot": 1
  }
}
```

### 5.2 Node Mode (advanced)

```json
{
  "mode": "node",
  "nodes": [
    { "id": "n1", "type": "input",      "data": { "field": "abs_move_pct" } },
    { "id": "n2", "type": "input",      "data": { "field": "pct_into_slot" } },
    { "id": "n3", "type": "indicator",  "data": { "fn": "EMA", "period": 20, "field": "mid_up" } },
    { "id": "n4", "type": "comparator", "data": { "operator": ">", "value": 3.0 } },
    { "id": "n5", "type": "comparator", "data": { "operator": "between", "value": [0.1, 0.3] } },
    { "id": "n6", "type": "logic",      "data": { "operator": "AND" } },
    { "id": "n7", "type": "action",     "data": { "signal": "buy", "outcome": "UP", "size_usdc": 50 } }
  ],
  "edges": [
    { "source": "n1", "target": "n4" },
    { "source": "n2", "target": "n5" },
    { "source": "n4", "target": "n6" },
    { "source": "n5", "target": "n6" },
    { "source": "n6", "target": "n7" }
  ]
}
```

### 5.3 Available Indicators

| Indicator | Description | Stateless |
|---|---|---|
| `abs_move_pct` | Absolute move % since slot start | ✅ |
| `dir_move_pct` | Directional move % (+ UP, - DOWN) | ✅ |
| `spread_up` / `spread_down` | Current spread | ✅ |
| `size_ratio_up` | bid_size / ask_size ratio | ✅ |
| `pct_into_slot` | % progress in current slot | ✅ |
| `mid_up` / `mid_down` | Mid price | ✅ |
| `chainlink_price` | BTC spot price | ✅ |
| `hour_utc` | UTC hour 0-23 | ✅ |
| `day_of_week` | Day 0-6 | ✅ |
| `EMA(n, field)` | Exponential moving average | ❌ stateful |
| `SMA(n, field)` | Simple moving average | ❌ stateful |
| `RSI(n, field)` | RSI oscillator | ❌ stateful |
| `VWAP(field)` | Volume-weighted avg price | ❌ stateful |
| `cross_above(a, b)` | Crossover detection | ❌ stateful |
| `cross_below(a, b)` | Crossunder detection | ❌ stateful |

---

## 6. Laravel Routes (Inertia)

Avec Inertia, pas d'API REST séparée — les routes retournent des pages Inertia (GET) ou traitent des actions (POST/PUT/DELETE) puis redirigent. Les données sont passées via `Inertia::render()` comme props.

```php
// routes/web.php

// Auth (Laravel Breeze / starter kit)
Route::get('/login', [AuthController::class, 'show']);
Route::post('/login', [AuthController::class, 'login']);
Route::post('/logout', [AuthController::class, 'logout']);
Route::get('/register', [AuthController::class, 'register']);

Route::middleware('auth')->group(function () {

    // Dashboard
    Route::get('/', [DashboardController::class, 'index']);

    // Strategies
    Route::get('/strategies', [StrategyController::class, 'index']);
    Route::get('/strategies/create', [StrategyController::class, 'create']);
    Route::post('/strategies', [StrategyController::class, 'store']);
    Route::get('/strategies/{strategy}', [StrategyController::class, 'show']);
    Route::put('/strategies/{strategy}', [StrategyController::class, 'update']);
    Route::delete('/strategies/{strategy}', [StrategyController::class, 'destroy']);
    Route::post('/strategies/{strategy}/activate', [StrategyController::class, 'activate']);
    Route::post('/strategies/{strategy}/deactivate', [StrategyController::class, 'deactivate']);

    // Wallets
    Route::get('/wallets', [WalletController::class, 'index']);
    Route::post('/wallets', [WalletController::class, 'store']);        # génère clé + chiffre
    Route::delete('/wallets/{wallet}', [WalletController::class, 'destroy']);
    Route::post('/wallets/{wallet}/strategies', [WalletController::class, 'assignStrategy']);
    Route::delete('/wallets/{wallet}/strategies/{strategy}', [WalletController::class, 'removeStrategy']);

    // Backtests
    Route::post('/strategies/{strategy}/backtest', [BacktestController::class, 'run']);
    Route::get('/backtests', [BacktestController::class, 'index']);
    Route::get('/backtests/{result}', [BacktestController::class, 'show']);

    // Markets (données depuis ClickHouse, cachées Redis)
    Route::get('/markets', [MarketController::class, 'index']);

    // Copy trading
    Route::get('/copy-trading', [CopyTradingController::class, 'index']);       # liste des leaders publics
    Route::post('/copy-trading/follow', [CopyTradingController::class, 'follow']);
    Route::delete('/copy-trading/{relationship}', [CopyTradingController::class, 'unfollow']);
    Route::get('/copy-trading/stats/{wallet}', [CopyTradingController::class, 'leaderStats']); # stats publiques


    Route::get('/billing', [BillingController::class, 'index']);
    Route::post('/billing/subscribe', [BillingController::class, 'subscribe']);
    Route::post('/billing/portal', [BillingController::class, 'portal']);

});

// Webhooks Stripe (pas d'auth middleware)
Route::post('/webhooks/stripe', [StripeWebhookController::class, 'handle']);
```

### Inertia data flow

```php
// Exemple StrategyController::index()
public function index()
{
    return Inertia::render('Strategy/Index', [
        'strategies' => Strategy::where('user_id', auth()->id())
            ->withCount('wallets')
            ->latest()
            ->get(),
    ]);
}

// Exemple StrategyController::activate()
public function activate(Strategy $strategy)
{
    $this->authorize('update', $strategy);
    $this->engineService->activateStrategy($strategy);
    $strategy->update(['is_active' => true]);
    return back()->with('success', 'Strategy activated');
}
```

---

## 7. Rust Internal API (Axum)

Laravel communicates with the Rust engine via this internal HTTP API (not exposed publicly).

```
POST   /internal/strategy/activate
       Body: { wallet_id, strategy_id, graph: {...}, markets: [...] }

POST   /internal/strategy/deactivate
       Body: { wallet_id, strategy_id }

GET    /internal/wallet/{id}/state
       Returns: { position, pnl, last_signal, last_tick_at }

POST   /internal/backtest/run
       Body: { strategy_graph, market_filter, date_from, date_to }
       Returns: { total_trades, win_rate, pnl, trades: [...] }

GET    /internal/engine/status
       Returns: { active_wallets, ticks_per_sec, kafka_lag, ... }
```

---

## 8. Rust Engine — Main Loop

```rust
// src/main.rs — simplified

#[tokio::main]
async fn main() {
    // 1. Start Axum internal API server
    tokio::spawn(api::server::run());

    // 2. Start Kafka consumer → dispatch to strategy engine
    tokio::spawn(strategy::engine::run());

    // 3. Start wallet watcher → copy trading fan-out
    tokio::spawn(watcher::run());

    // 4. Start fetcher loop (every second, 20 markets)
    fetcher::run_loop().await;
}

// src/watcher/polymarket.rs
// Poll l'API Polymarket trades pour chaque adresse externe surveillée
// Endpoint : GET /data-api/v2/trades?maker_address={address}&limit=1
// Tourner toutes les secondes, détecter les nouveaux trades via last_seen_at
pub async fn run() {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let watched = get_all_watched_wallets().await;  // depuis Redis (cache)

        futures::future::join_all(
            watched.iter().map(|w| check_new_trades(w))
        ).await
        .into_iter()
        .flatten()
        .for_each(|(address, trade)| {
            // Fan-out vers tous les followers actifs de cette adresse
            let followers = get_followers_for_address(&address).await;
            for follower in followers {
                let signal = build_copy_signal(&trade, &follower);
                match signal {
                    Some(s) => execution::queue::push_copy(follower.wallet_id, s, &trade).await,
                    None => log_skipped(&follower, &trade, "max_position_reached"),
                }
            }
        });
    }
}

// src/strategy/engine.rs — inchangé, gère uniquement les stratégies
pub async fn run() {
    let mut consumer = kafka::create_consumer("ticks").await;
    loop {
        let tick = consumer.recv().await;
        let assignments = get_active_assignments(&tick.symbol).await;
        let signals: Vec<(WalletId, Signal)> = assignments
            .par_iter()
            .filter_map(|a| {
                let signal = a.strategy.on_tick(&tick);
                match signal { Signal::Hold => None, s => Some((a.wallet_id, s)) }
            })
            .collect();
        for (wallet_id, signal) in signals {
            execution::queue::push(wallet_id, signal).await;
        }
    }
}
```

---

## 9. Pricing Plans

| Plan | Price | Wallets | Strategies | Backtest | Copy Trading | RevShare |
|---|---|---|---|---|---|---|
| **Free** | $0 | 1 | 2 | 30 days | Follow uniquement (1 leader) | ❌ |
| **Starter** | $29/mo | 5 | 10 | Full history | Follow (5 leaders) | ✅ |
| **Pro** | $79/mo | 25 | Unlimited | Full history | Follow + être leader public | ✅ |
| **Enterprise** | $249/mo | Unlimited | Unlimited | Full history + API | Illimité + leader fee custom | ✅ |

### Plan enforcement (Laravel middleware)
```php
// app/Http/Middleware/CheckPlanLimits.php
// Checks wallet count, strategy count, backtest date range
// against user's current subscription plan
```

---

## 10. Security

- **Wallet model** : wallets générés par la plateforme (pas de clé importée par l'user). L'utilisateur dépose des USDC sur l'adresse fournie — c'est un wallet dédié Oddex, pas son wallet principal. Psychologiquement bien plus rassurant.
- **Private keys** chiffrées AES-256 dans PostgreSQL via `ENCRYPTION_KEY` — jamais loggées, jamais exposées en API response
- **WalletService** : seul service autorisé à déchiffrer les clés, uniquement au moment de signer un ordre
- **Internal Rust API** accessible uniquement sur le réseau Docker interne (non exposé publiquement)
- **Builder API keys** (Polymarket) dans les variables d'environnement, jamais en base
- **Auth** via Laravel Breeze + sessions (Inertia fullstack, pas de token API)
- **Rate limiting** sur toutes les routes sensibles (Laravel throttle middleware)
- **Authorization** via Laravel Policies — chaque action vérifie que la ressource appartient à l'user authentifié
- Tous les secrets via `.env` / Docker secrets en production

---

## 11. Environment Variables

```env
# Laravel
APP_KEY=
APP_URL=https://oddex.io
DB_CONNECTION=pgsql
DB_HOST=postgres
DB_PORT=5432
DB_DATABASE=oddex
DB_USERNAME=oddex
DB_PASSWORD=
REDIS_HOST=redis
STRIPE_KEY=
STRIPE_SECRET=
STRIPE_WEBHOOK_SECRET=
ENGINE_INTERNAL_URL=http://engine:8080
ENCRYPTION_KEY=                    # AES-256 key for wallet private keys

# Rust Engine
CLICKHOUSE_URL=http://clickhouse:8123
KAFKA_BROKERS=kafka:9092
REDIS_URL=redis://redis:6379
POLYMARKET_API_URL=https://clob.polymarket.com
POLYMARKET_BUILDER_API_KEY=
POLYMARKET_BUILDER_SECRET=
POLYMARKET_BUILDER_PASSPHRASE=
INTERNAL_API_PORT=8080
ENCRYPTION_KEY=                    # même clé que Laravel pour déchiffrer les private keys
```

---

## 12. Docker Compose

```yaml
# docker-compose.yml (à la racine du projet)
services:
  app:
    build:
      context: .
      dockerfile: infra/docker/app.Dockerfile
    ports: ["8000:80", "5173:5173"]
    volumes:
      - ./web:/var/www/html
      - ./.env:/var/www/html/.env
      - vendor_data:/var/www/html/vendor
      - node_modules_data:/var/www/html/node_modules
    depends_on: [postgres, redis]
    env_file: .env

  engine:
    build: ./engine
    depends_on: [kafka, clickhouse, redis]
    env_file: .env
    # NON exposé publiquement

  postgres:
    image: postgres:17
    environment:
      POSTGRES_DB: oddex
      POSTGRES_USER: oddex
      POSTGRES_PASSWORD: oddex_secret
    volumes: [postgres_data:/var/lib/postgresql/data]
    ports: ["5432:5432"]

  clickhouse:
    image: clickhouse/clickhouse-server:26.1
    environment:
      CLICKHOUSE_USER: default
      CLICKHOUSE_PASSWORD: clickhouse
      CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT: 1
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./infra/clickhouse/init.sql:/docker-entrypoint-initdb.d/init.sql
    ports: ["8123:8123"]

  redis:
    image: redis:7-alpine
    volumes: [redis_data:/data]
    ports: ["6379:6379"]

  kafka:
    image: confluentinc/cp-kafka:7.9.0
    environment:
      KAFKA_NODE_ID: 1
      KAFKA_PROCESS_ROLES: broker,controller
      KAFKA_LISTENERS: PLAINTEXT://kafka:9092,CONTROLLER://kafka:9093
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_CONTROLLER_LISTENER_NAMES: CONTROLLER
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
      KAFKA_CONTROLLER_QUORUM_VOTERS: 1@kafka:9093
      CLUSTER_ID: MkU3OEVBNTcwNTJENDM2Qk

  grafana:
    image: grafana/grafana:latest
    ports: ["3001:3000"]
    volumes: [grafana_data:/var/lib/grafana]

volumes:
  postgres_data:
  clickhouse_data:
  redis_data:
  grafana_data:
  vendor_data:
  node_modules_data:
```

---

## 13. Build Order for Claude Code

Follow this sequence to build incrementally:

### Phase 1 — Infrastructure
1. `docker-compose.yml` with all services
2. ClickHouse schema + migrations
3. PostgreSQL migrations (Laravel)

### Phase 2 — Data Pipeline
4. Rust fetcher: Polymarket API client → fetch 1 market
5. Rust fetcher: parallel fetch N markets every second
6. Rust ClickHouse batch writer
7. Rust Kafka producer
8. Verify data flowing into ClickHouse

### Phase 3 — Strategy Engine
9. Rust Tick struct + Kafka consumer
10. Strategy trait + StatelessStrategy (simple comparators)
11. StatefulStrategy (EMA, SMA, RSI with sliding window)
12. JSON graph interpreter (form mode first, node mode second)
13. Rayon parallel dispatch across wallet/strategy pairs
14. Redis state persistence + recovery

### Phase 4 — Execution + Copy Trading
15. Execution queue avec throttling rate limits Polymarket
16. Order signing avec feeRateBps (Builder attribution)
17. Multi-wallet manager (buy, sell, stoploss, take profit)
18. Wallet watcher : poll `GET /data-api/v2/trades?maker_address=` pour chaque adresse surveillée
19. Détection nouveaux trades via `last_seen_at` + fan-out vers followers
20. `push_copy()` avec calcul slippage a posteriori + écriture `copy_trades`
21. Laravel job périodique : recalcul stats `watched_wallets` (win_rate, avg_slippage)

### Phase 5 — Backtest
18. ClickHouse tick replay iterator
19. Backtest runner (same Strategy trait, different tick source)
20. Result aggregation (PnL, win rate, Sharpe, drawdown)

### Phase 6 — Internal API
21. Axum server with all `/internal/*` endpoints
22. Laravel EngineService (HTTP calls to Axum)

### Phase 7 — Laravel + Inertia
23. Auth (Laravel Breeze avec stack Inertia/React)
24. Strategy CRUD + pages Inertia
25. WalletService : génération keypair Polygon + chiffrement AES-256
26. Wallet CRUD + pages Inertia
27. Backtest trigger + résultats
28. Plan limit middleware (CheckPlanLimits)
29. Stripe Cashier billing + webhook

### Phase 8 — Frontend Inertia/React
30. Layout applicatif (sidebar, navigation)
31. Dashboard (positions ouvertes, PnL global, stratégies actives)
32. Strategy Builder — mode formulaire (SI/ET/ALORS)
33. Strategy Builder — mode node editor (React Flow)
34. Page Backtest (déclenchement + graphiques résultats)
35. Page Wallets (création, assignation stratégies, solde)
36. Page Billing (plan actuel, upgrade, portal Stripe)

### Phase 9 — Monitoring
36. Grafana dashboards (ticks/sec, active wallets, PnL, Kafka lag)

---

## 14. Key Constraints & Notes

- **ClickHouse insert pattern** : jamais row-by-row, toujours batch minimum 100 rows ou flush toutes les 10 secondes
- **Kafka topics** : `ticks`, `signals`, `strategy-updates`
- **Strategy activation flow** : Inertia form submit → Laravel Controller → EngineService HTTP → Axum → charge le graph en mémoire → spawn Tokio task par wallet
- **Strategy graph transmis au boot** : si le Rust engine redémarre, Laravel re-push toutes les `wallet_strategies` `is_running=true` au démarrage via un Artisan command ou observer
- **Backtest** utilise le même trait `Strategy` que le live engine — zéro duplication de logique
- **Wallet private keys** : jamais dans les logs, jamais dans les réponses HTTP, déchiffrées uniquement dans WalletService au moment de signer
- **Wallet model** : généré par la plateforme (keypair Ethereum/Polygon), l'user dépose des USDC dessus — pas d'import de clé existante
- **pct_into_slot** = `minutes_into_slot / (slot_duration / 60)` — calculé à l'ingestion dans le fetcher Rust
- **Rate limits Polymarket** : Unverified=100/day, Verified=3000/day, Partner=unlimited — enforcer dans la queue d'exécution Rust
- **feeRateBps** : fetcher dynamiquement par market avant chaque ordre — jamais hardcodé
- **Multi-tenancy** : toutes les queries PostgreSQL scopées par `user_id`, toutes les tasks Rust keyed par `(wallet_id, strategy_id)`
- **team_id** : présent dans la table `users` mais nullable et non utilisé en V1 — préparé pour une future feature teams sans migration breaking
- **Copy trading** : surveillance de n'importe quelle adresse Polygon publique — pas uniquement les users Oddex. Le watcher poll `GET /data-api/v2/trades?maker_address={address}` chaque seconde pour chaque adresse suivie. Latence réaliste leader→follower : 1-3 secondes, à afficher clairement avant toute souscription.
- **Copy trading transparence** : `copy_trades` stocke systématiquement le prix leader (détecté) ET le prix follower (exécuté) + slippage calculé. Les stats `watched_wallets` incluent le slippage moyen historique visible avant de follow.
- **watched_wallets en cache Redis** : la liste des adresses à surveiller est chargée en mémoire Redis au démarrage du watcher, mise à jour à chaque nouvelle `copy_relationship`. Le watcher ne lit jamais PostgreSQL en boucle.