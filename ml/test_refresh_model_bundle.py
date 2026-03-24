from __future__ import annotations

from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))

from refresh_model_bundle import RefreshConfig, evaluate_auto_promotion


class AutoPromotionDecisionTest(unittest.TestCase):
    def make_config(self) -> RefreshConfig:
        return RefreshConfig(
            engine_internal_url="http://engine:8080",
            artifacts_dir=Path("/tmp/models"),
            data_dir=Path("/tmp/data"),
            model_name="btc-15m-xgb-policy",
            slot_duration=900,
            symbols="BTC",
            hours=720.0,
            sample_every=6,
            limit=5000,
            max_rows=0,
            verbose_eval=50,
            rl_gamma=0.999,
            auto_promote_enabled=True,
            min_candidate_rows_for_promotion=50000,
            min_policy_total_pnl_delta=0.0,
            min_entry_total_reward_delta=0.0,
            min_efficiency_ratio=0.9,
            max_drawdown_ratio=1.1,
            min_trade_ratio=0.5,
        )

    def test_rejects_candidate_when_dataset_too_small(self) -> None:
        decision = evaluate_auto_promotion(
            self.make_config(),
            {"rows": 20000},
            {
                "policy": {"recommended": {"total_pnl_per_1usdc": 100.0}},
                "rl_like": {"entry_policy": {"recommended": {"total_reward_per_contract": 100.0}}},
            },
            {
                "policy": {
                    "recommended": {
                        "total_pnl_per_1usdc": 120.0,
                        "pnl_to_drawdown": 2.0,
                        "max_drawdown_per_1usdc": 10.0,
                        "trades": 1000,
                    }
                },
                "rl_like": {
                    "entry_policy": {
                        "recommended": {
                            "total_reward_per_contract": 130.0,
                            "reward_to_drawdown": 3.0,
                            "max_drawdown_per_contract": 1.0,
                            "trades": 1000,
                        }
                    }
                },
            },
        )

        self.assertFalse(decision["eligible"])
        self.assertEqual(decision["verdict"], "reject")
        self.assertFalse(decision["gates"]["min_candidate_rows"]["passed"])

    def test_promotes_candidate_when_all_gates_pass(self) -> None:
        decision = evaluate_auto_promotion(
            self.make_config(),
            {"rows": 120000},
            {
                "policy": {
                    "recommended": {
                        "total_pnl_per_1usdc": 100.0,
                        "pnl_to_drawdown": 2.0,
                        "max_drawdown_per_1usdc": 10.0,
                        "trades": 1000,
                    }
                },
                "rl_like": {
                    "entry_policy": {
                        "recommended": {
                            "total_reward_per_contract": 100.0,
                            "reward_to_drawdown": 3.0,
                            "max_drawdown_per_contract": 1.0,
                            "trades": 1000,
                        }
                    }
                },
            },
            {
                "policy": {
                    "recommended": {
                        "total_pnl_per_1usdc": 120.0,
                        "pnl_to_drawdown": 2.1,
                        "max_drawdown_per_1usdc": 10.5,
                        "trades": 900,
                    }
                },
                "rl_like": {
                    "entry_policy": {
                        "recommended": {
                            "total_reward_per_contract": 130.0,
                            "reward_to_drawdown": 3.1,
                            "max_drawdown_per_contract": 1.05,
                            "trades": 800,
                        }
                    }
                },
            },
        )

        self.assertTrue(decision["eligible"])
        self.assertEqual(decision["verdict"], "promote")


if __name__ == "__main__":
    unittest.main()
