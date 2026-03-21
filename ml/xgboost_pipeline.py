from __future__ import annotations

import json
import math
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np
import xgboost as xgb

FEATURE_PREFIX = "f_"
TARGET_FIELD = "target_up"
DEFAULT_MODEL_NAME = "model.json"
DEFAULT_METADATA_NAME = "metadata.json"


@dataclass
class Dataset:
    rows: list[dict[str, Any]]
    feature_names: list[str]
    X: np.ndarray
    y: np.ndarray
    group_keys: list[tuple[int, int, str]]
    captured_at: list[datetime]
    ask_up: np.ndarray
    ask_down: np.ndarray


@dataclass
class DataSplit:
    name: str
    X: np.ndarray
    y: np.ndarray
    ask_up: np.ndarray
    ask_down: np.ndarray
    rows: list[dict[str, Any]]

    @property
    def size(self) -> int:
        return int(self.y.shape[0])


@dataclass
class SplitBundle:
    train: DataSplit
    val: DataSplit
    test: DataSplit
    feature_names: list[str]


def parse_timestamp(value: str) -> datetime:
    normalized = value.replace("Z", "+00:00")
    return datetime.fromisoformat(normalized).astimezone(timezone.utc)


def load_ndjson_dataset(path: str | Path) -> Dataset:
    source = Path(path)
    rows: list[dict[str, Any]] = []
    with source.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))

    if not rows:
        raise ValueError(f"dataset is empty: {source}")

    feature_names = sorted(key for key in rows[0].keys() if key.startswith(FEATURE_PREFIX))
    if not feature_names:
        raise ValueError("dataset does not contain any feature columns prefixed with 'f_'")

    X = np.asarray(
        [[float(row.get(feature, 0.0)) for feature in feature_names] for row in rows],
        dtype=np.float32,
    )
    y = np.asarray([float(row[TARGET_FIELD]) for row in rows], dtype=np.float32)
    ask_up = np.asarray([float(row.get("f_ask_up", 0.0)) for row in rows], dtype=np.float32)
    ask_down = np.asarray([float(row.get("f_ask_down", 0.0)) for row in rows], dtype=np.float32)
    group_keys = [
        (int(row["slot_duration"]), int(row["slot_ts"]), str(row["symbol"]))
        for row in rows
    ]
    captured_at = [parse_timestamp(str(row["captured_at"])) for row in rows]

    return Dataset(
        rows=rows,
        feature_names=feature_names,
        X=X,
        y=y,
        group_keys=group_keys,
        captured_at=captured_at,
        ask_up=ask_up,
        ask_down=ask_down,
    )


def build_split(dataset: Dataset, row_indices: np.ndarray, name: str) -> DataSplit:
    index_list = row_indices.tolist()
    return DataSplit(
        name=name,
        X=dataset.X[row_indices],
        y=dataset.y[row_indices],
        ask_up=dataset.ask_up[row_indices],
        ask_down=dataset.ask_down[row_indices],
        rows=[dataset.rows[index] for index in index_list],
    )


