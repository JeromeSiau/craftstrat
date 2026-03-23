# AGENTS.md

This file provides guidance to coding agents working in this repository.

## Project Overview

CraftStrat (repo name: `oddex`) is a SaaS platform for automated trading on Polymarket prediction markets. Users build strategies via a no-code form builder or a visual node editor, backtest against historical order book data, and run them live across multiple Ethereum/Polygon wallets.

## Monorepo Structure

```text
oddex/
├── web/           # Laravel 12 + Inertia.js/React SPA
├── engine/        # Rust trading engine (Tokio/Axum)
├── infra/         # Dockerfiles, Nginx config, ClickHouse init SQL, backup script
├── docs/          # SPEC.md + implementation plans
├── docker-compose.yml          # Production services
└── docker-compose.override.yml # Dev overrides (bind mounts, extra ports)
```

Working directory conventions:

- Repo-level infra and Docker commands run from the repository root.
- Laravel, frontend, and test commands below run from `web/` unless stated otherwise.
- In development, `docker compose up -d` automatically merges `docker-compose.override.yml`.
- In production, always pin the compose file explicitly with `docker compose -f docker-compose.yml ...`.

## Common Commands

### Development

```bash
cd web && composer run dev     # Starts server, queue, logs, Vite concurrently
docker compose up -d           # Dev (auto-merges override file)
```

### Testing

```bash
cd web && php artisan test --compact
cd web && php artisan test --compact tests/Feature/SomeTest.php
cd web && php artisan test --compact --filter=testName
```

Tests use SQLite in-memory (`web/phpunit.xml`) with `RefreshDatabase` on feature tests.

### Linting & Formatting

```bash
cd web && vendor/bin/pint --dirty --format agent
cd web && npm run lint
cd web && npm run format
cd web && npm run types
```

### Building

```bash
cd web && npm run build
cd web && npm run build:ssr
```

### Rebuilding the Rust Engine

If you modify any Rust code in `engine/`, you must rebuild the Docker image and restart the container:

```bash
docker compose -f docker-compose.yml build --no-cache engine
docker compose -f docker-compose.yml up -d engine
```

### Wayfinder

Wayfinder auto-generates TypeScript route helpers on Vite dev/build. Generated files live in:

- `web/resources/js/actions/`
- `web/resources/js/routes/`

## Production Access

Critical production access details are intentionally documented here because they are required to inspect the live databases safely.

- Server: `ploi@94.130.218.197`
- Deployed app root: `/home/ploi/craftstrat.com`
- Production compose entrypoint: `docker compose -f docker-compose.yml`
- Never use `docker-compose.override.yml` in production.
- Production env is loaded from the repository root `.env`.

### SSH

```bash
ssh ploi@94.130.218.197
```

After connecting, go to the deployed project root, the directory that contains `.env` and `docker-compose.yml`.

### Load Production Env In Shell

Use this before DB commands so compose and client commands pick up the deployed credentials:

```bash
set -a; source .env; set +a
```

### Production Compose / Logs

```bash
docker compose -f docker-compose.yml ps
docker compose -f docker-compose.yml logs -f app
docker compose -f docker-compose.yml logs -f engine
```

### PostgreSQL Access

Interactive `psql` session:

```bash
set -a; source .env; set +a
docker compose -f docker-compose.yml exec -T postgres \
  psql -U "${DB_USERNAME:-craftstrat}" "${DB_DATABASE:-craftstrat}"
```

One-off query:

```bash
set -a; source .env; set +a
docker compose -f docker-compose.yml exec -T postgres \
  psql -U "${DB_USERNAME:-craftstrat}" "${DB_DATABASE:-craftstrat}" \
  -c "SELECT now();"
```

Consistent dump pattern used by `infra/backup.sh`:

```bash
set -a; source .env; set +a
docker compose -f docker-compose.yml exec -T postgres \
  pg_dump -U "${DB_USERNAME:-craftstrat}" "${DB_DATABASE:-craftstrat}"
```

### ClickHouse Access

Interactive client:

```bash
set -a; source .env; set +a
docker compose -f docker-compose.yml exec -T clickhouse \
  clickhouse-client --password="${CLICKHOUSE_PASSWORD:-clickhouse}"
```

One-off query:

```bash
set -a; source .env; set +a
docker compose -f docker-compose.yml exec -T clickhouse \
  clickhouse-client --password="${CLICKHOUSE_PASSWORD:-clickhouse}" \
  --query="SELECT name FROM system.tables WHERE database = 'default';"
```

