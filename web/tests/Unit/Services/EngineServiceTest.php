<?php

use App\Services\EngineService;
use Illuminate\Support\Facades\Http;

uses(Tests\TestCase::class);

beforeEach(function () {
    $this->service = new EngineService(
        baseUrl: 'http://engine:8080',
        timeout: 10,
    );
});

it('sends activate strategy request', function () {
    Http::fake(['engine:8080/internal/strategy/activate' => Http::response(null, 200)]);

    $this->service->activateStrategy(1, 100, ['mode' => 'form'], ['btc-15m']);

    Http::assertSent(fn ($request) => $request->url() === 'http://engine:8080/internal/strategy/activate'
        && $request['wallet_id'] === 1
        && $request['strategy_id'] === 100
    );
});

it('sends deactivate strategy request', function () {
    Http::fake(['engine:8080/internal/strategy/deactivate' => Http::response(null, 200)]);

    $this->service->deactivateStrategy(1, 100);

    Http::assertSent(fn ($request) => $request->url() === 'http://engine:8080/internal/strategy/deactivate'
        && $request['wallet_id'] === 1
        && $request['strategy_id'] === 100
    );
});

it('fetches wallet state', function () {
    Http::fake(['engine:8080/internal/wallet/42/state' => Http::response([
        'wallet_id' => 42,
        'assignments' => [],
    ])]);

    $result = $this->service->walletState(42);

    expect($result)->toHaveKey('wallet_id', 42);
});

it('runs backtest', function () {
    Http::fake(['engine:8080/internal/backtest/run' => Http::response([
        'total_trades' => 5,
        'win_rate' => 0.6,
        'total_pnl_usdc' => 42.5,
        'max_drawdown' => 0.15,
        'sharpe_ratio' => 1.2,
        'trades' => [],
    ])]);

    $result = $this->service->runBacktest(['mode' => 'form'], ['btc-15m'], '2025-01-01T00:00:00Z', '2025-02-01T00:00:00Z');

    expect($result)
        ->toHaveKey('total_trades', 5)
        ->toHaveKey('win_rate', 0.6);
});

it('fetches engine status', function () {
    Http::fake(['engine:8080/internal/engine/status' => Http::response([
        'active_wallets' => 3,
        'active_assignments' => 7,
        'ticks_processed' => 150000,
        'uptime_secs' => 3600,
    ])]);

    $result = $this->service->engineStatus();

    expect($result)->toHaveKey('active_wallets', 3);
});

it('sends watch leader request', function () {
    Http::fake(['engine:8080/internal/copy/watch' => Http::response(null, 200)]);

    $this->service->watchLeader('0xabc123');

    Http::assertSent(fn ($request) => $request['leader_address'] === '0xabc123'
    );
});

it('sends unwatch leader request', function () {
    Http::fake(['engine:8080/internal/copy/unwatch' => Http::response(null, 200)]);

    $this->service->unwatchLeader('0xabc123');

    Http::assertSent(fn ($request) => $request['leader_address'] === '0xabc123'
    );
});

it('throws on engine error', function () {
    Http::fake(['engine:8080/internal/engine/status' => Http::response(null, 500)]);

    $this->service->engineStatus();
})->throws(\Illuminate\Http\Client\RequestException::class);