def split_dataset(
    dataset: Dataset,
    train_fraction: float = 0.70,
    val_fraction: float = 0.15,
) -> SplitBundle:
    if not 0.0 < train_fraction < 1.0:
        raise ValueError("train_fraction must be between 0 and 1")
    if not 0.0 < val_fraction < 1.0:
        raise ValueError("val_fraction must be between 0 and 1")
    if train_fraction + val_fraction >= 1.0:
        raise ValueError("train_fraction + val_fraction must be < 1")

    grouped: dict[tuple[int, int, str], list[int]] = {}
    group_time: dict[tuple[int, int, str], datetime] = {}
    for idx, group_key in enumerate(dataset.group_keys):
        grouped.setdefault(group_key, []).append(idx)
        group_time.setdefault(group_key, dataset.captured_at[idx])

    ordered_groups = sorted(grouped.keys(), key=lambda key: (group_time[key], key[0], key[2]))
    total_groups = len(ordered_groups)
    if total_groups < 3:
        raise ValueError("dataset needs at least 3 distinct slots to create train/val/test splits")

    train_cut = max(1, int(total_groups * train_fraction))
    val_cut = max(train_cut + 1, int(total_groups * (train_fraction + val_fraction)))
    val_cut = min(val_cut, total_groups - 1)
    train_cut = min(train_cut, val_cut - 1)

    train_groups = ordered_groups[:train_cut]
    val_groups = ordered_groups[train_cut:val_cut]
    test_groups = ordered_groups[val_cut:]

    if not train_groups or not val_groups or not test_groups:
        raise ValueError("split produced an empty partition; add more slots or adjust fractions")

    def flatten(groups: list[tuple[int, int, str]]) -> np.ndarray:
        return np.asarray(
            [idx for group in groups for idx in grouped[group]],
            dtype=np.int64,
        )

    return SplitBundle(
        train=build_split(dataset, flatten(train_groups), "train"),
        val=build_split(dataset, flatten(val_groups), "val"),
        test=build_split(dataset, flatten(test_groups), "test"),
        feature_names=dataset.feature_names,
    )


def build_matrix(X: np.ndarray, y: np.ndarray | None, feature_names: list[str]) -> xgb.DMatrix:
    matrix = xgb.DMatrix(X, feature_names=feature_names)
    if y is not None:
        matrix.set_label(y)
    return matrix


def sigmoid(values: np.ndarray) -> np.ndarray:
    clipped = np.clip(values, -35.0, 35.0)
    return 1.0 / (1.0 + np.exp(-clipped))


def fit_platt_scaler(
    margins: np.ndarray,
    labels: np.ndarray,
    max_iter: int = 50,
    l2: float = 1e-4,
) -> tuple[float, float]:
    a = 1.0
    b = 0.0
    positives = float(np.sum(labels == 1.0))
    negatives = float(np.sum(labels == 0.0))
    high_target = (positives + 1.0) / (positives + 2.0) if positives > 0 else 0.75
    low_target = 1.0 / (negatives + 2.0) if negatives > 0 else 0.25
    smoothed = np.where(labels == 1.0, high_target, low_target)

    for _ in range(max_iter):
        z = a * margins + b
        probs = sigmoid(z)
        weights = np.clip(probs * (1.0 - probs), 1e-6, None)

        grad_a = np.sum((probs - smoothed) * margins) + l2 * a
        grad_b = np.sum(probs - smoothed)

        h_aa = np.sum(weights * margins * margins) + l2
        h_ab = np.sum(weights * margins)
        h_bb = np.sum(weights)

        det = h_aa * h_bb - h_ab * h_ab
        if abs(det) < 1e-12:
            break

        step_a = (h_bb * grad_a - h_ab * grad_b) / det
        step_b = (-h_ab * grad_a + h_aa * grad_b) / det

        a -= step_a
        b -= step_b

        if abs(step_a) < 1e-6 and abs(step_b) < 1e-6:
            break

    return float(a), float(b)


def apply_platt_scaler(margins: np.ndarray, a: float, b: float) -> np.ndarray:
    return sigmoid((a * margins) + b)


def logloss(labels: np.ndarray, probs: np.ndarray) -> float:
    safe = np.clip(probs, 1e-7, 1.0 - 1e-7)
    return float(-np.mean(labels * np.log(safe) + (1.0 - labels) * np.log(1.0 - safe)))


def brier_score(labels: np.ndarray, probs: np.ndarray) -> float:
    return float(np.mean((probs - labels) ** 2))


