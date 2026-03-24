#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

import xgboost as xgb

from xgboost_pipeline import (
    AUX_MODEL_FILES,
    DEFAULT_METADATA_NAME,
    DEFAULT_MODEL_NAME,
    apply_platt_scaler,
    build_matrix,
    build_policy_inputs,
    derive_rl_like_targets,
    evaluate_policy,
    evaluate_entry_policy,
    fit_platt_scaler,
    load_bundle,
    load_ndjson_dataset,
    model_metrics,
    predict_regression,
    regression_metrics,
    save_metadata,
    score_rows,
    split_dataset,
    sweep_entry_value_thresholds,
    sweep_edge_thresholds,
)


def train_regression_booster(
    train_X,
    train_y,
    val_X,
    val_y,
    feature_names,
    args: argparse.Namespace,
    seed_offset: int = 0,
) -> xgb.Booster:
    train_matrix = build_matrix(train_X, train_y, feature_names)
    val_matrix = build_matrix(val_X, val_y, feature_names)
    params = {
        "objective": "reg:squarederror",
        "eval_metric": "rmse",
        "eta": args.eta,
        "max_depth": args.max_depth,
        "min_child_weight": args.min_child_weight,
        "subsample": args.subsample,
        "colsample_bytree": args.colsample_bytree,
        "lambda": args.reg_lambda,
        "alpha": args.reg_alpha,
        "tree_method": "hist",
        "seed": args.seed + seed_offset,
    }
    return xgb.train(
        params=params,
        dtrain=train_matrix,
        num_boost_round=args.num_boost_round,
        evals=[(train_matrix, "train"), (val_matrix, "val")],
        early_stopping_rounds=args.early_stopping_rounds,
        verbose_eval=False,
    )


