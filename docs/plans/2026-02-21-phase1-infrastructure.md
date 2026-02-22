# Phase 1 — Infrastructure Implementation Plan (100% Docker)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up the complete development infrastructure for CraftStrat — Laravel 12 app with React/Inertia, all Docker services, PostgreSQL migrations, ClickHouse schema, and Rust engine skeleton. Everything runs in Docker via OrbStack.

**Architecture:** Laravel 12 with React starter kit (Inertia 2, TypeScript, shadcn/ui, Tailwind) running behind Nginx/PHP-FPM in Docker. All dependencies (PostgreSQL 17, ClickHouse 26.1, Redis 7, Kafka KRaft, Grafana) containerized via Docker Compose. Dev workflow: source code mounted as volume, all commands run via `docker compose exec`. OrbStack for fast I/O on macOS.

**Tech Stack:** PHP 8.4, Laravel 12, React 19, Inertia 2, TypeScript, TailwindCSS, shadcn/ui, PostgreSQL 17, ClickHouse 26.1, Redis 7, Kafka (Confluent 7.9 KRaft), Grafana, Rust (Tokio, Axum), Docker Compose, Nginx, OrbStack

---

## Task 1: Create Docker infrastructure files

Create all Docker config files BEFORE scaffolding Laravel. These files exist independently of the Laravel code.

**Files:**
- Create: `Dockerfile`
- Create: `infra/nginx/default.conf`
- Create: `docker-compose.yml` (root, not infra/ — simpler for daily use)

### Step 1: Create Nginx config

```nginx
# infra/nginx/default.conf
server {
    listen 80;
    server_name localhost;
    root /var/www/html/public;
    index index.php;

    client_max_body_size 20M;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass 127.0.0.1:9000;
        fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
        include fastcgi_params;
        fastcgi_hide_header X-Powered-By;
    }

    location ~ /\.(?!well-known).* {
        deny all;
    }
}
```

### Step 2: Create the Dockerfile

The Dockerfile installs PHP-FPM, Nginx, Composer, Node 22, and Supervisor. It serves as both dev and prod base.

```dockerfile
FROM php:8.4-fpm AS base

# System deps
RUN apt-get update && apt-get install -y \
    git curl zip unzip libpq-dev libzip-dev libicu-dev \
    nginx supervisor \
    && docker-php-ext-install pdo_pgsql pgsql zip intl pcntl bcmath \
    && pecl install redis && docker-php-ext-enable redis \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Composer
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

# Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Laravel installer (for scaffolding)
RUN composer global require laravel/installer

ENV PATH="/root/.composer/vendor/bin:${PATH}"

# Nginx config
COPY infra/nginx/default.conf /etc/nginx/sites-available/default

# Supervisor config (PHP-FPM + Nginx)
RUN printf '[supervisord]\nnodaemon=true\n\n\
[program:php-fpm]\ncommand=php-fpm\nautostart=true\nautorestart=true\n\n\
[program:nginx]\ncommand=nginx -g "daemon off;"\nautostart=true\nautorestart=true\n' \
    > /etc/supervisor/conf.d/app.conf

WORKDIR /var/www/html

EXPOSE 80

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/app.conf"]
```

### Step 3: Create docker-compose.yml

```yaml
# docker-compose.yml
services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8000:80"
      - "5173:5173"
    volumes:
      - .:/var/www/html
      - vendor_data:/var/www/html/vendor
      - node_modules_data:/var/www/html/node_modules
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    env_file:
      - .env
    networks:
      - craftstrat

  postgres:
    image: postgres:17
    environment:
      POSTGRES_DB: craftstrat
      POSTGRES_USER: craftstrat
      POSTGRES_PASSWORD: craftstrat_secret
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U craftstrat"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - craftstrat

  clickhouse:
    image: clickhouse/clickhouse-server:26.1
    volumes:
      - clickhouse_data:/var/lib/clickhouse
      - ./infra/clickhouse/init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "8123:8123"
    networks:
      - craftstrat

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - craftstrat

  kafka:
    image: confluentinc/cp-kafka:7.9.0
    environment:
      KAFKA_NODE_ID: 1
      KAFKA_PROCESS_ROLES: broker,controller
      KAFKA_LISTENERS: PLAINTEXT://0.0.0.0:9092,CONTROLLER://0.0.0.0:9093
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_CONTROLLER_LISTENER_NAMES: CONTROLLER
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT
      KAFKA_CONTROLLER_QUORUM_VOTERS: 1@kafka:9093
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: 1
      KAFKA_TRANSACTION_STATE_LOG_MIN_ISR: 1
      KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS: 0
      CLUSTER_ID: craftstrat-kafka-cluster-001
    ports:
      - "9092:9092"
    networks:
      - craftstrat

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana_data:/var/lib/grafana
    networks:
      - craftstrat

networks:
  craftstrat:
    driver: bridge

volumes:
  postgres_data:
  clickhouse_data:
  redis_data:
  grafana_data:
  vendor_data:
  node_modules_data:
```