def roc_auc(labels: np.ndarray, probs: np.ndarray) -> float:
    labels = labels.astype(np.int32)
    positives = int(labels.sum())
    negatives = int(labels.shape[0] - positives)
    if positives == 0 or negatives == 0:
        return 0.5

    order = np.argsort(probs)
    ranks = np.empty_like(order, dtype=np.float64)
    ranks[order] = np.arange(1, probs.shape[0] + 1, dtype=np.float64)
    pos_rank_sum = float(np.sum(ranks[labels == 1]))
    auc = (pos_rank_sum - positives * (positives + 1) / 2.0) / (positives * negatives)
    return float(auc)


def realized_pnl(side: str, label_up: float, ask_up: float, ask_down: float) -> float:
    if side == "UP":
        return (1.0 - ask_up) if label_up == 1.0 else -ask_up
    return (1.0 - ask_down) if label_up == 0.0 else -ask_down


@dataclass
class PolicyInputs:
    spread_up_rel: np.ndarray
    spread_down_rel: np.ndarray
    pct_into_slot: np.ndarray
    log_volume: np.ndarray


def feature_array(
    rows: list[dict[str, Any]],
    key: str,
    default: float = 0.0,
) -> np.ndarray:
    return np.asarray([float(row.get(key, default)) for row in rows], dtype=np.float32)


def build_policy_inputs(rows: list[dict[str, Any]]) -> PolicyInputs:
    return PolicyInputs(
        spread_up_rel=feature_array(rows, "f_spread_up_rel"),
        spread_down_rel=feature_array(rows, "f_spread_down_rel"),
        pct_into_slot=feature_array(rows, "f_pct_into_slot"),
        log_volume=feature_array(rows, "f_log_volume"),
    )


def max_drawdown(pnl: np.ndarray) -> float:
    if pnl.size == 0:
        return 0.0

    cumulative = np.cumsum(pnl, dtype=np.float64)
    peaks = np.maximum.accumulate(cumulative)
    drawdowns = peaks - cumulative
    return float(np.max(drawdowns)) if drawdowns.size > 0 else 0.0


def sharpe_like(pnl: np.ndarray) -> float:
    if pnl.size < 2:
        return float(pnl[0]) if pnl.size == 1 else 0.0

    std = float(np.std(pnl))
    if std <= 1e-12:
        return float(np.mean(pnl))

    return float(np.mean(pnl) / std * math.sqrt(pnl.size))


def recommended_policy(metadata: dict[str, Any]) -> dict[str, float]:
    policy = metadata.get("policy", {}).get("recommended")
    if not isinstance(policy, dict):
        policy = metadata.get("thresholds", {}).get("validation_best", {})

    return {
        "min_edge": float(
            policy.get(
                "min_edge",
                metadata.get("thresholds", {}).get("recommended_min_edge", 0.0),
            )
        ),
        "max_spread_rel": float(policy.get("max_spread_rel", 0.25)),
        "min_pct_into_slot": float(policy.get("min_pct_into_slot", 0.05)),
        "max_pct_into_slot": float(policy.get("max_pct_into_slot", 0.90)),
        "min_log_volume": float(policy.get("min_log_volume", 0.0)),
    }


def policy_masks(
    probs_up: np.ndarray,
    ask_up: np.ndarray,
    ask_down: np.ndarray,
    policy_inputs: PolicyInputs,
    policy: dict[str, float],
) -> dict[str, np.ndarray]:
    probs_down = 1.0 - probs_up
    edges_up = probs_up - ask_up
    edges_down = probs_down - ask_down

    min_edge = float(policy.get("min_edge", 0.0))
    max_spread_rel = float(policy.get("max_spread_rel", 0.25))
    min_pct_into_slot = float(policy.get("min_pct_into_slot", 0.05))
    max_pct_into_slot = float(policy.get("max_pct_into_slot", 0.90))
    min_log_volume = float(policy.get("min_log_volume", 0.0))

    common_mask = (
        (policy_inputs.pct_into_slot >= min_pct_into_slot)
        & (policy_inputs.pct_into_slot <= max_pct_into_slot)
        & (policy_inputs.log_volume >= min_log_volume)
    )
    eligible_up = (
        common_mask
        & (ask_up > 0.0)
        & (policy_inputs.spread_up_rel <= max_spread_rel)
        & (edges_up >= min_edge)
    )
    eligible_down = (
        common_mask
        & (ask_down > 0.0)
        & (policy_inputs.spread_down_rel <= max_spread_rel)
        & (edges_down >= min_edge)
    )
    prefer_up = edges_up >= edges_down
    take_up = eligible_up & (~eligible_down | prefer_up)
    take_down = eligible_down & (~eligible_up | ~prefer_up)

    return {
        "probs_down": probs_down,
        "edges_up": edges_up,
        "edges_down": edges_down,
        "take_up": take_up,
        "take_down": take_down,
        "take_trade": take_up | take_down,
    }


