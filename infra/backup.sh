#!/bin/bash
set -euo pipefail

# ──────────────────────────────────────────────
# CraftStrat – Full backup script
# Backs up: PostgreSQL, ClickHouse, project files
# Destination synced to cloud via Ploi
# ──────────────────────────────────────────────

BACKUP_DIR="/home/ploi/server/backups/craftstrat"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
set -a; source "$PROJECT_DIR/.env"; set +a
DATE=$(date +%Y-%m-%d-%H%M%S)
PREFIX="craftstrat"
RETENTION_DAYS=7

mkdir -p "$BACKUP_DIR"

echo "[$DATE] Starting backup..."

# ── PostgreSQL ───────────────────────────────
echo "Dumping PostgreSQL..."
docker compose -f "$PROJECT_DIR/docker-compose.yml" exec -T postgres \
  pg_dump -U "${DB_USERNAME:-craftstrat}" "${DB_DATABASE:-craftstrat}" \
  | gzip > "$BACKUP_DIR/${PREFIX}-db-postgres-${DATE}.sql.gz"
echo "PostgreSQL done."

# ── ClickHouse (all tables) ──────────────────
echo "Dumping ClickHouse..."
CH="docker compose -f $PROJECT_DIR/docker-compose.yml exec -T clickhouse clickhouse-client --password=${CLICKHOUSE_PASSWORD:-clickhouse}"
TABLES=$($CH --query="SELECT name FROM system.tables WHERE database = 'default' AND engine NOT LIKE '%Log'" | tr -d '\r')

# Schema (all CREATE TABLE statements in one file)
{
  for TABLE in $TABLES; do
    $CH --query="SHOW CREATE TABLE default.${TABLE} FORMAT TabSeparatedRaw"
    echo ";"
  done
} | gzip > "$BACKUP_DIR/${PREFIX}-db-clickhouse-schema-${DATE}.sql.gz"

# Data (one file per table in Native format)
for TABLE in $TABLES; do
  echo "  - $TABLE"
  $CH --query="SELECT * FROM default.${TABLE} FORMAT Native" \
    | gzip > "$BACKUP_DIR/${PREFIX}-db-clickhouse-${TABLE}-${DATE}.native.gz"
done
echo "ClickHouse done."

# ── Project files ────────────────────────────
echo "Archiving project files..."
tar czf "$BACKUP_DIR/${PREFIX}-files-${DATE}.tar.gz" \
  -C "$(dirname "$PROJECT_DIR")" \
  --exclude="$(basename "$PROJECT_DIR")/web/vendor" \
  --exclude="$(basename "$PROJECT_DIR")/web/node_modules" \
  --exclude="$(basename "$PROJECT_DIR")/engine/target" \
  --exclude="$(basename "$PROJECT_DIR")/.git" \
  "$(basename "$PROJECT_DIR")"
echo "Files done."

# ── Cleanup old backups ─────────────────────
echo "Cleaning backups older than ${RETENTION_DAYS} days..."
find "$BACKUP_DIR" -maxdepth 1 -type f -mtime +$RETENTION_DAYS -delete

echo "[$DATE] Backup complete."
ls -lh "$BACKUP_DIR/${PREFIX}-"*"${DATE}"*