### Step 4: Commit

```bash
git init -b main
git add Dockerfile docker-compose.yml infra/nginx/default.conf
git commit -m "infra: add Dockerfile, docker-compose, Nginx config"
```

---

## Task 2: Create ClickHouse init schema

**Files:**
- Create: `infra/clickhouse/init.sql`

### Step 1: Create the schema file

Copy exactly from SPEC.md section 3.2 — the `slot_snapshots` table with MergeTree engine, partitioned by month, ordered by (symbol, slot_duration, captured_at).

```sql
-- infra/clickhouse/init.sql
CREATE TABLE IF NOT EXISTS slot_snapshots (
    captured_at       DateTime64(3),
    symbol            LowCardinality(String),
    slot_ts           UInt32,
    slot_duration     UInt32,
    minutes_into_slot Float32,
    pct_into_slot     Float32,
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
    bid_up_l2         Float32,
    ask_up_l2         Float32,
    bid_up_l3         Float32,
    ask_up_l3         Float32,
    bid_down_l2       Float32,
    ask_down_l2       Float32,
    bid_down_l3       Float32,
    ask_down_l3       Float32,
    mid_up            Float32,
    mid_down          Float32,
    size_ratio_up     Float32,
    size_ratio_down   Float32,
    chainlink_price   Float32,
    dir_move_pct      Float32,
    abs_move_pct      Float32,
    hour_utc          UInt8,
    day_of_week       UInt8,
    market_volume_usd Float32,
    winner            Nullable(Enum8('UP' = 1, 'DOWN' = 2)),
    btc_price_start   Float32,
    btc_price_end     Float32
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(captured_at)
ORDER BY (symbol, slot_duration, captured_at)
SETTINGS index_granularity = 8192;
```

### Step 2: Commit

```bash
git add infra/clickhouse/init.sql
git commit -m "infra: add ClickHouse slot_snapshots schema"
```

---

## Task 3: Create .env files

**Files:**
- Create: `.env`
- Create: `.env.example`

### Step 1: Create .env and .env.example (identical for now)

```env
APP_NAME=CraftStrat
APP_ENV=local
APP_KEY=
APP_DEBUG=true
APP_URL=http://localhost:8000

LOG_CHANNEL=stack

DB_CONNECTION=pgsql
DB_HOST=postgres
DB_PORT=5432
DB_DATABASE=craftstrat
DB_USERNAME=craftstrat
DB_PASSWORD=craftstrat_secret

REDIS_HOST=redis
REDIS_PORT=6379

CACHE_STORE=redis
SESSION_DRIVER=redis
QUEUE_CONNECTION=redis

# CraftStrat-specific
ENGINE_INTERNAL_URL=http://engine:8080
ENCRYPTION_KEY=

# Stripe
STRIPE_KEY=
STRIPE_SECRET=
STRIPE_WEBHOOK_SECRET=

# ClickHouse
CLICKHOUSE_URL=http://clickhouse:8123

# Kafka
KAFKA_BROKERS=kafka:9092

# Polymarket
POLYMARKET_API_URL=https://clob.polymarket.com
POLYMARKET_BUILDER_API_KEY=
POLYMARKET_BUILDER_SECRET=
POLYMARKET_BUILDER_PASSPHRASE=
```

