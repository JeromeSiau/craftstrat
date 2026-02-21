<?php

use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;

uses(Tests\TestCase::class, Illuminate\Foundation\Testing\RefreshDatabase::class);

it('has many strategies', function () {
    $user = User::factory()->create();
    Strategy::factory()->count(3)->create(['user_id' => $user->id]);

    expect($user->strategies)->toHaveCount(3);
});

it('has many wallets', function () {
    $user = User::factory()->create();
    Wallet::factory()->count(2)->create(['user_id' => $user->id]);

    expect($user->wallets)->toHaveCount(2);
});

it('has many backtest results', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);
    BacktestResult::factory()->create(['user_id' => $user->id, 'strategy_id' => $strategy->id]);

    expect($user->backtestResults)->toHaveCount(1);
});

it('returns correct plan limits for free plan', function () {
    $user = User::factory()->create(['plan' => 'free']);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBe(1)
        ->and($limits['max_strategies'])->toBe(2)
        ->and($limits['max_leaders'])->toBe(1)
        ->and($limits['backtest_days'])->toBe(30);
});

it('returns correct plan limits for pro plan', function () {
    $user = User::factory()->create(['plan' => 'pro']);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBe(25)
        ->and($limits['max_strategies'])->toBeNull()
        ->and($limits['max_leaders'])->toBeNull()
        ->and($limits['backtest_days'])->toBeNull();
});

it('returns correct plan limits for enterprise plan', function () {
    $user = User::factory()->create(['plan' => 'enterprise']);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBeNull()
        ->and($limits['max_strategies'])->toBeNull();
});

it('defaults to free plan limits when plan is not set', function () {
    $user = User::factory()->create();
    $user->plan = null;
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBe(1);
});
