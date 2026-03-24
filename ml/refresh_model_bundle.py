#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, replace
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib import parse, request

from xgboost_pipeline import DEFAULT_METADATA_NAME, DEFAULT_MODEL_NAME


LATEST_CANDIDATE_FILE = "latest-candidate.json"
LAST_PROMOTION_FILE = "last-promotion.json"


@dataclass(frozen=True)
class RefreshConfig:
    engine_internal_url: str
    artifacts_dir: Path
    data_dir: Path
    model_name: str
    slot_duration: int
    symbols: str
    hours: float
    sample_every: int
    limit: int
    max_rows: int
    verbose_eval: int
    rl_gamma: float

    @property
    def candidates_dir(self) -> Path:
        return self.artifacts_dir / "candidates"

    @property
    def backups_dir(self) -> Path:
        return self.artifacts_dir / "backups"

    @property
    def datasets_dir(self) -> Path:
        return self.data_dir / "datasets"

    @property
    def live_dir(self) -> Path:
        return self.artifacts_dir / self.model_name


def _env(name: str, default: str) -> str:
    return str(os.environ.get(name, default))


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def utc_timestamp() -> str:
    return utc_now().strftime("%Y%m%d-%H%M%S")


def load_json(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def parse_symbols(value: str) -> list[str]:
    return [part.strip().upper() for part in value.split(",") if part.strip()]


def config_from_env() -> RefreshConfig:
    return RefreshConfig(
        engine_internal_url=_env("ENGINE_INTERNAL_URL", "http://engine:8080"),
        artifacts_dir=Path(_env("ML_ARTIFACTS_DIR", "/models")).resolve(),
        data_dir=Path(_env("ML_DATA_DIR", "/data")).resolve(),
        model_name=_env("ML_MODEL_NAME", "btc-15m-xgb-policy"),
        slot_duration=int(_env("ML_REFRESH_SLOT_DURATION", "900")),
        symbols=_env("ML_REFRESH_SYMBOLS", "BTC"),
        hours=float(_env("ML_REFRESH_HOURS", "720")),
        sample_every=int(_env("ML_REFRESH_SAMPLE_EVERY", "6")),
        limit=int(_env("ML_REFRESH_LIMIT", "5000")),
        max_rows=int(_env("ML_REFRESH_MAX_ROWS", "0")),
        verbose_eval=int(_env("ML_TRAIN_VERBOSE_EVAL", "50")),
        rl_gamma=float(_env("ML_TRAIN_RL_GAMMA", "0.999")),
    )


def apply_overrides(config: RefreshConfig, overrides: dict[str, Any] | None) -> RefreshConfig:
    if not overrides:
        return config

    updates: dict[str, Any] = {}
    for key in [
        "slot_duration",
        "symbols",
        "hours",
        "sample_every",
        "limit",
        "max_rows",
        "verbose_eval",
        "rl_gamma",
    ]:
        value = overrides.get(key)
        if value in (None, ""):
            continue
        updates[key] = value

    casts = {
        "slot_duration": int,
        "symbols": str,
        "hours": float,
        "sample_every": int,
        "limit": int,
        "max_rows": int,
        "verbose_eval": int,
        "rl_gamma": float,
    }
    for key, caster in casts.items():
        if key in updates:
            updates[key] = caster(updates[key])

    return replace(config, **updates)


def fetch_dataset(config: RefreshConfig, dataset_path: Path) -> dict[str, Any]:
    dataset_path.parent.mkdir(parents=True, exist_ok=True)
    offset = 0
    total = 0
    pages = 0
    started_at = utc_now().isoformat()
    symbols = ",".join(parse_symbols(config.symbols))

    with dataset_path.open("w", encoding="utf-8") as handle:
        while True:
            query = parse.urlencode(
                {
                    "slot_duration": config.slot_duration,
                    "symbols": symbols or None,
                    "hours": config.hours,
                    "sample_every": config.sample_every,
                    "limit": config.limit,
                    "offset": offset,
                }
            )
            url = f"{config.engine_internal_url.rstrip('/')}/internal/stats/slots/ml-dataset?{query}"
            with request.urlopen(url, timeout=120) as response:
                payload = json.loads(response.read().decode("utf-8"))

            rows = payload.get("rows", [])
            for row in rows:
                if config.max_rows > 0 and total >= config.max_rows:
                    break
                handle.write(json.dumps(row, separators=(",", ":")) + "\n")
                total += 1

            pages += 1
            count = len(rows)
            offset += count

            if count < config.limit:
                break
            if config.max_rows > 0 and total >= config.max_rows:
                break

    return {
        "started_at": started_at,
        "finished_at": utc_now().isoformat(),
        "dataset_path": str(dataset_path),
        "rows": total,
        "pages": pages,
        "slot_duration": config.slot_duration,
        "symbols": parse_symbols(config.symbols),
        "hours": config.hours,
        "sample_every": config.sample_every,
        "limit": config.limit,
        "max_rows": config.max_rows,
    }


def train_bundle(config: RefreshConfig, dataset_path: Path, candidate_dir: Path) -> dict[str, Any]:
    candidate_dir.parent.mkdir(parents=True, exist_ok=True)
    command = [
        sys.executable,
        str(Path(__file__).with_name("train_xgboost.py")),
        "train",
        "--dataset",
        str(dataset_path),
        "--output-dir",
        str(candidate_dir),
        "--verbose-eval",
        str(config.verbose_eval),
        "--rl-gamma",
        str(config.rl_gamma),
    ]
    started = time.perf_counter()
    subprocess.run(command, check=True)
    elapsed = time.perf_counter() - started
    return {
        "command": command,
        "elapsed_sec": round(elapsed, 3),
    }


def summarize_metadata(metadata: dict[str, Any]) -> dict[str, Any]:
    return {
        "created_at": metadata.get("created_at"),
        "policy": metadata.get("policy", {}).get("recommended", {}),
        "rl_like": metadata.get("rl_like", {}),
        "split_sizes": metadata.get("split_sizes", {}),
        "metrics": metadata.get("metrics", {}),
        "regression_metrics": metadata.get("regression_metrics", {}),
    }


def compare_bundles(live_metadata: dict[str, Any] | None, candidate_metadata: dict[str, Any]) -> dict[str, Any]:
    if not live_metadata:
        return {"has_live_bundle": False}

    live_policy = live_metadata.get("policy", {}).get("recommended", {})
    candidate_policy = candidate_metadata.get("policy", {}).get("recommended", {})
    live_entry = live_metadata.get("rl_like", {}).get("entry_policy", {}).get("recommended", {})
    candidate_entry = candidate_metadata.get("rl_like", {}).get("entry_policy", {}).get("recommended", {})

    return {
        "has_live_bundle": True,
        "policy_total_pnl_delta": round(
            float(candidate_policy.get("total_pnl_per_1usdc", 0.0))
            - float(live_policy.get("total_pnl_per_1usdc", 0.0)),
            6,
        ),
        "policy_win_rate_delta": round(
            float(candidate_policy.get("win_rate", 0.0))
            - float(live_policy.get("win_rate", 0.0)),
            6,
        ),
        "entry_total_reward_delta": round(
            float(candidate_entry.get("total_reward_per_contract", 0.0))
            - float(live_entry.get("total_reward_per_contract", 0.0)),
            6,
        ),
        "entry_win_rate_delta": round(
            float(candidate_entry.get("win_rate", 0.0))
            - float(live_entry.get("win_rate", 0.0)),
            6,
        ),
    }


def refresh_candidate(config: RefreshConfig, overrides: dict[str, Any] | None = None) -> dict[str, Any]:
    config = apply_overrides(config, overrides)
    run_id = utc_timestamp()
    candidate_name = f"{config.model_name}-candidate-{run_id}"
    dataset_path = config.datasets_dir / f"{candidate_name}.ndjson"
    candidate_dir = config.candidates_dir / candidate_name

    export_summary = fetch_dataset(config, dataset_path)
    train_summary = train_bundle(config, dataset_path, candidate_dir)

    metadata_path = candidate_dir / DEFAULT_METADATA_NAME
    candidate_metadata = load_json(metadata_path)
    if candidate_metadata is None:
        raise FileNotFoundError(f"missing trained metadata file: {metadata_path}")

    live_metadata = load_json(config.live_dir / DEFAULT_METADATA_NAME)

    report = {
        "ok": True,
        "run_id": run_id,
        "candidate_name": candidate_name,
        "candidate_dir": str(candidate_dir),
        "live_dir": str(config.live_dir),
        "export": export_summary,
        "train": train_summary,
        "candidate": summarize_metadata(candidate_metadata),
        "comparison": compare_bundles(live_metadata, candidate_metadata),
        "generated_at": utc_now().isoformat(),
    }

    write_json(candidate_dir / "report.json", report)
    write_json(config.artifacts_dir / LATEST_CANDIDATE_FILE, report)
    return report


def resolve_candidate_dir(config: RefreshConfig, candidate_name: str | None) -> Path:
    if candidate_name:
        candidate_dir = (config.candidates_dir / candidate_name).resolve()
    else:
        latest = load_json(config.artifacts_dir / LATEST_CANDIDATE_FILE)
        if latest and latest.get("candidate_name"):
            candidate_dir = (config.candidates_dir / str(latest["candidate_name"])).resolve()
        else:
            candidates = sorted(
                [
                    path.resolve()
                    for path in config.candidates_dir.glob(f"{config.model_name}-candidate-*")
                    if path.is_dir()
                ]
            )
            if not candidates:
                raise FileNotFoundError("no candidate bundle available to promote")
            candidate_dir = candidates[-1]

    if not candidate_dir.is_relative_to(config.candidates_dir.resolve()):
        raise ValueError("candidate_name resolves outside candidates directory")

    if not (candidate_dir / DEFAULT_MODEL_NAME).exists() or not (candidate_dir / DEFAULT_METADATA_NAME).exists():
        raise FileNotFoundError(f"candidate bundle is incomplete: {candidate_dir}")

    return candidate_dir


def promote_candidate(config: RefreshConfig, candidate_name: str | None = None) -> dict[str, Any]:
    config.artifacts_dir.mkdir(parents=True, exist_ok=True)
    config.backups_dir.mkdir(parents=True, exist_ok=True)
    config.candidates_dir.mkdir(parents=True, exist_ok=True)

    candidate_dir = resolve_candidate_dir(config, candidate_name)
    live_dir = config.live_dir
    backup_dir = config.backups_dir / f"{config.model_name}-{utc_timestamp()}"

    if live_dir.exists():
        live_dir.rename(backup_dir)

    try:
        candidate_dir.rename(live_dir)
    except Exception:
        if backup_dir.exists() and not live_dir.exists():
            backup_dir.rename(live_dir)
        raise

    metadata = load_json(live_dir / DEFAULT_METADATA_NAME) or {}
    report = {
        "ok": True,
        "promoted_at": utc_now().isoformat(),
        "live_dir": str(live_dir),
        "backup_dir": str(backup_dir) if backup_dir.exists() else None,
        "promoted_from": candidate_dir.name,
        "live_bundle": summarize_metadata(metadata),
    }
    write_json(config.artifacts_dir / LAST_PROMOTION_FILE, report)
    return report


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Refresh and promote ML model bundles.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    refresh = subparsers.add_parser("refresh", help="export a fresh dataset and train a candidate bundle")
    refresh.add_argument("--slot-duration", type=int)
    refresh.add_argument("--symbols")
    refresh.add_argument("--hours", type=float)
    refresh.add_argument("--sample-every", type=int)
    refresh.add_argument("--limit", type=int)
    refresh.add_argument("--max-rows", type=int)
    refresh.add_argument("--verbose-eval", type=int)
    refresh.add_argument("--rl-gamma", type=float)

    promote = subparsers.add_parser("promote", help="promote a candidate bundle into the live model path")
    promote.add_argument("--candidate-name")
    return parser


def main() -> int:
    args = build_parser().parse_args()
    config = config_from_env()

    if args.command == "refresh":
        payload = {
            "slot_duration": args.slot_duration,
            "symbols": args.symbols,
            "hours": args.hours,
            "sample_every": args.sample_every,
            "limit": args.limit,
            "max_rows": args.max_rows,
            "verbose_eval": args.verbose_eval,
            "rl_gamma": args.rl_gamma,
        }
        print(json.dumps(refresh_candidate(config, payload), indent=2, sort_keys=True))
        return 0

    if args.command == "promote":
        print(json.dumps(promote_candidate(config, args.candidate_name), indent=2, sort_keys=True))
        return 0

    raise ValueError(f"unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