### Step 2: Commit

```bash
git add .env.example
git commit -m "config: add .env.example for Docker development"
```

Note: `.env` is NOT committed (will be in .gitignore after Laravel scaffolding).

---

## Task 4: Build Docker image and scaffold Laravel via container

This is the key task — scaffolding Laravel entirely via Docker.

### Step 1: Build the Docker image

```bash
docker compose build app
```

### Step 2: Scaffold Laravel via a one-off container

We need a temporary container to run `laravel new`. Since the Dockerfile has the Laravel installer:

```bash
docker compose run --rm --no-deps -w /var/www/html app bash -c "\
  laravel new temp-project --react --database=pgsql --pest --npm --no-interaction && \
  cp -a temp-project/. /var/www/html/ && \
  rm -rf temp-project"
```

This creates a Laravel project in a temp directory then copies everything to the mounted volume.

**Important:** After this, the local `oddex/` directory will have all Laravel files. The `.env` we created in Task 3 may be overwritten by Laravel's default — we'll re-apply our Docker-specific values.

### Step 3: Re-apply Docker .env values

Overwrite the `.env` with our Docker-specific values from Task 3, preserving the `APP_KEY` that Laravel generated.

### Step 4: Verify Laravel works

```bash
docker compose up -d
docker compose exec app php artisan --version
```

Expected: `Laravel Framework 12.x.x`

### Step 5: Verify SPEC.md and docs/ survived

Check that SPEC.md and docs/plans/ still exist. If overwritten, restore them.

### Step 6: Update .gitignore

Ensure `.gitignore` includes:
- `vendor/`
- `node_modules/`
- `.env`
- `engine/target/`

### Step 7: Commit

```bash
git add -A
git commit -m "chore: scaffold Laravel 12 with React/Inertia starter kit via Docker"
```

---

## Task 5: Create PostgreSQL migration — Users extension

**Context:** The React starter kit creates a base `users` table. We add `plan`, `stripe_id`, `team_id` in a separate migration.

All `artisan` commands run via Docker:

```bash
docker compose exec app php artisan make:migration add_plan_stripe_team_to_users_table --table=users
```

### Migration content:

```php
public function up(): void
{
    Schema::table('users', function (Blueprint $table) {
        $table->string('plan', 20)->default('free');
        $table->string('stripe_id', 255)->nullable();
        $table->bigInteger('team_id')->nullable();
    });
}

public function down(): void
{
    Schema::table('users', function (Blueprint $table) {
        $table->dropColumn(['plan', 'stripe_id', 'team_id']);
    });
}
```

### Commit:
```bash
git add database/migrations/*add_plan_stripe_team*
git commit -m "db: add plan, stripe_id, team_id to users table"
```

---

## Task 6: Create PostgreSQL migration — Wallets

```bash
docker compose exec app php artisan make:migration create_wallets_table
```

```php
public function up(): void
{
    Schema::create('wallets', function (Blueprint $table) {
        $table->id();
        $table->foreignId('user_id')->constrained()->cascadeOnDelete();
        $table->string('label')->nullable();
        $table->string('address')->unique();
        $table->text('private_key_enc');
        $table->decimal('balance_usdc', 18, 6)->default(0);
        $table->boolean('is_active')->default(true);
        $table->timestamp('created_at')->useCurrent();
    });
}

public function down(): void
{
    Schema::dropIfExists('wallets');
}
```

### Commit:
```bash
git add database/migrations/*create_wallets*
git commit -m "db: create wallets table"
```

---

## Task 7: Create PostgreSQL migration — Strategies

```bash
docker compose exec app php artisan make:migration create_strategies_table
```

```php
public function up(): void
{
    Schema::create('strategies', function (Blueprint $table) {
        $table->id();
        $table->foreignId('user_id')->constrained()->cascadeOnDelete();
        $table->string('name');
        $table->text('description')->nullable();
        $table->jsonb('graph');
        $table->string('mode', 10)->default('form');
        $table->boolean('is_active')->default(false);
        $table->timestamps();
    });

    DB::statement('CREATE INDEX idx_strategies_graph ON strategies USING GIN (graph)');
}

public function down(): void
{
    Schema::dropIfExists('strategies');
}
```