Schema export pattern used by `infra/backup.sh`:

```bash
set -a; source .env; set +a
docker compose -f docker-compose.yml exec -T clickhouse \
  clickhouse-client --password="${CLICKHOUSE_PASSWORD:-clickhouse}" \
  --query="SHOW CREATE TABLE default.slot_snapshots FORMAT TabSeparatedRaw"
```

### Laravel Commands In Production

```bash
docker compose -f docker-compose.yml exec -T app php artisan about
docker compose -f docker-compose.yml exec -T app php artisan migrate:status
```

### Production Safety Rules

- Prefer read-only inspection when looking at production data.
- Do not run `migrate`, `db:seed`, ad-hoc `UPDATE`/`DELETE`, or service restarts unless explicitly requested.
- Do not use bare `docker compose up` on prod because the override file is for development only.
- Do not copy secrets from `.env` into committed files.
- Do not copy tracked source files directly onto production as a deployment method. Deploy via Git push only.

### Production Deploy Workflow

When code changes need to go live, use this order:

1. Make and verify the change locally.
2. Commit locally on `main` and `git push origin main`.
3. Wait about 5 minutes for the automatic deploy to update `/home/ploi/craftstrat.com`. This deploy already handles `web/app`.
4. If Rust code changed in `engine/`, SSH to prod and confirm the checkout is clean with `git status --short`.
5. If Rust code changed in `engine/`, run `docker compose -f docker-compose.yml build --no-cache engine`.
6. If Rust code changed in `engine/`, run `docker compose -f docker-compose.yml up -d engine`.
7. Verify with `docker compose -f docker-compose.yml ps` and targeted logs if needed.

If a tracked file was copied manually onto prod for debugging, restore it before the next deploy:

```bash
git restore path/to/file
```

## Architecture

### Web ↔ Engine Communication

Laravel talks to the Rust engine via HTTP (`EngineService` → `ENGINE_INTERNAL_URL`). Key internal endpoints:

- `POST /internal/strategy/activate`
- `POST /internal/strategy/deactivate`
- `POST /internal/backtest/run`
- `GET /internal/wallet/{id}/state`
- `GET /internal/engine/status`
- `POST /internal/copy/watch`
- `POST /internal/copy/unwatch`
- `GET /internal/stats/slots`

### Data Stores

| Store | Purpose |
|-------|---------|
| PostgreSQL 17 | Business data: users, strategies, wallets, trades |
| ClickHouse 26.1 | Time-series slot snapshots and analytics |
| Redis 7 | Cache, queues, transient state |
| Kafka | Tick distribution from engine to ClickHouse |

### Backend Key Patterns

- Services: `EngineService`, `WalletService`, `StrategyActivationService`, `BillingService`
- Wallet encryption: AES-256-GCM via `ENCRYPTION_KEY`
- Authorization: Laravel policies (`StrategyPolicy`, `WalletPolicy`, `BacktestResultPolicy`)
- Plan limits: `CheckPlanLimits` middleware with tiers in `web/config/plans.php`
- Deferred props: `Inertia::defer()` for live stats on strategy pages
- Engine HTTP mocking in tests: `Http::fake()`

### Frontend Key Patterns

- Routing: Wayfinder imports from `@/actions/...`
- UI: shadcn/ui + Radix primitives in `web/resources/js/components/ui/`
- Strategy editor: React Flow for node mode, custom form builder for form mode
- Charts: Recharts
- Shared TS models: `web/resources/js/types/models.ts`
- Shared formatters: `web/resources/js/lib/formatters.ts`

### Rust Engine

Key modules in `engine/src/`:

- `strategy/` for evaluation, interpreters, indicators, state
- `execution/` for order signing, fees, execution queue
- `fetcher/` for Polymarket WebSocket and Gamma API ingestion
- `backtest/` for historical replay against ClickHouse
- `watcher/` for copy trading
- `stats/` for analytical queries
- `storage/` for PostgreSQL, ClickHouse, Redis integration

## Project Conventions

- Follow existing patterns in sibling files before introducing a new structure.
- Reuse existing services, components, policies, and utilities before creating new ones.
- Do not add or change dependencies without explicit approval.
- If you edit PHP files, run Pint afterwards.
- If you edit frontend code, run the relevant `lint`, `format`, `types`, or build command for the scope of the change.
- If a frontend change does not appear locally, the missing step is often `npm run dev`, `npm run build`, or `composer run dev`.
- Only create additional documentation files when explicitly requested.
