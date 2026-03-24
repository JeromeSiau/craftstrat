#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import threading
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

from refresh_model_bundle import (
    LAST_PROMOTION_FILE,
    LATEST_CANDIDATE_FILE,
    config_from_env,
    load_json,
    promote_candidate,
    refresh_candidate,
)


class RefreshLock:
    def __init__(self):
        self._lock = threading.Lock()

    def acquire(self) -> bool:
        return self._lock.acquire(blocking=False)

    def release(self) -> None:
        self._lock.release()


def build_handler():
    config = config_from_env()
    refresh_lock = RefreshLock()

    class Handler(BaseHTTPRequestHandler):
        server_version = "craftstrat-ml-trainer/0.1"

        def _send_json(self, payload: dict[str, Any], status: int = HTTPStatus.OK) -> None:
            body = json.dumps(payload).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def _read_json(self) -> dict[str, Any]:
            content_length = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(content_length)
            if not raw:
                return {}
            payload = json.loads(raw)
            if not isinstance(payload, dict):
                raise ValueError("payload must be a JSON object")
            return payload

        def do_GET(self) -> None:  # noqa: N802
            if self.path != "/health":
                self._send_json({"error": "not found"}, status=HTTPStatus.NOT_FOUND)
                return

            self._send_json(
                {
                    "ok": True,
                    "engine_internal_url": config.engine_internal_url,
                    "artifacts_dir": str(config.artifacts_dir),
                    "data_dir": str(config.data_dir),
                    "model_name": config.model_name,
                    "refresh_defaults": {
                        "slot_duration": config.slot_duration,
                        "symbols": config.symbols,
                        "hours": config.hours,
                        "sample_every": config.sample_every,
                        "limit": config.limit,
                        "max_rows": config.max_rows,
                        "verbose_eval": config.verbose_eval,
                        "rl_gamma": config.rl_gamma,
                    },
                    "latest_candidate": load_json(config.artifacts_dir / LATEST_CANDIDATE_FILE),
                    "last_promotion": load_json(config.artifacts_dir / LAST_PROMOTION_FILE),
                }
            )

        def do_POST(self) -> None:  # noqa: N802
            try:
                payload = self._read_json()
            except (json.JSONDecodeError, ValueError) as exc:
                self._send_json({"error": str(exc)}, status=HTTPStatus.BAD_REQUEST)
                return

            if self.path == "/refresh":
                if not refresh_lock.acquire():
                    self._send_json({"error": "refresh already running"}, status=HTTPStatus.CONFLICT)
                    return

                try:
                    report = refresh_candidate(config, payload)
                except Exception as exc:
                    self._send_json({"error": str(exc)}, status=HTTPStatus.INTERNAL_SERVER_ERROR)
                else:
                    self._send_json(report)
                finally:
                    refresh_lock.release()
                return

            if self.path == "/promote":
                try:
                    report = promote_candidate(config, payload.get("candidate_name"))
                except Exception as exc:
                    self._send_json({"error": str(exc)}, status=HTTPStatus.INTERNAL_SERVER_ERROR)
                else:
                    self._send_json(report)
                return

            self._send_json({"error": "not found"}, status=HTTPStatus.NOT_FOUND)

        def log_message(self, format: str, *args: Any) -> None:  # noqa: A003
            return

    return Handler


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Serve the ML trainer control plane over HTTP.")
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8011)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    handler = build_handler()
    server = ThreadingHTTPServer((args.host, args.port), handler)
    print(f"serving trainer on http://{args.host}:{args.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