### Commit:
```bash
git add database/migrations/*create_strategies*
git commit -m "db: create strategies table with GIN index on graph"
```

---

## Task 8: Create PostgreSQL migration — Wallet Strategies

```bash
docker compose exec app php artisan make:migration create_wallet_strategies_table
```

```php
public function up(): void
{
    Schema::create('wallet_strategies', function (Blueprint $table) {
        $table->id();
        $table->foreignId('wallet_id')->constrained()->cascadeOnDelete();
        $table->foreignId('strategy_id')->constrained()->cascadeOnDelete();
        $table->jsonb('markets')->default('[]');
        $table->decimal('max_position_usdc', 18, 6)->default(100);
        $table->boolean('is_running')->default(false);
        $table->timestamp('started_at')->nullable();
        $table->unique(['wallet_id', 'strategy_id']);
    });
}

public function down(): void
{
    Schema::dropIfExists('wallet_strategies');
}
```

### Commit:
```bash
git add database/migrations/*create_wallet_strategies*
git commit -m "db: create wallet_strategies table"
```

---

## Task 9: Create PostgreSQL migration — Watched Wallets

```bash
docker compose exec app php artisan make:migration create_watched_wallets_table
```

```php
public function up(): void
{
    Schema::create('watched_wallets', function (Blueprint $table) {
        $table->id();
        $table->string('address')->unique();
        $table->string('label')->nullable();
        $table->integer('follower_count')->default(0);
        $table->decimal('win_rate', 5, 4)->nullable();
        $table->decimal('total_pnl_usdc', 18, 6)->nullable();
        $table->decimal('avg_slippage', 8, 6)->nullable();
        $table->timestamp('last_seen_at')->nullable();
        $table->timestamp('updated_at')->useCurrent();
    });
}

public function down(): void
{
    Schema::dropIfExists('watched_wallets');
}
```

### Commit:
```bash
git add database/migrations/*create_watched_wallets*
git commit -m "db: create watched_wallets table"
```

---

## Task 10: Create PostgreSQL migration — Trades (FK deferred)

```bash
docker compose exec app php artisan make:migration create_trades_table
```

The `copy_relationship_id` column is created but the FK constraint is deferred to Task 13 (circular dependency with `copy_relationships`).

```php
public function up(): void
{
    Schema::create('trades', function (Blueprint $table) {
        $table->id();
        $table->foreignId('wallet_id')->constrained();
        $table->foreignId('strategy_id')->nullable()->constrained();
        $table->unsignedBigInteger('copy_relationship_id')->nullable();
        $table->string('market_id', 100)->nullable();
        $table->string('side', 10)->nullable();
        $table->string('outcome', 10)->nullable();
        $table->decimal('price', 10, 6)->nullable();
        $table->decimal('size_usdc', 18, 6)->nullable();
        $table->string('order_type', 20)->nullable();
        $table->string('status', 20)->default('pending');
        $table->string('polymarket_order_id')->nullable();
        $table->smallInteger('fee_bps')->nullable();
        $table->timestamp('executed_at')->nullable();
        $table->timestamp('created_at')->useCurrent();
        $table->index(['wallet_id', 'created_at'], 'idx_trades_wallet');
        $table->index('market_id', 'idx_trades_market');
    });
}

public function down(): void
{
    Schema::dropIfExists('trades');
}
```

### Commit:
```bash
git add database/migrations/*create_trades*
git commit -m "db: create trades table (copy_relationship FK deferred)"
```

---

## Task 11: Create PostgreSQL migration — Copy Relationships

```bash
docker compose exec app php artisan make:migration create_copy_relationships_table
```

```php
public function up(): void
{
    Schema::create('copy_relationships', function (Blueprint $table) {
        $table->id();
        $table->foreignId('follower_wallet_id')->constrained('wallets')->cascadeOnDelete();
        $table->foreignId('watched_wallet_id')->constrained('watched_wallets')->cascadeOnDelete();
        $table->string('size_mode', 20)->default('proportional');
        $table->decimal('size_value', 18, 6);
        $table->decimal('max_position_usdc', 18, 6)->default(100);
        $table->jsonb('markets_filter')->nullable();
        $table->boolean('is_active')->default(true);
        $table->timestamp('created_at')->useCurrent();
        $table->unique(['follower_wallet_id', 'watched_wallet_id']);
    });
}

public function down(): void
{
    Schema::dropIfExists('copy_relationships');
}
```

