<?php

namespace App\Services;

use Illuminate\Http\Client\PendingRequest;
use Illuminate\Support\Carbon;
use Illuminate\Support\Facades\Http;

class EngineService
{
    public function __construct(
        private readonly string $baseUrl,
        private readonly int $timeout,
    ) {}

    /**
     * Deploy a Gnosis Safe wallet via the Builder Relayer.
     *
     * @return array{safe_address: string, transaction_hash: string}
     */
    public function deploySafe(int $walletId, string $signerAddress, string $privateKeyEnc): array
    {
        return $this->client()
            ->timeout($this->timeout * 5)
            ->post('/internal/wallet/deploy-safe', [
                'wallet_id' => $walletId,
                'signer_address' => $signerAddress,
                'private_key_enc' => $privateKeyEnc,
            ])
            ->throw()
            ->json();
    }

    public function activateStrategy(
        int $walletId,
        int $strategyId,
        array $graph,
        array $markets,
        float $maxPositionUsdc = 1000.0,
        bool $isPaper = false,
        string $privateKeyEnc = '',
        string $safeAddress = '',
    ): void {
        $this->client()->post('/internal/strategy/activate', [
            'wallet_id' => $walletId,
            'strategy_id' => $strategyId,
            'graph' => $graph,
            'markets' => $markets,
            'max_position_usdc' => $maxPositionUsdc,
            'is_paper' => $isPaper,
            'private_key_enc' => $privateKeyEnc,
            'safe_address' => $safeAddress,
        ])->throw();
    }

    public function deactivateStrategy(int $walletId, int $strategyId): void
    {
        $this->client()->post('/internal/strategy/deactivate', [
            'wallet_id' => $walletId,
            'strategy_id' => $strategyId,
        ])->throw();
    }

    public function killStrategy(int $walletId, int $strategyId): void
    {
        $this->client()->post('/internal/strategy/kill', [
            'wallet_id' => $walletId,
            'strategy_id' => $strategyId,
        ])->throw();
    }

    public function unkillStrategy(int $walletId, int $strategyId): void
    {
        $this->client()->post('/internal/strategy/unkill', [
            'wallet_id' => $walletId,
            'strategy_id' => $strategyId,
        ])->throw();
    }

    public function walletState(int $walletId): array
    {
        return $this->client()
            ->get("/internal/wallet/{$walletId}/state")
            ->throw()
            ->json();
    }

    public function runBacktest(array $strategyGraph, array $marketFilter, string $dateFrom, string $dateTo): array
    {
        return $this->client()
            ->timeout($this->timeout * 3)
            ->post('/internal/backtest/run', [
                'strategy_graph' => $strategyGraph,
                'market_filter' => $marketFilter,
                'date_from' => Carbon::parse($dateFrom)->startOfDay()->toIso8601ZuluString(),
                'date_to' => Carbon::parse($dateTo)->endOfDay()->toIso8601ZuluString(),
            ])
            ->throw()
            ->json();
    }

    public function engineStatus(): array
    {
        return $this->client()
            ->get('/internal/engine/status')
            ->throw()
            ->json();
    }

    public function slotStats(int $slotDuration, array $symbols = [], float $hours = 168.0): array
    {
        return $this->client()
            ->get('/internal/stats/slots', array_filter([
                'slot_duration' => $slotDuration,
                'symbols' => ! empty($symbols) ? implode(',', $symbols) : null,
                'hours' => $hours,
            ]))
            ->throw()
            ->json();
    }

    public function watchLeader(string $leaderAddress): void
    {
        $this->client()->post('/internal/copy/watch', [
            'leader_address' => $leaderAddress,
        ])->throw();
    }

    public function unwatchLeader(string $leaderAddress): void
    {
        $this->client()->post('/internal/copy/unwatch', [
            'leader_address' => $leaderAddress,
        ])->throw();
    }

    private function client(): PendingRequest
    {
        return Http::baseUrl($this->baseUrl)
            ->timeout($this->timeout)
            ->acceptJson();
    }
}
