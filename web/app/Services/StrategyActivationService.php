<?php

namespace App\Services;

use App\Models\Strategy;
use App\Models\Wallet;
use Illuminate\Http\Client\RequestException;
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

        // Ensure all assigned wallets have a deployed Safe
        foreach ($assignments as $assignment) {
            if (! $assignment->wallet->isDeployed()) {
                throw new \RuntimeException("Wallet #{$assignment->wallet_id} Safe is not deployed yet.");
            }
        }

        DB::transaction(function () use ($strategy, $assignments): void {
            foreach ($assignments as $assignment) {
                $this->engine->activateStrategy(
                    $assignment->wallet_id,
                    $strategy->id,
                    $strategy->graph,
                    $assignment->markets ?? [],
                    (float) $assignment->max_position_usdc,
                    (bool) $assignment->is_paper,
                    $assignment->wallet->private_key_enc,
                    $assignment->wallet->safe_address,
                );

                $assignment->update(['is_running' => true, 'started_at' => now()]);
            }

            $strategy->update(['is_active' => true]);
        });
    }

    /**
     * @throws RequestException
     */
    public function deactivate(Strategy $strategy): void
    {
        $assignments = $strategy->walletStrategies()
            ->where('is_running', true)
            ->get();

        DB::transaction(function () use ($strategy, $assignments): void {
            foreach ($assignments as $assignment) {
                $this->engine->deactivateStrategy($assignment->wallet_id, $strategy->id);
                $assignment->update(['is_running' => false, 'started_at' => null]);
            }

            $strategy->update(['is_active' => false]);
        });
    }

    public function deactivateAllForStrategy(Strategy $strategy): void
    {
        $assignments = $strategy->walletStrategies()
            ->where('is_running', true)
            ->get();

        foreach ($assignments as $assignment) {
            $this->engine->deactivateStrategy($assignment->wallet_id, $strategy->id);
        }
    }

    public function deactivateAllForWallet(Wallet $wallet): void
    {
        $assignments = $wallet->walletStrategies()
            ->where('is_running', true)
            ->get();

        foreach ($assignments as $assignment) {
            $this->engine->deactivateStrategy($wallet->id, $assignment->strategy_id);
        }
    }
}