def train_command(args: argparse.Namespace) -> int:
    dataset = load_ndjson_dataset(args.dataset)
    split = split_dataset(
        dataset,
        train_fraction=args.train_fraction,
        val_fraction=args.val_fraction,
    )

    train_matrix = build_matrix(split.train.X, split.train.y, split.feature_names)
    val_matrix = build_matrix(split.val.X, split.val.y, split.feature_names)
    test_matrix = build_matrix(split.test.X, split.test.y, split.feature_names)

    positives = float(split.train.y.sum())
    negatives = float(split.train.size - positives)
    scale_pos_weight = (negatives / positives) if positives > 0 else 1.0

    params = {
        "objective": "binary:logistic",
        "eval_metric": "logloss",
        "eta": args.eta,
        "max_depth": args.max_depth,
        "min_child_weight": args.min_child_weight,
        "subsample": args.subsample,
        "colsample_bytree": args.colsample_bytree,
        "lambda": args.reg_lambda,
        "alpha": args.reg_alpha,
        "tree_method": "hist",
        "seed": args.seed,
        "scale_pos_weight": scale_pos_weight,
    }

    booster = xgb.train(
        params=params,
        dtrain=train_matrix,
        num_boost_round=args.num_boost_round,
        evals=[(train_matrix, "train"), (val_matrix, "val")],
        early_stopping_rounds=args.early_stopping_rounds,
        verbose_eval=args.verbose_eval,
    )

    train_raw = booster.predict(train_matrix)
    val_raw = booster.predict(val_matrix)
    test_raw = booster.predict(test_matrix)

    val_margins = booster.predict(val_matrix, output_margin=True)
    test_margins = booster.predict(test_matrix, output_margin=True)
    platt_a, platt_b = fit_platt_scaler(val_margins, split.val.y)
    candidate_val_calibrated = apply_platt_scaler(val_margins, platt_a, platt_b)
    candidate_test_calibrated = apply_platt_scaler(test_margins, platt_a, platt_b)

    val_raw_metrics = model_metrics(split.val.y, val_raw)
    val_calibrated_metrics = model_metrics(split.val.y, candidate_val_calibrated)
    use_platt = val_calibrated_metrics["logloss"] < val_raw_metrics["logloss"]
    if use_platt:
        val_calibrated = candidate_val_calibrated
        test_calibrated = candidate_test_calibrated
        active_platt = {"a": platt_a, "b": platt_b, "enabled": True}
    else:
        val_calibrated = val_raw
        test_calibrated = test_raw
        active_platt = {"a": 1.0, "b": 0.0, "enabled": False}

    threshold_summary = sweep_edge_thresholds(
        probs_up=val_calibrated,
        labels=split.val.y,
        ask_up=split.val.ask_up,
        ask_down=split.val.ask_down,
        rows=split.val.rows,
    )
    test_policy = evaluate_policy(
        probs_up=test_calibrated,
        labels=split.test.y,
        ask_up=split.test.ask_up,
        ask_down=split.test.ask_down,
        policy_inputs=build_policy_inputs(split.test.rows),
        policy=threshold_summary,
    )

    rl_train = derive_rl_like_targets(split.train.rows, split.train.y, gamma=args.rl_gamma)
    rl_val = derive_rl_like_targets(split.val.rows, split.val.y, gamma=args.rl_gamma)
    rl_test = derive_rl_like_targets(split.test.rows, split.test.y, gamma=args.rl_gamma)

    entry_up_model = train_regression_booster(
        split.train.X,
        rl_train.entry_up,
        split.val.X,
        rl_val.entry_up,
        split.feature_names,
        args,
        seed_offset=11,
    )
    entry_down_model = train_regression_booster(
        split.train.X,
        rl_train.entry_down,
        split.val.X,
        rl_val.entry_down,
        split.feature_names,
        args,
        seed_offset=23,
    )
    exit_up_model = train_regression_booster(
        split.train.X,
        rl_train.hold_advantage_up,
        split.val.X,
        rl_val.hold_advantage_up,
        split.feature_names,
        args,
        seed_offset=37,
    )
    exit_down_model = train_regression_booster(
        split.train.X,
        rl_train.hold_advantage_down,
        split.val.X,
        rl_val.hold_advantage_down,
        split.feature_names,
        args,
        seed_offset=53,
    )

    entry_up_val = predict_regression(entry_up_model, split.val.X, split.feature_names)
    entry_down_val = predict_regression(entry_down_model, split.val.X, split.feature_names)
    entry_up_test = predict_regression(entry_up_model, split.test.X, split.feature_names)
    entry_down_test = predict_regression(entry_down_model, split.test.X, split.feature_names)
    exit_up_val = predict_regression(exit_up_model, split.val.X, split.feature_names)
    exit_down_val = predict_regression(exit_down_model, split.val.X, split.feature_names)
    exit_up_test = predict_regression(exit_up_model, split.test.X, split.feature_names)
    exit_down_test = predict_regression(exit_down_model, split.test.X, split.feature_names)

    entry_policy_summary = sweep_entry_value_thresholds(
        entry_up_val,
        entry_down_val,
        rl_val.entry_up,
        rl_val.entry_down,
    )
    entry_test_policy = evaluate_entry_policy(
        entry_up_test,
        entry_down_test,
        rl_test.entry_up,
        rl_test.entry_down,
        float(entry_policy_summary["min_value"]),
    )

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    booster.save_model(output_dir / DEFAULT_MODEL_NAME)
    entry_up_model.save_model(output_dir / AUX_MODEL_FILES["entry_up"])
    entry_down_model.save_model(output_dir / AUX_MODEL_FILES["entry_down"])
    exit_up_model.save_model(output_dir / AUX_MODEL_FILES["exit_up"])
    exit_down_model.save_model(output_dir / AUX_MODEL_FILES["exit_down"])

    feature_importance = booster.get_score(importance_type="gain")
    metadata = {
        "created_at": datetime.now(timezone.utc).isoformat(),
        "dataset_path": str(Path(args.dataset).resolve()),
        "feature_names": split.feature_names,
        "train_fraction": args.train_fraction,
        "val_fraction": args.val_fraction,
        "best_iteration": int(getattr(booster, "best_iteration", args.num_boost_round - 1)),
        "best_score": float(getattr(booster, "best_score", 0.0)),
        "params": params,
        "platt_scaler": active_platt,
        "thresholds": {
            "recommended_min_edge": threshold_summary["min_edge"],
            "validation_best": threshold_summary,
        },
        "policy": {
            "selection_metric": "validation_total_pnl_per_1usdc",
            "recommended": threshold_summary,
            "test": test_policy,
        },
        "rl_like": {
            "discount": args.rl_gamma,
            "entry_policy": {
                "selection_metric": "validation_total_reward_per_contract",
                "recommended": entry_policy_summary,
                "test": entry_test_policy,
            },
            "exit_policy": {
                "selection_metric": "hold_advantage_threshold_zero",
                "recommended": {
                    "hold_advantage_threshold": 0.0,
                    "up_exit_rate": round(float((exit_up_val <= 0.0).mean()), 6),
                    "down_exit_rate": round(float((exit_down_val <= 0.0).mean()), 6),
                },
                "test": {
                    "hold_advantage_threshold": 0.0,
                    "up_exit_rate": round(float((exit_up_test <= 0.0).mean()), 6),
                    "down_exit_rate": round(float((exit_down_test <= 0.0).mean()), 6),
                },
            },
        },
        "split_sizes": {
            "train_rows": split.train.size,
            "val_rows": split.val.size,
            "test_rows": split.test.size,
        },
        "metrics": {
            "train_raw": model_metrics(split.train.y, train_raw),
            "val_raw": val_raw_metrics,
            "val_calibrated": model_metrics(split.val.y, val_calibrated),
            "test_raw": model_metrics(split.test.y, test_raw),
            "test_calibrated": model_metrics(split.test.y, test_calibrated),
            "entry_up_val": regression_metrics(rl_val.entry_up, entry_up_val),
            "entry_up_test": regression_metrics(rl_test.entry_up, entry_up_test),
            "entry_down_val": regression_metrics(rl_val.entry_down, entry_down_val),
            "entry_down_test": regression_metrics(rl_test.entry_down, entry_down_test),
            "exit_up_val": regression_metrics(rl_val.hold_advantage_up, exit_up_val),
            "exit_up_test": regression_metrics(rl_test.hold_advantage_up, exit_up_test),
            "exit_down_val": regression_metrics(rl_val.hold_advantage_down, exit_down_val),
            "exit_down_test": regression_metrics(rl_test.hold_advantage_down, exit_down_test),
        },
        "feature_importance_gain": {
            name: round(float(feature_importance.get(name, 0.0)), 6)
            for name in split.feature_names
            if feature_importance.get(name) is not None
        },
    }
    save_metadata(output_dir / DEFAULT_METADATA_NAME, metadata)

    print(json.dumps(metadata["metrics"], indent=2, sort_keys=True))
    print(json.dumps(metadata["thresholds"], indent=2, sort_keys=True))
    print(json.dumps(metadata["rl_like"], indent=2, sort_keys=True))
    print(f"saved model to {output_dir / DEFAULT_MODEL_NAME}")
    print(f"saved model to {output_dir / AUX_MODEL_FILES['entry_up']}")
    print(f"saved model to {output_dir / AUX_MODEL_FILES['entry_down']}")
    print(f"saved model to {output_dir / AUX_MODEL_FILES['exit_up']}")
    print(f"saved model to {output_dir / AUX_MODEL_FILES['exit_down']}")
    print(f"saved metadata to {output_dir / DEFAULT_METADATA_NAME}")

    return 0