def evaluate_policy(
    probs_up: np.ndarray,
    labels: np.ndarray,
    ask_up: np.ndarray,
    ask_down: np.ndarray,
    policy_inputs: PolicyInputs,
    policy: dict[str, float],
) -> dict[str, Any]:
    masks = policy_masks(probs_up, ask_up, ask_down, policy_inputs, policy)
    take_up = masks["take_up"]
    take_down = masks["take_down"]
    take_trade = masks["take_trade"]
    trade_count = int(np.sum(take_trade))

    summary = {
        "min_edge": round(float(policy.get("min_edge", 0.0)), 4),
        "max_spread_rel": round(float(policy.get("max_spread_rel", 0.25)), 4),
        "min_pct_into_slot": round(float(policy.get("min_pct_into_slot", 0.05)), 4),
        "max_pct_into_slot": round(float(policy.get("max_pct_into_slot", 0.90)), 4),
        "min_log_volume": round(float(policy.get("min_log_volume", 0.0)), 4),
        "trades": trade_count,
        "up_trades": int(np.sum(take_up)),
        "down_trades": int(np.sum(take_down)),
        "total_pnl_per_1usdc": 0.0,
        "avg_pnl_per_trade": 0.0,
        "win_rate": 0.0,
        "max_drawdown_per_1usdc": 0.0,
        "pnl_to_drawdown": 0.0,
        "sharpe_like": 0.0,
    }
    if trade_count == 0:
        return summary

    pnl = np.zeros(labels.shape[0], dtype=np.float64)
    pnl[take_up] = np.where(labels[take_up] == 1.0, 1.0 - ask_up[take_up], -ask_up[take_up])
    pnl[take_down] = np.where(
        labels[take_down] == 0.0,
        1.0 - ask_down[take_down],
        -ask_down[take_down],
    )
    trade_pnl = pnl[take_trade]
    total_pnl = float(np.sum(trade_pnl))
    avg_pnl = float(np.mean(trade_pnl))
    dd = max_drawdown(trade_pnl)

    summary.update(
        {
            "total_pnl_per_1usdc": round(total_pnl, 6),
            "avg_pnl_per_trade": round(avg_pnl, 6),
            "win_rate": round(float(np.mean(trade_pnl > 0.0)), 6),
            "max_drawdown_per_1usdc": round(dd, 6),
            "pnl_to_drawdown": round(total_pnl / dd, 6) if dd > 1e-9 else round(total_pnl, 6),
            "sharpe_like": round(sharpe_like(trade_pnl), 6),
        }
    )
    return summary


