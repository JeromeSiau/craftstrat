<?php

namespace App\Services;

use App\Models\Strategy;
use App\Models\Wallet;
use App\Models\WalletStrategy;
use Illuminate\Http\Client\RequestException;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\DB;

class StrategyActivationService
{
    public function __construct(private readonly EngineService $engine) {}

    /**
     * @throws RequestException
     */
    public function activate(Strategy $strategy): void
    {
        $assignments = $strategy->walletStrategies()
            ->where('is_running', false)
            ->with('wallet')
            ->get();

        $this->ensureWalletsAreDeployed($assignments);
        $this->performEngineActions(
            $assignments,
            fn (WalletStrategy $assignment) => $this->activateAssignmentOnEngine(
                $strategy,
                $assignment,
                $assignment->wallet,
            ),
            fn (WalletStrategy $assignment) => $this->engine->deactivateStrategy(
                $assignment->wallet_id,
                $strategy->id,
            ),
        );

        try {
            DB::transaction(function () use ($strategy, $assignments): void {
                foreach ($assignments as $assignment) {
                    $assignment->update(['is_running' => true, 'started_at' => now()]);
                }

                $strategy->update(['is_active' => true]);
            });
        } catch (\Throwable $e) {
            $this->rollbackAssignments(
                $assignments,
                fn (WalletStrategy $assignment) => $this->engine->deactivateStrategy(
                    $assignment->wallet_id,
                    $strategy->id,
                ),
            );

            throw $e;
        }
    }

    /**
     * @throws RequestException
     */
    public function deactivate(Strategy $strategy): void
    {
        $assignments = $strategy->walletStrategies()
            ->where('is_running', true)
            ->with('wallet')
            ->get();

        $this->performEngineActions(
            $assignments,
            fn (WalletStrategy $assignment) => $this->engine->deactivateStrategy(
                $assignment->wallet_id,
                $strategy->id,
            ),
            fn (WalletStrategy $assignment) => $this->activateAssignmentOnEngine(
                $strategy,
                $assignment,
                $assignment->wallet,
            ),
        );

        try {
            DB::transaction(function () use ($strategy, $assignments): void {
                foreach ($assignments as $assignment) {
                    $assignment->update(['is_running' => false, 'started_at' => null]);
                }

                $strategy->update(['is_active' => false]);
            });
        } catch (\Throwable $e) {
            $this->rollbackAssignments(
                $assignments,
                fn (WalletStrategy $assignment) => $this->activateAssignmentOnEngine(
                    $strategy,
                    $assignment,
                    $assignment->wallet,
                ),
            );

            throw $e;
        }
    }

    public function deactivateAllForStrategy(Strategy $strategy): void
    {
        $assignments = $strategy->walletStrategies()
            ->where('is_running', true)
            ->with('wallet')
            ->get();

        $this->performEngineActions(
            $assignments,
            fn (WalletStrategy $assignment) => $this->engine->deactivateStrategy(
                $assignment->wallet_id,
                $strategy->id,
            ),
            fn (WalletStrategy $assignment) => $this->activateAssignmentOnEngine(
                $strategy,
                $assignment,
                $assignment->wallet,
            ),
        );
    }

    public function deactivateAllForWallet(Wallet $wallet): void
    {
        $assignments = $wallet->walletStrategies()
            ->where('is_running', true)
            ->with('strategy')
            ->get();

        $this->performEngineActions(
            $assignments,
            fn (WalletStrategy $assignment) => $this->engine->deactivateStrategy(
                $wallet->id,
                $assignment->strategy_id,
            ),
            fn (WalletStrategy $assignment) => $this->activateAssignmentOnEngine(
                $assignment->strategy,
                $assignment,
                $wallet,
            ),
        );
    }

    private function ensureWalletsAreDeployed(Collection $assignments): void
    {
        foreach ($assignments as $assignment) {
            if (! $assignment->wallet->isDeployed()) {
                throw new \RuntimeException("Wallet #{$assignment->wallet_id} Safe is not deployed yet.");
            }
        }
    }

    private function activateAssignmentOnEngine(Strategy $strategy, WalletStrategy $assignment, Wallet $wallet): void
    {
        $this->engine->activateStrategy(
            $assignment->wallet_id,
            $strategy->id,
            $strategy->graph,
            $assignment->markets ?? [],
            (float) $assignment->max_position_usdc,
            (bool) $assignment->is_paper,
            $wallet->private_key_enc,
            $wallet->safe_address ?? '',
        );
    }

    private function performEngineActions(Collection $assignments, callable $action, callable $compensation): void
    {
        $completed = collect();

        try {
            foreach ($assignments as $assignment) {
                $action($assignment);
                $completed->push($assignment);
            }
        } catch (RequestException $e) {
            $this->rollbackAssignments($completed, $compensation);

            throw $e;
        }
    }

    private function rollbackAssignments(Collection $assignments, callable $compensation): void
    {
        foreach ($assignments->reverse()->values() as $assignment) {
            try {
                $compensation($assignment);
            } catch (RequestException $rollbackException) {
                report($rollbackException);
            }
        }
    }
}