### Commit:
```bash
git add database/migrations/*create_copy_relationships*
git commit -m "db: create copy_relationships table"
```

---

## Task 12: Create PostgreSQL migration — Copy Trades

```bash
docker compose exec app php artisan make:migration create_copy_trades_table
```

```php
public function up(): void
{
    Schema::create('copy_trades', function (Blueprint $table) {
        $table->id();
        $table->foreignId('copy_relationship_id')->constrained();
        $table->foreignId('follower_trade_id')->nullable()->constrained('trades');
        $table->string('leader_address');
        $table->string('leader_market_id', 100)->nullable();
        $table->string('leader_outcome', 10)->nullable();
        $table->decimal('leader_price', 10, 6)->nullable();
        $table->decimal('leader_size_usdc', 18, 6)->nullable();
        $table->string('leader_tx_hash')->nullable();
        $table->decimal('follower_price', 10, 6)->nullable();
        $table->decimal('slippage_pct', 8, 6)->nullable();
        $table->string('status', 20)->default('pending');
        $table->string('skip_reason')->nullable();
        $table->timestamp('detected_at');
        $table->timestamp('executed_at')->nullable();
        $table->timestamp('created_at')->useCurrent();
    });
}

public function down(): void
{
    Schema::dropIfExists('copy_trades');
}
```

### Commit:
```bash
git add database/migrations/*create_copy_trades*
git commit -m "db: create copy_trades table"
```

---

## Task 13: Add deferred FK — trades.copy_relationship_id

```bash
docker compose exec app php artisan make:migration add_copy_relationship_fk_to_trades_table --table=trades
```

```php
public function up(): void
{
    Schema::table('trades', function (Blueprint $table) {
        $table->foreign('copy_relationship_id')
              ->references('id')
              ->on('copy_relationships')
              ->nullOnDelete();
    });
}

public function down(): void
{
    Schema::table('trades', function (Blueprint $table) {
        $table->dropForeign(['copy_relationship_id']);
    });
}
```

### Commit:
```bash
git add database/migrations/*add_copy_relationship_fk*
git commit -m "db: add deferred FK copy_relationship_id on trades"
```

---

## Task 14: Create PostgreSQL migration — Backtest Results

```bash
docker compose exec app php artisan make:migration create_backtest_results_table
```

```php
public function up(): void
{
    Schema::create('backtest_results', function (Blueprint $table) {
        $table->id();
        $table->foreignId('user_id')->constrained();
        $table->foreignId('strategy_id')->constrained();
        $table->jsonb('market_filter')->nullable();
        $table->timestamp('date_from')->nullable();
        $table->timestamp('date_to')->nullable();
        $table->integer('total_trades')->nullable();
        $table->decimal('win_rate', 5, 4)->nullable();
        $table->decimal('total_pnl_usdc', 18, 6)->nullable();
        $table->decimal('max_drawdown', 5, 4)->nullable();
        $table->decimal('sharpe_ratio', 8, 4)->nullable();
        $table->jsonb('result_detail')->nullable();
        $table->timestamp('created_at')->useCurrent();
    });
}

public function down(): void
{
    Schema::dropIfExists('backtest_results');
}
```

### Commit:
```bash
git add database/migrations/*create_backtest_results*
git commit -m "db: create backtest_results table"
```

---

## Task 15: Create PostgreSQL migrations — Subscriptions (Cashier)

```bash
docker compose exec app php artisan make:migration create_subscriptions_table
docker compose exec app php artisan make:migration create_subscription_items_table
```