def sweep_edge_thresholds(
    probs_up: np.ndarray,
    labels: np.ndarray,
    ask_up: np.ndarray,
    ask_down: np.ndarray,
    rows: list[dict[str, Any]],
) -> dict[str, Any]:
    thresholds = np.arange(0.0, 0.2001, 0.01, dtype=np.float32)
    max_spread_thresholds = np.asarray([0.03, 0.05, 0.07, 0.10, 0.15, 0.25], dtype=np.float32)
    timing_windows = [
        (0.05, 0.90),
        (0.10, 0.85),
        (0.15, 0.80),
        (0.20, 0.75),
        (0.25, 0.70),
    ]
    policy_inputs = build_policy_inputs(rows)
    finite_volume = policy_inputs.log_volume[np.isfinite(policy_inputs.log_volume)]
    if finite_volume.size == 0:
        volume_thresholds = np.asarray([0.0], dtype=np.float32)
    else:
        volume_thresholds = np.unique(
            np.round(np.quantile(finite_volume, [0.0, 0.25, 0.5, 0.75]), 4)
        ).astype(np.float32)

    min_trades = max(25, labels.shape[0] // 200)
    best: dict[str, Any] | None = None

    def is_better(candidate: dict[str, Any], incumbent: dict[str, Any] | None) -> bool:
        if incumbent is None:
            return True

        candidate_key = (
            candidate["total_pnl_per_1usdc"],
            candidate["pnl_to_drawdown"],
            candidate["avg_pnl_per_trade"],
            candidate["sharpe_like"],
            -candidate["trades"],
        )
        incumbent_key = (
            incumbent["total_pnl_per_1usdc"],
            incumbent["pnl_to_drawdown"],
            incumbent["avg_pnl_per_trade"],
            incumbent["sharpe_like"],
            -incumbent["trades"],
        )
        return candidate_key > incumbent_key

    for threshold in thresholds:
        for max_spread_rel in max_spread_thresholds:
            for min_log_volume in volume_thresholds:
                for min_pct_into_slot, max_pct_into_slot in timing_windows:
                    candidate = evaluate_policy(
                        probs_up=probs_up,
                        labels=labels,
                        ask_up=ask_up,
                        ask_down=ask_down,
                        policy_inputs=policy_inputs,
                        policy={
                            "min_edge": float(threshold),
                            "max_spread_rel": float(max_spread_rel),
                            "min_pct_into_slot": float(min_pct_into_slot),
                            "max_pct_into_slot": float(max_pct_into_slot),
                            "min_log_volume": float(min_log_volume),
                        },
                    )
                    if candidate["trades"] < min_trades:
                        continue
                    if is_better(candidate, best):
                        best = candidate

    return best or {
        "min_edge": 0.0,
        "max_spread_rel": 0.25,
        "min_pct_into_slot": 0.05,
        "max_pct_into_slot": 0.90,
        "min_log_volume": 0.0,
        "trades": 0,
        "up_trades": 0,
        "down_trades": 0,
        "total_pnl_per_1usdc": 0.0,
        "avg_pnl_per_trade": 0.0,
        "win_rate": 0.0,
        "max_drawdown_per_1usdc": 0.0,
        "pnl_to_drawdown": 0.0,
        "sharpe_like": 0.0,
    }


def kelly_fraction(probability: float, price: float, fraction: float = 1.0) -> float:
    if price <= 0.0 or price >= 1.0:
        return 0.0
    b = (1.0 - price) / price
    raw = ((probability * b) - (1.0 - probability)) / b
    return float(max(0.0, min(1.0, raw * fraction)))


def model_metrics(labels: np.ndarray, probs: np.ndarray) -> dict[str, float]:
    return {
        "logloss": round(logloss(labels, probs), 6),
        "brier": round(brier_score(labels, probs), 6),
        "auc": round(roc_auc(labels, probs), 6),
    }


def save_metadata(path: str | Path, payload: dict[str, Any]) -> None:
    destination = Path(path)
    destination.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def load_metadata(path: str | Path) -> dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def load_bundle(model_dir: str | Path) -> tuple[xgb.Booster, dict[str, Any]]:
    base = Path(model_dir)
    booster = xgb.Booster()
    booster.load_model(base / DEFAULT_MODEL_NAME)
    metadata = load_metadata(base / DEFAULT_METADATA_NAME)
    return booster, metadata


def rows_to_matrix(rows: list[dict[str, Any]], feature_names: list[str]) -> np.ndarray:
    return np.asarray(
        [[float(row.get(feature, 0.0)) for feature in feature_names] for row in rows],
        dtype=np.float32,
    )


def score_rows(
    rows: list[dict[str, Any]],
    booster: xgb.Booster,
    metadata: dict[str, Any],
) -> list[dict[str, Any]]:
    feature_names = metadata["feature_names"]
    X = rows_to_matrix(rows, feature_names)
    matrix = build_matrix(X, None, feature_names)
    margins = booster.predict(matrix, output_margin=True)
    platt = metadata.get("platt_scaler", {"a": 1.0, "b": 0.0})
    probs_up = apply_platt_scaler(margins, float(platt["a"]), float(platt["b"]))
    policy = recommended_policy(metadata)
    ask_up_array = feature_array(rows, "f_ask_up")
    ask_down_array = feature_array(rows, "f_ask_down")
    masks = policy_masks(
        probs_up=probs_up,
        ask_up=ask_up_array,
        ask_down=ask_down_array,
        policy_inputs=build_policy_inputs(rows),
        policy=policy,
    )
    recommended_min_edge = float(policy["min_edge"])

    predictions: list[dict[str, Any]] = []
    for idx, row in enumerate(rows):
        proba_up = float(probs_up[idx])
        proba_down = float(1.0 - proba_up)
        ask_up = float(row.get("f_ask_up", 0.0))
        ask_down = float(row.get("f_ask_down", 0.0))
        edge_up = proba_up - ask_up if ask_up > 0.0 else None
        edge_down = proba_down - ask_down if ask_down > 0.0 else None
        signal_side = None
        signal_edge = 0.0
        if edge_up is not None or edge_down is not None:
            if (edge_up or -math.inf) >= (edge_down or -math.inf):
                signal_side = "UP"
                signal_edge = float(edge_up or 0.0)
            else:
                signal_side = "DOWN"
                signal_edge = float(edge_down or 0.0)

        predictions.append(
            {
                "proba_up": round(proba_up, 6),
                "proba_down": round(proba_down, 6),
                "edge_up": round(float(edge_up), 6) if edge_up is not None else None,
                "edge_down": round(float(edge_down), 6) if edge_down is not None else None,
                "signal_side": signal_side,
                "signal_edge": round(signal_edge, 6),
                "take_trade": bool(masks["take_trade"][idx]),
                "recommended_min_edge": round(recommended_min_edge, 6),
                "policy_take_trade": bool(masks["take_trade"][idx]),
                "policy_take_up": bool(masks["take_up"][idx]),
                "policy_take_down": bool(masks["take_down"][idx]),
                "policy_signal": 1 if bool(masks["take_up"][idx]) else (-1 if bool(masks["take_down"][idx]) else 0),
                "policy_edge": round(
                    float(masks["edges_up"][idx]) if bool(masks["take_up"][idx]) else (
                        float(masks["edges_down"][idx]) if bool(masks["take_down"][idx]) else 0.0
                    ),
                    6,
                ),
                "policy_min_edge": round(float(policy["min_edge"]), 6),
                "policy_max_spread_rel": round(float(policy["max_spread_rel"]), 6),
                "policy_min_pct_into_slot": round(float(policy["min_pct_into_slot"]), 6),
                "policy_max_pct_into_slot": round(float(policy["max_pct_into_slot"]), 6),
                "policy_min_log_volume": round(float(policy["min_log_volume"]), 6),
                "kelly_up_full": round(kelly_fraction(proba_up, ask_up, 1.0), 6),
                "kelly_up_half": round(kelly_fraction(proba_up, ask_up, 0.5), 6),
                "kelly_down_full": round(kelly_fraction(proba_down, ask_down, 1.0), 6),
                "kelly_down_half": round(kelly_fraction(proba_down, ask_down, 0.5), 6),
            }
        )

    return predictions
