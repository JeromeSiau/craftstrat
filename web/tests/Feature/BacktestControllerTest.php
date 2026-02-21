<?php

use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Models\User;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays backtests index page', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    BacktestResult::factory()->count(2)->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('backtests/index', false)
            ->has('results.data', 2)
        );
});

it('shows a backtest result', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.show', $result))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('backtests/show', false)
            ->has('result')
        );
});

it('prevents viewing another users backtest result', function () {
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $other->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $other->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.show', $result))
        ->assertForbidden();
});

it('runs a backtest via the engine and stores the result', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 42,
        'win_rate' => 0.65,
        'pnl' => 123.45,
        'max_drawdown' => 0.12,
        'sharpe_ratio' => 1.5,
        'trades' => [],
    ])]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(7)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertRedirect();

    $result = BacktestResult::where('strategy_id', $strategy->id)->first();
    expect($result)->not->toBeNull()
        ->and($result->total_trades)->toBe(42)
        ->and($result->win_rate)->toBe('0.6500');
});

it('validates backtest request fields', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [])
        ->assertSessionHasErrors(['date_from', 'date_to']);
});

it('enforces backtest_days limit for free plan', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(60)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertSessionHasErrors(['date_from']);
});

it('allows full history backtest for starter plan', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 10,
        'win_rate' => 0.50,
        'pnl' => 50.0,
        'max_drawdown' => 0.05,
        'sharpe_ratio' => 1.0,
        'trades' => [],
    ])]);

    $user = User::factory()->create(['plan' => 'starter']);
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    $this->actingAs($user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(365)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertRedirect();
});

it('requires authentication', function () {
    $this->get(route('backtests.index'))->assertRedirect('/login');
});
