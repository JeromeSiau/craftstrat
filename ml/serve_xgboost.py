#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from xgboost_pipeline import load_bundle, score_rows


def build_handler(model_dir: str):
    booster, metadata = load_bundle(model_dir)

    class Handler(BaseHTTPRequestHandler):
        server_version = "craftstrat-ml/0.1"

        def _send_json(self, payload: dict[str, Any], status: int = HTTPStatus.OK) -> None:
            body = json.dumps(payload).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self) -> None:  # noqa: N802
            if self.path == "/health":
                self._send_json(
                    {
                        "ok": True,
                        "model_dir": str(Path(model_dir).resolve()),
                        "features": metadata["feature_names"],
                        "recommended_min_edge": metadata.get("thresholds", {}).get(
                            "recommended_min_edge",
                            0.0,
                        ),
                    }
                )
                return

            self._send_json({"error": "not found"}, status=HTTPStatus.NOT_FOUND)

        def do_POST(self) -> None:  # noqa: N802
            if self.path != "/predict":
                self._send_json({"error": "not found"}, status=HTTPStatus.NOT_FOUND)
                return

            content_length = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(content_length)
            try:
                payload = json.loads(raw or b"{}")
            except json.JSONDecodeError as exc:
                self._send_json(
                    {"error": f"invalid json: {exc.msg}"},
                    status=HTTPStatus.BAD_REQUEST,
                )
                return

            if isinstance(payload, dict) and "rows" in payload:
                rows = payload["rows"]
            elif isinstance(payload, dict) and "row" in payload:
                rows = [payload["row"]]
            elif isinstance(payload, dict):
                rows = [payload]
            else:
                self._send_json(
                    {"error": "payload must be an object with row or rows"},
                    status=HTTPStatus.BAD_REQUEST,
                )
                return

            if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
                self._send_json(
                    {"error": "rows must be a list of objects"},
                    status=HTTPStatus.BAD_REQUEST,
                )
                return

            predictions = score_rows(rows, booster, metadata)
            response: dict[str, Any] = {
                "count": len(predictions),
                "predictions": predictions,
            }
            if len(predictions) == 1:
                response.update(predictions[0])

            self._send_json(response)

        def log_message(self, format: str, *args: Any) -> None:  # noqa: A003
            return

    return Handler


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Serve the trained XGBoost model over HTTP.")
    parser.add_argument("--model-dir", required=True, help="directory containing model.json and metadata.json")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8010)
    return parser


def main() -> int:
    args = build_parser().parse_args()
    handler = build_handler(args.model_dir)
    server = ThreadingHTTPServer((args.host, args.port), handler)
    print(f"serving model from {Path(args.model_dir).resolve()} on http://{args.host}:{args.port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