def predict_command(args: argparse.Namespace) -> int:
    bundle = load_bundle(args.model_dir)

    rows = []
    source = Path(args.input)
    with source.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            rows.append(json.loads(line))

    predictions = score_rows(rows, bundle)

    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("w", encoding="utf-8") as handle:
            for prediction in predictions:
                handle.write(json.dumps(prediction, separators=(",", ":")) + "\n")
    else:
        for prediction in predictions:
            print(json.dumps(prediction, separators=(",", ":")))

    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Train or score the slot model with XGBoost.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    train = subparsers.add_parser("train", help="train a model from the exported NDJSON dataset")
    train.add_argument("--dataset", required=True, help="NDJSON dataset exported by ml:export-slot-dataset")
    train.add_argument("--output-dir", required=True, help="directory where model.json and metadata.json are written")
    train.add_argument("--train-fraction", type=float, default=0.70)
    train.add_argument("--val-fraction", type=float, default=0.15)
    train.add_argument("--num-boost-round", type=int, default=400)
    train.add_argument("--early-stopping-rounds", type=int, default=40)
    train.add_argument("--eta", type=float, default=0.05)
    train.add_argument("--max-depth", type=int, default=6)
    train.add_argument("--min-child-weight", type=float, default=2.0)
    train.add_argument("--subsample", type=float, default=0.85)
    train.add_argument("--colsample-bytree", type=float, default=0.85)
    train.add_argument("--reg-lambda", type=float, default=1.0)
    train.add_argument("--reg-alpha", type=float, default=0.0)
    train.add_argument("--rl-gamma", type=float, default=0.999)
    train.add_argument("--seed", type=int, default=42)
    train.add_argument("--verbose-eval", type=int, default=50)
    train.set_defaults(func=train_command)

    predict = subparsers.add_parser("predict", help="score NDJSON rows with an existing model")
    predict.add_argument("--model-dir", required=True, help="directory containing model.json and metadata.json")
    predict.add_argument("--input", required=True, help="NDJSON file to score")
    predict.add_argument("--output", help="optional NDJSON output path")
    predict.set_defaults(func=predict_command)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