**subscriptions:**
```php
public function up(): void
{
    Schema::create('subscriptions', function (Blueprint $table) {
        $table->id();
        $table->foreignId('user_id')->constrained()->cascadeOnDelete();
        $table->string('type');
        $table->string('stripe_id')->unique();
        $table->string('stripe_status');
        $table->string('stripe_price')->nullable();
        $table->integer('quantity')->default(1);
        $table->timestamp('trial_ends_at')->nullable();
        $table->timestamp('ends_at')->nullable();
        $table->timestamps();
    });
}

public function down(): void
{
    Schema::dropIfExists('subscriptions');
}
```

**subscription_items:**
```php
public function up(): void
{
    Schema::create('subscription_items', function (Blueprint $table) {
        $table->id();
        $table->foreignId('subscription_id')->constrained()->cascadeOnDelete();
        $table->string('stripe_id')->unique();
        $table->string('stripe_product')->nullable();
        $table->string('stripe_price');
        $table->integer('quantity')->default(1);
        $table->timestamps();
    });
}

public function down(): void
{
    Schema::dropIfExists('subscription_items');
}
```

### Commit:
```bash
git add database/migrations/*create_subscription*
git commit -m "db: create subscriptions and subscription_items tables"
```

---

## Task 16: Create Rust engine skeleton

**Files:**
- Create: `engine/Cargo.toml`
- Create: `engine/src/main.rs`

### Step 1: Create Cargo.toml

```toml
[package]
name = "craftstrat-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rdkafka = { version = "0.37", features = ["cmake-build"] }
clickhouse = { version = "0.13", features = ["time"] }
redis = { version = "0.27", features = ["tokio-comp"] }
chrono = { version = "0.4", features = ["serde"] }
rayon = "1.10"
reqwest = { version = "0.12", features = ["json"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Step 2: Create main.rs

```rust
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::init();
    tracing::info!("CraftStrat engine starting...");
}
```

### Step 3: Add to .gitignore

Append `engine/target/` to `.gitignore`.

### Step 4: Commit (don't compile yet — no Rust in Docker at this stage)

```bash
git add engine/Cargo.toml engine/src/main.rs .gitignore
git commit -m "engine: add Rust skeleton with dependencies"
```

---

## Task 17: Run migrations and verify everything works

### Step 1: Start all services

```bash
docker compose up -d
```

### Step 2: Verify containers are running

```bash
docker compose ps
```

Expected: all 6 services up.

### Step 3: Run Laravel migrations

```bash
docker compose exec app php artisan migrate
```

Expected: all migrations pass.

### Step 4: Verify app responds

```bash
curl -s -o /dev/null -w "%{http_code}" http://localhost:8000
```

Expected: `200`

### Step 5: Verify ClickHouse

```bash
curl "http://localhost:8123/?query=SHOW+TABLES"
```

Expected: `slot_snapshots`

### Step 6: Verify Redis

```bash
docker compose exec redis redis-cli ping
```

Expected: `PONG`

### Step 7: Verify Kafka

```bash
docker compose exec kafka kafka-topics --bootstrap-server localhost:9092 --list
```

Expected: no errors.

### Step 8: Final commit if fixes were needed

```bash
git add -A
git commit -m "infra: fix issues from integration verification"
```

---

## Summary

| Task | Description | Method |
|------|-------------|--------|
| 1 | Dockerfile + docker-compose + Nginx | Write files directly |
| 2 | ClickHouse schema | Write file directly |
| 3 | .env configuration | Write file directly |
| 4 | Scaffold Laravel 12 + React via Docker | `docker compose run` |
| 5 | Migration: users extension | `docker compose exec` |
| 6 | Migration: wallets | `docker compose exec` |
| 7 | Migration: strategies | `docker compose exec` |
| 8 | Migration: wallet_strategies | `docker compose exec` |
| 9 | Migration: watched_wallets | `docker compose exec` |
| 10 | Migration: trades (FK deferred) | `docker compose exec` |
| 11 | Migration: copy_relationships | `docker compose exec` |
| 12 | Migration: copy_trades | `docker compose exec` |
| 13 | Migration: deferred FK on trades | `docker compose exec` |
| 14 | Migration: backtest_results | `docker compose exec` |
| 15 | Migration: subscriptions | `docker compose exec` |
| 16 | Rust engine skeleton | Write files directly |
| 17 | Boot + verify everything | `docker compose up` |
