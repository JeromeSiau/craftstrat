#!/bin/bash
set -euo pipefail

# ──────────────────────────────────────────────
# CraftStrat – Full backup script
# Backs up: PostgreSQL, ClickHouse, project files
# Destination synced to cloud via Ploi
# ──────────────────────────────────────────────

BACKUP_DIR="/home/ploi/server/backups/craftstrat"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
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

# ── ClickHouse (full dump) ───────────────────
echo "Dumping ClickHouse..."
docker compose -f "$PROJECT_DIR/docker-compose.yml" exec -T clickhouse \
  clickhouse-client --password="${CLICKHOUSE_PASSWORD:-clickhouse}" \
  --query="BACKUP DATABASE default TO Disk('backups', '${DATE}')" \
  2>/dev/null
docker compose -f "$PROJECT_DIR/docker-compose.yml" exec -T clickhouse \
  tar czf - -C /var/lib/clickhouse/backups/"${DATE}" . \
  | cat > "$BACKUP_DIR/${PREFIX}-db-clickhouse-${DATE}.tar.gz"
docker compose -f "$PROJECT_DIR/docker-compose.yml" exec -T clickhouse \
  rm -rf /var/lib/clickhouse/backups/"${DATE}"
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
